use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ParsedRecipe {
    pub name: String,
    pub description: Option<String>,
    pub ingredients: Vec<String>,
    pub instructions: Vec<String>,
    pub prep_time_minutes: Option<i32>,
    pub cook_time_minutes: Option<i32>,
    pub servings: Option<i32>,
    pub source_url: String,
}

#[derive(Debug, Deserialize)]
struct SchemaOrgRecipe {
    name: Option<String>,
    description: Option<String>,
    recipeIngredient: Option<Vec<String>>,
    recipeInstructions: Option<SchemaInstructions>,
    prepTime: Option<String>,
    cookTime: Option<String>,
    totalTime: Option<String>,
    recipeYield: Option<String>,
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum SchemaInstructions {
    String(String),
    Strings(Vec<String>),
    Steps(Vec<SchemaStep>),
}

#[derive(Debug, Deserialize)]
struct SchemaStep {
    #[serde(rename = "@type")]
    type_: Option<String>,
    text: Option<String>,
}

pub fn extract_recipe_from_html(html: &str, fallback_url: &str) -> Option<ParsedRecipe> {
    let document = scraper::Html::parse_document(html);
    let selector = scraper::Selector::parse(r#"script[type="application/ld+json"]"#).ok()?;

    for element in document.select(&selector) {
        let json_text = element.text().collect::<String>();
        let json_text = json_text.trim();

        if let Some(recipe) = try_parse_json_ld(json_text, fallback_url) {
            return Some(recipe);
        }
    }

    None
}

fn try_parse_json_ld(json_text: &str, fallback_url: &str) -> Option<ParsedRecipe> {
    let value: Value = serde_json::from_str(json_text).ok()?;

    let recipes = if let Some(graph) = value.get("@graph").and_then(|g| g.as_array()) {
        graph.iter().collect::<Vec<_>>()
    } else {
        vec![&value]
    };

    for item in recipes {
        let type_field = item.get("@type");
        let is_recipe = match type_field {
            Some(Value::String(s)) => s == "Recipe",
            Some(Value::Array(arr)) => arr.iter().any(|v| v.as_str() == Some("Recipe")),
            _ => false,
        };

        if !is_recipe {
            continue;
        }

        let recipe: SchemaOrgRecipe = serde_json::from_value(item.clone()).ok()?;

        let name = recipe.name?;
        let ingredients = recipe.recipeIngredient.unwrap_or_default();
        let instructions = parse_instructions(recipe.recipeInstructions);
        let prep_time = recipe.prepTime.as_deref().and_then(parse_iso_duration);
        let cook_time = recipe.cookTime.as_deref().and_then(parse_iso_duration);
        let servings = recipe.recipeYield.as_deref().and_then(parse_servings);
        let source_url = recipe.url.unwrap_or_else(|| fallback_url.to_string());

        return Some(ParsedRecipe {
            name,
            description: recipe.description,
            ingredients,
            instructions,
            prep_time_minutes: prep_time,
            cook_time_minutes: cook_time,
            servings,
            source_url,
        });
    }

    None
}

fn parse_instructions(instructions: Option<SchemaInstructions>) -> Vec<String> {
    match instructions {
        Some(SchemaInstructions::String(s)) => vec![s],
        Some(SchemaInstructions::Strings(arr)) => arr,
        Some(SchemaInstructions::Steps(steps)) => {
            steps.into_iter().filter_map(|s| s.text).collect()
        }
        None => Vec::new(),
    }
}

fn parse_iso_duration(duration: &str) -> Option<i32> {
    if !duration.starts_with("PT") {
        return None;
    }

    let duration = &duration[2..];
    let mut minutes = 0i32;

    let hours = extract_number_before_char(duration, 'H');
    minutes += hours.unwrap_or(0) * 60;

    let mins = extract_number_before_char(duration, 'M');
    minutes += mins.unwrap_or(0);

    if minutes > 0 {
        Some(minutes)
    } else {
        None
    }
}

fn extract_number_before_char(s: &str, c: char) -> Option<i32> {
    let parts: Vec<&str> = s.split(c).collect();
    if parts.len() < 2 {
        return None;
    }
    parts[0].parse::<i32>().ok()
}

fn parse_servings(yield_str: &str) -> Option<i32> {
    let digits: String = yield_str.chars().filter(|c| c.is_ascii_digit()).collect();
    digits.parse::<i32>().ok()
}
