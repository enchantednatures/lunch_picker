use std::fmt::Debug;

use sqlx::Pool;
use sqlx::Postgres;
use thiserror::Error;

use crate::user::UserId;

use super::Homie;

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

trait GetAllHomies {
    async fn get_all_homies(&self, params: UserId) -> Result<Vec<Homie>, sqlx::Error>;
}

impl GetAllHomies for Pool<Postgres> {
    async fn get_all_homies(&self, params: UserId) -> Result<Vec<Homie>, sqlx::Error> {
        let homie = sqlx::query_as!(
            Homie,
            r#"select id, name from homies where user_id = $1"#,
            params.as_i32()
        )
        .fetch_all(self)
        .await?;
        Ok(homie)
    }
}
