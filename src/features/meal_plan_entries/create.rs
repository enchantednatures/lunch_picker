use std::fmt::Debug;

use sqlx::Pool;
use sqlx::Sqlite;
use thiserror::Error;
use tracing::Instrument;

use super::models::{MealPlanEntry, MealPlanEntryRow, MealSlot};
use crate::user::UserId;

#[tracing::instrument(skip(db))]
pub async fn create_meal_plan_entry(
    recipe_id: i32,
    date: chrono::NaiveDate,
    meal_slot: MealSlot,
    user_id: impl Into<UserId> + Debug,
    db: &impl CreateMealPlanEntry,
) -> Result<MealPlanEntry, CreateMealPlanEntryError> {
    let params = CreateMealPlanEntryParams::new(recipe_id, date, meal_slot, user_id.into());
    let entry = db.create_meal_plan_entry(params).await?;
    Ok(entry)
}

#[derive(Debug)]
pub(crate) struct CreateMealPlanEntryParams {
    recipe_id: i32,
    date: chrono::NaiveDate,
    meal_slot: String,
    user_id: i32,
}

impl CreateMealPlanEntryParams {
    fn new(
        recipe_id: i32,
        date: chrono::NaiveDate,
        meal_slot: MealSlot,
        user_id: UserId,
    ) -> Self {
        Self {
            recipe_id,
            date,
            meal_slot: meal_slot.as_str().to_string(),
            user_id: user_id.into(),
        }
    }
}

#[derive(Error, Debug)]
pub enum CreateMealPlanEntryError {
    #[error("Meal plan slot already occupied")]
    SlotOccupied,

    #[error("Unknown db error")]
    UnknownDbError(#[from] sqlx::Error),

    #[error("Unknown error")]
    Unknown,
}

pub(crate) trait CreateMealPlanEntry {
    async fn create_meal_plan_entry(
        &self,
        params: CreateMealPlanEntryParams,
    ) -> Result<MealPlanEntry, CreateMealPlanEntryError>;
}

impl CreateMealPlanEntry for Pool<Sqlite> {
    #[tracing::instrument(skip(self, params))]
    async fn create_meal_plan_entry(
        &self,
        params: CreateMealPlanEntryParams,
    ) -> Result<MealPlanEntry, CreateMealPlanEntryError> {
        let row: MealPlanEntryRow = sqlx::query_as(
            r#"
            INSERT INTO meal_plan_entries (user_id, recipe_id, date, meal_slot)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(user_id, date, meal_slot) DO UPDATE SET
                recipe_id = excluded.recipe_id,
                updated_at = CURRENT_TIMESTAMP
            RETURNING id, user_id, recipe_id,
                (SELECT name FROM recipes WHERE id = recipe_id) as recipe_name,
                date, meal_slot
            "#,
        )
        .bind(params.user_id)
        .bind(params.recipe_id)
        .bind(params.date)
        .bind(params.meal_slot)
        .fetch_one(self)
        .instrument(tracing::info_span!("Upsert Meal Plan Entry"))
        .await
        .map_err(|e| CreateMealPlanEntryError::UnknownDbError(e))?;

        Ok(row.into())
    }
}
