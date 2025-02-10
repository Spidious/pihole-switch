use std::collections::HashMap;

pub struct AuthPiHoleAPI {
    host: String,
    key: String,
}

impl Clone for AuthPiHoleAPI {
    fn clone(&self) -> Self {
        AuthPiHoleAPI {
            host: self.host.clone(), // Clone the field
            key: self.key.clone(),         // Primitive types like i32 implement Clone automatically
        }
    }
}

impl AuthPiHoleAPI {
    /// Create new AuthPiHoleAPI
    pub fn new(host: String, key: String) -> Self {
        Self {host, key}
    }

    /// Open the dashboard in the default browser
    pub fn open_dashboard(&self) {
        // Format address string
        let addr = format!("{}/admin", self.host);

        // Open the address
        match open::that(addr) {
            Ok(_) => {}
            Err(e) => {eprintln!("Error in open: {}", e);}
        }
    }

    /// Disable pihole for n seconds
    pub async fn disable(&self, seconds: u64) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
        // Format the url
        let url = format!("{}/admin/api.php?disable{}&auth={}",
            self.host,
            (if seconds != 0 {format!("={}", seconds)} else {"".to_string()}),
            self.key
        );
    
        // Call api
        let resp = reqwest::get(url)
            .await?
            .json::<HashMap<String, String>>()
            .await?;
        
        // Pass the response back
        Ok(resp)
    }

    /// enable the pihole
    pub async fn enable(&self) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
        // Format the url
        let url = format!("{}/admin/api.php?enable&auth={}",
            self.host,
            self.key
        );

        // Call the api
        let resp = reqwest::get(url)
            .await?
            .json::<HashMap<String, String>>()
            .await?;

        // Pass the response back
        Ok(resp)
    }

    // Retrieve the status of the pihole (enabled or disabled)
    pub async fn status(&self) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
        // Format the url (Because the api crate doesn't include this call)
        let url = format!("{}/admin/api.php?status&auth={}", 
            self.host,
            self.key
        );

        // Call the api
        let resp = reqwest::get(url)
            .await?
            .json::<HashMap<String, String>>()
            .await?;
        
        // Pass the response back
        Ok(resp)
    }
}