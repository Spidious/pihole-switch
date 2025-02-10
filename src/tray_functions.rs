use crate::piapi_handler;

// For async handling, just to make it shorter
macro_rules! block_on {
    ($expr:expr) => {{
        tokio::runtime::Runtime::new().unwrap().block_on($expr)
    }};
}

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
                            // log_err!(format!("Error trying to disable: {}", e));
                            eprintln!("Error trying to disable: {}", e);
                        }
                    }
                }
                Some("disabled") => {
                    // enable pihole
                    match piapi.enable().await {
                        Ok(_) => {}
                        Err(e) => {
                            // log_err!(format!("Issue trying to enable: {}", e));
                            eprintln!("Error trying to enable: {}", e);
                        }
                    }
                }
                Some(other) => {
                    // log_err!(format!("Unexpected value in status: {}", other));
                    eprintln!("Unexpected value in status: {}", other);
                }
                None => {
                    // log_err!("Key \"status\" not found in status api function");
                    eprintln!("Key \"status\" not found in status api function");
                }
            }

        }
        Err(e) => {
            // log_err!(format!("Issue getting status {}", e));
            eprintln!("ERROR getting status {}", e);
        }
    }
}

// Open the dashboard and log action
pub async fn open_browser(pi_api: &piapi_handler::AuthPiHoleAPI) {
    // Call action in pi_api
    pi_api.open_dashboard();
    // log_info!("Action Received: Open Dashboard");
}

pub async fn toggle(pi_api: &piapi_handler::AuthPiHoleAPI) {
    block_on!(async {toggle_pihole(&pi_api).await});
}

pub async fn disable_sec(pi_api: &piapi_handler::AuthPiHoleAPI, time: u64) {
    println!("Disable!!! {} seconds", time);
    // log_info!("Action Received: Disable 30 Seconds");

    // Disable for 30 seconds
    if let Err(e) = block_on!(async {pi_api.disable(time).await}) {
        // log_err!(format!("Action Failed: Disable 30 seconds => {}", e));
        eprintln!("Error calling disable: {}", e);
    }
}