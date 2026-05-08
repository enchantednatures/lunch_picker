use sqlx::Pool;
use sqlx::Sqlite;
use thiserror::Error;
use tracing::Instrument;

use super::models::{Tag, TagRow};

#[tracing::instrument(skip(db))]
pub async fn list_tags(db: &impl ListTags) -> Result<Vec<Tag>, ListTagsError> {
    let tags = db.list_tags().await?;
    Ok(tags.into_iter().map(|t| t.into()).collect())
}

#[derive(Error, Debug)]
pub enum ListTagsError {
    #[error("Unknown db error")]
    UnknownDbError(#[from] sqlx::Error),

    #[error("Unknown error")]
    Unknown,
}

pub(crate) trait ListTags {
    async fn list_tags(&self) -> Result<Vec<TagRow>, ListTagsError>;
}

impl ListTags for Pool<Sqlite> {
    #[tracing::instrument(skip(self))]
    async fn list_tags(&self) -> Result<Vec<TagRow>, ListTagsError> {
        let rows = sqlx::query_as::<_, TagRow>(
            r#"SELECT id, name FROM tags ORDER BY name"#,
        )
        .fetch_all(self)
        .instrument(tracing::info_span!("List Tags"))
        .await
        .map_err(|e| ListTagsError::UnknownDbError(e))?;

        Ok(rows)
    }
}
