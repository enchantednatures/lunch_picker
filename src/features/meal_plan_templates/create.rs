use std::fmt::Debug;

use sqlx::Pool;
use sqlx::Sqlite;
use thiserror::Error;
use tracing::Instrument;

use super::models::{MealPlanTemplate, MealPlanTemplateRow};
use crate::features::meal_plan_entries::MealSlot;
use crate::user::UserId;

#[tracing::instrument(skip(db))]
pub async fn create_meal_plan_template(
    name: String,
    day_of_week: i32,
    meal_slot: MealSlot,
    recipe_id: i32,
    user_id: impl Into<UserId> + Debug,
    db: &impl CreateMealPlanTemplate,
) -> Result<MealPlanTemplate, CreateMealPlanTemplateError> {
    let params = CreateMealPlanTemplateParams::new(
        name,
        day_of_week,
        meal_slot,
        recipe_id,
        user_id.into(),
    );
    let template = db.create_meal_plan_template(params).await?;
    Ok(template)
}

#[derive(Debug)]
pub(crate) struct CreateMealPlanTemplateParams {
    name: String,
    day_of_week: i32,
    meal_slot: String,
    recipe_id: i32,
    user_id: i32,
}

impl CreateMealPlanTemplateParams {
    fn new(
        name: String,
        day_of_week: i32,
        meal_slot: MealSlot,
        recipe_id: i32,
        user_id: UserId,
    ) -> Self {
        Self {
            name,
            day_of_week,
            meal_slot: meal_slot.as_str().to_string(),
            recipe_id,
            user_id: user_id.into(),
        }
    }
}

#[derive(Error, Debug)]
pub enum CreateMealPlanTemplateError {
    #[error("Template already exists for this day and slot")]
    AlreadyExists,

    #[error("Unknown db error")]
    UnknownDbError(#[from] sqlx::Error),

    #[error("Unknown error")]
    Unknown,
}

pub(crate) trait CreateMealPlanTemplate {
    async fn create_meal_plan_template(
        &self,
        params: CreateMealPlanTemplateParams,
    ) -> Result<MealPlanTemplate, CreateMealPlanTemplateError>;
}

impl CreateMealPlanTemplate for Pool<Sqlite> {
    #[tracing::instrument(skip(self, params))]
    async fn create_meal_plan_template(
        &self,
        params: CreateMealPlanTemplateParams,
    ) -> Result<MealPlanTemplate, CreateMealPlanTemplateError> {
        let row: MealPlanTemplateRow = sqlx::query_as(
            r#"
            INSERT INTO meal_plan_templates (user_id, name, day_of_week, meal_slot, recipe_id)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(user_id, day_of_week, meal_slot) DO UPDATE SET
                name = excluded.name,
                recipe_id = excluded.recipe_id,
                updated_at = CURRENT_TIMESTAMP
            RETURNING id, user_id, name, day_of_week, meal_slot, recipe_id,
                (SELECT name FROM recipes WHERE id = recipe_id) as recipe_name
            "#,
        )
        .bind(params.user_id)
        .bind(params.name)
        .bind(params.day_of_week)
        .bind(params.meal_slot)
        .bind(params.recipe_id)
        .fetch_one(self)
        .instrument(tracing::info_span!("Upsert Meal Plan Template"))
        .await
        .map_err(|e| CreateMealPlanTemplateError::UnknownDbError(e))?;

        Ok(row.into())
    }
}
