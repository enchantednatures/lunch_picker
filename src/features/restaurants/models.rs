use thiserror::Error;

#[derive(Debug, PartialEq, Eq)]
pub struct Restaurant {
    pub id: i32,
    pub name: String,
}

impl Restaurant {
    pub fn new(id: i32, name: RestaurantName) -> Self {
        Self { id, name: name.0 }
    }

    pub fn as_view(&self) -> RestaurantView {
        RestaurantView {
            id: self.id,
            name: &self.name,
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

#[derive(Debug)]
pub struct RestaurantId(i32);

#[derive(Debug)]
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
