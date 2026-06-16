use crate::network::http::http_get;

use anyhow::Result;

#[derive(serde::Deserialize)]
pub struct StockQuote {
    pub c: f32,
    pub d: Option<f32>,
    pub dp: Option<f32>,
    pub pc: f32
}

pub fn fetch_stock(symbol: &str) -> Result<f32> {
    let url = &format!("https://finnhub.io/api/v1/quote?symbol={symbol}&token={}", env!("FINNHUB_TOKEN"));
    let json = http_get(url);
    let res = serde_json::from_str::<StockQuote>(&json?);
    Ok(res?.c)
}
