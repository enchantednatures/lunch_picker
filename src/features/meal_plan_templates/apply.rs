use sqlx::Pool;
use sqlx::Sqlite;
use thiserror::Error;
use tracing::Instrument;

use chrono::Datelike;

use super::models::MealPlanTemplate;
use crate::user::UserId;

#[tracing::instrument(skip(db))]
pub async fn apply_meal_plan_templates(
    user_id: impl Into<UserId> + std::fmt::Debug,
    start_date: chrono::NaiveDate,
    end_date: chrono::NaiveDate,
    db: &impl ApplyMealPlanTemplates,
) -> Result<Vec<MealPlanTemplate>, ApplyMealPlanTemplatesError> {
    let applied = db
        .apply_meal_plan_templates(user_id.into(), start_date, end_date)
        .await?;
    Ok(applied)
}

#[derive(Error, Debug)]
pub enum ApplyMealPlanTemplatesError {
    #[error("Unknown db error")]
    UnknownDbError(#[from] sqlx::Error),

    #[error("Unknown error")]
    Unknown,
}

pub(crate) trait ApplyMealPlanTemplates {
    async fn apply_meal_plan_templates(
        &self,
        user_id: UserId,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    ) -> Result<Vec<MealPlanTemplate>, ApplyMealPlanTemplatesError>;
}

impl ApplyMealPlanTemplates for Pool<Sqlite> {
    #[tracing::instrument(skip(self))]
    async fn apply_meal_plan_templates(
        &self,
        user_id: UserId,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    ) -> Result<Vec<MealPlanTemplate>, ApplyMealPlanTemplatesError> {
        let mut current = start_date;
        let mut applied = Vec::new();

        while current <= end_date {
            let day_of_week = current.weekday().num_days_from_sunday() as i32;

            let templates = sqlx::query_as::<_, super::models::MealPlanTemplateRow>(
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
                WHERE mpt.user_id = ? AND mpt.day_of_week = ?
                "#,
            )
            .bind(user_id.as_i32())
            .bind(day_of_week)
            .fetch_all(self)
            .instrument(tracing::info_span!("Get Templates For Day"))
            .await
            .map_err(|e| ApplyMealPlanTemplatesError::UnknownDbError(e))?;

            for template in templates {
                let exists: bool = sqlx::query_scalar(
                    r#"
                    SELECT EXISTS(
                        SELECT 1 FROM meal_plan_entries
                        WHERE user_id = ? AND date = ? AND meal_slot = ?
                    )
                    "#,
                )
                .bind(user_id.as_i32())
                .bind(current)
                .bind(&template.meal_slot)
                .fetch_one(self)
                .await
                .map_err(|e| ApplyMealPlanTemplatesError::UnknownDbError(e))?;

                if !exists {
                    sqlx::query(
                        r#"
                        INSERT INTO meal_plan_entries (user_id, recipe_id, date, meal_slot)
                        VALUES (?, ?, ?, ?)
                        "#,
                    )
                    .bind(user_id.as_i32())
                    .bind(template.recipe_id)
                    .bind(current)
                    .bind(&template.meal_slot)
                    .execute(self)
                    .await
                    .map_err(|e| ApplyMealPlanTemplatesError::UnknownDbError(e))?;

                    applied.push(template.into());
                }
            }

            current = current.succ_opt().unwrap_or(current);
        }

        Ok(applied)
    }
}
