use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;

#[derive(serde::Serialize)]
pub struct Message {
    pub message: String,
}

pub async fn health_check() -> Json<Message> {
    Json(Message {
        message: String::from("Hello, World!"),
    })
}
