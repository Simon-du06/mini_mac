use std::{
    thread,
    time::{Duration, Instant},
};

use anyhow::Result;
use embedded_graphics::{
    mono_font::{MonoTextStyleBuilder, ascii::FONT_10X20},
    pixelcolor::BinaryColor,
};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{gpio::PinDriver, modem::Modem, peripherals::Peripherals},
    log::EspLogger,
    nvs::EspDefaultNvsPartition,
    wifi::{BlockingWifi, EspWifi},
};

use mini_mac::{
    glucose::sync_glucose::{GlucoseDatas, fetch_glucose},
    market::{sync_crypto::fetch_btc_price, sync_market::fetch_stock},
    network::{connect_wifi, geo::fetch_geo_info},
    time::sync_time,
    ui::{
        Screen,
        display::{init_display, show_boot_image},
        draw_clock, draw_glucose, draw_market, draw_weather,
    },
    weather::sync_weather::fetch_weather,
};

fn init_wifi(
    modem: Modem,
    sys_loop: EspSystemEventLoop,
    nvs: EspDefaultNvsPartition,
) -> Result<BlockingWifi<EspWifi<'static>>> {
    let mut wifi = BlockingWifi::wrap(EspWifi::new(modem, sys_loop.clone(), Some(nvs))?, sys_loop)?;
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

    let touch = PinDriver::input(peripherals.pins.gpio4)?;

    let mut display = init_display(
        peripherals.i2c0,
        peripherals.pins.gpio6,
        peripherals.pins.gpio7,
    )?;
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
    const SCREEN_CHANGE_INTERVAL: Duration = Duration::from_secs(20);
    let mut btc_history: Vec<f32> = vec![fetch_btc_price()?];
    let mut last_fetch = Instant::now();
    log::info!("BTC price: ${:.0}", btc_history[0]);

    let mut stock_history: Vec<f32> = vec![fetch_stock("QCOM")?];
    log::info!("QCOM price: ${:.0}", stock_history[0]);

    const PROXY_IP: u8 = 34;
    const GLUCOSE_BROADCAST: u16 = 17580;
    let mut glucose_history: Vec<GlucoseDatas> = vec![];
    match fetch_glucose(PROXY_IP, GLUCOSE_BROADCAST) {
        Ok(glucose) if glucose.is_empty() => {
            log::warn!("Juggluco returned no glucose data");
        }
        Ok(glucose) => {
            log::info!("Glucose: {}", glucose[0].sgv);
            glucose_history = glucose;
        }
        Err(error) => {
            log::warn!("Failed to fetch glucose data : {error}");
        }
    }
    const GLUCOSE_REFRESH_INTERVAL: Duration = Duration::from_mins(1);
    let mut glucose_refresh = Instant::now();

    sync_time::sync_ntp()?;
    let (mut h, mut m, mut s) = sync_time::get_local_time(geo.offset);
    log::info!("Local time: {h:02}:{m:02}:{s:02}");

    let style = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(BinaryColor::On)
        .build();

    let mut was_touched = false;

    let mut current_screen = Screen::Clock;

    let mut rotation_clock = Instant::now();

    loop {
        let is_touched = touch.is_high();

        if is_touched && !was_touched {
            current_screen = current_screen.next();
            rotation_clock = Instant::now();
        } else if rotation_clock.elapsed() >= SCREEN_CHANGE_INTERVAL {
            current_screen = current_screen.next();
            rotation_clock = Instant::now();
        }
        was_touched = is_touched;

        if last_fetch.elapsed() >= REFRESH_INTERVAL {
            if let Ok(price) = fetch_btc_price() {
                btc_history.push(price);
                if btc_history.len() > MAX_HISTORY {
                    btc_history.remove(0);
                }
                log::info!("BTC price: ${price:.0}");
            }
            if let Ok(weather_up) = fetch_weather(geo.lat, geo.lon) {
                weather = weather_up;
                log::info!("Weather: {}°C, code {}", weather.temperature_2m,weather.weathercode);
            }
            if let Ok(price) = fetch_stock("QCOM") {
                stock_history.push(price);
                if stock_history.len() > MAX_HISTORY {
                    stock_history.remove(0);
                }
                log::info!("QCOM price: ${price:.0}");
            }
            last_fetch = Instant::now();
        }

        if glucose_refresh.elapsed() >= GLUCOSE_REFRESH_INTERVAL {
            match fetch_glucose(PROXY_IP, GLUCOSE_BROADCAST) {
                Ok(glucose) if glucose.is_empty() => {
                    log::warn!("Juggluco returned no glucose data");
                }
                Ok(glucose) => {
                    log::info!("Glucose: {}", glucose[0].sgv);
                    glucose_history = glucose;
                }
                Err(error) => {
                    log::warn!("Failed to fetch glucose data : {error}");
                }
            }
            glucose_refresh = Instant::now();
        }

        match current_screen {
            Screen::Clock => {
                draw_clock(&mut display, h, m, s, style)?;
                (h, m, s) = sync_time::get_local_time(geo.offset);
            }
            Screen::Weather => {
                draw_weather(&mut display, &weather, &geo, style)?;
            }
            Screen::Crypto => {
                draw_market(&mut display, "BTC", &btc_history, style)?;
            }
            Screen::Market => {
                draw_market(&mut display, "QCOM", &stock_history, style)?;
            }
            Screen::Glucose => {
                draw_glucose(&mut display, &glucose_history, style)?;
            }
        }

        thread::sleep(Duration::from_millis(200));
    }
}
