use std::fmt::Debug;

use sqlx::Pool;
use sqlx::Sqlite;
use thiserror::Error;
use tracing::Instrument;

use crate::user::UserId;

#[tracing::instrument(skip(db))]
pub async fn add_recipe_tag(
    recipe_id: i32,
    tag_id: i32,
    user_id: impl Into<UserId> + Debug,
    db: &impl AddRecipeTag,
) -> Result<(), AddRecipeTagError> {
    db.add_recipe_tag(recipe_id, tag_id, user_id.into()).await?;
    Ok(())
}

#[derive(Error, Debug)]
pub enum AddRecipeTagError {
    #[error("Tag already assigned to this recipe")]
    AlreadyExists,

    #[error("Unknown db error")]
    UnknownDbError(#[from] sqlx::Error),

    #[error("Unknown error")]
    Unknown,
}

pub(crate) trait AddRecipeTag {
    async fn add_recipe_tag(
        &self,
        recipe_id: i32,
        tag_id: i32,
        user_id: UserId,
    ) -> Result<(), AddRecipeTagError>;
}

impl AddRecipeTag for Pool<Sqlite> {
    #[tracing::instrument(skip(self))]
    async fn add_recipe_tag(
        &self,
        recipe_id: i32,
        tag_id: i32,
        user_id: UserId,
    ) -> Result<(), AddRecipeTagError> {
        sqlx::query(
            r#"
            INSERT INTO recipe_tags (recipe_id, tag_id, user_id)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(recipe_id)
        .bind(tag_id)
        .bind(user_id.as_i32())
        .execute(self)
        .instrument(tracing::info_span!("Insert Recipe Tag"))
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(db_error) => {
                if db_error.is_unique_violation() {
                    return AddRecipeTagError::AlreadyExists;
                }
                AddRecipeTagError::UnknownDbError(sqlx::Error::Database(db_error))
            }
            _ => AddRecipeTagError::UnknownDbError(e),
        })?;

        Ok(())
    }
}
