use std::{thread, time::{Duration, Instant}};

use anyhow::{Ok, Result, anyhow};
use embedded_graphics::{image::{Image, ImageRaw}, mono_font::{MonoTextStyle, MonoTextStyleBuilder, ascii::FONT_10X20}, pixelcolor::{BinaryColor, Rgb565}, prelude::*, primitives::{Polyline, PrimitiveStyle}, text::{Alignment, Baseline, Text, TextStyle, TextStyleBuilder}};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        gpio::{Gpio4, Gpio6, Gpio7, PinDriver},
        i2c::{I2C0, I2cDriver, config::Config as I2cConfig},
        modem::Modem,
        peripherals::Peripherals,
        prelude::Hertz,
    },
    log::EspLogger,
    nvs::EspDefaultNvsPartition,
    wifi::{BlockingWifi, EspWifi},
};
use mini_mac::{market::sync_market::fetch_btc_price, network::{connect_wifi, geo::{GeoInfo, fetch_geo_info}}, weather::{icons, sync_weather::{CurrentWeather, get_weather_icon}}};
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

const CENTER_MIDDLE_TEXT_STYLE: TextStyle = TextStyleBuilder::new()
    .alignment(Alignment::Center)
    .baseline(Baseline::Middle)
    .build();

const CENTER_RIGHT_TEXT_STYLE: TextStyle = TextStyleBuilder::new()
    .alignment(Alignment::Right)
    .baseline(Baseline::Middle)
    .build();

enum Screen {
    Clock,
    Weather,
    Market,
}

impl Screen {
    fn next(self) -> Self {
        match self {
            Screen::Clock => Screen::Weather,
            Screen::Weather => Screen::Market,
            Screen::Market  => Screen::Clock,
        }
    }
}

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

fn draw_clock(
    display: &mut Display,
    h: u8, m: u8, s: u8,
    style: MonoTextStyle<BinaryColor>,
) -> Result<()> {
    display
        .clear(BinaryColor::Off)
        .map_err(|err| anyhow!("Failed to clear display: {err:?}"))?;
    Text::with_text_style(
        &format!("{h:02}:{m:02}:{s:02}"),
        Point::new(64, 32),
        style,  CENTER_MIDDLE_TEXT_STYLE,
    )
    .draw(display)
    .map_err(|err| anyhow!("Failed to draw clock: {err:?}"))?;
    display
        .flush()
        .map_err(|err| anyhow!("Failed to flush display buffer: {err:?}"))?;

    Ok(())
}

fn draw_weather(
    display: &mut Display,
    weather: &CurrentWeather,
    geo: &GeoInfo,
    style: MonoTextStyle<BinaryColor>
) -> Result<()> {
    let raw: ImageRaw<BinaryColor> = ImageRaw::new(get_weather_icon(weather.weathercode), icons::ICON_LARGE_WIDTH);
    let image = Image::new(&raw, Point::new(5, 5));

    display
        .clear(BinaryColor::Off)
        .map_err(|err| anyhow!("Failed to clear display: {err:?}"))?;
    image
        .draw(&mut display.color_converted())
        .map_err(|err| anyhow!("Failed to draw icon bitmap: {err:?}"))?;
    Text::with_text_style(
        &format!("{}", geo.city),
        Point::new(128, 54),
        style, CENTER_RIGHT_TEXT_STYLE
    )
    .draw(display)
    .map_err(|err| anyhow!("Failed to draw city: {err:?}"))?;
    Text::with_text_style(
        &format!("{:.1}°C", weather.temperature_2m),
        Point::new(128, 32),
        style,  CENTER_RIGHT_TEXT_STYLE,
    )
    .draw(display)
    .map_err(|err| anyhow!("Failed to draw temperature: {err:?}"))?;
    display
        .flush()
        .map_err(|err| anyhow!("Failed to flush display buffer: {err:?}"))?;
    Ok(())
}


fn draw_market(
    display: &mut Display,
    history: &[f32],
    style: MonoTextStyle<BinaryColor>,
) -> Result<()> {
    display
        .clear(BinaryColor::Off)
        .map_err(|err| anyhow!("Failed to clear display: {err:?}"))?;

    let current_price = *history.last().unwrap_or(&0.0);
    Text::with_text_style(
        &format!("BTC ${current_price:.0}"),
        Point::new(64, 12),
        style, CENTER_MIDDLE_TEXT_STYLE,
    )
    .draw(display)
    .map_err(|err| anyhow!("Failed to draw BTC price: {err:?}"))?;

    if history.len() >= 2 {
        let min = history.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = history.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let range = (max - min).max(1.0);

        const GRAPH_LEFT: i32 = 4;
        const GRAPH_RIGHT: i32 = 124;
        const GRAPH_TOP: i32 = 26;
        const GRAPH_HEIGHT: i32 = 32;

        let span = (GRAPH_RIGHT - GRAPH_LEFT) as f32 / (history.len() - 1) as f32;
        let points: Vec<Point> = history
            .iter()
            .enumerate()
            .map(|(i, &price)| {
                let x = GRAPH_LEFT + (i as f32 * span) as i32;
                let y = GRAPH_TOP + GRAPH_HEIGHT - ((price - min) / range * GRAPH_HEIGHT as f32) as i32;
                Point::new(x, y)
            })
            .collect();

        Polyline::new(&points)
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(display)
            .map_err(|err| anyhow!("Failed to draw price trend: {err:?}"))?;
    }

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

    let mut touch = PinDriver::input(peripherals.pins.gpio4)?;

    let mut display = init_display(peripherals.i2c0, peripherals.pins.gpio6, peripherals.pins.gpio7)?;
    show_boot_image(&mut display)?;

    let _wifi = init_wifi(peripherals.modem, sys_loop, nvs)?;

    let geo = fetch_geo_info()?;
    log::info!("City: {}, Timezone offset: {}s", geo.city, geo.offset);

    let mut weather = fetch_weather(geo.lat, geo.lon)?;
    log::info!(
        "Weather: {}°C, code {}",
        weather.temperature_2m,
        weather.weathercode
    );

    const MAX_HISTORY: usize = 30;
    const REFRESH_INTERVAL: Duration = Duration::from_secs(5 * 60);
    let mut btc_history: Vec<f32> = vec![fetch_btc_price()?];
    let mut last_fetch = Instant::now();
    log::info!("BTC price: ${:.0}", btc_history[0]);

    sync_time::sync_ntp()?;
    let (mut h, mut m, mut s) = sync_time::get_local_time(geo.offset);
    log::info!("Local time: {h:02}:{m:02}:{s:02}");

    let style =MonoTextStyleBuilder::new()
    .font(&FONT_10X20)
    .text_color(BinaryColor::On)
    .build();

    let mut was_touched = false;

    let mut current_screen = Screen::Clock;

    loop {
        let is_touched = touch.is_high();

        if is_touched && !was_touched {
            current_screen = current_screen.next();
        }
        was_touched = is_touched;
        
        if last_fetch.elapsed() >= REFRESH_INTERVAL {
            if let Result::Ok(price) = fetch_btc_price() {
                btc_history.push(price);
                if btc_history.len() > MAX_HISTORY {
                    btc_history.remove(0);
                }
                log::info!("BTC price: ${price:.0}");
            }
            if let Result::Ok(weather_up) = fetch_weather(geo.lat, geo.lon) {
                weather = weather_up;
                log::info!("Weather: {}°C, code {}", weather.temperature_2m,weather.weathercode);
            }
            last_fetch = Instant::now();
        }

        match current_screen {
            Screen::Clock => {
                draw_clock(&mut display, h, m, s, style)?;
                (h, m, s) = sync_time::get_local_time(geo.offset);
            }
            Screen::Weather => {
                draw_weather(&mut display, &weather, &geo, style)?;
            }
            Screen::Market => {
                draw_market(&mut display, &btc_history, style)?;
            }
        }

        thread::sleep(Duration::from_millis(200));
    }
}
