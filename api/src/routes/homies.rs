use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{response, Json};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use utoipa::ToSchema;

use common::Homie;

use crate::routes;

#[derive(Deserialize, ToSchema)]
pub struct CreateHomieRequest {
    #[schema(example = "Hunter")]
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct CreateHomieResponse {
    #[schema(example = "1")]
    #[schema(example = "2")]
    pub id: i64,

    #[schema(example = "Sienna")]
    #[schema(example = "Hunter")]
    pub name: String,
}

#[utoipa::path(put, path = "/homies", responses((status = 200, description = "Create a homie", body = CreateHomieResponse), (status = NOT_FOUND, description = "Pet was not found")))]
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

#[utoipa::path(put, path = "/homies/{homie_id}/", responses((status = 200, description = "Create a homie", body = CreateHomieResponse), (status = NOT_FOUND, description = "Pet was not found")))]
pub async fn patch_homie(
    State(db_pool): State<SqlitePool>,
    Path(homie_id): Path<i64>,
    Json(payload): Json<CreateHomieRequest>,
) -> impl IntoResponse {
    sqlx::query(
        r#"
        UPDATE homies
        SET name = ?
        WHERE id = ?
        "#,
    )
    .bind(payload.name)
    .bind(homie_id)
    .execute(&db_pool)
    .await
    .unwrap();

    (
        StatusCode::ACCEPTED,
        Json(Homie::new(homie_id, payload.name)),
    )
}

#[utoipa::path(
get,
path = "/homies",
responses((status = 200, description = "Get all", body = [CreateHomieResponse]))
)]
pub async fn get_homies(
    State(db_pool): State<SqlitePool>,
) -> response::Result<impl IntoResponse, (StatusCode, String)> {
    let homies = sqlx::query_as!(
        CreateHomieResponse,
        r#"
        SELECT id, name
        FROM homies
        "#
    )
    .fetch_all(&db_pool)
    .await
    .map_err(routes::internal_error)?;

    Ok(Json(homies))
}
