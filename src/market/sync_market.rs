use crate::network::http::http_get;

use anyhow::Result;

#[derive(serde::Deserialize)]
struct PriceResponse {
    bitcoin: BtcPrice,
}

#[derive(serde::Deserialize)]
pub struct BtcPrice {
    pub usd: f32,
}

pub fn fetch_btc_price() -> Result<f32> {
    let json = http_get("https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd");
    let res = serde_json::from_str::<PriceResponse>(&json?);
    Ok(res?.bitcoin.usd)
}
