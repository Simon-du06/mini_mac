use std::{thread, time::Duration};

use anyhow::{anyhow, Result};
use embedded_graphics::{image::Image, mono_font::{MonoTextStyleBuilder, ascii::FONT_10X20}, pixelcolor::{BinaryColor, Rgb565}, prelude::*, text::Text};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        gpio::{Gpio6, Gpio7},
        i2c::{config::Config as I2cConfig, I2cDriver, I2C0},
        modem::Modem,
        peripherals::Peripherals,
        prelude::Hertz,
    },
    log::EspLogger,
    nvs::EspDefaultNvsPartition,
    wifi::{BlockingWifi, EspWifi},
};
use mini_mac::network::{connect_wifi, geo::fetch_geo_info};
use mini_mac::time::sync_time;
use mini_mac::weather::sync_weather::fetch_weather;
use ssd1306::{
    mode::{BufferedGraphicsMode, DisplayConfig},
    prelude::{DisplaySize128x64, DisplayRotation, I2CInterface},
    I2CDisplayInterface, Ssd1306,
};
use tinybmp::Bmp;

/// Alias pour le type concret de l'écran : sans lui, chaque signature de
/// fonction qui manipule l'écran devrait répéter ce type générique complet.
type Display = Ssd1306<
    I2CInterface<I2cDriver<'static>>,
    DisplaySize128x64,
    BufferedGraphicsMode<DisplaySize128x64>,
>;

fn init_display(i2c0: I2C0, sda: Gpio6, scl: Gpio7) -> Result<Display> {
    let i2c = I2cDriver::new(i2c0, sda, scl, &I2cConfig::new().baudrate(Hertz(400_000)))?;

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display
        .init()
        .map_err(|err| anyhow!("Failed to initialize display: {err:?}"))?;

    Ok(display)
}

fn show_boot_image(display: &mut Display) -> Result<()> {
    let bmp = Bmp::from_slice(include_bytes!("../asset/images/hello2.bmp"))
        .map_err(|err| anyhow!("Failed to parse boot bitmap: {err:?}"))?;
    let image: Image<Bmp<Rgb565>> = Image::new(&bmp, Point::new(0, 0));

    image
        .draw(&mut display.color_converted())
        .map_err(|err| anyhow!("Failed to draw boot bitmap: {err:?}"))?;
    display
        .flush()
        .map_err(|err| anyhow!("Failed to flush display buffer: {err:?}"))?;

    Ok(())
}

fn init_wifi(
    modem: Modem,
    sys_loop: EspSystemEventLoop,
    nvs: EspDefaultNvsPartition,
) -> Result<BlockingWifi<EspWifi<'static>>> {
    let mut wifi =
        BlockingWifi::wrap(EspWifi::new(modem, sys_loop.clone(), Some(nvs))?, sys_loop)?;
    connect_wifi::connect_wifi(&mut wifi)?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    log::info!("Wi-Fi DHCP info: {ip_info:?}");

    Ok(wifi)
}

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    EspLogger::initialize_default();

    log::info!("Starting main");

    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut display = init_display(peripherals.i2c0, peripherals.pins.gpio6, peripherals.pins.gpio7)?;
    show_boot_image(&mut display)?;

    let _wifi = init_wifi(peripherals.modem, sys_loop, nvs)?;

    let geo = fetch_geo_info()?;
    log::info!("City: {}, Timezone offset: {}s", geo.city, geo.offset);

    let weather = fetch_weather(geo.lat, geo.lon)?;
    log::info!(
        "Weather: {}°C, code {}",
        weather.temperature_2m,
        weather.weathercode
    );

    sync_time::sync_ntp()?;
    let (mut h, mut m, mut s) = sync_time::get_local_time(geo.offset);
    log::info!("Local time: {h:02}:{m:02}:{s:02}");

    let style =MonoTextStyleBuilder::new()
    .font(&FONT_10X20)
    .text_color(BinaryColor::On)   // pour un écran monochrome SSD1306
    .build();

    loop {
        display.clear(BinaryColor::Off);
        Text::new(
            &format!("{h:02}:{m:02}:{s:02}"), 
            Point::new(24, 32), style).draw(&mut display);
        display.flush();
        (h, m, s) = sync_time::get_local_time(geo.offset);
        thread::sleep(Duration::from_secs(1));
    }
}
