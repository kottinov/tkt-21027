use std::net::TcpListener;
use std::time::Duration;

use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::EnvFilter;

use ping_pong::{connect_to_database, run};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_timer(UtcTime::rfc_3339())
        .with_env_filter(filter)
        .init();

    let pool = loop {
        match connect_to_database().await {
            Ok(pool) => break pool,
            Err(e) => {
                tracing::warn!("Failed to connect to database: {}. Retrying in 2 seconds...", e);
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    };

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid port number");

    let address = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&address)?;

    tracing::info!("Server started in port {}", port);

    run(listener, pool)?.await
}
