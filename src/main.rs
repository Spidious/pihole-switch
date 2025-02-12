// Run without terminal if not in debug
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// todo: Figure out doing this for linux

//Begin imports
#[cfg(target_os = "linux")]
use gtk::prelude::*;


use dotenv::dotenv;
pub mod tray_functions;
pub mod tray_handler;
pub mod piapi_handler;
pub mod windows;
// pub mod linux;


// // For async handling, just to make it shorter
// macro_rules! block_on {
//     ($expr:expr) => {{
//         tokio::runtime::Runtime::new().unwrap().block_on($expr)
//     }};
// }

// Create general logging message macro
#[macro_export]
macro_rules! log_message {
    ($level:expr, $msg:expr) => {{
        use std::fs::OpenOptions;
        use std::io::Write;
        use chrono::Local;

        // Format timestamp MM-DD-YYYY hh:mm:ss
        let timestamp = Local::now().format("%m-%d-%Y %H:%M:%S").to_string();
        // Format string
        let log_entry = format!("[{}][{}] {} {}\n", $level, timestamp, if ($level == "WARN") {"Warning:"} else {if ($level == "ERROR") {"Error:"} else {""}},$msg);

        // Open or create the log file
        // If it doesn't exist, create it. then, append to it
        let mut file = OpenOptions::new()
            .create(true)  
            .append(true)  
            .open("output.log")
            .expect("Failed to open log file");

        // Write the log entry to the file
        file.write_all(log_entry.as_bytes())
            .expect("Failed to write to log file");
    }};
}

// Define other macros for specific log levels
#[macro_export]
macro_rules! log_info {
    ($msg:expr) => {
        log_message!("INFO", $msg);
    };
}

#[macro_export]
macro_rules! log_warn {
    ($msg:expr) => {
        log_message!("WARN", $msg);
    };
}

#[macro_export]
macro_rules! log_err {
    ($msg:expr) => {
        log_message!("ERROR", $msg);
    };
}

// #[tokio::main]
fn main() {
    // Prep retrieval of environment variables
    dotenv().ok();

    #[cfg(target_os = "linux")]
    gtk::init();

    // Retrieve env variables and create the api handler
    let pi_api = piapi_handler::AuthPiHoleAPI::new(
        std::env::var("PI_HOLE_ADDR").expect("PI_HOLE_ADDR must be set").clone(),
        std::env::var("PI_HOLE_KEY").expect("PI_HOLE_KEY must be set").clone(),
    );

    // if being compiled for non-release, use this TrayIcon
    #[cfg(debug_assertions)]
    let pi_tray = tray_handler::TrayIcon::new("Pi-Hole (Non Release)", 2);

    // if being compiled for release, use this TrayIcon
    #[cfg(not(debug_assertions))]
    let mut pi_tray = tray_handler::TrayIcon::new("Pi-Hole", 2); 
    
    #[cfg(target_os = "linux")]
    {
        let pi_api_clone = pi_api.clone();
        // Setup open in browser button
        pi_tray.tray.add_menu_item("Open in Browser", move || {
            tray_functions::open_browser(&pi_api_clone);
        })
        .unwrap();
        // Add break line
        pi_tray.tray.inner_mut().add_separator().unwrap();
    
        // Setup enable button
        let pi_api_clone = pi_api.clone();
        pi_tray.tray.add_menu_item("Toggle", move || {
            tray_functions::toggle(&pi_api_clone);
        })
        .unwrap();

        // Setup disable button
        let pi_api_clone = pi_api.clone();
        pi_tray.tray.add_menu_item("Disable 10 Seconds", move || {
            tray_functions::disable_sec(&pi_api_clone, 10);
        })
        .unwrap();

        // Setup disable button
        let pi_api_clone = pi_api.clone();
        pi_tray.tray.add_menu_item("Disable 30 Seconds", move || {
            tray_functions::disable_sec(&pi_api_clone, 30);
        })
        .unwrap();

        // Setup disable button
        let pi_api_clone = pi_api.clone();
        pi_tray.tray.add_menu_item("Disable 5 minutes", move || {
            tray_functions::disable_sec(&pi_api_clone, 60*5);
        })
        .unwrap();
    
        // Add break line
        pi_tray.tray.inner_mut().add_separator().unwrap();

        // Add quit button (exits the app)
        pi_tray.tray.add_menu_item("Quit", move || {
            gtk::main_quit();
        })
        .unwrap();
    }

    log_info!("Setup Complete! Entering mainloop");

    // infinite loop to keep app from dying

    #[cfg(target_os = "linux")]  // LINUX mainloop
    gtk::main();

    #[cfg(target_os = "windows")] // WINDOWS mainloop
    windows::main(&pi_api, pi_tray);

    log_warn!("Loop exited program ending");
}
