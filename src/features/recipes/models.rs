use serde::Serialize;
use sqlx::FromRow;
use thiserror::Error;

#[derive(Debug, PartialEq, Serialize)]
pub struct Recipe {
    pub id: RecipeId,
    pub user_id: i32,
    pub name: RecipeName,
    pub description: Option<String>,
    pub prep_time: Option<i32>,
    pub cook_time: Option<i32>,
    pub servings: Option<i32>,
    pub source_url: Option<String>,
    pub imported_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, PartialEq, FromRow)]
pub struct RecipeRow {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub prep_time: Option<i32>,
    pub cook_time: Option<i32>,
    pub servings: Option<i32>,
    pub source_url: Option<String>,
    pub imported_at: Option<chrono::NaiveDateTime>,
}

impl From<RecipeRow> for Recipe {
    fn from(row: RecipeRow) -> Self {
        Self {
            id: RecipeId(row.id),
            user_id: row.user_id,
            name: RecipeName::from_string_unchecked(row.name),
            description: row.description,
            prep_time: row.prep_time,
            cook_time: row.cook_time,
            servings: row.servings,
            source_url: row.source_url,
            imported_at: row.imported_at,
        }
    }
}

impl Recipe {
    pub fn new(id: impl Into<RecipeId>, user_id: i32, name: impl Into<RecipeName>) -> Self {
        Self {
            id: id.into(),
            user_id,
            name: name.into(),
            description: None,
            prep_time: None,
            cook_time: None,
            servings: None,
            source_url: None,
            imported_at: None,
        }
    }
}

#[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct RecipeId(i32);

impl RecipeId {
    pub fn as_i32(&self) -> i32 {
        self.0
    }
}

impl From<i32> for RecipeId {
    fn from(id: i32) -> Self {
        RecipeId(id)
    }
}

impl From<&i32> for RecipeId {
    fn from(id: &i32) -> Self {
        RecipeId(*id)
    }
}

#[derive(Debug, Serialize, PartialEq, Eq, Clone)]
pub struct RecipeName(String);

impl RecipeName {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn from_string_unchecked(name: String) -> Self {
        Self(name)
    }
}

impl TryFrom<String> for RecipeName {
    type Error = RecipeNameValidationError;

    fn try_from(name: String) -> Result<Self, Self::Error> {
        let tr = name.trim();
        if tr.is_empty() {
            Err(RecipeNameValidationError::EmptyName)
        } else {
            Ok(RecipeName(tr.to_string()))
        }
    }
}

#[derive(Error, Debug)]
pub enum RecipeNameValidationError {
    #[error("No name provided")]
    EmptyName,
}
