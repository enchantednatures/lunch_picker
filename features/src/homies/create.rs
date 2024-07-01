use std::fmt::Debug;

use models::UserId;

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
