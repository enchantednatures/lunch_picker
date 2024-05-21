use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Recipe {
    id: i32,
}

impl PartialEq for Recipe {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Ingredient {
    id: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct RecipeIngrediateSection {
    id: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct RecipeIngredient {
    ingredient_id: i32,
    recipe_id: i32,
}
