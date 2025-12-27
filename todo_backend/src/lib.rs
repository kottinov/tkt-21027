use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use actix_cors::Cors;
use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: usize,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTodo {
    pub content: String,
}

#[derive(Clone)]
struct AppState {
    todos: Arc<Mutex<Vec<Todo>>>,
    next_id: Arc<Mutex<usize>>,
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/json; charset=utf-8")
        .body(r#"{"status":"ok"}"#)
}

async fn get_todos(state: web::Data<AppState>) -> HttpResponse {
    let todos = state.todos.lock().unwrap();
    HttpResponse::Ok()
        .content_type("application/json; charset=utf-8")
        .json(todos.clone())
}

async fn create_todo(
    state: web::Data<AppState>,
    new_todo: web::Json<CreateTodo>,
) -> HttpResponse {
    let content = new_todo.content.trim();

    if content.is_empty() {
        return HttpResponse::BadRequest()
            .content_type("application/json; charset=utf-8")
            .json(serde_json::json!({
                "error": "Todo content cannot be empty"
            }));
    }

    if content.len() > 140 {
        return HttpResponse::BadRequest()
            .content_type("application/json; charset=utf-8")
            .json(serde_json::json!({
                "error": "Todo content must be 140 characters or less"
            }));
    }

    let mut next_id = state.next_id.lock().unwrap();
    let id = *next_id;
    *next_id += 1;
    drop(next_id);

    let todo = Todo {
        id,
        content: content.to_string(),
    };

    let mut todos = state.todos.lock().unwrap();
    todos.push(todo.clone());
    drop(todos);

    tracing::info!("Created new todo with id {}: {}", id, content);

    HttpResponse::Created()
        .content_type("application/json; charset=utf-8")
        .json(todo)
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let state = web::Data::new(AppState {
        todos: Arc::new(Mutex::new(Vec::new())),
        next_id: Arc::new(Mutex::new(1)),
    });

    let server = HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .app_data(state.clone())
            .wrap(cors)
            .wrap(tracing_actix_web::TracingLogger::default())
            .route("/todos", web::get().to(get_todos))
            .route("/todos", web::post().to(create_todo))
            .route("/healthz", web::get().to(health_check))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
