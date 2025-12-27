use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer};
use chrono::Utc;

struct AppState {
    instance_id: String,
}

async fn status(data: web::Data<AppState>) -> HttpResponse {
    let timestamp = Utc::now().to_rfc3339();
    let body = format!("{} {}", timestamp, data.instance_id);

    HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body(body)
}

pub fn run(listener: TcpListener, instance_id: String) -> Result<Server, std::io::Error> {
    let state = web::Data::new(AppState { instance_id });

    let server = HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .route("/log-output", web::get().to(status))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
