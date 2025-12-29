use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer};
use chrono::Utc;

struct AppState {
    instance_id: String,
    greeter_url: String,
}

async fn status(data: web::Data<AppState>) -> HttpResponse {
    let timestamp = Utc::now().to_rfc3339();

    let greeting = match reqwest::get(&data.greeter_url).await {
        Ok(response) => match response.text().await {
            Ok(text) => text,
            Err(_) => "Hello".to_string(),
        },
        Err(_) => "Hello".to_string(),
    };

    let body = format!("{}: {} {}", greeting, timestamp, data.instance_id);

    HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body(body)
}

async fn root(data: web::Data<AppState>) -> HttpResponse {
    let timestamp = Utc::now().to_rfc3339();

    let greeting = match reqwest::get(&data.greeter_url).await {
        Ok(response) => match response.text().await {
            Ok(text) => text,
            Err(_) => "Hello".to_string(),
        },
        Err(_) => "Hello".to_string(),
    };

    let body = format!("{}: {} {}", greeting, timestamp, data.instance_id);

    HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body(body)
}

pub fn run(listener: TcpListener, instance_id: String, greeter_url: String) -> Result<Server, std::io::Error> {
    let state = web::Data::new(AppState { instance_id, greeter_url });

    let server = HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .route("/", web::get().to(root))
            .route("/log-output", web::get().to(status))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
