use serde::Serialize;
use sqlx::FromRow;
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Homie {
    pub id: HomieId,
    pub name: HomiesName,
}

#[derive(Debug, PartialEq, Eq, FromRow)]
pub struct HomieRow {
    id: i32,
    user_id: i32,
    name: String,
}

impl From<HomieRow> for Homie {
    fn from(value: HomieRow) -> Self {
        Self {
            id: HomieId(value.id),
            name: HomiesName(value.name),
        }
    }
}

impl Homie {
    pub fn new(id: impl Into<HomieId>, name: impl Into<HomiesName>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
        }
    }

    pub fn as_view(&self) -> HomieView {
        HomieView {
            id: self.id.0,
            name: &self.name.0,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct HomieView<'a> {
    id: i32,
    pub name: &'a str,
}

#[derive(Error, Debug)]
pub enum HomieNameValidationError {
    #[error("Invalid name for homie: {:?}", name)]
    InvalidName { name: String },

    #[error("No name provided")]
    EmptyName,
}

#[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct HomieId(i32);

impl HomieId {
    pub fn as_i32(&self) -> i32 {
        self.0
    }
}

#[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct HomiesName(String);

impl HomiesName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// trait TryIntoHomieName: TryInto<HomiesName, Error = HomieNameValidationError> + Debug {}

impl TryFrom<String> for HomiesName {
    type Error = HomieNameValidationError;

    fn try_from(name: String) -> Result<Self, Self::Error> {
        let tr = name.trim();
        match tr.is_empty() {
            true => Err(HomieNameValidationError::EmptyName),
            false => Ok(HomiesName(tr.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{HomieNameValidationError, HomiesName};

    #[test]
    fn homie_name_validation_fails_on_empty_string() {
        let h: Result<HomiesName, HomieNameValidationError> = "    ".to_string().try_into();
        assert_eq!(
            HomieNameValidationError::EmptyName.to_string(),
            h.unwrap_err().to_string()
        );
    }

    #[test]
    fn valid_names_are_valid() {
        let h: Result<HomiesName, HomieNameValidationError> = "Bob".to_string().try_into();
        assert_eq!("Bob", h.unwrap().as_str());
    }
}
