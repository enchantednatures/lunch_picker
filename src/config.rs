use serde::Deserialize;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
}

#[derive(Deserialize, Clone)]
pub enum DatabaseSettings {
    #[cfg(feature = "postgres")]
    Postgres(PostgresSettings),
    #[cfg(feature = "sqlite")]
    Sqlite(SqliteSettings),
}

#[cfg(feature = "postgres")]
#[derive(Deserialize, Clone)]
pub struct PostgresSettings {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database_name: String,
}

#[cfg(feature = "sqlite")]
#[derive(Deserialize, Clone)]
pub struct SqliteSettings {
    pub filename: PathBuf,
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

pub struct SettingsBuilder {
    config_file: Option<PathBuf>,
}
impl SettingsBuilder {
    fn new() -> Self {
        Self { config_file: None }
    }
    pub fn with_config_file(mut self, config_file: impl AsRef<Path>) -> Self {
        self.config_file = Some(config_file.as_ref().to_path_buf());
        self
    }

    pub fn build(self) -> Result<Settings, ConfigError> {
        let config_file = self
            .config_file
            .unwrap_or_else(|| "~/.config/local/lunch.json".into());
        match config_file.exists() {
            true => {
                let config_file = config_file.to_str().ok_or(ConfigError::UnableToParsePath)?;
                let settings = std::fs::read_to_string(config_file)?;
                Ok(serde_json::from_str(&settings)?)
            }
            false => return Err(ConfigError::Unknown),
        }
    }
}

impl Settings {
    pub fn builder() -> SettingsBuilder {
        SettingsBuilder::new()
    }
}
