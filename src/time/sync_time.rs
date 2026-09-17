use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Ok, Result, anyhow, Context};
use esp_idf_svc::sntp::{EspSntp, SyncStatus};

pub fn sync_ntp() -> Result<()> {
    let sntp = EspSntp::new_default()?;
    let started_at = Instant::now();

    while sntp.get_sync_status() != SyncStatus::Completed {
        if started_at.elapsed() >= Duration::from_secs(10) {
            return Err(anyhow!("NTP synchronization timed out"));
        }
        std::thread::sleep(Duration::from_secs(1));
    }
    Ok(())
}

pub fn get_local_time(offset_secs: i32) -> Result<(u8, u8, u8)> {
    let timestamp_utc = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("System clock is earlier than Unix epoch")?
        .as_secs();

    let local_secs = (timestamp_utc as i64 + offset_secs as i64) as u64;
    let seconds_per_day = local_secs % 86400;

    let hours = seconds_per_day / 3600;
    let minutes = (seconds_per_day % 3600) / 60;
    let seconds = seconds_per_day % 60;

    Ok((hours as u8, minutes as u8, seconds as u8))
}
