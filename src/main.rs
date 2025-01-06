// Run without terminal if not in debug
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// use std::sync::mpsc;
use dotenv::dotenv;
use std::thread::sleep;
use std::time::Duration;
use std::sync::mpsc;
use tray_item::{IconSource, TrayItem};
pub mod piapi_handler;

enum Message {
    Open,
    Quit,
    Disable10,
    Disable30,
    Disable5min,
    Toggle,
}

async fn toggle_pihole(piapi: &piapi_handler::AuthPiHoleAPI) {
    match piapi.status().await {
        Ok(status) => {
            match status.get("status").map(String::as_str) {
                Some("enabled") => {
                    // disable pihole
                    match piapi.disable(0).await {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("Error trying to disable: {}", e);
                        }
                    }
                }
                Some("disabled") => {
                    // enable pihole
                    match piapi.enable().await {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("Error trying to enable: {}", e);
                        }
                    }
                }
                Some(other) => {
                    eprintln!("Unexpected value in status: {}", other);
                }
                None => {
                    eprintln!("Key \"status\" not found in status api function");
                }
            }

        }
        Err(e) => {
            eprintln!("ERROR getting status {}", e);
        }
    }
}

#[tokio::main]
async fn main() {
    // Prep retrieval of environment variables
    dotenv().ok();

    // Retrieve env variables and create the api handler
    let pi_api = piapi_handler::AuthPiHoleAPI::new(
        std::env::var("PI_HOLE_ADDR").expect("PI_HOLE_ADDR must be set").to_string(),
        std::env::var("PI_HOLE_KEY").expect("PI_HOLE_KEY must be set").to_string(),
    );

    // Setup Tray Item
    let mut tray = match TrayItem::new(
        "Pi-Hole", 
        IconSource::Resource("APPICON")) 
        {
        Ok(tray) => tray,
        Err(e) => {
            eprintln!("Failed to create tray item: {}", e);
            return;
        },
    };


    // Add to the tray
    let status_label = tray.inner_mut().add_label_with_id("status_label").unwrap();


    // Setup tx/rx channel
    let (tx, rx) = mpsc::sync_channel(1);
    
    // Add break line
    tray.inner_mut().add_separator().unwrap();

    
    
    // Setup open in browser button
    let open_browser_tx = tx.clone();
    tray.add_menu_item("Open in Browser", move || {
        open_browser_tx.send(Message::Open).unwrap();
    })
    .unwrap();

    // Add break line
    tray.inner_mut().add_separator().unwrap();

    // Setup enable button
    let toggle_tx = tx.clone();
    tray.add_menu_item("Toggle", move || {
        toggle_tx.send(Message::Toggle).unwrap();
    })
    .unwrap();
    
    // Setup disable button
    let disable_tx = tx.clone();
    tray.add_menu_item("Disable 10 Seconds", move || {
        disable_tx.send(Message::Disable10).unwrap();
    })
    .unwrap();

    // Setup disable button
    let disable_tx = tx.clone();
    tray.add_menu_item("Disable 30 Seconds", move || {
        disable_tx.send(Message::Disable30).unwrap();
    })
    .unwrap();

    // Setup disable button
    let disable_tx = tx.clone();
    tray.add_menu_item("Disable 5 minutes", move || {
        disable_tx.send(Message::Disable5min).unwrap();
    })
    .unwrap();

    // Add break line
    tray.inner_mut().add_separator().unwrap();

    // Add quit button (exits the app)
    let quit_tx = tx.clone();
    tray.add_menu_item("Quit", move || {
        quit_tx.send(Message::Quit).unwrap();
    })
    .unwrap();

    loop {
        // Update a status label to display the status of the app
        match pi_api.status().await {
            Ok(status) => {
                if let Some(rpi_status) = status.get("status") {
                    tray.inner_mut().set_menu_item_label(format!("Status: {}", rpi_status).as_str(), status_label).unwrap();
                }
            }
            Err(e) => {eprintln!("Error calling status: {}", e);}
        }


        // Handle the button presses from the system tray
        match rx.recv() {
            Ok(Message::Open) => {
                // Open the dashboard in a browser
                pi_api.open_dashboard();
            }
            Ok(Message::Quit) => {
                println!("Quit");
                break;
            }
            Ok(Message::Disable10) => {
                // Handle disable call
                println!("Disable!!! 10 seconds");

                // 20 second disable
                match pi_api.disable(10).await {
                    Ok(_) => {}
                    Err(e) => {eprintln!("Error calling disable: {}", e);}
                };
            }
            Ok(Message::Disable30) => {
                // Handle disable call
                println!("Disable!!! 30 seconds");

                // 20 second disable
                match pi_api.disable(30).await {
                    Ok(_) => {}
                    Err(e) => {eprintln!("Error calling disable: {}", e);}
                };
            }
            Ok(Message::Disable5min) => {
                // Handle disable call
                println!("Disable!!! 5 minutes");

                // 20 second disable
                match pi_api.disable(60*5).await {
                    Ok(_) => {}
                    Err(e) => {eprintln!("Error calling disable: {}", e);}
                };
            }
            Ok(Message::Toggle) => {
                // Handle enable call
                println!("Toggle");
                
                // Call the toggle functionality
                toggle_pihole(&pi_api).await;
            }
            _ => {}
        }
    }

    // Wait 50 milliseconds
    sleep(Duration::from_millis(500));
}
