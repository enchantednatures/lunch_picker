use std::fmt::Debug;

use sqlx::Pool;
use sqlx::Postgres;
use thiserror::Error;

use crate::features::Homie;
use crate::features::HomieRow;
use crate::user::UserId;

use super::HomieNameValidationError;
use super::HomiesName;

#[tracing::instrument(name = "Getting Homie by Name", skip(db))]
pub async fn get_homie(
    user_id: impl Into<UserId> + Debug,
    homie_name: impl TryInto<HomiesName, Error = HomieNameValidationError> + Debug,
    db: &impl GetHomie,
) -> Result<Homie, GetHomieError> {
    let retrieved_homie = db
        .get_homie(GetHomieParams::new(
            user_id.into().as_i32(),
            homie_name.try_into().unwrap().as_str(),
        ))
        .await?;

    Ok(retrieved_homie)
}

#[derive(Error, Debug)]
pub enum GetHomieError {
    #[error(transparent)]
    InvalidName(#[from] HomieNameValidationError),
    #[error(transparent)]
    DbError(#[from] sqlx::Error),
}

struct GetHomieParams<'a> {
    user_id: &'a i32,
    name: &'a str,
}
impl<'a> GetHomieParams<'a> {
    fn new(user_id: &'a i32, name: &'a str) -> Self {
        Self { user_id, name }
    }
}

trait GetHomie {
    async fn get_homie<'a>(
        &self,
        params: impl Into<GetHomieParams<'a>>,
    ) -> Result<Homie, sqlx::Error>;
}

impl GetHomie for Pool<Postgres> {
    async fn get_homie<'a>(
        &self,
        params: impl Into<GetHomieParams<'a>>,
    ) -> Result<Homie, sqlx::Error> {
        let params: GetHomieParams<'a> = params.into();
        let homie: HomieRow = sqlx::query_as(
            r#"SELECT id, user_id, name FROM homies WHERE name = $1 and user_id = $2"#,
        )
        .bind(params.name)
        .bind(params.user_id)
        .fetch_one(self)
        .await?;
        Ok(homie.into())
    }
}
