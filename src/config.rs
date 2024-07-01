use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use thiserror::Error;

#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub telemetry_enabled: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            database: DatabaseSettings::default(),
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

pub struct SettingsBuilder {
    config_file: Option<PathBuf>,
}
impl SettingsBuilder {
    fn new() -> Self {
        Self { config_file: None }
    }
    pub fn with_config_file(mut self, config_file: impl AsRef<Path>) -> Self {
        let prefix = &config_file.as_ref().parent().unwrap();
        fs::create_dir_all(prefix).unwrap();
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
            false => {
                fs::write(
                    config_file,
                    serde_json::to_string_pretty(&Settings::default()).unwrap(),
                );
                Ok(Settings::default())
            }
        }
    }
}

impl Settings {
    pub fn builder() -> SettingsBuilder {
        SettingsBuilder::new()
    }
}
