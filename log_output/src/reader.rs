use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer};
use std::fs;
use std::net::TcpListener;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::EnvFilter;

const FILE_PATH: &str = "/usr/src/app/files/log.txt";
const COUNTER_FILE: &str = "/usr/src/app/shared/pingpong_count.txt";

async fn log_output() -> HttpResponse {
    let log_content = match fs::read_to_string(FILE_PATH) {
        Ok(content) => content.lines().last().unwrap_or("").to_string(),
        Err(e) => {
            tracing::error!("Failed to read log file: {}", e);
            return HttpResponse::InternalServerError()
                .content_type("text/plain; charset=utf-8")
                .body(format!("Error reading log file: {}", e));
        }
    };

    let ping_count = fs::read_to_string(COUNTER_FILE)
        .ok()
        .and_then(|c| c.trim().parse::<usize>().ok())
        .unwrap_or(0);

    let response = format!("{}.\nPing / Pongs: {}", log_content, ping_count);

    HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body(response)
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