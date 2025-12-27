use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer};
use std::fs;
use std::net::TcpListener;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::EnvFilter;

const FILE_PATH: &str = "/usr/src/app/files/log.txt";
const CONFIG_FILE_PATH: &str = "/usr/src/app/config/information.txt";
const PING_PONG_URL: &str = "http://ping-pong-svc:3000/pings";

async fn log_output() -> HttpResponse {
    let config_content = match fs::read_to_string(CONFIG_FILE_PATH) {
        Ok(content) => content.trim().to_string(),
        Err(e) => {
            tracing::error!("Failed to read config file: {}", e);
            "config file not found".to_string()
        }
    };

    let message = std::env::var("MESSAGE").unwrap_or_else(|_| "env variable not set".to_string());

    let log_content = match fs::read_to_string(FILE_PATH) {
        Ok(content) => content.lines().last().unwrap_or("").to_string(),
        Err(e) => {
            tracing::error!("Failed to read log file: {}", e);
            return HttpResponse::InternalServerError()
                .content_type("text/plain; charset=utf-8")
                .body(format!("Error reading log file: {}", e));
        }
    };

    let ping_count = match reqwest::get(PING_PONG_URL).await {
        Ok(response) => match response.text().await {
            Ok(text) => text.trim().parse::<usize>().unwrap_or(0),
            Err(e) => {
                tracing::error!("Failed to parse ping count response: {}", e);
                0
            }
        },
        Err(e) => {
            tracing::error!("Failed to fetch ping count from {}: {}", PING_PONG_URL, e);
            0
        }
    };

    let response = format!(
        "file content: {}\nenv variable: MESSAGE={}\n{}.\nPing / Pongs: {}",
        config_content, message, log_content, ping_count
    );

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