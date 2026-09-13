use crate::network::http::http_get;

use anyhow::Result;

const proxy_ip: u8 = ;
const glucose_broadcast: u16 = ;

#[derive(serde::Deserialize)]
struct GlucoseDatas {
    pub date: u32,
    pub sgv: u16, 
    pub delta: f32,
    pub direction: String,
    pub noise: i8,
    pub units_hint: Option<String>,
}

pub fn fetch_glucose(ip: u8, port: u16) -> Result<GlucoseDatas> {
    let url = &format!("192.168.1.{ip}:{port}/sgv.json?count=20&interval=90&brief_mode=Y");
    let json = http_get(url);
    let res = serde_json::from_str::GlucoseDatas(&json?);
    Ok(res)
}
