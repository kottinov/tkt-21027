use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer};

async fn health_check() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/json; charset=utf-8")
        .body(r#"{"status":"ok"}"#)
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .wrap(tracing_actix_web::TracingLogger::default())
            .route("/", web::get().to(health_check))
            .route("/healthz", web::get().to(health_check))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
