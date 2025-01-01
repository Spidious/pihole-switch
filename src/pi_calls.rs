use pi_hole_api::{AuthenticatedPiHoleAPI, PiHoleAPIConfigWithKey};
use std::collections::HashMap;

trait _PiHoleAPIHost {
    fn get_host(&self) -> &str;
}

trait _PiHoleAPIKey {
    fn get_api_key(&self) -> &str;
}

/// Disable the pihole for n seconds
pub async fn disable(addr: &String, api_key: &String, seconds: u64) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    // Format the url
    let url = format!("{}/admin/api.php?disable{}&auth={}",
        addr,
        (if seconds != 0 {format!("={}", seconds)} else {"".to_string()}),
        api_key
    );

    // Call api
    let resp = reqwest::get(url)
        .await?
        .json::<HashMap<String, String>>()
        .await?;
    
    Ok(resp)
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