use chrono::Utc;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::thread;
use std::time::Duration;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

const INTERVAL_SECS: u64 = 5;
const FILE_PATH: &str = "/usr/src/app/files/log.txt";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_timer(UtcTime::rfc_3339())
        .with_env_filter(filter)
        .init();

    let instance_id = Uuid::new_v4().to_string();
    tracing::info!("Log writer started with instance ID: {}", instance_id);

    if let Some(parent) = Path::new(FILE_PATH).parent() {
        fs::create_dir_all(parent)?;
    }

    loop {
        let timestamp = Utc::now().to_rfc3339();
        let log_line = format!("{}: {}\n", timestamp, instance_id);

        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(FILE_PATH)?;
        file.write_all(log_line.as_bytes())?;
        file.sync_all()?;

        tracing::info!("Wrote log entry: {}", log_line.trim());
        thread::sleep(Duration::from_secs(INTERVAL_SECS));
    }
}