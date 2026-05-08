use std::fmt::Debug;

use sqlx::Pool;
use sqlx::Sqlite;
use thiserror::Error;
use tracing::Instrument;

use super::models::{Tag, TagName, TagNameValidationError, TagRow};

#[tracing::instrument(skip(db))]
pub async fn create_tag(
    tag_name: impl TryInto<TagName, Error = TagNameValidationError> + Debug,
    db: &impl CreateTag,
) -> Result<Tag, CreateTagError> {
    let tag_name: TagName = tag_name.try_into()?;
    let tag = db.create_tag(tag_name.as_str()).await?;
    Ok(tag)
}

#[derive(Error, Debug)]
pub enum CreateTagError {
    #[error(transparent)]
    ValidationError(#[from] TagNameValidationError),

    #[error("Tag already exists")]
    TagAlreadyExists,

    #[error("Unknown db error")]
    UnknownDbError(#[from] sqlx::Error),

    #[error("Unknown error")]
    Unknown,
}

pub(crate) trait CreateTag {
    async fn create_tag(&self, name: &str) -> Result<Tag, CreateTagError>;
}

impl CreateTag for Pool<Sqlite> {
    #[tracing::instrument(skip(self))]
    async fn create_tag(&self, name: &str) -> Result<Tag, CreateTagError> {
        let row: TagRow = sqlx::query_as(
            r#"INSERT INTO tags (name) VALUES (?) RETURNING id, name"#,
        )
        .bind(name)
        .fetch_one(self)
        .instrument(tracing::info_span!("Insert Tag"))
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(db_error) => {
                if db_error.is_unique_violation() {
                    return CreateTagError::TagAlreadyExists;
                }
                CreateTagError::UnknownDbError(sqlx::Error::Database(db_error))
            }
            _ => CreateTagError::UnknownDbError(e),
        })?;

        Ok(row.into())
    }
}
