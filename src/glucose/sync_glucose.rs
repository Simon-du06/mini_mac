use crate::network::http::http_get;

use anyhow::Result;

#[derive(serde::Deserialize)]
pub struct GlucoseDatas {
    pub date: u64,
    pub sgv: u16,
    pub delta: Option<f32>,
    pub direction: String,
    pub noise: Option<i8>,
    pub units_hint: Option<String>,
}

impl GlucoseDatas {
    pub fn age_minutes(&self, now_ms: u128) -> u128 {
        now_ms.saturating_sub(u128::from(self.date)) / 60_000
    }
}

pub fn fetch_glucose(ip: u8, port: u16) -> Result<Vec<GlucoseDatas>> {
    let url = &format!("http://192.168.1.{ip}:{port}/sgv.json?count=12&interval=60&brief_mode=Y");
    let json = http_get(url);
    let res = serde_json::from_str::<Vec<GlucoseDatas>>(&json?);
    Ok(res?)
}
