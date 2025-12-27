use std::fs;
use std::net::TcpListener;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer};

const IMAGE_PATH: &str = "/usr/src/app/cache/image.jpg";
const IMAGE_TIMESTAMP_PATH: &str = "/usr/src/app/cache/image_timestamp.txt";
const IMAGE_REFRESH_SECS: u64 = 600;

#[derive(Clone)]
struct AppState {
    client: reqwest::Client,
    image_lock: Arc<Mutex<()>>,
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/json; charset=utf-8")
        .body(r#"{"status":"ok"}"#)
}

async fn index(state: web::Data<AppState>) -> HttpResponse {
    if let Err(e) = ensure_image(&state.client, &state.image_lock).await {
        tracing::error!("Failed to ensure image: {}", e);
    }

    let html = r#"<!DOCTYPE html>
    <html>
    <head>
        <title>The Project</title>
        <style>
            body { font-family: Arial, sans-serif; max-width: 1200px; margin: 0 auto; padding: 20px; }
            h1 { color: #333; }
            img { max-width: 100%; height: auto; border: 2px solid #ddd; border-radius: 4px; margin-top: 20px; }
        </style>
    </head>
    <body>
        <h1>The Project</h1>
        <p>Random image (refreshes every 10 minutes):</p>
        <img src="/the-project/image.jpg" alt="Random image from Lorem Picsum">
    </body>
    </html>"#;

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

async fn serve_image() -> HttpResponse {
    match fs::read(IMAGE_PATH) {
        Ok(image_data) => HttpResponse::Ok()
            .content_type("image/jpeg")
            .body(image_data),
        Err(e) => {
            tracing::error!("Failed to read image file: {}", e);
            HttpResponse::NotFound().body("Image not found")
        }
    }
}

async fn ensure_image(
    client: &reqwest::Client,
    lock: &Arc<Mutex<()>>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !needs_refresh()? {
        return Ok(());
    }

    let _guard = lock.lock().map_err(|e| format!("Lock error: {}", e))?;

    if !needs_refresh()? {
        return Ok(());
    }

    tracing::info!("Fetching new image from Lorem Picsum");

    let response = client.get("https://picsum.photos/1200").send().await?;
    let image_bytes = response.bytes().await?;

    if let Some(parent) = Path::new(IMAGE_PATH).parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(IMAGE_PATH, &image_bytes)?;

    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs();
    fs::write(IMAGE_TIMESTAMP_PATH, timestamp.to_string())?;

    tracing::info!("Successfully cached new image");

    Ok(())
}

fn needs_refresh() -> Result<bool, Box<dyn std::error::Error>> {
    if !Path::new(IMAGE_PATH).exists() || !Path::new(IMAGE_TIMESTAMP_PATH).exists() {
        return Ok(true);
    }

    let timestamp_str = fs::read_to_string(IMAGE_TIMESTAMP_PATH)?;
    let timestamp: u64 = timestamp_str.trim().parse()?;

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs();

    Ok(now.saturating_sub(timestamp) >= IMAGE_REFRESH_SECS)
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let state = web::Data::new(AppState {
        client: reqwest::Client::new(),
        image_lock: Arc::new(Mutex::new(())),
    });

    let server = HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(tracing_actix_web::TracingLogger::default())
            .service(
                web::scope("/the-project")
                    .route("", web::get().to(index))
                    .route("/", web::get().to(index))
                    .route("/image.jpg", web::get().to(serve_image))
                    .route("/healthz", web::get().to(health_check)),
            )
    })
    .listen(listener)?
    .run();

    Ok(server)
}
