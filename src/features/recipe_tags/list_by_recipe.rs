use sqlx::Pool;
use sqlx::Sqlite;
use thiserror::Error;
use tracing::Instrument;

use super::models::RecipeTag;

#[tracing::instrument(skip(db))]
pub async fn list_recipe_tags(
    recipe_id: i32,
    db: &impl ListRecipeTags,
) -> Result<Vec<RecipeTag>, ListRecipeTagsError> {
    let tags = db.list_recipe_tags(recipe_id).await?;
    Ok(tags)
}

#[derive(Error, Debug)]
pub enum ListRecipeTagsError {
    #[error("Unknown db error")]
    UnknownDbError(#[from] sqlx::Error),

    #[error("Unknown error")]
    Unknown,
}

pub(crate) trait ListRecipeTags {
    async fn list_recipe_tags(
        &self,
        recipe_id: i32,
    ) -> Result<Vec<RecipeTag>, ListRecipeTagsError>;
}

impl ListRecipeTags for Pool<Sqlite> {
    #[tracing::instrument(skip(self))]
    async fn list_recipe_tags(
        &self,
        recipe_id: i32,
    ) -> Result<Vec<RecipeTag>, ListRecipeTagsError> {
        let rows = sqlx::query_as::<_, RecipeTag>(
            r#"
            SELECT rt.recipe_id, rt.tag_id, t.name as tag_name
            FROM recipe_tags rt
            JOIN tags t ON t.id = rt.tag_id
            WHERE rt.recipe_id = ?
            ORDER BY t.name
            "#,
        )
        .bind(recipe_id)
        .fetch_all(self)
        .instrument(tracing::info_span!("List Recipe Tags"))
        .await
        .map_err(|e| ListRecipeTagsError::UnknownDbError(e))?;

        Ok(rows)
    }
}
