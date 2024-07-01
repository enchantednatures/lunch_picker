use sqlx::prelude::FromRow;
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, FromRow)]
pub struct RestaurantRow {
    id: i32,
    user_id: i32,
    name: String,
}

impl RestaurantRow {
    pub fn name(&self) -> String {
        self.name.clone()
    }
}

impl From<RestaurantRow> for Restaurant {
    fn from(row: RestaurantRow) -> Self {
        Self::new_unchecked(row.id, row.name)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Restaurant {
    pub id: RestaurantId,
    pub name: RestaurantName,
}

impl Restaurant {
    pub fn new(id: impl Into<RestaurantId>, name: impl Into<RestaurantName>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
        }
    }

    fn new_unchecked(id: i32, name: String) -> Self {
        Self {
            id: RestaurantId(id),
            name: RestaurantName::from_string_unchecked(name),
        }
    }

    pub fn as_view(&self) -> RestaurantView {
        RestaurantView {
            id: self.id.0,
            name: &self.name.0,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct RestaurantView<'a> {
    id: i32,
    pub name: &'a str,
}

#[derive(Error, Debug)]
pub enum RestaurantNameValidationError {
    #[error("Invalid name for homie: {:?}", name)]
    InvalidName { name: String },

    #[error("No name provided")]
    EmptyName,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct RestaurantId(i32);

impl From<i32> for RestaurantId {
    fn from(id: i32) -> Self {
        RestaurantId(id)
    }
}

impl From<&i32> for RestaurantId {
    fn from(id: &i32) -> Self {
        RestaurantId(*id)
    }
}

impl RestaurantId {
    pub fn as_i32(&self) -> &i32 {
        &self.0
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RestaurantName(String);

impl RestaurantName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl RestaurantName {
    fn from_string_unchecked(name: String) -> Self {
        Self(name)
    }
}

impl TryFrom<String> for RestaurantName {
    type Error = RestaurantNameValidationError;

    fn try_from(name: String) -> Result<Self, Self::Error> {
        let tr = name.trim();
        match tr.is_empty() {
            true => Err(RestaurantNameValidationError::EmptyName),
            false => Ok(RestaurantName(tr.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{RestaurantName, RestaurantNameValidationError};

    #[test]
    fn homie_name_validation_fails_on_empty_string() {
        let h: Result<RestaurantName, RestaurantNameValidationError> =
            "    ".to_string().try_into();
        assert_eq!(
            RestaurantNameValidationError::EmptyName.to_string(),
            h.unwrap_err().to_string()
        );
    }

    #[test]
    fn valid_names_are_valid() {
        let h: Result<RestaurantName, RestaurantNameValidationError> = "Bob".to_string().try_into();
        assert_eq!("Bob", h.unwrap().as_str());
    }
}
