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
    Sqlite(SqliteSettings),
}

impl Default for DatabaseSettings {
    fn default() -> Self {
        Self::Sqlite(SqliteSettings {
            filename: PathBuf::from_str("~/.local/state/lunch.db").unwrap(),
        })
    }
}
trait ToDatabaseUrl {
    fn to_url(&self) -> String;
}

impl ToDatabaseUrl for DatabaseSettings {
    fn to_url(&self) -> String {
        match self {
            DatabaseSettings::Sqlite(settings) => settings.to_url(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SqliteSettings {
    pub filename: PathBuf,
}

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
