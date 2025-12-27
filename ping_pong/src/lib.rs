use std::net::TcpListener;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer};

struct AppState {
    counter: Arc<AtomicUsize>,
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/json; charset=utf-8")
        .body(r#"{"status":"ok"}"#)
}

async fn ping_pong(data: web::Data<AppState>) -> HttpResponse {
    let count = data.counter.fetch_add(1, Ordering::Relaxed) + 1;

    HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body(format!("pong {}", count))
}

async fn get_pings(data: web::Data<AppState>) -> HttpResponse {
    let count = data.counter.load(Ordering::Relaxed);

    HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body(count.to_string())
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let state = web::Data::new(AppState {
        counter: Arc::new(AtomicUsize::new(0)),
    });

    let server = HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(tracing_actix_web::TracingLogger::default())
            .route("/pingpong", web::get().to(ping_pong))
            .route("/pings", web::get().to(get_pings))
            .route("/healthz", web::get().to(health_check))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
