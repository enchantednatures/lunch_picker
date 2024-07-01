use std::fmt::Debug;

use models::UserId;
use sqlx::Pool;

#[cfg(feature = "postgres")]
use sqlx::Postgres;

#[cfg(feature = "sqlite")]
use sqlx::Sqlite;
use thiserror::Error;
use tracing::Instrument;

use super::models::Homie;
use super::HomieNameValidationError;
use super::HomieRow;
use super::HomiesName;

#[tracing::instrument(skip(db))]
pub async fn create_homie(
    homie_name: impl TryInto<HomiesName, Error = HomieNameValidationError> + Debug,
    user_id: impl Into<UserId> + Debug,
    db: &impl CreateHomie,
) -> Result<Homie, CreateHomieError> {
    let homies_name: HomiesName = homie_name.try_into()?;
    let homie = CreateHomieParams::new(user_id.into(), &homies_name);

    let created_homie = db.create_homie(homie).await?;

    Ok(created_homie)
}

#[derive(Debug)]
struct CreateHomieParams<'a> {
    user_id: i32,
    name: &'a str,
}

impl<'a> CreateHomieParams<'a> {
    fn new(user_id: UserId, name: &'a HomiesName) -> Self {
        Self {
            user_id: user_id.into(),
            name: name.as_str(),
        }
    }
}

#[derive(Error, Debug)]
pub enum CreateHomieError {
    #[error(transparent)]
    ValidationError(#[from] HomieNameValidationError),

    #[error("Invalid User")]
    ForeignKeyViolation { constraint: String },

    #[error("Homie already exists: {:?}", name)]
    HomieAlreadyExists { name: String },

    #[error("Unknown db error")]
    UnknownDbError(#[from] sqlx::Error),

    #[error("Unknown error")]
    Unknown,
}

pub trait CreateHomie {
    async fn create_homie<'a>(
        &self,
        params: CreateHomieParams<'a>,
    ) -> Result<Homie, CreateHomieError>;
}



#[cfg(feature = "sqlite")]
impl CreateHomie for Pool<Sqlite> {
    #[tracing::instrument(skip(self, params))]
    async fn create_homie<'a>(
        &self,
        params: CreateHomieParams<'a>,
    ) -> Result<Homie, CreateHomieError> {
        let homie: HomieRow = sqlx::query_as(
            r#"INSERT INTO homies (user_id, name) VALUES (?, ?) RETURNING id, user_id, name"#,
        )
        .bind(params.user_id)
        .bind(params.name.to_string())
        // .bind((params.user_id, params.name.to_string()))
        .fetch_one(self)
        .instrument(tracing::info_span!("Insert Homie Query"))
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(db_error) => {
                if db_error.is_unique_violation() {
                    return CreateHomieError::HomieAlreadyExists {
                        name: params.name.to_string(),
                    };
                } else if db_error.is_foreign_key_violation() {
                    return CreateHomieError::ForeignKeyViolation {
                        constraint: db_error
                            .constraint()
                            .expect("Constraint should be named if it is a ForeignKeyViolation")
                            .to_string(),
                    };
                }
                CreateHomieError::UnknownDbError(sqlx::Error::Database(db_error))
            }
            _ => CreateHomieError::UnknownDbError(e),
        })?;
        Ok(homie.into())
    }
}
