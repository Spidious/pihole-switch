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
    Green,
    Red,
}


fn main() {
    // Get api key from environment variable
    dotenv().ok();
    let pihole_addr = std::env::var("PI_HOLE_ADDR").expect("PI_HOLE_ADDR must be set");
    let pihole_key = std::env::var("PI_HOLE_KEY").expect("PI_HOLE_KEY must be set");

    // Replace the address and key with those of your Pi Hole
    let _pihole_api = PiHoleAPIConfigWithKey::new(
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

    let red_tx = tx.clone();
    tray.add_menu_item("Red", move || {
        red_tx.send(Message::Red).unwrap();
    })
    .unwrap();

    let green_tx = tx.clone();
    tray.add_menu_item("Green", move || {
        green_tx.send(Message::Green).unwrap();
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
            Ok(Message::Red) => {
                println!("Red");
                tray.set_icon(IconSource::Resource("another-name-from-rc-file"))
                    .unwrap();
            }
            Ok(Message::Green) => {
                println!("Green");
                tray.set_icon(IconSource::Resource("name-of-icon-in-rc-file"))
                    .unwrap()
            }
            _ => {}
        }
    }
    // sleep(Duration::from_millis(10));
}
