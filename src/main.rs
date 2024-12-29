use dotenv::dotenv;
use pi_hole_api::PiHoleAPIConfigWithKey;
use std::thread::sleep;
use std::time::Duration;
pub mod pi_calls;

fn main() {
    // Get api key from environment variable
    dotenv().ok();
    let pihole_addr = std::env::var("PI_HOLE_ADDR").expect("PI_HOLE_ADDR must be set");
    let pihole_key = std::env::var("PI_HOLE_KEY").expect("PI_HOLE_KEY must be set");

    // Replace the address and key with those of your Pi Hole
    let pihole_api = PiHoleAPIConfigWithKey::new(
        pihole_addr.to_string(),
        pihole_key.to_string(),
    );
    
    loop {
        // Sleep main thread for 10 ms
        sleep(Duration::from_millis(10));
    }
}
