use pi_hole_api::{AuthenticatedPiHoleAPI, PiHoleAPIConfigWithKey};

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