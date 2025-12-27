use log_output::{generate_string, run};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::time::UtcTime;

fn main() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_timer(UtcTime::rfc_3339())
        .with_env_filter(filter)
        .init();

    run(&generate_string());
}
