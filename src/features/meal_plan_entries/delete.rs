use std::fmt::Debug;

use sqlx::Pool;
use sqlx::Sqlite;
use thiserror::Error;
use tracing::Instrument;

use crate::user::UserId;

#[tracing::instrument(skip(db))]
pub async fn delete_meal_plan_entry(
    entry_id: i32,
    user_id: impl Into<UserId> + Debug,
    db: &impl DeleteMealPlanEntry,
) -> Result<(), DeleteMealPlanEntryError> {
    db.delete_meal_plan_entry(entry_id, user_id.into()).await?;
    Ok(())
}

#[derive(Error, Debug)]
pub enum DeleteMealPlanEntryError {
    #[error("Entry not found")]
    NotFound,

    #[error("Unknown db error")]
    UnknownDbError(#[from] sqlx::Error),

    #[error("Unknown error")]
    Unknown,
}

pub(crate) trait DeleteMealPlanEntry {
    async fn delete_meal_plan_entry(
        &self,
        entry_id: i32,
        user_id: UserId,
    ) -> Result<(), DeleteMealPlanEntryError>;
}

impl DeleteMealPlanEntry for Pool<Sqlite> {
    #[tracing::instrument(skip(self))]
    async fn delete_meal_plan_entry(
        &self,
        entry_id: i32,
        user_id: UserId,
    ) -> Result<(), DeleteMealPlanEntryError> {
        let result = sqlx::query(
            r#"DELETE FROM meal_plan_entries WHERE id = ? AND user_id = ?"#,
        )
        .bind(entry_id)
        .bind(user_id.as_i32())
        .execute(self)
        .instrument(tracing::info_span!("Delete Meal Plan Entry"))
        .await
        .map_err(|e| DeleteMealPlanEntryError::UnknownDbError(e))?;

        if result.rows_affected() == 0 {
            return Err(DeleteMealPlanEntryError::NotFound);
        }

        Ok(())
    }
}
