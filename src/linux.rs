/*
    Move linux specific items to this file where possible. 
    Keep them separate from linux commands for readability

    Involved items are pi_tray setup and mainloop
 */
use crate::*;
use gtk;
use gtk_sys;

pub fn main(pi_api: &piapi_handler::AuthPiHoleAPI, mut pi_tray:tray_handler::TrayIcon) {

    // Add "Open in Browser" Button
    // Open the pihole dashboard in the default browser
    let pi_api_clone = pi_api.clone();
    pi_tray.tray.add_menu_item("Open in Browser", move || {
        block_on!(async{tray_functions::open_browser(&pi_api_clone).await});
    })
    .unwrap();

    // Add a break in the tray
    pi_tray.tray.inner_mut().add_separator().unwrap();

    // Add the "toggle" Button
    // Toggle the state of pihole
    let pi_api_clone = pi_api.clone();
    pi_tray.tray.add_menu_item("Toggle", move || {
        block_on!(async{tray_functions::toggle_pihole(&pi_api_clone).await});
    })
    .unwrap();

    // Setup disable button
    // Disable pihole 10 seconds
    let pi_api_clone = pi_api.clone();
    pi_tray.tray.add_menu_item("Disable 10 Seconds", move || {
        block_on!(async{tray_functions::disable_sec(&pi_api_clone, 10).await});
    })
    .unwrap();

    // Setup disable button
    // Disable pihole 30 seconds
    let pi_api_clone = pi_api.clone();
    pi_tray.tray.add_menu_item("Disable 30 Seconds", move || {
        block_on!(async{tray_functions::disable_sec(&pi_api_clone, 30).await});
    })
    .unwrap();

    // Setup disable button
    // Disable pihole 5 minutes
    let pi_api_clone = pi_api.clone();
    pi_tray.tray.add_menu_item("Disable 5 minutes", move || {
        block_on!(async{tray_functions::disable_sec(&pi_api_clone, 60*5).await});
    })
    .unwrap();

    // Add break line
    pi_tray.tray.inner_mut().add_separator().unwrap();

    // Add quit button (exits the app)
    pi_tray.tray.add_menu_item("Quit", move || {
        unsafe { gtk_sys::gtk_main_quit(); } // TODO: Recommended method from the docs but should ideally try to find a better method
    })
    .unwrap();

    log_info!("Setup Complete! Entering Mainloop.");
    gtk::main();
}