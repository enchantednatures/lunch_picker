use sqlx::Pool;
use sqlx::Sqlite;
use thiserror::Error;
use tracing::Instrument;

use super::models::{MealPlanEntry, MealPlanEntryRow};
use crate::user::UserId;

#[tracing::instrument(skip(db))]
pub async fn list_meal_plan_entries(
    user_id: impl Into<UserId> + std::fmt::Debug,
    start_date: chrono::NaiveDate,
    end_date: chrono::NaiveDate,
    db: &impl ListMealPlanEntries,
) -> Result<Vec<MealPlanEntry>, ListMealPlanEntriesError> {
    let entries = db
        .list_meal_plan_entries(user_id.into(), start_date, end_date)
        .await?;
    Ok(entries.into_iter().map(|e| e.into()).collect())
}

#[derive(Error, Debug)]
pub enum ListMealPlanEntriesError {
    #[error("Unknown db error")]
    UnknownDbError(#[from] sqlx::Error),

    #[error("Unknown error")]
    Unknown,
}

pub(crate) trait ListMealPlanEntries {
    async fn list_meal_plan_entries(
        &self,
        user_id: UserId,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    ) -> Result<Vec<MealPlanEntryRow>, ListMealPlanEntriesError>;
}

impl ListMealPlanEntries for Pool<Sqlite> {
    #[tracing::instrument(skip(self))]
    async fn list_meal_plan_entries(
        &self,
        user_id: UserId,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    ) -> Result<Vec<MealPlanEntryRow>, ListMealPlanEntriesError> {
        let rows = sqlx::query_as::<_, MealPlanEntryRow>(
            r#"
            SELECT
                mpe.id,
                mpe.user_id,
                mpe.recipe_id,
                r.name as recipe_name,
                mpe.date,
                mpe.meal_slot
            FROM meal_plan_entries mpe
            JOIN recipes r ON r.id = mpe.recipe_id
            WHERE mpe.user_id = ? AND mpe.date BETWEEN ? AND ?
            ORDER BY mpe.date, mpe.meal_slot
            "#,
        )
        .bind(user_id.as_i32())
        .bind(start_date)
        .bind(end_date)
        .fetch_all(self)
        .instrument(tracing::info_span!("List Meal Plan Entries"))
        .await
        .map_err(|e| ListMealPlanEntriesError::UnknownDbError(e))?;

        Ok(rows)
    }
}
