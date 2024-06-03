use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::{self};

use sqlx::Pool;
use sqlx::Postgres;

use crate::models::Homie;
use crate::validator::Validator;

#[tracing::instrument(name = "Getting all Homies", skip(db))]
pub async fn get_all_homies(db: &Pool<Postgres>, user_id: i32) -> Result<Vec<Homie>, GetAllHomiesError> {
    let retrieved_homies = db.get_all_homies(user_id.into()).await?;

    Ok(retrieved_homies)
}
impl Error for GetAllHomiesError {}
struct GetAllHomiesParams {
    user_id: i32,
}
impl GetAllHomiesParams {
    fn new(user_id: i32) -> Self {
        Self { user_id }
    }
}
impl From<i32> for GetAllHomiesParams {
    fn from(user_id: i32) -> Self {
        Self::new(user_id)
    }
}

enum GetAllHomiesParamsError {
    InvalidName,
}

enum GetAllHomiesDbError {
    HomieNotFound { name: String },
}

#[derive(Debug)]
pub enum GetAllHomiesError {
    InvalidName,
    HomieNotFound { name: String },
}

impl Display for GetAllHomiesError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            GetAllHomiesError::InvalidName => write!(f, "Invalid name"),
            GetAllHomiesError::HomieNotFound { name } => write!(f, "Homie not found {}", name),
        }
    }
}

impl From<GetAllHomiesDbError> for GetAllHomiesError {
    fn from(value: GetAllHomiesDbError) -> Self {
        match value {
            GetAllHomiesDbError::HomieNotFound { name } => GetAllHomiesError::HomieNotFound { name },
        }
    }
}
impl From<GetAllHomiesParamsError> for GetAllHomiesError {
    fn from(value: GetAllHomiesParamsError) -> Self {
        match value {
            GetAllHomiesParamsError::InvalidName => GetAllHomiesError::InvalidName,
        }
    }
}

impl Validator for GetAllHomiesParams {
    type E = GetAllHomiesParamsError;

    fn validate(&self) -> Result<(), Self::E> {
        match self.user_id == 0 {
            true => return Err(GetAllHomiesParamsError::InvalidName),
            false => return Ok(()),
        }
    }
}


trait GetAllHomies {
    async fn get_all_homies(&self, params: GetAllHomiesParams) -> Result<Vec<Homie>, GetAllHomiesDbError>;
}

impl GetAllHomies for Pool<Postgres> {
    async fn get_all_homies(&self, params: GetAllHomiesParams) -> Result<Vec<Homie>, GetAllHomiesDbError> {
        let homie = sqlx::query_as!(
            Homie,
            r#"select id, name from homies where user_id = $1"#,
            params.user_id
        )
        .fetch_all(self)
        .await
        .map_err(|e| -> GetAllHomiesDbError {
            println!("Error: {:?}", e);
            GetAllHomiesDbError::HomieNotFound { name: "unknown".into() }
        })?;
        Ok(homie)
    }
}
