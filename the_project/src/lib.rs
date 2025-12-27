use std::fs;
use std::net::TcpListener;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer};
use serde::{Deserialize, Serialize};

const IMAGE_REFRESH_SECS: u64 = 600;
const IMAGE_PATH: &str = "/usr/src/app/cache/image.jpg";
const TODO_BACKEND_URL: &str = "http://todo-backend-svc:3000/todos";
const IMAGE_TIMESTAMP_PATH: &str = "/usr/src/app/cache/image_timestamp.txt";

#[derive(Clone)]
struct AppState {
    client: reqwest::Client,
    image_lock: Arc<Mutex<()>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Todo {
    id: usize,
    content: String,
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

    let todos = match fetch_todos(&state.client).await {
        Ok(todos) => todos,
        Err(e) => {
            tracing::error!("Failed to fetch todos: {}", e);
            Vec::new()
        }
    };

    let todo_items_html: String = todos
        .iter()
        .map(|todo| {
            format!(
                r#"<li class="todo-item">{}</li>"#,
                html_escape(&todo.content)
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
            .todo-item {{ padding: 12px; margin-bottom: 8px; background-color: #f8f9fa; border-left: 3px solid #007bff; border-radius: 4px; }}
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

async fn fetch_todos(client: &reqwest::Client) -> Result<Vec<Todo>, Box<dyn std::error::Error>> {
    let response = client.get(TODO_BACKEND_URL).send().await?;
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
