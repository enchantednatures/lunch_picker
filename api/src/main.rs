#![allow(dead_code)]

use std::net::SocketAddr;
use std::time::Duration;

use anyhow::Result;
use axum::error_handling::HandleErrorLayer;
use axum::http::StatusCode;
use axum::routing::get;
use axum::routing::patch;
use axum::Router;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{migrate, SqlitePool};
use tokio::time::error::Elapsed;
use tower::{BoxError, ServiceBuilder};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;

use routes::health;
use routes::health::health_check;
use routes::homies;
use routes::homies::{get_homies, put_homie};

mod routes;

#[tokio::main]
async fn main() -> Result<()> {
    #[derive(OpenApi)]
    #[openapi(
    paths(
    homies::get_homies,
    homies::put_homie,
    health::health_check
    ),
    components(schemas(homies::CreateHomieRequest, homies::CreateHomieResponse)),
    modifiers(& SecurityAddon),
    tags((name = "shitty lunch picker", description = "Shitty lunch picker management API"))
    )]
    struct ApiDoc;

    struct SecurityAddon;

    impl Modify for SecurityAddon {
        fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
            if let Some(components) = openapi.components.as_mut() {
                components.add_security_scheme(
                    "api_key",
                    SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("todo_apikey"))),
                )
            }
        }
    }
    let db_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "./shitty_lunch_picker.db".into());
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "shitty_lunch_picker=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    // let pool = SqlitePoolOptions::new()
    //     .max_connections(5)
    //     .connect(&db_url)
    //     .await?;

    let pool: SqlitePool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    migrate!("./../migrations").run(&pool).await?;

    // build our application with some routes
    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/health_check", get(health_check))
        .route("/homies", get(get_homies).put(put_homie))
        .route(
            "/homies/:id",
            patch(|| async { "patch" }).delete(|| async { "delete" }),
        )
        .route("/recipes", get(get_homies).put(put_homie))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|error: BoxError| async move {
                    if error.is::<Elapsed>() {
                        Ok(StatusCode::REQUEST_TIMEOUT)
                    } else {
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Unhandled internal error: {}", error),
                        ))
                    }
                }))
                .timeout(Duration::from_secs(10))
                .layer(TraceLayer::new_for_http())
                .into_inner(),
        )
        .with_state(pool);

    // run it with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 6969));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}
