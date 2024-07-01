#![allow(async_fn_in_trait)]

pub mod cli_args;
mod config;
pub mod db;
pub mod features;
mod interaction;
pub mod user;

pub use config::*;
pub use interaction::*;

#[cfg(feature = "postgres")]
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("migrations/postgres");

#[cfg(not(feature = "postgres"))]
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("migrations/sqlite");
