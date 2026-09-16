use std::{thread, time::{Duration, Instant}};

use anyhow::{Result, anyhow};
use embedded_graphics::{image::{Image, ImageRaw}, mono_font::{MonoTextStyle, MonoTextStyleBuilder, ascii::FONT_10X20}, pixelcolor::{BinaryColor, Rgb565}, prelude::*, primitives::{Polyline, PrimitiveStyle}, text::{Alignment, Baseline, Text, TextStyle, TextStyleBuilder}};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        gpio::{Gpio6, Gpio7, PinDriver},
        i2c::{I2C0, I2cDriver, config::Config as I2cConfig},
        modem::Modem,
        peripherals::Peripherals,
        prelude::Hertz,
    },
    log::EspLogger,
    nvs::EspDefaultNvsPartition,
    wifi::{BlockingWifi, EspWifi},
};
use mini_mac::time::sync_time;
use ssd1306::{
    mode::{BufferedGraphicsMode, DisplayConfig},
    prelude::{DisplaySize128x64, DisplayRotation, I2CInterface},
    I2CDisplayInterface, Ssd1306,
};
use tinybmp::Bmp;

use mini_mac::ui::{
    draw_clock,
    draw_glucose,
    draw_market,
    draw_weather,
    display::{init_display, show_boot_image},
    Screen,
};

fn draw_glucose(
    display: &mut Display,
    history: &[GlucoseDatas],
    style: MonoTextStyle<BinaryColor>
) -> Result<()> {
    display
        .clear(BinaryColor::Off)
        .map_err(|err| anyhow!("Failed to clear display: {err:?}"))?;

    let now = SystemTime::now();
    let duration = now.duration_since(UNIX_EPOCH)?;
    let now_ms = duration.as_millis();

    let current_sugar = history.first();
    let message = match current_sugar {
        None => "NO DATA".to_string(),
        Some(sugar) if sugar.age_minutes(now_ms) >= 10 => {
            format!("{} OLD", sugar.sgv)
        }
        Some(sugar) => format!("{} mg/dL", sugar.sgv),
    };

    Text::with_text_style(
        &message,
        Point::new(64, 12),
        style, CENTER_MIDDLE_TEXT_STYLE,
    )
    .draw(display)
    .map_err(|err| anyhow!("Failed to draw glucose: {err:?}"))?;

    let glucose_values: Vec<f32> = history
        .iter()
        .rev()
        .map(|x| x.sgv as f32)
        .collect();

    if glucose_values.len() > 3 {
        let min = glucose_values.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = glucose_values.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let range = (max - min).max(1.0);

        const GRAPH_LEFT: i32 = 4;
        const GRAPH_RIGHT: i32 = 124;
        const GRAPH_TOP: i32 = 26;
        const GRAPH_HEIGHT: i32 = 32;

        let span = (GRAPH_RIGHT - GRAPH_LEFT) as f32 / (history.len() - 1) as f32;
        let points: Vec<Point> = glucose_values
            .iter()
            .enumerate()
            .map(|(i, &glucose)| {
                let x = GRAPH_LEFT + (i as f32 * span) as i32;
                let y = GRAPH_TOP + GRAPH_HEIGHT - ((glucose - min) / range * GRAPH_HEIGHT as f32) as i32;
                Point::new(x, y)
            })
            .collect();

        Polyline::new(&points)
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(display)
            .map_err(|err| anyhow!("Failed to draw glucose trend: {err:?}"))?;
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

    let touch = PinDriver::input(peripherals.pins.gpio4)?;

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
        Result::Ok(glucose) if glucose.is_empty() => {
            log::warn!("Juggluco returned no glucose data");
        }
        Result::Ok(glucose) => {
            log::info!("Glucose: {}", glucose[0].sgv);
            glucose_history = glucose;
        }
        Result::Err(error) => {
            log::warn!("Failed to fetch glucose data : {error}");
        }
    }
    const GLUCOSE_REFRESH_INTERVAL: Duration = Duration::from_mins(1);
    let mut glucose_refresh = Instant::now();

    sync_time::sync_ntp()?;
    let (mut h, mut m, mut s) = sync_time::get_local_time(geo.offset);
    log::info!("Local time: {h:02}:{m:02}:{s:02}");

    let style =MonoTextStyleBuilder::new()
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
            if let Result::Ok(price) = fetch_stock("QCOM") {
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
                Result::Ok(glucose) if glucose.is_empty() => {
                    log::warn!("Juggluco returned no glucose data");
                }
                Result::Ok(glucose) => {
                    log::info!("Glucose: {}", glucose[0].sgv);
                    glucose_history = glucose;
                } 
                Result::Err(error) => {
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
