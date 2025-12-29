use serde::{Deserialize, Serialize};
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Todo {
    id: i32,
    content: String,
    done: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TodoEvent {
    action: String,
    todo: Todo,
}

#[derive(Debug, Clone, Serialize)]
struct TelegramMessage {
    chat_id: String,
    text: String,
}

async fn connect_to_nats() -> Result<async_nats::Client, async_nats::ConnectError> {
    let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://my-nats:4222".to_string());

    tracing::info!("Connecting to NATS at {}", nats_url);

    let client = async_nats::connect(&nats_url).await?;

    tracing::info!("Connected to NATS successfully");

    Ok(client)
}

async fn send_telegram(bot_token: &str, chat_id: &str, text: String) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let url = format!("https://api.telegram.org/bot{}/sendMessage", bot_token);

    let message = TelegramMessage {
        chat_id: chat_id.to_string(),
        text,
    };

    tracing::info!("Sending Telegram message: {}", message.text);

    client
        .post(&url)
        .json(&message)
        .send()
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_timer(UtcTime::rfc_3339())
        .with_env_filter(filter)
        .init();

    let telegram_bot_token = std::env::var("TELEGRAM_BOT_TOKEN")
        .expect("TELEGRAM_BOT_TOKEN environment variable must be set");

    let telegram_chat_id = std::env::var("TELEGRAM_CHAT_ID")
        .expect("TELEGRAM_CHAT_ID environment variable must be set");

    tracing::info!("Broadcaster starting with Telegram chat ID: {}", telegram_chat_id);

    let client = connect_to_nats().await?;

    let mut subscriber = client.queue_subscribe("todo.events", "broadcasters".to_string()).await?;

    tracing::info!("Subscribed to todo.events with queue group 'broadcasters'");

    while let Some(message) = subscriber.next().await {
        let payload = String::from_utf8_lossy(&message.payload);

        tracing::debug!("Received message: {}", payload);

        match serde_json::from_str::<TodoEvent>(&payload) {
            Ok(event) => {
                let message_text = match event.action.as_str() {
                    "created" => format!("A todo was created: \"{}\"", event.todo.content),
                    "updated" => format!("A todo was updated: \"{}\" (done: {})", event.todo.content, event.todo.done),
                    _ => format!("Unknown action on todo: \"{}\"", event.todo.content),
                };

                if let Err(e) = send_telegram(&telegram_bot_token, &telegram_chat_id, message_text).await {
                    tracing::error!("Failed to send Telegram message: {}", e);
                } else {
                    tracing::info!("Successfully sent Telegram notification");
                }
            }
            Err(e) => {
                tracing::error!("Failed to parse todo event: {}", e);
            }
        }
    }

    Ok(())
}