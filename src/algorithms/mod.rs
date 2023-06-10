use std::collections::HashMap;

use crate::models::db_rows::HomiesFavorite;

pub async fn get_most_favorited_recipes(homies_favorites: &[HomiesFavorite]) -> Vec<&i64> {
    let mut recipe_counts = HashMap::<&i64, u32>::new();
    for homies_favorite in homies_favorites.iter() {
        let recipe = &homies_favorite.recipe_id;
        let count = recipe_counts.entry(recipe).or_insert(0);
        *count += 1;
    }
    let mut recipe_counts = recipe_counts.into_iter().collect::<Vec<(&i64, u32)>>();
    recipe_counts.sort_by(|a, b| b.1.cmp(&a.1));
    let max_favorite = recipe_counts.iter().fold(0, |acc, (_, count)| {
        if acc < *count {
            return *count;
        }
        acc
    });
    let most_favorited_recipes = recipe_counts
        .iter()
        .filter(|(_, count)| &max_favorite == count)
        .map(|(recipe, _)| *recipe)
        .collect();
    most_favorited_recipes
}
