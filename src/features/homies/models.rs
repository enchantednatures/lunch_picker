use thiserror::Error;

#[derive(Debug, PartialEq, Eq)]
pub struct Homie {
    pub id: i32,
    pub name: String,
}

impl Homie {
    pub fn new(id: i32, name: HomiesName) -> Self {
        Self { id, name: name.0 }
    }

    pub fn as_view(&self) -> HomieView {
        HomieView {
            id: self.id,
            name: &self.name,
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

#[derive(Debug)]
pub struct HomieId(i32);

#[derive(Debug)]
pub struct HomiesName(String);

impl HomiesName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl HomiesName {
    fn from_string_unchecked(name: String) -> Self {
        Self(name)
    }
}

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
