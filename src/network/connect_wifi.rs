extern crate alloc;
use alloc::string::String;
use esp_radio::wifi::{ClientConfig, ModeConfig, WifiController, WifiError};

pub fn connect_wifi(wifi_controller: &mut WifiController) -> Result<(), WifiError> {
    let config = ClientConfig::default()
        .with_ssid(String::from(env!("WIFI_SSID")))
        .with_password(String::from(env!("WIFI_PASSWORD")));
    
    wifi_controller.set_config(&ModeConfig::Client(config))?;
    wifi_controller.start()?;
    wifi_controller.connect()?;

    esp_println::println!("WiFi connected!");
    Ok(())
}
