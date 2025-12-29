use std::net::TcpListener;

use actix_cors::Cors;
use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: i32,
    pub content: String,
    pub done: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTodo {
    pub content: String,
}

#[derive(Clone)]
struct AppState {
    db_pool: PgPool,
    nats_client: async_nats::Client,
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/json; charset=utf-8")
        .body(r#"{"status":"ok"}"#)
}

async fn get_todos(state: web::Data<AppState>) -> HttpResponse {
    match fetch_todos(&state.db_pool).await {
        Ok(todos) => HttpResponse::Ok()
            .content_type("application/json; charset=utf-8")
            .json(todos),
        Err(e) => {
            tracing::error!("Failed to fetch todos: {}", e);
            HttpResponse::InternalServerError()
                .content_type("application/json; charset=utf-8")
                .json(serde_json::json!({
                    "error": "Failed to fetch todos"
                }))
        }
    }
}

async fn create_todo(
    state: web::Data<AppState>,
    new_todo: web::Json<CreateTodo>,
) -> HttpResponse {
    let content = new_todo.content.trim();

    tracing::info!("Received todo creation request: \"{}\" (length: {} chars)", content, content.len());

    if content.is_empty() {
        tracing::warn!("Rejected todo: content is empty");
        return HttpResponse::BadRequest()
            .content_type("application/json; charset=utf-8")
            .json(serde_json::json!({
                "error": "Todo content cannot be empty"
            }));
    }

    if content.len() > 140 {
        tracing::warn!(
            "Rejected todo: content exceeds 140 character limit (length: {} chars): \"{}\"",
            content.len(),
            content
        );
        return HttpResponse::BadRequest()
            .content_type("application/json; charset=utf-8")
            .json(serde_json::json!({
                "error": "Todo content must be 140 characters or less"
            }));
    }

    match insert_todo(&state.db_pool, content).await {
        Ok(todo) => {
            tracing::info!("Created new todo with id {}: {}", todo.id, todo.content);

            let message = serde_json::json!({
                "action": "created",
                "todo": todo
            });
            if let Err(e) = state.nats_client.publish("todo.events", message.to_string().into()).await {
                tracing::warn!("Failed to publish todo creation to NATS: {}", e);
            }

            HttpResponse::Created()
                .content_type("application/json; charset=utf-8")
                .json(todo)
        }
        Err(e) => {
            tracing::error!("Failed to create todo: {}", e);
            HttpResponse::InternalServerError()
                .content_type("application/json; charset=utf-8")
                .json(serde_json::json!({
                    "error": "Failed to create todo"
                }))
        }
    }
}

async fn fetch_todos(pool: &PgPool) -> Result<Vec<Todo>, sqlx::Error> {
    let todos = sqlx::query_as::<_, (i32, String, bool)>(
        "SELECT id, content, done FROM todos ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(id, content, done)| Todo { id, content, done })
    .collect();

    Ok(todos)
}

async fn insert_todo(pool: &PgPool, content: &str) -> Result<Todo, sqlx::Error> {
    let row = sqlx::query_as::<_, (i32, String, bool)>(
        "INSERT INTO todos (content) VALUES ($1) RETURNING id, content, done"
    )
    .bind(content)
    .fetch_one(pool)
    .await?;

    Ok(Todo {
        id: row.0,
        content: row.1,
        done: row.2,
    })
}

async fn update_todo(
    state: web::Data<AppState>,
    id: web::Path<i32>,
) -> HttpResponse {
    let todo_id = id.into_inner();

    tracing::info!("Marking todo {} as done", todo_id);

    match mark_todo_done(&state.db_pool, todo_id).await {
        Ok(todo) => {
            tracing::info!("Successfully marked todo {} as done", todo.id);

            let message = serde_json::json!({
                "action": "updated",
                "todo": todo
            });
            if let Err(e) = state.nats_client.publish("todo.events", message.to_string().into()).await {
                tracing::warn!("Failed to publish todo update to NATS: {}", e);
            }

            HttpResponse::Ok()
                .content_type("application/json; charset=utf-8")
                .json(todo)
        }
        Err(e) => {
            tracing::error!("Failed to update todo {}: {}", todo_id, e);
            HttpResponse::NotFound()
                .content_type("application/json; charset=utf-8")
                .json(serde_json::json!({
                    "error": "Todo not found"
                }))
        }
    }
}

async fn mark_todo_done(pool: &PgPool, id: i32) -> Result<Todo, sqlx::Error> {
    let row = sqlx::query_as::<_, (i32, String, bool)>(
        "UPDATE todos SET done = TRUE WHERE id = $1 RETURNING id, content, done"
    )
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(Todo {
        id: row.0,
        content: row.1,
        done: row.2,
    })
}

pub async fn connect_to_database() -> Result<PgPool, sqlx::Error> {
    let postgres_host = std::env::var("POSTGRES_HOST")
        .unwrap_or_else(|_| "todo-postgres-stset-0.todo-postgres-svc".to_string());
    let postgres_port = std::env::var("POSTGRES_PORT").unwrap_or_else(|_| "5432".to_string());
    let postgres_db = std::env::var("POSTGRES_DB").unwrap_or_else(|_| "todos".to_string());
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

pub async fn connect_to_nats() -> Result<async_nats::Client, async_nats::ConnectError> {
    let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://my-nats:4222".to_string());

    tracing::info!("Connecting to NATS at {}", nats_url);

    let client = async_nats::connect(&nats_url).await?;

    tracing::info!("Connected to NATS successfully");

    Ok(client)
}

pub fn run(listener: TcpListener, pool: PgPool, nats_client: async_nats::Client) -> Result<Server, std::io::Error> {
    let state = web::Data::new(AppState { db_pool: pool, nats_client });

    let server = HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .app_data(state.clone())
            .wrap(cors)
            .wrap(tracing_actix_web::TracingLogger::default())
            .route("/todos", web::get().to(get_todos))
            .route("/todos", web::post().to(create_todo))
            .route("/todos/{id}", web::put().to(update_todo))
            .route("/healthz", web::get().to(health_check))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
