use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::{self};

use sqlx::Pool;
use sqlx::Postgres;
use tracing::Instrument;

use crate::models::Homie;
use crate::user::UserId;
use crate::validator::Validator;

#[tracing::instrument(skip(db))]
pub async fn create_homie(
    homie_name: String,
    user_id: UserId,
    db: &Pool<Postgres>,
) -> Result<Homie, CreateHomieError> {
    let homie = CreateHomieParams::new(user_id, &homie_name);

    homie.validate()?;
    let created_homie = db.create_homie(homie).await?;

    Ok(created_homie)
}

impl Error for CreateHomieError {}

#[derive(Debug)]
struct CreateHomieParams<'a> {
    user_id: i32,
    name: &'a str,
}

impl<'a> CreateHomieParams<'a> {
    fn new(user_id: UserId, name: &'a str) -> Self {
        Self {
            user_id: user_id.into(),
            name,
        }
    }
}

enum CreateHomieParamsError {
    InvalidName,
}

enum CreateHomieDbError {
    Unknown,
    ForeignKeyViolation { constraint: String },
    HomieAlreadyExists,
}

#[derive(Debug)]
pub enum CreateHomieError {
    Unknown,
    UnknownDbError(String),
    InvalidName,
    ForeignKeyViolation { constraint: String },
    HomieAlreadyExists,
}

impl Display for CreateHomieError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            CreateHomieError::InvalidName => write!(f, "Invalid name"),
            CreateHomieError::HomieAlreadyExists => write!(f, "Homie already exists"),
            CreateHomieError::Unknown => write!(f, "Unknown error"),
            CreateHomieError::UnknownDbError(e) => write!(f, "Unknown db error: {}", e),
            CreateHomieError::ForeignKeyViolation { constraint } => {
                write!(f, "Foreign key violation: {}", constraint)
            }
        }
    }
}

impl From<CreateHomieDbError> for CreateHomieError {
    fn from(value: CreateHomieDbError) -> Self {
        match value {
            CreateHomieDbError::HomieAlreadyExists => CreateHomieError::HomieAlreadyExists,
            CreateHomieDbError::Unknown => CreateHomieError::Unknown,
            CreateHomieDbError::ForeignKeyViolation { constraint } => {
                CreateHomieError::ForeignKeyViolation { constraint }
            }
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

impl Validator for CreateHomieParams<'_> {
    type E = CreateHomieParamsError;

    #[tracing::instrument(skip(self))]
    fn validate(&self) -> Result<(), Self::E> {
        match self.name.is_empty() {
            true => Err(CreateHomieParamsError::InvalidName),
            false => Ok(()),
        }
    }
}


trait CreateHomie {
    async fn create_homie<'a>(
        &self,
        params: CreateHomieParams<'a>,
    ) -> Result<Homie, CreateHomieDbError>;
}

impl CreateHomie for Pool<Postgres> {

    #[tracing::instrument(skip(self, params))]
    async fn create_homie<'a>(
        &self,
        params: CreateHomieParams<'a>,
    ) -> Result<Homie, CreateHomieDbError> {
        let homie = sqlx::query_as!(
            Homie,
            r#"INSERT INTO homies (user_id, name) VALUES ($1, $2) RETURNING id, name"#,
            params.user_id,
            params.name
        )
        .fetch_one(self)
        .instrument(tracing::info_span!("create_homie_db_query"))
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(db_error) => {
                if db_error.is_unique_violation() {
                    return CreateHomieDbError::HomieAlreadyExists;
                } else if db_error.is_foreign_key_violation() {
                    return CreateHomieDbError::ForeignKeyViolation {
                        constraint: db_error
                            .constraint()
                            .expect("Constraint should be named if it is a ForeignKeyViolation")
                            .to_string(),
                    };
                }
                CreateHomieDbError::Unknown
            }
            _ => CreateHomieDbError::Unknown,
        })?;
        Ok(homie)
    }
}
