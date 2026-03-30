use std::{thread, time::Duration};

use anyhow::{anyhow, Result};
use embedded_graphics::{image::Image, pixelcolor::Rgb565, prelude::*};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        i2c::{config::Config as I2cConfig, I2cDriver},
        peripherals::Peripherals,
        prelude::Hertz,
    },
    log::EspLogger,
    nvs::EspDefaultNvsPartition,
    wifi::{BlockingWifi, EspWifi},
};
use mini_mac::network::connect_wifi;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use tinybmp::Bmp;

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    EspLogger::initialize_default();

    log::info!("Starting main");

    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let modem = peripherals.modem;
    let i2c0 = peripherals.i2c0;
    let pins = peripherals.pins;

    let i2c = I2cDriver::new(
        i2c0,
        pins.gpio6,
        pins.gpio7,
        &I2cConfig::new().baudrate(Hertz(400_000)),
    )?;

    let interface = I2CDisplayInterface::new(i2c);
    let mut display =
        Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();
    display
        .init()
        .map_err(|err| anyhow!("Failed to initialize display: {err:?}"))?;

    let bmp = Bmp::from_slice(include_bytes!("../asset/images/hello2.bmp"))
        .map_err(|err| anyhow!("Failed to parse boot bitmap: {err:?}"))?;
    let image: Image<Bmp<Rgb565>> = Image::new(&bmp, Point::new(0, 0));

    image
        .draw(&mut display.color_converted())
        .map_err(|err| anyhow!("Failed to draw boot bitmap: {err:?}"))?;
    display
        .flush()
        .map_err(|err| anyhow!("Failed to flush display buffer: {err:?}"))?;

    let mut wifi =
        BlockingWifi::wrap(EspWifi::new(modem, sys_loop.clone(), Some(nvs))?, sys_loop)?;
    connect_wifi::connect_wifi(&mut wifi)?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    log::info!("Wi-Fi DHCP info: {ip_info:?}");

    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
