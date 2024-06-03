use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::{self};

use sqlx::Pool;
use sqlx::Postgres;

use crate::models::Homie;
use crate::validator::Validator;

#[tracing::instrument(name = "Getting Homie by Name", skip(db))]
pub async fn get_homie(homie_name: String, db: &Pool<Postgres>) -> Result<Homie, GetHomieError> {
    let homie: GetHomieParams = homie_name.into();
    homie.validate()?;
    let retrieved_homie = db.get_homie(homie).await?;

    Ok(retrieved_homie)
}
impl Error for GetHomieError {}
struct GetHomieParams {
    name: String,
}
impl GetHomieParams {
    fn new(name: String) -> Self {
        Self { name }
    }
}
impl From<String> for GetHomieParams {
    fn from(name: String) -> Self {
        Self::new(name)
    }
}

enum GetHomieParamsError {
    InvalidName,
}

enum GetHomieDbError {
    HomieNotFound { name: String },
}

#[derive(Debug)]
pub enum GetHomieError {
    InvalidName,
    HomieNotFound { name: String },
}

impl Display for GetHomieError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            GetHomieError::InvalidName => write!(f, "Invalid name"),
            GetHomieError::HomieNotFound { name } => write!(f, "Homie not found {}", name),
        }
    }
}

impl From<GetHomieDbError> for GetHomieError {
    fn from(value: GetHomieDbError) -> Self {
        match value {
            GetHomieDbError::HomieNotFound { name } => GetHomieError::HomieNotFound { name },
        }
    }
}
impl From<GetHomieParamsError> for GetHomieError {
    fn from(value: GetHomieParamsError) -> Self {
        match value {
            GetHomieParamsError::InvalidName => GetHomieError::InvalidName,
        }
    }
}

impl Validator for GetHomieParams {
    type E = GetHomieParamsError;

    fn validate(&self) -> Result<(), Self::E> {
        match self.name.is_empty() {
            true => return Err(GetHomieParamsError::InvalidName),
            false => return Ok(()),
        }
    }
}


trait GetHomie {
    async fn get_homie(&self, params: GetHomieParams) -> Result<Homie, GetHomieDbError>;
}

impl GetHomie for Pool<Postgres> {
    async fn get_homie(&self, params: GetHomieParams) -> Result<Homie, GetHomieDbError> {
        let homie = sqlx::query_as!(
            Homie,
            r#"SELECT id, name FROM homies WHERE name = $1"#,
            params.name
        )
        .fetch_one(self)
        .await
        .map_err(|e| -> GetHomieDbError {
            println!("Error: {:?}", e);
            GetHomieDbError::HomieNotFound { name: params.name }
        })?;
        Ok(homie)
    }
}
