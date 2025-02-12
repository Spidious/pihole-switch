// Run without terminal if not in debug
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//Begin imports
// #[cfg(target_os = "linux")]
// use gtk::prelude::*;

#[cfg(target_os = "windows")]
use std::sync::mpsc;

use dotenv::dotenv;
pub mod tray_functions;
pub mod tray_handler;
pub mod piapi_handler;
// pub mod linux;
// pub mod windows;


// For async handling, just to make it shorter
macro_rules! block_on {
    ($expr:expr) => {{
        tokio::runtime::Runtime::new().unwrap().block_on($expr)
    }};
}

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

/// Determine the current state of pihole and toggle it on or off respectively
// todo: This is wehre toggle_pihole went

// #[tokio::main]
fn main() {
    // Prep retrieval of environment variables
    dotenv().ok();

    // #[cfg(target_os = "linux")]
    // gtk::init();

    // Retrieve env variables and create the api handler
    let pi_api = piapi_handler::AuthPiHoleAPI::new(
        std::env::var("PI_HOLE_ADDR").expect("PI_HOLE_ADDR must be set").clone(),
        std::env::var("PI_HOLE_KEY").expect("PI_HOLE_KEY must be set").clone(),
    );

    // if being compiled for non-release, use this TrayIcon
    #[cfg(debug_assertions)]
    let mut pi_tray = tray_handler::TrayIcon::new("Pi-Hole (Non Release)", 2);

    // if being compiled for release, use this TrayIcon
    #[cfg(not(debug_assertions))]
    let mut pi_tray = tray_handler::TrayIcon::new("Pi-Hole", 2); 


    // Setup tx/rx channel
    #[cfg(target_os = "windows")]
    let (tx, rx) = mpsc::sync_channel(1);
    
    // Add break line
    pi_tray.tray.inner_mut().add_separator().unwrap();

    
    // Setup open in browser button
    #[cfg(target_os = "windows")]
    let open_browser_tx = tx.clone();
    let pi_api_clone = pi_api.clone();
    pi_tray.tray.add_menu_item("Open in Browser", move || {
        #[cfg(target_os = "windows")]
        open_browser_tx.send(Message::Open).unwrap();

        #[cfg(target_os = "linux")]
        tray_functions::open_browser(&pi_api_clone);
    })
    .unwrap();

    // Add break line
    pi_tray.tray.inner_mut().add_separator().unwrap();

    // Setup enable button
    #[cfg(target_os = "windows")]
    let toggle_tx = tx.clone();
    let pi_api_clone = pi_api.clone();
    pi_tray.tray.add_menu_item("Toggle", move || {
        #[cfg(target_os = "windows")]
        toggle_tx.send(Message::Toggle).unwrap();

        #[cfg(target_os = "linux")]
        tray_functions::toggle(&pi_api_clone);
    })
    .unwrap();
    
    // Setup disable button
    #[cfg(target_os = "windows")]
    let disable_tx = tx.clone();
    let pi_api_clone = pi_api.clone();
    pi_tray.tray.add_menu_item("Disable 10 Seconds", move || {
        #[cfg(target_os = "windows")]
        disable_tx.send(Message::Disable10).unwrap();

        #[cfg(target_os = "linux")]
        tray_functions::disable_sec(&pi_api_clone, 10);
    })
    .unwrap();

    // Setup disable button
    #[cfg(target_os = "windows")]
    let disable_tx = tx.clone();
    let pi_api_clone = pi_api.clone();
    pi_tray.tray.add_menu_item("Disable 30 Seconds", move || {
        #[cfg(target_os = "windows")]
        disable_tx.send(Message::Disable30).unwrap();

        #[cfg(target_os = "linux")]
        tray_functions::disable_sec(&pi_api_clone, 30);
    })
    .unwrap();

    // Setup disable button
    #[cfg(target_os = "windows")]
    let disable_tx = tx.clone();
    let pi_api_clone = pi_api.clone();
    pi_tray.tray.add_menu_item("Disable 5 minutes", move || {
        #[cfg(target_os = "windows")]
        disable_tx.send(Message::Disable5min).unwrap();

        #[cfg(target_os = "linux")]
        tray_functions::disable_sec(&pi_api_clone, 60*5);
    })
    .unwrap();

    // Add break line
    pi_tray.tray.inner_mut().add_separator().unwrap();

    // Add quit button (exits the app)
    #[cfg(target_os = "windows")]
    let quit_tx = tx.clone();
    pi_tray.tray.add_menu_item("Quit", move || {
        #[cfg(target_os = "windows")]
        quit_tx.send(Message::Quit).unwrap();

        // #[cfg(target_os = "linux")]
        // gtk::main_quit();
    })
    .unwrap();

    log_info!("Setup Complete! Entering mainloop");

    // infinite loop to keep app from dying

    // #[cfg(target_os = "linux")] 
    // gtk::main();

    #[cfg(target_os = "windows")]
    loop {
        match pi_tray.test(|| {
            // Use block_on to call the async function in a synchronous context
            block_on!(async {
                pi_api.status().await  // Call the async function and await its result
            })
        }) {
            Ok(response) => {
                // Parse the output of the api call
                let status = response.get("status").unwrap();

                // check enabled or disabled
                if status == "enabled" {
                    // Display enabled
                    pi_tray.show_enabled();
                } else {
                    // Display disabled
                    pi_tray.show_disabled();
                }
            },
            Err(count) => {
                if count >= pi_tray.max_fail() {
                    // Display disabled
                    pi_tray.show_disabled();
                }
            }
        }

        // Handle the button presses from the system tray
        // Specifically stop here for 50ms because the status above needs to execute
        if let Ok(message) = rx.recv_timeout(std::time::Duration::from_millis(100)) {
            if message == Message::Open {
                // Open dashboard in browser
                tray_functions::open_browser(&pi_api);
            } else if message == Message::Quit {
                // Close the application
                println!("Quit");
                log_info!("Action Received: Quit");
                break;
            } else if message == Message::Disable10 {
                tray_functions::disable_sec(&pi_api, 10);
            } else if message == Message::Disable30 {
                tray_functions::disable_sec(&pi_api, 30);
            } else if message == Message::Disable5min {
                tray_functions::disable_sec(&pi_api, 60*5);
            } else if message == Message::Toggle {
                println!("Toggle");
                log_info!("Action Received: Toggle");
                block_on!(async {tray_functions::toggle_pihole(&pi_api).await});
            }
        }
    }

    log_warn!("Loop exited program ending");
    // // Wait 50 milliseconds to decrease cpu usage while idle
    // sleep(Duration::from_millis(500));
}
