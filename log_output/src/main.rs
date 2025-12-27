use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::EnvFilter;

use log_output::{generate_string, run, start_logger};

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_timer(UtcTime::rfc_3339())
        .with_env_filter(filter)
        .init();

    let instance_id = generate_string();
    start_logger(instance_id.clone());

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid port number");
    let address = format!("0.0.0.0:{}", port);
    let listener = std::net::TcpListener::bind(&address)?;

    tracing::info!("Server started in port {}", port);

    run(listener, instance_id)?.await
}
