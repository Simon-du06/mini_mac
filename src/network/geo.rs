use crate::network::http::http_get;

use anyhow::{Result};

#[derive(serde::Deserialize)]
pub struct GeoInfo {
    pub timezone: String,
    pub offset: i32,
    pub lat: f32,
    pub lon: f32,
    pub city: String,
    pub country: String
}

pub fn fetch_geo_info() -> Result<GeoInfo> {
    let json = http_get("http://ip-api.com/json?fields=status,city,country,timezone,offset,lat,lon");
    
    let res = serde_json::from_str::<GeoInfo>(&json?);
    Ok(res?)
}