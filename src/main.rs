#![allow(dead_code)]

use std::fs;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;

use axum::http::HeaderName;
use axum::routing::put;
use axum::{routing::get, Router};
use sqlx::migrate;
use sqlx::sqlite::SqlitePoolOptions;

use api::create_recipe;
use startup::AppState;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::trace::TraceLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse};
use tower_http::ServiceBuilderExt;

use crate::api::create_homie;
use crate::api::{health_check, MyMakeRequestId};

mod algorithms;
mod api;
mod data_access;
mod models;
mod startup;
mod stuff;
mod user_input;

static CONFIG_FILE: &str = "./.shitty_lunch_picker.config";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut is_setup = false;

    let mut db_url = String::new();

    if !stuff::check_if_file_exists(CONFIG_FILE) {
        println!("Config file doesn't exist");
        let mut file = File::create(CONFIG_FILE).unwrap();
        db_url = "./.shitty_lunch_picker.db".into();
        file.write_all(db_url.as_bytes()).unwrap();
    }

    if db_url.is_empty() {
        db_url = fs::read_to_string(CONFIG_FILE).expect("Failed to read config file");
    }
    if !stuff::check_if_file_exists(&db_url) {
        println!("Database file doesn't exist");
        File::create(&db_url).unwrap();
    } else {
        is_setup = true;
    }

    let db_pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    if !is_setup {
        migrate!("./migrations").run(&db_pool).await?;
    }

    let state = Arc::new(AppState::new(db_pool));
    tracing_subscriber::fmt::init();

    let _x_request_id = HeaderName::from_static("x-request-id");

    let svc = ServiceBuilder::new()
        .set_x_request_id(MyMakeRequestId::default())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_response(DefaultOnResponse::new().include_headers(true)),
        )
        .propagate_x_request_id()
        .layer(CompressionLayer::new());

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/health_check", get(health_check))
        .route("/create_recipe", put(create_recipe))
        .route("/create_homie", put(create_homie))
        .with_state(state)
        .layer(svc);

    axum::Server::bind(&"0.0.0.0:6969".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}
