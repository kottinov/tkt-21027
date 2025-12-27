use std::net::TcpListener;

use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::EnvFilter;

use todo_backend::{connect_to_database, run};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_timer(UtcTime::rfc_3339())
        .with_env_filter(filter)
        .init();

    let pool = connect_to_database()
        .await
        .expect("Failed to connect to database");

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid port number");

    let address = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&address)?;

    tracing::info!("Todo backend server started on port {}", port);

    run(listener, pool)?.await
}
