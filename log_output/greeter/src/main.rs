use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use std::env;

#[get("/")]
async fn greet() -> impl Responder {
    let greeting = env::var("GREETING").unwrap_or_else(|_| "Hello".to_string());
    HttpResponse::Ok().body(greeting)
}

#[get("/healthz")]
async fn healthz() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let bind_addr = format!("0.0.0.0:{}", port);

    let version = env::var("VERSION").unwrap_or_else(|_| "unknown".to_string());
    let greeting = env::var("GREETING").unwrap_or_else(|_| "Hello".to_string());

    log::info!("Starting greeter service version {} on {}", version, bind_addr);
    log::info!("Greeting: {}", greeting);

    HttpServer::new(|| {
        App::new()
            .service(greet)
            .service(healthz)
    })
    .bind(&bind_addr)?
    .run()
    .await
}