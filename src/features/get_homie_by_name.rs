use std::fmt::Debug;

use sqlx::Pool;
use sqlx::Postgres;
use thiserror::Error;

use crate::features::Homie;

use super::HomieNameValidationError;
use super::HomiesName;

#[tracing::instrument(name = "Getting Homie by Name", skip(db))]
pub async fn get_homie(
    homie_name: impl TryInto<HomiesName, Error = HomieNameValidationError> + Debug,
    db: &impl GetHomie,
) -> Result<Homie, GetHomieError> {
    let retrieved_homie = db.get_homie(homie_name.try_into()?).await?;

    Ok(retrieved_homie)
}

#[derive(Error, Debug)]
pub enum GetHomieError {
    #[error(transparent)]
    InvalidName(#[from] HomieNameValidationError),
    #[error(transparent)]
    DbError(#[from] sqlx::Error),
}

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

trait GetHomie {
    async fn get_homie(&self, params: HomiesName) -> Result<Homie, sqlx::Error>;
}

impl GetHomie for Pool<Postgres> {
    async fn get_homie(&self, params: HomiesName) -> Result<Homie, sqlx::Error> {
        let homie = sqlx::query_as!(
            Homie,
            r#"SELECT id, name FROM homies WHERE name = $1"#,
            params.as_str()
        )
        .fetch_one(self)
        .await?;
        Ok(homie)
    }
}
