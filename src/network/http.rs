use anyhow::{anyhow, Result};
use embedded_svc::http::client::Client;
use embedded_svc::http::Method;
use embedded_svc::utils::io;
use esp_idf_svc::http::client::{Configuration, EspHttpConnection};
use log::{error, info};

pub fn http_get(url: &str) -> Result<String> {
    let config = Configuration {
        crt_bundle_attach: Some(esp_idf_svc::sys::esp_crt_bundle_attach),
        ..Default::default()
    };
    let mut client = Client::wrap(EspHttpConnection::new(&config)?);
    let headers = [("accept", "text/plain")];
    let request = client.request(Method::Get, url, &headers)?;
    let mut response = request.submit()?;

    let status = response.status();
    if status != 200 {
        return Err(anyhow!("HTTP request failed with status {status}"));
    }

    let mut buf = [0u8; 1024];
    let bytes_read = io::try_read_full(&mut response, &mut buf).map_err(|e| e.0)?;

    match std::str::from_utf8(&buf[0..bytes_read]) {
        Ok(body_string) => {
            info!("Response body ({} bytes): {:?}", bytes_read, body_string);
            Ok(body_string.to_string())
        }
        Err(e) => {
            error!("Error decoding response body: {e}");
            Err(anyhow!("Failed to decode response body as UTF-8: {e}"))
        }
    }
}