pub fn log_entry(value: &str) {
    tracing::info!(payload = %value, "log entry");
}
