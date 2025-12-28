use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer};
use sqlx::postgres::PgPool;

#[derive(Clone)]
struct AppState {
    db_pool: PgPool,
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/json; charset=utf-8")
        .body(r#"{"status":"ok"}"#)
}


async fn ping_pong(data: web::Data<AppState>) -> HttpResponse {
    let count = match increment_counter(&data.db_pool).await {
        Ok(count) => count,
        Err(e) => {
            tracing::error!("Failed to increment counter: {}", e);
            return HttpResponse::InternalServerError()
                .content_type("text/plain; charset=utf-8")
                .body(format!("Database error: {}", e));
        }
    };

    HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body(format!("pong {}", count))
}

async fn get_pings(data: web::Data<AppState>) -> HttpResponse {
    let count = match get_counter(&data.db_pool).await {
        Ok(count) => count,
        Err(e) => {
            tracing::error!("Failed to get counter: {}", e);
            return HttpResponse::InternalServerError()
                .content_type("text/plain; charset=utf-8")
                .body(format!("Database error: {}", e));
        }
    };

    HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body(count.to_string())
}

async fn increment_counter(pool: &PgPool) -> Result<i32, sqlx::Error> {
    let row = sqlx::query_as::<_, (i32,)>(
        "UPDATE counter SET count = count + 1 WHERE id = 1 RETURNING count"
    )
    .fetch_one(pool)
    .await?;

    Ok(row.0)
}

async fn get_counter(pool: &PgPool) -> Result<i32, sqlx::Error> {
    let row = sqlx::query_as::<_, (i32,)>("SELECT count FROM counter WHERE id = 1")
        .fetch_one(pool)
        .await?;

    Ok(row.0)
}

pub async fn connect_to_database() -> Result<PgPool, sqlx::Error> {
    let postgres_host = std::env::var("POSTGRES_HOST")
        .unwrap_or_else(|_| "postgres-stset-0.postgres-svc".to_string());
    let postgres_port = std::env::var("POSTGRES_PORT").unwrap_or_else(|_| "5432".to_string());
    let postgres_db = std::env::var("POSTGRES_DB").unwrap_or_else(|_| "pingpong".to_string());
    let postgres_user = std::env::var("POSTGRES_USER").unwrap_or_else(|_| "postgres".to_string());
    let postgres_password =
        std::env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| "postgres".to_string());

    let connection_string = format!(
        "postgres://{}:{}@{}:{}/{}",
        postgres_user, postgres_password, postgres_host, postgres_port, postgres_db
    );

    tracing::info!(
        "Connecting to database at {}:{}",
        postgres_host,
        postgres_port
    );

    let pool = PgPool::connect(&connection_string).await?;

    tracing::info!("Connected to database successfully");

    Ok(pool)
}

pub fn run(listener: TcpListener, pool: PgPool) -> Result<Server, std::io::Error> {
    let state = web::Data::new(AppState { db_pool: pool });

    let server = HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(tracing_actix_web::TracingLogger::default())
            .route("/", web::get().to(ping_pong))
            .route("/pings", web::get().to(get_pings))
            .route("/healthz", web::get().to(health_check))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
