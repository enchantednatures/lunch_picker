use sqlx::Pool;
use sqlx::Sqlite;
use thiserror::Error;
use tracing::Instrument;

use super::models::{MealPlanTemplate, MealPlanTemplateRow};
use crate::user::UserId;

#[tracing::instrument(skip(db))]
pub async fn list_meal_plan_templates(
    user_id: impl Into<UserId> + std::fmt::Debug,
    db: &impl ListMealPlanTemplates,
) -> Result<Vec<MealPlanTemplate>, ListMealPlanTemplatesError> {
    let templates = db.list_meal_plan_templates(user_id.into()).await?;
    Ok(templates.into_iter().map(|t| t.into()).collect())
}

#[derive(Error, Debug)]
pub enum ListMealPlanTemplatesError {
    #[error("Unknown db error")]
    UnknownDbError(#[from] sqlx::Error),

    #[error("Unknown error")]
    Unknown,
}

pub(crate) trait ListMealPlanTemplates {
    async fn list_meal_plan_templates(
        &self,
        user_id: UserId,
    ) -> Result<Vec<MealPlanTemplateRow>, ListMealPlanTemplatesError>;
}

impl ListMealPlanTemplates for Pool<Sqlite> {
    #[tracing::instrument(skip(self))]
    async fn list_meal_plan_templates(
        &self,
        user_id: UserId,
    ) -> Result<Vec<MealPlanTemplateRow>, ListMealPlanTemplatesError> {
        let rows = sqlx::query_as::<_, MealPlanTemplateRow>(
            r#"
            SELECT
                mpt.id,
                mpt.user_id,
                mpt.name,
                mpt.day_of_week,
                mpt.meal_slot,
                mpt.recipe_id,
                r.name as recipe_name
            FROM meal_plan_templates mpt
            JOIN recipes r ON r.id = mpt.recipe_id
            WHERE mpt.user_id = ?
            ORDER BY mpt.day_of_week, mpt.meal_slot
            "#,
        )
        .bind(user_id.as_i32())
        .fetch_all(self)
        .instrument(tracing::info_span!("List Meal Plan Templates"))
        .await
        .map_err(|e| ListMealPlanTemplatesError::UnknownDbError(e))?;

        Ok(rows)
    }
}
