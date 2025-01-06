// Run without terminal if not in debug
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//Begin imports
use dotenv::dotenv;
use std::sync::mpsc;
use tray_item::{IconSource, TrayItem};
pub mod piapi_handler;

// Used for rx/tx of the system tray menu
enum Message {
    Open,
    Quit,
    Disable10,
    Disable30,
    Disable5min,
    Toggle,
}

/// Determine the current state of pihole and toggle it on or off respectively
async fn toggle_pihole(piapi: &piapi_handler::AuthPiHoleAPI) {
    // Start match for the status call
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
        std::env::var("PI_HOLE_ADDR").expect("PI_HOLE_ADDR must be set").clone(),
        std::env::var("PI_HOLE_KEY").expect("PI_HOLE_KEY must be set").clone(),
    );

    // Setup Tray Item
    let mut tray = match TrayItem::new(
        "Pi-Hole", 
        IconSource::Resource("APPICON_DISABLED")) 
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

    // infinite loop to keep app from dying
    loop {
        if let Ok(status) = pi_api.status().await {
            if let Some(rpi_status) = status.get("status") {
                tray.inner_mut().set_menu_item_label(format!("Status: {}", rpi_status).as_str(), status_label).unwrap();
                // Set tray icon status
                if rpi_status == "enabled" {
                    tray.set_icon(IconSource::Resource("APPICON_ENABLED")).unwrap();
                } else {
                    tray.set_icon(IconSource::Resource("APPICON_DISABLED")).unwrap();
                }
            } else {
                tray.set_icon(IconSource::Resource("APPICON_DISABLED")).unwrap();
            }
        } else {
            // Set tray icon status
            tray.set_icon(IconSource::Resource("APPICON_DISABLED")).unwrap();
        }

        // Handle the button presses from the system tray
        // Specifically stop here for 50ms because the status above needs to execute
        match rx.recv_timeout(std::time::Duration::from_millis(100)) {
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

    // Wait 50 milliseconds to decrease cpu usage while idle
    // sleep(Duration::from_millis(500));
}
