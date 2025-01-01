use pi_hole_api::{AuthenticatedPiHoleAPI, PiHoleAPIConfigWithKey};
use std::collections::HashMap;

/// Disable the pihole for n seconds
pub fn disable(piapi: &PiHoleAPIConfigWithKey, seconds: u64) {
    match piapi.disable(seconds){
        Ok(status) => println!("Disable Success: {:?}", status),
        Err(e) => panic!("Request to disable pihole failed {:?}", e),
    };
}

/// enable the pihole
pub fn enable(piapi: &PiHoleAPIConfigWithKey) {
    match piapi.enable(){
        Ok(status) => println!("Enable Success: {:?}", status),
        Err(e) => panic!("Request to enable pihole failed {:?}", e),
    };
}

// Retrieve the status of the pihole (enabled or disabled)
pub async fn status(addr: &String, api_key: &String) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    // Format the url (Because the api crate doesn't include this call)
    let url = format!("{}/admin/api.php?status&auth={}", 
        addr,
        api_key
    );

    let resp = reqwest::get(url)
        .await?
        .json::<HashMap<String, String>>()
        .await?;
    
    Ok(resp)
}