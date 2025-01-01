// use std::sync::mpsc;
use dotenv::dotenv;
use pi_hole_api::PiHoleAPIConfigWithKey;
use std::thread::sleep;
use std::time::Duration;
use std::sync::mpsc;
use tray_item::{IconSource, TrayItem};
pub mod pi_calls;

enum Message {
    Open,
    Quit,
    Disable,
    Enable,
}

#[tokio::main]
async fn main() {
    // Get api key from environment variable
    dotenv().ok();
    let pihole_addr = std::env::var("PI_HOLE_ADDR").expect("PI_HOLE_ADDR must be set");
    let pihole_key = std::env::var("PI_HOLE_KEY").expect("PI_HOLE_KEY must be set");

    // Replace the address and key with those of your Pi Hole
    let pihole_api = PiHoleAPIConfigWithKey::new(
        pihole_addr.to_string(),
        pihole_key.to_string(),
    );

    // Setup Tray Item
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
        match pi_calls::status().await {
            Ok(status) => {
                if let Some(rpi_status) = status.get("status") {
                    tray.inner_mut().set_menu_item_label(rpi_status, status_label).unwrap();
                }
            }
            Err(e) => {eprintln!("Error calling status: {}", e);}
        }


        match rx.recv() {
            Ok(Message::Open) => {
                println!("Opening in browser...");
                let addr = pihole_addr.to_string() + "/admin";
                
                match open::that(addr) {
                    Ok(_) => {}
                    Err(e) => {eprintln!("Error in crate Open: {}", e);}
                };
            }
            Ok(Message::Quit) => {
                println!("Quit");
                break;
            }
            Ok(Message::Disable) => {
                // Handle disable call
                println!("Disable!!! 20 seconds");

                // 20 second disable
                pi_calls::disable(&pihole_api, 20);
            }
            Ok(Message::Enable) => {
                // Handle enable call
                println!("Enable");
                
                // Handle enable
                pi_calls::enable(&pihole_api);

            }
            _ => {}
        }
    }

    // Wait 50 milliseconds
    sleep(Duration::from_millis(500));
}
