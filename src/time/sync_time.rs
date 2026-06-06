use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use esp_idf_svc::sntp::{EspSntp, SyncStatus};

pub fn sync_ntp() -> Result<()> {
    let sntp = EspSntp::new_default()?;

    while sntp.get_sync_status() != SyncStatus::Completed  {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    Ok(())
}

pub fn get_local_time(offset_secs: i32) -> (u8, u8, u8) {
    let timestamp_utc = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let local_secs = timestamp_utc + offset_secs as u64;

    let seconds_per_day = local_secs % 86400;
    let hours = seconds_per_day / 3600;
    let minutes = (seconds_per_day % 3600) / 60;
    let seconds = seconds_per_day % 60;

    (hours as u8, minutes as u8, seconds as u8)
}