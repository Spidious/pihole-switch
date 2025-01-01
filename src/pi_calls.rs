use pi_hole_api::{AuthenticatedPiHoleAPI, PiHoleAPIConfigWithKey};
use std::collections::HashMap;

pub fn disable(piapi: &PiHoleAPIConfigWithKey, seconds: u64) {
    match piapi.disable(seconds){
        Ok(status) => println!("Disable Success: {:?}", status),
        Err(e) => panic!("Request to disable pihole failed {:?}", e),
    };
}

pub fn enable(piapi: &PiHoleAPIConfigWithKey) {
    match piapi.enable(){
        Ok(status) => println!("Enable Success: {:?}", status),
        Err(e) => panic!("Request to enable pihole failed {:?}", e),
    };
}

pub async fn status() -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let url = format!("{}/admin/api.php?status&auth={}", 
        std::env::var("PI_HOLE_ADDR").expect("PI_HOLE_ADDR must be set").to_string(),
        std::env::var("PI_HOLE_KEY").expect("PI_HOLE_KEY must be set").to_string()
    );

    let resp = reqwest::get(url)
        .await?
        .json::<HashMap<String, String>>()
        .await?;
    
    Ok(resp)
}