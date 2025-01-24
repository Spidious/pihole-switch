use tray_item::{IconSource, TrayItem};

pub struct TrayIcon {
    pub tray: TrayItem,
    status: bool,
    fail_count: u8,
    fail_limit: u8,

}

// add updates for these
impl TrayIcon {
    // Constructor function to build and setup the trayIcon
    pub fn new(title: &str, fail_limit: u8) -> Self {
        // Create TrayItem
        let tray = TrayItem::new(
            title, 
            IconSource::Resource("APPICON_DISABLED"))
            .unwrap();

        // Init tray status
        let status = false;
        // Init tray fail_count
        let fail_count = 0;


        Self {tray, status, fail_count, fail_limit}
    }
    
    // handler to reset fail_count
    pub fn pass(&mut self) {
        if self.fail_count != 0 {
            self.fail_count = 0;
        }
    }

    // handler to increment fail_count
    pub fn fail(&mut self) -> Result<u8, u8>{
        self.fail_count += 1; // increment the count (this will cap at 255 due to the type u8)

        if self.fail_count < self.fail_limit {
            Ok(self.fail_count)     // Return the count as Ok
        } else {
            Err(self.fail_count) // Return the count as Err (limit has been reached or exceeded)
        }
    }


    pub fn show_enabled(&mut self) {
        // Only do this if the status is not already enabled
        if !self.status {
            self.tray.set_icon(IconSource::Resource("APPICON_ENABLED")).unwrap(); // set the enabled icon
            self.status = true;
        }
    }

    pub fn show_disabled(&mut self) {
        if self.status {
            self.tray.set_icon(IconSource::Resource("APPICON_DISABLED")).unwrap();
            self.status = false;
        }
    }

    // get the status variable value
    pub fn is_enabled(&mut self) -> bool{
        return self.status;
    }

    // Handle pass/fail on a given Result<> function
    pub fn test<F, T, U>(&mut self, func: F) -> Result<T, u8>
    where
        F: Fn() -> Result<T, U>,   // The closure should return a value of type T if successful. Not concerned with Err
    {
        // call func. If Ok, mark as pass and return the output T
        if let Ok(value) = func() {
            self.pass();
            return Ok(value);
        }

        // test function output at this point is always Err. map the Ok and Err from fail marker to Err
        match self.fail() {
            Ok(count) => Err(count),
            Err(count) => Err(count)
        }
    }
}
