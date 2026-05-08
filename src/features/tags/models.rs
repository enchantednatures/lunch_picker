use serde::Serialize;
use sqlx::FromRow;
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Tag {
    pub id: TagId,
    pub name: TagName,
}

#[derive(Debug, PartialEq, Eq, FromRow)]
pub struct TagRow {
    pub id: i32,
    pub name: String,
}

impl From<TagRow> for Tag {
    fn from(row: TagRow) -> Self {
        Self {
            id: TagId(row.id),
            name: TagName::from_string_unchecked(row.name),
        }
    }
}

#[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct TagId(i32);

impl TagId {
    pub fn as_i32(&self) -> i32 {
        self.0
    }
}

impl From<i32> for TagId {
    fn from(id: i32) -> Self {
        TagId(id)
    }
}

#[derive(Debug, Serialize, PartialEq, Eq, Clone)]
pub struct TagName(String);

impl TagName {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn from_string_unchecked(name: String) -> Self {
        Self(name)
    }
}

impl TryFrom<String> for TagName {
    type Error = TagNameValidationError;

    fn try_from(name: String) -> Result<Self, Self::Error> {
        let tr = name.trim();
        if tr.is_empty() {
            Err(TagNameValidationError::EmptyName)
        } else {
            Ok(TagName(tr.to_string()))
        }
    }
}

#[derive(Error, Debug)]
pub enum TagNameValidationError {
    #[error("No name provided")]
    EmptyName,
}
