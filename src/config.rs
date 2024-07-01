use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use thiserror::Error;

use crate::user_setup;

#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
    pub database_url: String,
    pub telemetry_enabled: bool,
}

impl Settings {
    pub fn new(database_url: String, telemetry_enabled: bool) -> Self {
        Self {
            database_url,
            telemetry_enabled,
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            database_url: DatabaseSettings::default().to_url(),
            telemetry_enabled: true,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum DatabaseSettings {
    #[cfg(feature = "postgres")]
    Postgres(PostgresSettings),
    #[cfg(feature = "sqlite")]
    Sqlite(SqliteSettings),
}

impl Default for DatabaseSettings {
    fn default() -> Self {
        #[cfg(feature = "postgres")]
        return Self::Postgres(PostgresSettings {
            host: "localhost".into(),
            port: 5432,
            username: "postgres".into(),
            password: "password".into(),
            database_name: "lunch_picker".into(),
        });
        #[cfg(feature = "sqlite")]
        return Self::Sqlite(SqliteSettings {
            filename: PathBuf::from_str("~/.local/state/lunch.db").unwrap(),
        });
    }
}

impl ToDatabaseUrl for DatabaseSettings {
    fn to_url(&self) -> String {
        match self {
            #[cfg(feature = "postgres")]
            DatabaseSettings::Postgres(settings) => settings.to_url(),
            #[cfg(feature = "sqlite")]
            DatabaseSettings::Sqlite(settings) => settings.to_url(),
        }
    }
}

#[cfg(feature = "postgres")]
#[derive(Serialize, Deserialize, Clone)]
pub struct PostgresSettings {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database_name: String,
}

pub trait ToDatabaseUrl {
    fn to_url(&self) -> String;
}

#[cfg(feature = "postgres")]
impl ToDatabaseUrl for PostgresSettings {
    fn to_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }
}

#[cfg(feature = "sqlite")]
#[derive(Serialize, Deserialize, Clone)]
pub struct SqliteSettings {
    pub filename: PathBuf,
}

#[cfg(feature = "sqlite")]
impl ToDatabaseUrl for SqliteSettings {
    fn to_url(&self) -> String {
        format!(
            "sqlite:{}",
            String::from(
                &self
                    .filename
                    .clone()
                    .into_os_string()
                    .into_string()
                    .unwrap(),
            )
        )
    }
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse config file")]
    Serde(#[from] serde_json::Error),

    #[error("Unknown Error")]
    UnableToParsePath,

    #[error("Unknown Error")]
    Unknown,
}
