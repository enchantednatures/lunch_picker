use dialoguer::MultiSelect;
use sqlx::sqlite::SqliteQueryResult;
use sqlx::{query_file, query_file_as, Pool, Sqlite, SqlitePool};

use crate::api::CreateRecipeRequest;
use crate::models::db_rows::{Homie, HomiesFavorite, RecentMeal, Recipe};

pub async fn add_homie(db_pool: &SqlitePool, name: &str) -> Result<(), sqlx::Error> {
    query_file!("src/sql/insert_homie.sql", name)
        .execute(db_pool)
        .await?;
    Ok(())
}

pub async fn get_all_homies(db_pool: &SqlitePool) -> Result<Vec<Homie>, sqlx::Error> {
    let homies = query_file_as!(Homie, "src/sql/get_all_homies.sql")
        .fetch_all(db_pool)
        .await?;
    Ok(homies)
}

async fn get_recent_meals(db_pool: &SqlitePool) -> Result<Vec<RecentMeal>, sqlx::Error> {
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

pub async fn get_home_homies(homies: &[Homie]) -> Vec<&Homie> {
    let homies_names = homies
        .iter()
        .map(|h| {
            return h.name.as_str();
        })
        .collect::<Vec<&str>>();
    let chosen = MultiSelect::new()
        .with_prompt("Who's home?")
        .items(&homies_names)
        .interact()
        .unwrap();
    if chosen.is_empty() {
        println!("No homies selected");
        return homies.iter().collect();
    } else {
        println!("Homies selected: {:?}", chosen);
    }
    let home_homies = chosen.iter().map(|&index| &homies[index]).collect();
    home_homies
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
    let homies_favorites =
        query_file_as!(HomiesFavorite, "src/sql/get_homies_favorites.sql", homie.id)
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

pub async fn create_recipe(
    recipe: &CreateRecipeRequest,
    db_pool: &SqlitePool,
) -> anyhow::Result<SqliteQueryResult> {
    Ok(query_file!("./src/sql/insert_recipe.sql", recipe.name)
        .execute(db_pool)
        .await?)
}
