#![allow(dead_code)]

use std::collections::HashMap;

use anyhow::Result;
use sqlx::{query_file, query_file_as, Pool, Sqlite, SqlitePool};

pub use domain::*;

pub mod db;
pub mod domain;

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

pub async fn add_homie(db_pool: &SqlitePool, name: &str) -> Result<()> {
    query_file!("src/sql/insert_homie.sql", name)
        .execute(db_pool)
        .await?;
    Ok(())
}

pub async fn get_all_homies(db_pool: &SqlitePool) -> Result<Vec<Homie>> {
    let homies = query_file_as!(Homie, "src/sql/get_all_homies.sql")
        .fetch_all(db_pool)
        .await?;
    Ok(homies)
}

async fn get_recent_meals(db_pool: &SqlitePool) -> Result<Vec<RecentMeal>> {
    let recent_meals = sqlx::query_as!(
        RecentMeal,
        r#"
       SELECT id as "id!", name as "name!"
       FROM recent_meals
       ORDER BY created_at DESC
       LIMIT 5
       "#
    )
    .fetch_all(db_pool)
    .await?;
    Ok(recent_meals)
}

pub async fn get_all_recipes(db_pool: &SqlitePool) -> Vec<Recipe> {
    let recipes = query_file_as!(Recipe, "src/sql/get_all_recipes.sql")
        .fetch_all(db_pool)
        .await
        .unwrap();
    println!("Recipes: {:?}", recipes);
    recipes
}

pub async fn add_recipe(db_pool: &SqlitePool, name: &str) -> Result<(), sqlx::Error> {
    query_file!("src/sql/insert_recipe.sql", name)
        .execute(db_pool)
        .await?;
    Ok(())
}

pub async fn get_favorites_for_home_homie(
    db_pool: &SqlitePool,
    homie: &Homie,
) -> Result<Vec<HomiesFavorite>, sqlx::Error> {
    let homie_id = homie.id;
    let homies_favorites =
        query_file_as!(HomiesFavorite, "src/sql/get_homies_favorites.sql", homie_id)
            .fetch_all(db_pool)
            .await?;
    Ok(homies_favorites)
}

async fn get_recents_for_homies(
    db_pool: &SqlitePool,
    home_homies: &[&Homie],
) -> Result<Vec<Recipe>, sqlx::Error> {
    let home_homies = home_homies
        .iter()
        .map(|h| {
            return h.name.as_str();
        })
        .collect::<Vec<&str>>()
        .join(", ");
    // TODO: this should return each homie's five most recent favorites, not the five most recent favorites of all homies
    let recipes = sqlx::query_as!(
        Recipe,
        r#"
       SELECT recipes.id as "id!", recipes.name as "name!"
       FROM recipes
       JOIN homies_favorites ON recipes.id = homies_favorites.recipe_id
       JOIN homies ON homies_favorites.homie_id = homies.id
       WHERE homies.name IN (?)
       ORDER BY recipes.created_at DESC
       LIMIT 5
       "#,
        home_homies
    )
    .fetch_all(db_pool)
    .await?;
    Ok(recipes)
}

pub async fn get_recipe(pool: &Pool<Sqlite>, recipe_id: &i64) -> Recipe {
    let recipe = sqlx::query_as!(
        Recipe,
        "SELECT id, name FROM recipes WHERE id = ?",
        recipe_id
    )
    .fetch_one(pool)
    .await
    .unwrap();
    println!("{:?}", recipe);
    recipe
}

pub fn check_if_file_exists(path: &str) -> bool {
    return std::path::Path::new(path).exists();
}

pub static CONFIG_FILE: &str = "./.shitty_lunch_picker.config";
