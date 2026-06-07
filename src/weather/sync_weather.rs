use crate::network::http::http_get;
use crate::weather::icons::{*};

use anyhow::{Result};

#[derive(serde::Deserialize)]
struct WeatherResponse {
    current: CurrentWeather,
}

#[derive(serde::Deserialize)]
pub struct CurrentWeather {
    pub temperature_2m: f32,
    pub weathercode: u8,
}

pub fn get_weather_icon(code: u8) -> &'static [u8] {
    match code {
        0 => &ICON_SUN_LARGE, // Clear sky
        1 | 2 => &ICON_CLOUD_SUNNY_LARGE, // Mainly clear, partly cloudy
        3 => &ICON_CLOUD_LARGE, // Overcast
        45 | 48 => &ICON_FOG_LARGE, // Fog and depositing rime fog
        51 | 53 | 55 | 61 | 63 | 65 | 80 | 81 | 82 => &ICON_RAIN_LARGE, // Drizzle: Light, moderate, and dense intensity
        56 | 57 | 66 | 67 => &ICON_SLEET_LARGE, // Freezing Rain: Light and heavy intensity
        71 | 73 | 75 | 77 | 85 | 86 => &ICON_SNOW_LARGE, // Snow fall: Slight, moderate, and heavy intensity
        95 | 96 | 99 => &ICON_THUNDERSTORM_LARGE, // Thunderstorm
        _ => &ICON_UNKNOWN_LARGE, // Unknown weather code
    }
}

pub fn fetch_weather(lat:f32, lon:f32) -> Result<CurrentWeather> {
    let url = format!(
        "http://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,weathercode",
        lat, lon);
    let json = http_get(&url);
    let res = serde_json::from_str::<WeatherResponse>(&json?);
    Ok(res?.current)
}

