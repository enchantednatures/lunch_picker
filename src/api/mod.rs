use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use axum::extract::Json;
use axum::extract::State;
use axum::http::Request;
use serde_json::{json, Value};
use sqlx::query_file;
use sqlx::types::Uuid;

use tower_http::request_id::MakeRequestId;
use tower_http::request_id::RequestId;

use crate::startup::AppState;

#[derive(Clone, Default)]
pub struct MyMakeRequestId {
    counter: Arc<AtomicU64>,
}

impl MakeRequestId for MyMakeRequestId {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        let request_id = self
            .counter
            .fetch_add(1, Ordering::SeqCst)
            .to_string()
            .parse()
            .unwrap();

        Some(RequestId::new(request_id))
    }
}

pub async fn health_check() -> Json<Value> {
    Json(json!({ "status": "OK"}))
}

#[derive(serde::Deserialize)]
pub struct CreateRecipeRequest {
    pub request_id: Uuid,
    pub name: String,
}

pub async fn create_recipe(
    State(state): State<Arc<AppState>>,
    Json(recipe): Json<CreateRecipeRequest>,
) -> Json<Value> {
    let query_result = query_file!("./src/sql/insert_recipe.sql", recipe.name)
        .execute(&state.db_pool)
        .await;
    match query_result {
        Ok(_) => Json(json!({"created": true})),
        Err(_) => Json(json!({"created": false})),
    }
}
