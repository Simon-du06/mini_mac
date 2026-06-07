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

pub fn fetch_weather(lat:f32, lon:f32) -> Result<CurrentWeather> {
    let url = format!(
        "http://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,weathercode",
        lat, lon);
    let json = http_get(&url);
    let res = serde_json::from_str::<WeatherResponse>(&json?);
    Ok(res?.current)
}

