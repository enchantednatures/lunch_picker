use std::fmt::Debug;

use sqlx::Pool;
use sqlx::Postgres;
use thiserror::Error;
use tracing::Instrument;

use crate::user::UserId;

use super::Homie;
use super::HomieRow;

#[tracing::instrument(name = "Getting all Homies", skip(db))]
pub async fn get_all_homies(
    user_id: impl Into<UserId> + Debug,
    db: &impl GetAllHomies,
) -> Result<Vec<Homie>, GetAllHomiesError> {
    let retrieved_homies = db.get_all_homies(user_id.into()).await?;

    Ok(retrieved_homies)
}
#[derive(Error, Debug)]
pub enum GetAllHomiesError {
    #[error(transparent)]
    DbError(#[from] sqlx::Error),
}

pub trait GetAllHomies {
    async fn get_all_homies(&self, params: UserId) -> Result<Vec<Homie>, sqlx::Error>;
}

impl GetAllHomies for Pool<Postgres> {
    #[tracing::instrument(name = "Getting all Homies", skip(self))]
    async fn get_all_homies(&self, params: UserId) -> Result<Vec<Homie>, sqlx::Error> {
        let homie: Vec<HomieRow> =
            sqlx::query_as(r#"select id, user_id, name from homies where user_id = $1"#)
                .bind(params.as_i32())
                .fetch_all(self)
                .instrument(tracing::info_span!("Querying all homies"))
                .await?;
        Ok(homie.into_iter().map(|x| x.into()).collect())
    }
}
