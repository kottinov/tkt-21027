use std::fs;
use std::net::TcpListener;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
struct Config {
    image_refresh_secs: u64,
    image_path: String,
    todo_backend_url: String,
    image_timestamp_path: String,
}

impl Config {
    fn from_env() -> Self {
        Self {
            image_refresh_secs: std::env::var("IMAGE_REFRESH_SECS")
                .unwrap_or_else(|_| "600".to_string())
                .parse()
                .expect("IMAGE_REFRESH_SECS must be a valid number"),
            image_path: std::env::var("IMAGE_PATH")
                .unwrap_or_else(|_| "/usr/src/app/cache/image.jpg".to_string()),
            todo_backend_url: std::env::var("TODO_BACKEND_URL")
                .unwrap_or_else(|_| "http://todo-backend-svc:3000/todos".to_string()),
            image_timestamp_path: std::env::var("IMAGE_TIMESTAMP_PATH")
                .unwrap_or_else(|_| "/usr/src/app/cache/image_timestamp.txt".to_string()),
        }
    }
}

#[derive(Clone)]
struct AppState {
    client: reqwest::Client,
    image_lock: Arc<Mutex<()>>,
    config: Config,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Todo {
    id: usize,
    content: String,
    done: bool,
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/json; charset=utf-8")
        .body(r#"{"status":"ok"}"#)
}

async fn index(state: web::Data<AppState>) -> HttpResponse {
    if let Err(e) = ensure_image(&state.client, &state.image_lock, &state.config).await {
        tracing::error!("Failed to ensure image: {}", e);
    }

    let todos = match fetch_todos(&state.client, &state.config).await {
        Ok(todos) => todos,
        Err(e) => {
            tracing::error!("Failed to fetch todos: {}", e);
            Vec::new()
        }
    };

    let todo_items_html: String = todos
        .iter()
        .map(|todo| {
            let done_class = if todo.done { " done" } else { "" };
            let done_button = if !todo.done {
                format!(r#"<button class="done-button" onclick="markDone({})">Done</button>"#, todo.id)
            } else {
                String::from(r#"<span class="done-badge">✓ Done</span>"#)
            };
            format!(
                r#"<li class="todo-item{}">{}{}</li>"#,
                done_class,
                html_escape(&todo.content),
                done_button
            )
        })
        .collect();

    let html = format!(
        r#"<!DOCTYPE html>
    <html>
    <head>
        <title>The Project</title>
        <style>
            body {{ font-family: Arial, sans-serif; max-width: 1200px; margin: 0 auto; padding: 20px; }}
            h1 {{ color: #333; }}
            img {{ max-width: 100%; height: auto; border: 2px solid #ddd; border-radius: 4px; margin-top: 20px; }}
            .todo-section {{ margin: 30px 0; }}
            .todo-form {{ display: flex; gap: 10px; margin-bottom: 20px; }}
            .todo-input {{ flex: 1; padding: 10px; font-size: 16px; border: 1px solid #ddd; border-radius: 4px; }}
            .todo-button {{ padding: 10px 20px; background-color: #007bff; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 16px; }}
            .todo-button:hover {{ background-color: #0056b3; }}
            .todo-list {{ list-style: none; padding: 0; }}
            .todo-item {{ padding: 12px; margin-bottom: 8px; background-color: #f8f9fa; border-left: 3px solid #007bff; border-radius: 4px; display: flex; justify-content: space-between; align-items: center; }}
            .todo-item.done {{ background-color: #e8f5e9; border-left-color: #4caf50; text-decoration: line-through; color: #666; }}
            .done-button {{ padding: 6px 12px; background-color: #28a745; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 14px; }}
            .done-button:hover {{ background-color: #218838; }}
            .done-badge {{ color: #28a745; font-weight: bold; }}
            .char-counter {{ font-size: 12px; color: #666; margin-top: 5px; }}
            .char-counter.warning {{ color: #dc3545; }}
            .empty-state {{ text-align: center; padding: 40px; color: #999; }}
        </style>
    </head>
    <body>
        <h1>The Project</h1>

        <div class="todo-section">
            <h2>Todo List</h2>
            <div class="todo-form">
                <input type="text"
                       id="todoInput"
                       class="todo-input"
                       placeholder="Enter a new todo (max 140 characters)"
                       maxlength="140"
                       oninput="updateCharCounter()">
                <button class="todo-button" onclick="addTodo()">Send</button>
            </div>
            <div id="charCounter" class="char-counter">0 / 140 characters</div>

            <h3>Existing Todos:</h3>
            <ul class="todo-list" id="todoList">
                {todo_items}
            </ul>
            {empty_state}
        </div>

        <div>
            <h2>Random Image</h2>
            <p>Random image (refreshes every 10 minutes):</p>
            <img src="/the-project/image.jpg" alt="Random image from Lorem Picsum">
        </div>

        <script>
            function updateCharCounter() {{
                const input = document.getElementById('todoInput');
                const counter = document.getElementById('charCounter');
                const length = input.value.length;
                counter.textContent = length + ' / 140 characters';

                if (length > 130) {{
                    counter.classList.add('warning');
                }} else {{
                    counter.classList.remove('warning');
                }}
            }}

            async function addTodo() {{
                const input = document.getElementById('todoInput');
                const todoText = input.value.trim();

                if (todoText === '') {{
                    alert('Please enter a todo item');
                    return;
                }}

                if (todoText.length > 140) {{
                    alert('Todo must be 140 characters or less');
                    return;
                }}

                try {{
                    const response = await fetch('/todos', {{
                        method: 'POST',
                        headers: {{
                            'Content-Type': 'application/json',
                        }},
                        body: JSON.stringify({{ content: todoText }})
                    }});

                    if (!response.ok) {{
                        const error = await response.json();
                        alert('Error: ' + (error.error || 'Failed to create todo'));
                        return;
                    }}

                    input.value = '';
                    updateCharCounter();

                    window.location.reload();
                }} catch (error) {{
                    alert('Error creating todo: ' + error.message);
                }}
            }}

            async function markDone(id) {{
                try {{
                    const response = await fetch(`/todos/${{id}}`, {{
                        method: 'PUT'
                    }});

                    if (!response.ok) {{
                        const error = await response.json();
                        alert('Error: ' + (error.error || 'Failed to mark todo as done'));
                        return;
                    }}

                    window.location.reload();
                }} catch (error) {{
                    alert('Error marking todo as done: ' + error.message);
                }}
            }}
        </script>
    </body>
    </html>"#,
        todo_items = todo_items_html,
        empty_state = if todos.is_empty() {
            r#"<div class="empty-state">No todos yet. Add one above!</div>"#
        } else {
            ""
        }
    );

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

async fn fetch_todos(client: &reqwest::Client, config: &Config) -> Result<Vec<Todo>, Box<dyn std::error::Error>> {
    let response = client.get(&config.todo_backend_url).send().await?;
    let todos: Vec<Todo> = response.json().await?;
    Ok(todos)
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

async fn serve_image(state: web::Data<AppState>) -> HttpResponse {
    match fs::read(&state.config.image_path) {
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
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    if !needs_refresh(config)? {
        return Ok(());
    }

    let _guard = lock.lock().map_err(|e| format!("Lock error: {}", e))?;

    if !needs_refresh(config)? {
        return Ok(());
    }

    tracing::info!("Fetching new image from Lorem Picsum");

    let response = client.get("https://picsum.photos/1200").send().await?;
    let image_bytes = response.bytes().await?;

    if let Some(parent) = Path::new(&config.image_path).parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&config.image_path, &image_bytes)?;

    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs();
    fs::write(&config.image_timestamp_path, timestamp.to_string())?;

    tracing::info!("Successfully cached new image");

    Ok(())
}

fn needs_refresh(config: &Config) -> Result<bool, Box<dyn std::error::Error>> {
    if !Path::new(&config.image_path).exists() || !Path::new(&config.image_timestamp_path).exists() {
        return Ok(true);
    }

    let timestamp_str = fs::read_to_string(&config.image_timestamp_path)?;
    let timestamp: u64 = timestamp_str.trim().parse()?;

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs();

    Ok(now.saturating_sub(timestamp) >= config.image_refresh_secs)
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let config = Config::from_env();

    let state = web::Data::new(AppState {
        client: reqwest::Client::new(),
        image_lock: Arc::new(Mutex::new(())),
        config,
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
