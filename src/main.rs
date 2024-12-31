// use std::sync::mpsc;
use dotenv::dotenv;
use pi_hole_api::PiHoleAPIConfigWithKey;
// use std::thread::sleep;
// use std::time::Duration;
use std::sync::mpsc;
use tray_item::{IconSource, TrayItem};
pub mod pi_calls;

enum Message {
    Quit,
    Disable,
    Enable,
}

fn main() {
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
        "Tray Example", 
        IconSource::Resource("APPICON")) 
        {
        Ok(tray) => tray,
        Err(e) => {
            eprintln!("Failed to create tray item: {}", e);
            return;
        },
    };


    // Add to the tray
    tray.add_label("Tray Label").unwrap();
    tray.add_menu_item("Hello", || {
        println!("Hello!");
    })
    .unwrap();
    tray.inner_mut().add_separator().unwrap();

    let (tx, rx) = mpsc::sync_channel(1);

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

    tray.inner_mut().add_separator().unwrap();

    let quit_tx = tx.clone();
    tray.add_menu_item("Quit", move || {
        quit_tx.send(Message::Quit).unwrap();
    })
    .unwrap();

    loop {
        match rx.recv() {
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
    // sleep(Duration::from_millis(10));
}
