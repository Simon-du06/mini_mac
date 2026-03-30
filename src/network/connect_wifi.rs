use core::convert::TryInto;

use anyhow::{Context, Result};
use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};
use esp_idf_svc::wifi::{BlockingWifi, EspWifi};

pub fn connect_wifi(wifi: &mut BlockingWifi<EspWifi<'static>>) -> Result<()> {
    let ssid = option_env!("WIFI_SSID").context(
        "WIFI_SSID is not set. Export it before building so the device can join your network.",
    )?;
    let password = option_env!("WIFI_PASSWORD")
        .or(option_env!("WIFI_PASS"))
        .unwrap_or_default();

    let auth_method = if password.is_empty() {
        AuthMethod::None
    } else {
        AuthMethod::WPA2Personal
    };

    let wifi_configuration = Configuration::Client(ClientConfiguration {
        ssid: ssid.try_into().expect("SSID exceeds ESP-IDF max length"),
        password: password
            .try_into()
            .expect("Password exceeds ESP-IDF max length"),
        auth_method,
        ..Default::default()
    });

    wifi.set_configuration(&wifi_configuration)?;
    wifi.start()?;
    log::info!("Wi-Fi started");

    wifi.connect()?;
    log::info!("Wi-Fi connected");

    wifi.wait_netif_up()?;
    log::info!("Wi-Fi netif is up");

    Ok(())
}
