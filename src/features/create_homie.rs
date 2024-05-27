use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::{self};

use sqlx::Pool;
use sqlx::Sqlite;
use tracing::info;

use crate::validator::Validator;

#[tracing::instrument]
pub async fn create_homie(
    homie_name: String,
    db: &Pool<Sqlite>,
) -> Result<Homie, CreateHomieError> {
    info!("Creating homie");

    let homie: CreateHomieParams = homie_name.into();
    homie.validate()?;
    let created_homie = db.create_homie(homie).await?;

    Ok(created_homie)
}
impl Error for CreateHomieError {}
struct CreateHomieParams {
    name: String,
}
impl CreateHomieParams {
    fn new(name: String) -> Self {
        Self { name }
    }
}

impl From<String> for CreateHomieParams {
    fn from(name: String) -> Self {
        Self::new(name)
    }
}

enum CreateHomieParamsError {
    InvalidName,
}

enum CreateHomieDbError {
    HomieAlreadyExists,
}

#[derive(Debug)]
pub enum CreateHomieError {
    InvalidName,
    HomieAlreadyExists,
}

impl Display for CreateHomieError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            CreateHomieError::InvalidName => write!(f, "Invalid name"),
            CreateHomieError::HomieAlreadyExists => write!(f, "Homie already exists"),
        }
    }
}

impl From<CreateHomieDbError> for CreateHomieError {
    fn from(value: CreateHomieDbError) -> Self {
        match value {
            CreateHomieDbError::HomieAlreadyExists => CreateHomieError::HomieAlreadyExists,
        }
    }
}
impl From<CreateHomieParamsError> for CreateHomieError {
    fn from(value: CreateHomieParamsError) -> Self {
        match value {
            CreateHomieParamsError::InvalidName => CreateHomieError::InvalidName,
        }
    }
}

impl Validator for CreateHomieParams {
    type E = CreateHomieParamsError;

    fn validate(&self) -> Result<(), Self::E> {
        match self.name.is_empty() {
            true => return Err(CreateHomieParamsError::InvalidName),
            false => return Ok(()),
        }
    }
}

#[derive(Debug)]
pub struct Homie {
    pub id: i64,
    pub name: String,
}

trait CreateHomie {
    async fn create_homie(&self, params: CreateHomieParams) -> Result<Homie, CreateHomieDbError>;
}

impl CreateHomie for Pool<Sqlite> {
    async fn create_homie(&self, params: CreateHomieParams) -> Result<Homie, CreateHomieDbError> {
        let homie = sqlx::query_as!(
            Homie,
            r#"INSERT INTO homies (name) VALUES (?) RETURNING id, name"#,
            params.name
        )
        .fetch_one(self)
        .await
        .map_err(|e| {
            println!("Error: {:?}", e);
            match e {
                sqlx::Error::Configuration(_) => todo!(),
                sqlx::Error::Database(db_error) => {
                    if db_error.is_unique_violation() {
                        return CreateHomieDbError::HomieAlreadyExists;
                    }
                    todo!()
                }
                sqlx::Error::Io(_) => todo!(),
                sqlx::Error::Tls(_) => todo!(),
                sqlx::Error::Protocol(_) => todo!(),
                sqlx::Error::RowNotFound => todo!(),
                sqlx::Error::TypeNotFound { type_name } => todo!(),
                sqlx::Error::ColumnIndexOutOfBounds { index, len } => todo!(),
                sqlx::Error::ColumnNotFound(_) => todo!(),
                sqlx::Error::ColumnDecode { index, source } => todo!(),
                sqlx::Error::Decode(_) => todo!(),
                sqlx::Error::AnyDriverError(_) => todo!(),
                sqlx::Error::PoolTimedOut => todo!(),
                sqlx::Error::PoolClosed => todo!(),
                sqlx::Error::WorkerCrashed => todo!(),
                sqlx::Error::Migrate(_) => todo!(),
                _ => CreateHomieDbError::HomieAlreadyExists,
            }
        })?;
        Ok(homie)
    }
}
