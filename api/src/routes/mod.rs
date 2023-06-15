use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Result};
use axum::Json;
use serde::Deserialize;
use serde::Serialize;
use serde_json::{json, Value};
use sqlx::SqlitePool;

use utoipa::{IntoParams, ToSchema};

#[derive(serde::Serialize)]
pub struct Message {
    pub message: String,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateHomieRequest {
    #[schema(example = "Hunter")]
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct CreateHomieResponse {
    #[schema(example = "1")]
    pub id: i64,

    #[schema(example = "Hunter")]
    pub name: String,
}

#[utoipa::path(
put,
path = "/homies",
responses(
(status = 200, description = "Create a homie", body = CreateHomieResponse),
)
)]
pub async fn put_homie(
    State(db_pool): State<SqlitePool>,
    Json(payload): Json<CreateHomieRequest>,
) -> impl IntoResponse {
    let homie = sqlx::query_as!(
        CreateHomieResponse,
        r#"
        INSERT INTO homies (name)
        VALUES (?)
        RETURNING id, name as "name!"
        "#,
        payload.name
    )
    .fetch_one(&db_pool)
    .await
    .unwrap();

    (StatusCode::CREATED, Json(homie))
}

#[utoipa::path(
get,
path = "/homies",
responses(
(status = 200, description = "Get all", body = [CreateHomieResponse]),
)
)]
pub async fn get_homies(
    State(db_pool): State<SqlitePool>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let homies = sqlx::query_as!(
        CreateHomieResponse,
        r#"
        SELECT id, name
        FROM homies
        "#
    )
    .fetch_all(&db_pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(homies))
}

#[derive(Serialize, Deserialize, ToSchema)]
enum HealthStatusEnum {
    Ok,
    Error,
}

#[derive(Deserialize, Serialize, ToSchema)]
struct HealthStatus {
    status: HealthStatusEnum,
}

impl HealthStatus {
    fn new() -> Self {
        HealthStatus {
            status: HealthStatusEnum::Ok,
        }
    }
}

#[utoipa::path(
get,
path = "/health_check",
responses(
(status = 200, description = "Check health", body = HealthStatus),
)
)]
pub async fn health_check() -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(Json(HealthStatus::new()))
}

/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
