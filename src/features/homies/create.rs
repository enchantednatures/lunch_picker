use std::fmt::Debug;

use sqlx::Pool;
use sqlx::Postgres;
use thiserror::Error;
use tracing::Instrument;

use super::models::Homie;
use super::HomieNameValidationError;
use crate::user::UserId;

use super::HomiesName;

trait TryIntoHomieName: TryInto<HomiesName, Error = HomieNameValidationError> + Debug {}

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

trait CreateHomie {
    async fn create_homie<'a>(
        &self,
        params: CreateHomieParams<'a>,
    ) -> Result<Homie, CreateHomieError>;
}

impl CreateHomie for Pool<Postgres> {
    #[tracing::instrument(skip(self, params))]
    async fn create_homie<'a>(
        &self,
        params: CreateHomieParams<'a>,
    ) -> Result<Homie, CreateHomieError> {
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
        Ok(homie)
    }
}
