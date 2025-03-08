/*
    Move windows specific items to this file where possible. 
    Keep them separate from linux commands for readability

    Involved items are pi_tray setup and mainloop
 */
use crate::*;
use std::sync::mpsc;

// Used for rx/tx of the system tray menu
#[derive(PartialEq)]
pub enum Message {
    Open,
    Quit,
    Disable10,
    Disable30,
    Disable5min,
    Toggle,
}

/// Mainloop function for windows
/// pi_api - Pihole API handler
/// pi_tray - tray handler
pub fn main(pi_api: piapi_handler::AuthPiHoleAPI, mut pi_tray:tray_handler::TrayIcon) {
    // Setup tx/rx channel
    let (tx, rx) = mpsc::sync_channel(1);

    // Add "Open in Browser" Button
    // Open the pihole dashboard in the default browser
    let open_browser_tx = tx.clone();
    pi_tray.tray.add_menu_item("Open in Browser", move || {
        open_browser_tx.send(Message::Open).unwrap();
    })
    .unwrap();

    // Add a break in the tray
    pi_tray.tray.inner_mut().add_separator().unwrap();

    // Add the "toggle" Button
    // Toggle the state of pihole
    let toggle_tx = tx.clone();
    pi_tray.tray.add_menu_item("Toggle", move || {
        toggle_tx.send(Message::Toggle).unwrap();
    })
    .unwrap();

    // Setup disable button
    // Disable pihole 10 seconds
    let disable_tx = tx.clone();
    pi_tray.tray.add_menu_item("Disable 10 Seconds", move || {
        disable_tx.send(Message::Disable10).unwrap();
    })
    .unwrap();

    // Setup disable button
    // Disable pihole 30 seconds
    let disable_tx = tx.clone();
    pi_tray.tray.add_menu_item("Disable 30 Seconds", move || {
        disable_tx.send(Message::Disable30).unwrap();
    })
    .unwrap();

    // Setup disable button
    // Disable pihole 5 minutes
    let disable_tx = tx.clone();
    pi_tray.tray.add_menu_item("Disable 5 minutes", move || {
        disable_tx.send(Message::Disable5min).unwrap();
    })
    .unwrap();

    // Add a break in the tray
    pi_tray.tray.inner_mut().add_separator().unwrap();

    // Add quit button (exits the app)
    let quit_tx = tx.clone();
    pi_tray.tray.add_menu_item("Quit", move || {
        quit_tx.send(Message::Quit).unwrap();
    })
    .unwrap();
    
    // Enter mainloop to keep app from dying
    loop {
        pi_tray.update_status_icon(&pi_api);
    
        // Handle the button presses from the system tray
        // Specifically stop here for 50ms because the status above needs to execute
        if let Ok(message) = rx.recv_timeout(std::time::Duration::from_millis(100)) {
            if message == Message::Open {
                // Open dashboard in browser
                block_on!(async {tray_functions::open_browser(&pi_api).await});
            } else if message == Message::Quit {
                // Close the application
                println!("Quit");
                log_info!("Action Received: Quit");
                break;
            } else if message == Message::Disable10 {
                block_on!(async {tray_functions::disable_sec(&pi_api, 10).await});
            } else if message == Message::Disable30 {
                block_on!(async {tray_functions::disable_sec(&pi_api, 30).await});
            } else if message == Message::Disable5min {
                block_on!(async {tray_functions::disable_sec(&pi_api, 60*5).await});
            } else if message == Message::Toggle {
                println!("Toggle");
                log_info!("Action Received: Toggle");
                block_on!(async {tray_functions::toggle_pihole(&pi_api).await});
            }
        }
    }
}
