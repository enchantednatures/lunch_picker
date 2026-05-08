use std::fmt::Debug;

use sqlx::Pool;
use sqlx::Sqlite;
use thiserror::Error;
use tracing::Instrument;

use crate::user::UserId;

#[tracing::instrument(skip(db))]
pub async fn delete_meal_plan_template(
    template_id: i32,
    user_id: impl Into<UserId> + Debug,
    db: &impl DeleteMealPlanTemplate,
) -> Result<(), DeleteMealPlanTemplateError> {
    db.delete_meal_plan_template(template_id, user_id.into()).await?;
    Ok(())
}

#[derive(Error, Debug)]
pub enum DeleteMealPlanTemplateError {
    #[error("Template not found")]
    NotFound,

    #[error("Unknown db error")]
    UnknownDbError(#[from] sqlx::Error),

    #[error("Unknown error")]
    Unknown,
}

pub(crate) trait DeleteMealPlanTemplate {
    async fn delete_meal_plan_template(
        &self,
        template_id: i32,
        user_id: UserId,
    ) -> Result<(), DeleteMealPlanTemplateError>;
}

impl DeleteMealPlanTemplate for Pool<Sqlite> {
    #[tracing::instrument(skip(self))]
    async fn delete_meal_plan_template(
        &self,
        template_id: i32,
        user_id: UserId,
    ) -> Result<(), DeleteMealPlanTemplateError> {
        let result = sqlx::query(
            r#"DELETE FROM meal_plan_templates WHERE id = ? AND user_id = ?"#,
        )
        .bind(template_id)
        .bind(user_id.as_i32())
        .execute(self)
        .instrument(tracing::info_span!("Delete Meal Plan Template"))
        .await
        .map_err(|e| DeleteMealPlanTemplateError::UnknownDbError(e))?;

        if result.rows_affected() == 0 {
            return Err(DeleteMealPlanTemplateError::NotFound);
        }

        Ok(())
    }
}
