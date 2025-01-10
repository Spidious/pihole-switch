// Run without terminal if not in debug
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//Begin imports
use dotenv::dotenv;
use std::sync::mpsc;
use tray_item::{IconSource, TrayItem};
pub mod piapi_handler;

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
            log_err!(format!("Failed to create tray item: {}", e));
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

    log_info!("Setup Complete! Entering mainloop");

    let mut connection_status: bool = false;

    // infinite loop to keep app from dying
    loop {
        if let Ok(status) = pi_api.status().await {
            if let Some(rpi_status) = status.get("status") {
                tray.inner_mut().set_menu_item_label(format!("Status: {}", rpi_status).as_str(), status_label).unwrap();
                // Set tray icon status
                if rpi_status == "enabled" && !connection_status { // If status is enabled and connection_status shows disabled
                    // Attempt to display enabled
                    if let Err(e) = tray.set_icon(IconSource::Resource("APPICON_ENABLED")) {
                        log_err!(format!("Could not set resource for APPICON_ENABLED: {}", e));
                        println!("Just allowing for a timestamp to be logged");
                    } else {
                        connection_status = true; // mark as enabled
                    }
                    
                } else if rpi_status == "disabled" && connection_status{ // if status is disabled and connection_status shows enabled
                    // Attempt to display disabled
                    if let Err(e) = tray.set_icon(IconSource::Resource("APPICON_DISABLED")) {
                        log_err!(format!("Could not set resource APPICON_DISABLED: {}", e));
                        println!("Just allowing for a timestamp to be logged");
                    } else {
                        connection_status = false;
                    }
                }
            } else {
                log_warn!("'status' was not returned by the status API call");
                tray.set_icon(IconSource::Resource("APPICON_DISABLED")).unwrap();
                connection_status = false;
            }
        } else {
            // Set tray icon status
            // This should run if a call to the api does not return (Pihole is not reachable for any reason)
            tray.set_icon(IconSource::Resource("APPICON_DISABLED")).unwrap();
            connection_status = false;
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
                if let Err(e) = pi_api.disable(10).await {
                    log_err!(format!("Action Failed: Disable 10 seconds => {}", e));
                    eprintln!("Error calling disable: {}", e);
                }
            } else if message == Message::Disable30 {
                println!("Disable!!! 30 seconds");
                log_info!("Action Received: Disable 30 Seconds");

                // Disable for 30 seconds
                if let Err(e) = pi_api.disable(30).await {
                    log_err!(format!("Action Failed: Disable 30 seconds => {}", e));
                    eprintln!("Error calling disable: {}", e);
                }
            } else if message == Message::Disable5min {
                println!("Disable!!! 5 minutes");
                log_info!("Action Received: Disable 5 Minutes");

                // Disable for 60 * 5 seconds
                if let Err(e) = pi_api.disable(60 * 5).await {
                    log_err!(format!("Action Failed: Disable 5 minutes => {}", e));
                    eprintln!("Error calling disable: {}", e);
                }
            } else if message == Message::Toggle {
                println!("Toggle");
                log_info!("Action Received: Toggle");
                toggle_pihole(&pi_api).await;
            }
        }
    }

    log_warn!("Loop exited program ending");
    // Wait 50 milliseconds to decrease cpu usage while idle
    // sleep(Duration::from_millis(500));
}
