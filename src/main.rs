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
    Disable,
    Enable,
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

    // Setup open in browser button
    let open_browser_tx = tx.clone();
    tray.add_menu_item("Open in Browser", move || {
        open_browser_tx.send(Message::Open).unwrap();
    })
    .unwrap();

    // Add break line
    tray.inner_mut().add_separator().unwrap();

    // Setup disable button
    let disable_tx = tx.clone();
    tray.add_menu_item("Disable", move || {
        disable_tx.send(Message::Disable).unwrap();
    })
    .unwrap();

    // Setup enable button
    let enable_tx = tx.clone();
    tray.add_menu_item("Enable", move || {
        enable_tx.send(Message::Enable).unwrap();
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
                    tray.inner_mut().set_menu_item_label(rpi_status, status_label).unwrap();
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
            Ok(Message::Disable) => {
                // Handle disable call
                println!("Disable!!! 20 seconds");

                // 20 second disable
                match pi_api.disable(20).await {
                    Ok(_) => {}
                    Err(e) => {eprintln!("Error calling disable: {}", e);}
                };
            }
            Ok(Message::Enable) => {
                // Handle enable call
                println!("Enable");
                
                // Handle enable
                match pi_api.enable().await {
                    Ok(_) => {}
                    Err(e) => {eprintln!("Error calling disable: {}", e);}
                };

            }
            _ => {}
        }
    }

    // Wait 50 milliseconds
    sleep(Duration::from_millis(500));
}
