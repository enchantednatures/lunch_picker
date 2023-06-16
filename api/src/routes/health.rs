use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{response, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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
pub async fn health_check() -> response::Result<impl IntoResponse, (StatusCode, String)> {
    Ok(Json(HealthStatus::new()))
}
