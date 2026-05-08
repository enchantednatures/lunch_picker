use serde::Serialize;
use sqlx::FromRow;
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Ingredient {
    pub id: IngredientId,
    pub name: IngredientName,
}

#[derive(Debug, PartialEq, Eq, FromRow)]
pub struct IngredientRow {
    id: i32,
    name: String,
}

impl From<IngredientRow> for Ingredient {
    fn from(value: IngredientRow) -> Self {
        Self {
            id: IngredientId(value.id),
            name: IngredientName(value.name),
        }
    }
}

impl Ingredient {
    pub fn new(id: impl Into<IngredientId>, name: impl Into<IngredientName>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
        }
    }
}

#[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct IngredientId(i32);

impl IngredientId {
    pub fn as_i32(&self) -> i32 {
        self.0
    }
}

impl From<i32> for IngredientId {
    fn from(id: i32) -> Self {
        IngredientId(id)
    }
}

impl From<&i32> for IngredientId {
    fn from(id: &i32) -> Self {
        IngredientId(*id)
    }
}

#[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct IngredientName(String);

impl IngredientName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for IngredientName {
    type Error = IngredientNameValidationError;

    fn try_from(name: String) -> Result<Self, Self::Error> {
        let tr = name.trim();
        if tr.is_empty() {
            Err(IngredientNameValidationError::EmptyName)
        } else {
            Ok(IngredientName(tr.to_string()))
        }
    }
}

#[derive(Error, Debug)]
pub enum IngredientNameValidationError {
    #[error("No name provided")]
    EmptyName,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Measure {
    Cup,
    Tbsp,
    Tsp,
    Oz,
    Lb,
    G,
    Kg,
    Ml,
    L,
    Each,
    Qty,
    Count,
}

impl Measure {
    pub fn as_str(&self) -> &'static str {
        match self {
            Measure::Cup => "cup",
            Measure::Tbsp => "tbsp",
            Measure::Tsp => "tsp",
            Measure::Oz => "oz",
            Measure::Lb => "lb",
            Measure::G => "g",
            Measure::Kg => "kg",
            Measure::Ml => "ml",
            Measure::L => "l",
            Measure::Each => "each",
            Measure::Qty => "qty",
            Measure::Count => "count",
        }
    }
}

impl TryFrom<String> for Measure {
    type Error = MeasureValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "cup" | "cups" => Ok(Measure::Cup),
            "tbsp" | "tablespoon" | "tablespoons" => Ok(Measure::Tbsp),
            "tsp" | "teaspoon" | "teaspoons" => Ok(Measure::Tsp),
            "oz" | "ounce" | "ounces" => Ok(Measure::Oz),
            "lb" | "pound" | "pounds" | "lbs" => Ok(Measure::Lb),
            "g" | "gram" | "grams" => Ok(Measure::G),
            "kg" | "kilogram" | "kilograms" => Ok(Measure::Kg),
            "ml" | "milliliter" | "milliliters" => Ok(Measure::Ml),
            "l" | "liter" | "liters" | "litre" | "litres" => Ok(Measure::L),
            "each" => Ok(Measure::Each),
            "qty" | "quantity" => Ok(Measure::Qty),
            "count" => Ok(Measure::Count),
            _ => Err(MeasureValidationError::InvalidMeasure(value)),
        }
    }
}

impl TryFrom<&str> for Measure {
    type Error = MeasureValidationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.to_string().try_into()
    }
}

#[derive(Error, Debug)]
pub enum MeasureValidationError {
    #[error("Invalid measure: {0}")]
    InvalidMeasure(String),
}

#[cfg(test)]
mod tests {
    use super::{IngredientName, IngredientNameValidationError, Measure};

    #[test]
    fn ingredient_name_validation_fails_on_empty_string() {
        let h: Result<IngredientName, IngredientNameValidationError> =
            "    ".to_string().try_into();
        assert_eq!(
            IngredientNameValidationError::EmptyName.to_string(),
            h.unwrap_err().to_string()
        );
    }

    #[test]
    fn valid_ingredient_names_are_valid() {
        let h: Result<IngredientName, IngredientNameValidationError> =
            "Flour".to_string().try_into();
        assert_eq!("Flour", h.unwrap().as_str());
    }

    #[test]
    fn measure_parsing_works() {
        assert_eq!(Measure::Cup, Measure::try_from("cup").unwrap());
        assert_eq!(Measure::Tbsp, Measure::try_from("tablespoons").unwrap());
        assert_eq!(Measure::G, Measure::try_from("grams").unwrap());
    }
}
