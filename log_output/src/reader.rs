use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer};
use std::fs;
use std::net::TcpListener;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::EnvFilter;

const FILE_PATH: &str = "/usr/src/app/files/log.txt";

async fn log_output() -> HttpResponse {
    match fs::read_to_string(FILE_PATH) {
        Ok(content) => HttpResponse::Ok()
            .content_type("text/plain; charset=utf-8")
            .body(content),
        Err(e) => {
            tracing::error!("Failed to read log file: {}", e);
            HttpResponse::InternalServerError()
                .content_type("text/plain; charset=utf-8")
                .body(format!("Error reading log file: {}", e))
        }
    }
}

fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(move || {
        App::new()
            .route("/", web::get().to(log_output))
            .route("/log-output", web::get().to(log_output))
    })
    .listen(listener)?
    .run();

    Ok(server)
}

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_timer(UtcTime::rfc_3339())
        .with_env_filter(filter)
        .init();

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid port number");
    let address = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&address)?;

    tracing::info!("Log reader server started on port {}", port);

    run(listener)?.await
}