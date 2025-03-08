// Run without terminal if not in debug
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// todo: Figure out doing this for linux


use dotenv::from_path;
pub mod tray_functions;
pub mod tray_handler;
pub mod piapi_handler;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "linux")]
mod icons {
    pub const BLANK_ICON: &[u8] = include_bytes!("../resources/Pi-hole_blank.ico");
    pub const DISABLED_ICON: &[u8] = include_bytes!("../resources/Pi-hole_disabled.ico");
    pub const ENABLED_ICON: &[u8] = include_bytes!("../resources/Pi-hole_enabled.ico");
}

#[cfg(target_os = "linux")]
use icons::*;

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

fn get_cargo_root() -> Option<std::path::PathBuf> {
    let mut exe_path = std::env::current_exe().ok()?;

    // Go up the directory tree from the executable's location
    while !exe_path.join(".env").exists() {
        if !exe_path.pop() {
            return None; // reached the root without finding .env
        }
    }

    // Return the path with .env appended
    Some(exe_path.join(".env"))
}

// #[tokio::main]
fn main() {

    // Store the result in a variable to extend the lifetime
    let cargo_root = get_cargo_root()
        .expect("Could not find env");

    // Now use it to create the env_path
    let env_path = std::path::Path::new(
        cargo_root.to_str()
            .expect("Path contains invalid Unicode")
    );

    from_path(env_path).expect("Failed to load .env file");
    // Prep retrieval of environment variables
    // dotenv().ok();

    // Retrieve env variables and create the api handler
    let pi_api = piapi_handler::AuthPiHoleAPI::new(
        std::env::var("PI_HOLE_ADDR").expect("PI_HOLE_ADDR must be set").clone(),
        std::env::var("PI_HOLE_KEY").expect("PI_HOLE_KEY must be set").clone(),
    );

    // If unable to initialize GTK then the app cannot run anyway. Submit log and quit
    // Must do this before pi_tray is created as it will cause rust to panic
    #[cfg(target_os = "linux")]
    if let Err(e) = gtk::init() {
        log_err!(format!("{}", e));
        return;
    }

    // if being compiled for non-release, use this TrayIcon
    #[cfg(debug_assertions)]
    let pi_tray = tray_handler::TrayIcon::new("Pi-Hole (Non Release)", 2);

    // if being compiled for release, use this TrayIcon
    #[cfg(not(debug_assertions))]
    let pi_tray = tray_handler::TrayIcon::new("Pi-Hole", 2); 
    
    // infinite loop to keep app from dying
    #[cfg(target_os = "linux")]  // LINUX mainloop
    linux::main(pi_api, pi_tray);
    

    #[cfg(target_os = "windows")] // WINDOWS mainloop
    windows::main(pi_api, pi_tray);

    log_warn!("Loop exited program ending");
}
