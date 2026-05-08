use regex::Regex;

use crate::features::ingredients::{Measure, MeasureValidationError};

#[derive(Debug, PartialEq)]
pub struct ParsedIngredient {
    pub quantity: f64,
    pub measure: Measure,
    pub name: String,
}

pub fn parse_ingredient(input: &str) -> Result<ParsedIngredient, String> {
    let input = input.trim();
    if input.is_empty() {
        return Err("Empty ingredient string".to_string());
    }

    let re = Regex::new(
        r"^(?:(?P<quantity>(?:\d+\s+)?(?:\d+/\d+|\d*\.?\d+)))?\s*(?:(?P<measure>cup|tbsp|tsp|oz|lb|g|kg|ml|l|each|qty|count)s?\b\.?\s+)?(?P<name>.+)$"
    ).map_err(|e| e.to_string())?;

    if let Some(caps) = re.captures(input) {
        let quantity_str = caps.name("quantity").map(|m| m.as_str()).unwrap_or("1");
        let quantity = parse_quantity(quantity_str)?;

        let measure_str = caps.name("measure").map(|m| m.as_str()).unwrap_or("each");
        let measure =
            Measure::try_from(measure_str).map_err(|e: MeasureValidationError| e.to_string())?;

        let name = caps
            .name("name")
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_else(|| input.to_string());

        if name.is_empty() {
            return Err("Empty ingredient name after parsing".to_string());
        }

        Ok(ParsedIngredient {
            quantity,
            measure,
            name,
        })
    } else {
        Err(format!("Could not parse ingredient: {}", input))
    }
}

fn parse_quantity(s: &str) -> Result<f64, String> {
    let s = s.trim();
    if s.is_empty() {
        return Ok(1.0);
    }

    if let Some(frac) = parse_fraction(s) {
        return Ok(frac);
    }

    s.parse::<f64>()
        .map_err(|_| format!("Invalid quantity: {}", s))
}

fn parse_fraction(s: &str) -> Option<f64> {
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() == 2 {
        let num = parts[0].trim().parse::<f64>().ok()?;
        let den = parts[1].trim().parse::<f64>().ok()?;
        if den != 0.0 {
            return Some(num / den);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_ingredient() {
        let result = parse_ingredient("2 cups flour").unwrap();
        assert_eq!(result.quantity, 2.0);
        assert_eq!(result.measure, Measure::Cup);
        assert_eq!(result.name, "flour");
    }

    #[test]
    fn test_parse_with_fraction() {
        let result = parse_ingredient("1/2 tsp salt").unwrap();
        assert_eq!(result.quantity, 0.5);
        assert_eq!(result.measure, Measure::Tsp);
        assert_eq!(result.name, "salt");
    }

    #[test]
    fn test_parse_without_quantity() {
        let result = parse_ingredient("tbsp olive oil").unwrap();
        assert_eq!(result.quantity, 1.0);
        assert_eq!(result.measure, Measure::Tbsp);
        assert_eq!(result.name, "olive oil");
    }

    #[test]
    fn test_parse_plural_measure() {
        let result = parse_ingredient("3 eggs").unwrap();
        assert_eq!(result.quantity, 3.0);
        assert_eq!(result.measure, Measure::Each);
        assert_eq!(result.name, "eggs");
    }
}
