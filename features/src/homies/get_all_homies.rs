use std::fmt::Debug;

use thiserror::Error;

use models::UserId;

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

// todo: make this not return a sqlx error
pub trait GetAllHomies {
    async fn get_all_homies(&self, params: UserId) -> Result<Vec<Homie>, sqlx::Error>;
}
