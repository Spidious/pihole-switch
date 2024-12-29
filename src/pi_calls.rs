use pi_hole_api::{AuthenticatedPiHoleAPI, PiHoleAPIConfigWithKey};

pub async fn disable(piapi: PiHoleAPIConfigWithKey) {
    match piapi.disable(10){
        Ok(status) => println!("Disable Success: {:?}", status),
        Err(e) => panic!("Request to disable pihole failed {:?}", e),
    };
}