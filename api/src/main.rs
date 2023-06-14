#![allow(dead_code)]

use std::fs;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use std::{net::SocketAddr, time::Duration};

use axum::extract::FromRequestParts;
use axum::extract::State;
use axum::http::request::Parts;
use axum::http::HeaderName;
use axum::http::StatusCode;
use axum::routing::put;
use axum::{routing::get, Router};
use sqlx::migrate;
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::sqlite::SqlitePoolOptions;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::trace::TraceLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::routes::health_check;

mod routes;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "shitty_lunch_picker=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // build our application with some routes
    let app = Router::new().route("/health_check", get(health_check));

    // run it with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
