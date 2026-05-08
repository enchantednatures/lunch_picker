use std::fmt::Debug;

use sqlx::Pool;
use sqlx::Sqlite;
use thiserror::Error;
use tracing::Instrument;

use crate::user::UserId;

#[tracing::instrument(skip(db))]
pub async fn remove_recipe_tag(
    recipe_id: i32,
    tag_id: i32,
    user_id: impl Into<UserId> + Debug,
    db: &impl RemoveRecipeTag,
) -> Result<(), RemoveRecipeTagError> {
    db.remove_recipe_tag(recipe_id, tag_id, user_id.into()).await?;
    Ok(())
}

#[derive(Error, Debug)]
pub enum RemoveRecipeTagError {
    #[error("Tag not found for this recipe")]
    NotFound,

    #[error("Unknown db error")]
    UnknownDbError(#[from] sqlx::Error),

    #[error("Unknown error")]
    Unknown,
}

pub(crate) trait RemoveRecipeTag {
    async fn remove_recipe_tag(
        &self,
        recipe_id: i32,
        tag_id: i32,
        user_id: UserId,
    ) -> Result<(), RemoveRecipeTagError>;
}

impl RemoveRecipeTag for Pool<Sqlite> {
    #[tracing::instrument(skip(self))]
    async fn remove_recipe_tag(
        &self,
        recipe_id: i32,
        tag_id: i32,
        user_id: UserId,
    ) -> Result<(), RemoveRecipeTagError> {
        let result = sqlx::query(
            r#"
            DELETE FROM recipe_tags
            WHERE recipe_id = ? AND tag_id = ? AND user_id = ?
            "#,
        )
        .bind(recipe_id)
        .bind(tag_id)
        .bind(user_id.as_i32())
        .execute(self)
        .instrument(tracing::info_span!("Remove Recipe Tag"))
        .await
        .map_err(|e| RemoveRecipeTagError::UnknownDbError(e))?;

        if result.rows_affected() == 0 {
            return Err(RemoveRecipeTagError::NotFound);
        }

        Ok(())
    }
}
