use tray_item::{IconSource, TrayItem};
#[cfg(target_os = "linux")]
use crate::*;

pub struct TrayIcon {
    pub tray: TrayItem,
    status: bool,
    fail_count: u8,
    fail_limit: u8,

}


#[cfg(target_os = "linux")]
struct Data {
    height: i32,
    width: i32,
    data: Vec<u8>,
}

// Load image from embedded bytes
#[cfg(target_os = "linux")]
fn load_embedded_image(image: &[u8]) -> Data {
    // Get ImageBuffer
    let img = image::load_from_memory(image)
        .expect("Failed to decode embedded image")
        .to_rgba8();

    // Grab current width and height
    let (mut width, height) = img.dimensions();
    let mut data = img.into_raw(); // Convert to raw RGBA bytes

    // Ensure the image is square
    if width < height {
        let pad = height - width;
        let mut new_data = Vec::with_capacity((height * height * 4) as usize);
        
        for row in data.chunks_exact((width * 4) as usize) {
            new_data.extend_from_slice(row);
            new_data.extend(vec![0; (pad * 4) as usize]); // Pad with transparent pixels
        }

        data = new_data;
        width = height; // Now it's square
    }

    // Apply color shift
    for pixel in data.chunks_exact_mut(4) {
        pixel.rotate_left(3); // Shift RGBA -> GBAR -> BARG -> ARGB
    }

    // Return a Data struct
    Data {
        height: height as i32,
        width: width as i32,
        data,
    }
}

// add updates for these
impl TrayIcon {
    // Constructor function to build and setup the trayIcon for linux
    #[cfg(target_os = "linux")]
    pub fn new(title: &str, fail_limit: u8) -> Self {
        // Create TrayItem
        let image_data = load_embedded_image(BLANK_ICON);

        let tray = TrayItem::new(
            title, 
            IconSource::Data {
                data: image_data.data,
                width: image_data.width,
                height: image_data.height,
            })
            .unwrap();

        // Init tray status
        let status = false;
        // Init tray fail_count
        let fail_count = 0;


        Self {tray, status, fail_count, fail_limit}
    }

    // Constructor function to build and setup the trayIcon for windows
    #[cfg(target_os = "windows")]
    pub fn new(title: &str, fail_limit: u8) -> Self {

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

        if self.fail_count  < self.fail_limit {
            self.fail_count += 1; // Do not want to increment past fail_limit
            Ok(self.fail_count)     // Return the count as Ok
        } else {
            Err(self.fail_count) // Return the count as Err (limit has been reached or exceeded)
        }
    }

    #[cfg(target_os = "linux")]
    pub fn show_enabled(&mut self) {
        // Only do this if the status is not already enabled
        if !self.status {
            // Retrieve image data from embedded image
            let image_data = load_embedded_image(ENABLED_ICON);

            self.tray.set_icon(IconSource::Data {
                data: image_data.data,
                width: image_data.width,
                height: image_data.height,
            }).unwrap(); // set the enabled icon
            self.status = true;
        }
    }

    #[cfg(target_os = "linux")]
    pub fn show_disabled(&mut self) {
        if self.status {
            // Retrieve image data from embedded image
            let image_data = load_embedded_image(DISABLED_ICON);

            self.tray.set_icon(IconSource::Data {
                data: image_data.data,
                width: image_data.width,
                height: image_data.height,
            }).unwrap();
            self.status = false;
        }
    }

    #[cfg(target_os = "windows")]
    pub fn show_enabled(&mut self) {
        // Only do this if the status is not already enabled
        if !self.status {
            self.tray.set_icon(IconSource::Resource("APPICON_ENABLED")).unwrap(); // set the enabled icon
            self.status = true;
        }
    }

    #[cfg(target_os = "windows")]
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

    pub fn max_fail(&self) -> u8 {
        return self.fail_limit;
    }
}
