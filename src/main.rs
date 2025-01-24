// Run without terminal if not in debug
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//Begin imports
use dotenv::dotenv;
use std::sync::mpsc;
pub mod piapi_handler;
pub mod tray_handler;

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
                            log_err!(format!("Error trying to disable: {}", e));
                            eprintln!("Error trying to disable: {}", e);
                        }
                    }
                }
                Some("disabled") => {
                    // enable pihole
                    match piapi.enable().await {
                        Ok(_) => {}
                        Err(e) => {
                            log_err!(format!("Issue trying to enable: {}", e));
                            eprintln!("Error trying to enable: {}", e);
                        }
                    }
                }
                Some(other) => {
                    log_err!(format!("Unexpected value in status: {}", other));
                    eprintln!("Unexpected value in status: {}", other);
                }
                None => {
                    log_err!("Key \"status\" not found in status api function");
                    eprintln!("Key \"status\" not found in status api function");
                }
            }

        }
        Err(e) => {
            log_err!(format!("Issue getting status {}", e));
            eprintln!("ERROR getting status {}", e);
        }
    }
}

// #[tokio::main]
fn main() {
    // Prep retrieval of environment variables
    dotenv().ok();

    // Retrieve env variables and create the api handler
    let pi_api = piapi_handler::AuthPiHoleAPI::new(
        std::env::var("PI_HOLE_ADDR").expect("PI_HOLE_ADDR must be set").clone(),
        std::env::var("PI_HOLE_KEY").expect("PI_HOLE_KEY must be set").clone(),
    );

    // if being compiled for non-release, use this TrayIcon
    #[cfg(debug_assertions)]
    let mut pi_tray = tray_handler::TrayIcon::new("Pi-Hole (Non Release)", 255);

    // if being compiled for release, use this TrayIcon
    #[cfg(not(debug_assertions))]
    let mut pi_tray = tray_handler::TrayIcon::new("Pi-Hole", 2); 

    // Add to the tray
    let status_label = pi_tray.tray.inner_mut().add_label_with_id("status_label").unwrap();


    // Setup tx/rx channel
    let (tx, rx) = mpsc::sync_channel(1);
    
    // Add break line
    pi_tray.tray.inner_mut().add_separator().unwrap();

    
    
    // Setup open in browser button
    let open_browser_tx = tx.clone();
    pi_tray.tray.add_menu_item("Open in Browser", move || {
        open_browser_tx.send(Message::Open).unwrap();
    })
    .unwrap();

    // Add break line
    pi_tray.tray.inner_mut().add_separator().unwrap();

    // Setup enable button
    let toggle_tx = tx.clone();
    pi_tray.tray.add_menu_item("Toggle", move || {
        toggle_tx.send(Message::Toggle).unwrap();
    })
    .unwrap();
    
    // Setup disable button
    let disable_tx = tx.clone();
    pi_tray.tray.add_menu_item("Disable 10 Seconds", move || {
        disable_tx.send(Message::Disable10).unwrap();
    })
    .unwrap();

    // Setup disable button
    let disable_tx = tx.clone();
    pi_tray.tray.add_menu_item("Disable 30 Seconds", move || {
        disable_tx.send(Message::Disable30).unwrap();
    })
    .unwrap();

    // Setup disable button
    let disable_tx = tx.clone();
    pi_tray.tray.add_menu_item("Disable 5 minutes", move || {
        disable_tx.send(Message::Disable5min).unwrap();
    })
    .unwrap();

    // Add break line
    pi_tray.tray.inner_mut().add_separator().unwrap();

    // Add quit button (exits the app)
    let quit_tx = tx.clone();
    pi_tray.tray.add_menu_item("Quit", move || {
        quit_tx.send(Message::Quit).unwrap();
    })
    .unwrap();

    log_info!("Setup Complete! Entering mainloop");

    // infinite loop to keep app from dying
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
                    pi_tray.tray.inner_mut().set_menu_item_label("Status: Enabled", status_label).unwrap();
                    pi_tray.show_enabled();
                } else {
                    // Display disabled
                    pi_tray.tray.inner_mut().set_menu_item_label("Status: Disabled", status_label).unwrap();
                    pi_tray.show_disabled();
                }
            },
            Err(count) => {
                if count >= pi_tray.max_fail() {
                    // Display disabled
                    pi_tray.tray.inner_mut().set_menu_item_label("Status: Disabled", status_label).unwrap();
                    pi_tray.show_disabled();
                }
            }
        }

        // Handle the button presses from the system tray
        // Specifically stop here for 50ms because the status above needs to execute
        if let Ok(message) = rx.recv_timeout(std::time::Duration::from_millis(100)) {
            if message == Message::Open {
                // Open dashboard in browser
                pi_api.open_dashboard();
                log_info!("Action Received: Open Dashboard");
            } else if message == Message::Quit {
                // Close the application
                println!("Quit");
                log_info!("Action Received: Quit");
                break;
            } else if message == Message::Disable10 {
                // Disable for 10 seconds
                println!("Disable!!! 10 seconds");
                log_info!("Action Received: Disable 10 Seconds");
                if let Err(e) = block_on!(async {pi_api.disable(10).await}) {
                    log_err!(format!("Action Failed: Disable 10 seconds => {}", e));
                    eprintln!("Error calling disable: {}", e);
                }
            } else if message == Message::Disable30 {
                println!("Disable!!! 30 seconds");
                log_info!("Action Received: Disable 30 Seconds");

                // Disable for 30 seconds
                if let Err(e) = block_on!(async {pi_api.disable(30).await}) {
                    log_err!(format!("Action Failed: Disable 30 seconds => {}", e));
                    eprintln!("Error calling disable: {}", e);
                }
            } else if message == Message::Disable5min {
                println!("Disable!!! 5 minutes");
                log_info!("Action Received: Disable 5 Minutes");

                // Disable for 60 * 5 seconds
                if let Err(e) = block_on!(async {pi_api.disable(60 * 5).await}) {
                    log_err!(format!("Action Failed: Disable 5 minutes => {}", e));
                    eprintln!("Error calling disable: {}", e);
                }
            } else if message == Message::Toggle {
                println!("Toggle");
                log_info!("Action Received: Toggle");
                block_on!(async {toggle_pihole(&pi_api).await});
            }
        }
    }

    log_warn!("Loop exited program ending");
    // // Wait 50 milliseconds to decrease cpu usage while idle
    // sleep(Duration::from_millis(500));
}
