use crate::network::http::http_get;

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

pub fn get_weather_icon(code: u8) -> &'static str {
    match code {
        0 => "☀️", // Clear sky
        1 | 2 => "🌤️", // Mainly clear, partly cloudy
        3 => "☁️", // Overcast
        45 | 48 => "🌫️", // Fog and depositing rime fog
        51 | 53 | 55 => "🌦️", // Drizzle: Light, moderate, and dense intensity
        56 | 57 => "🌨️", // Freezing Drizzle: Light and dense intensity
        61 | 63 | 65 => "🌧️", // Rain: Slight, moderate and heavy intensity
        66 | 67 => "🧊", // Freezing Rain: Light and heavy intensity
        71 | 73 | 75 => "❄️", // Snow fall: Slight, moderate, and heavy intensity
        77 => "🌨️", // Snow grains
        80 | 81 | 82 => "🌧️", // Rain showers: Slight, moderate, and violent
        85 | 86 => "🌨️", // Snow showers slight and heavy
        95 | 96 | 99 => "⛈️", // Thunderstorm
        _ => "❓", // Unknown weather code
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

