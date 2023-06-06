#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]
#![allow(dead_code)]

use std::fs;
use std::fs::File;
use std::io::Write;

use dialoguer::{Input, MultiSelect, Select};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{migrate, query_file, query_file_as, SqlitePool};

use models::db_rows::Homie;
use models::db_rows::HomiesFavorite;
use models::db_rows::RecentMeal;
use models::db_rows::Recipe;

mod models;

async fn add_homie(db_pool: &SqlitePool, name: &str) -> Result<(), sqlx::Error> {
    sqlx::query_file!("src/sql/insert_homie.sql", name)
        .execute(db_pool)
        .await?;
    return Ok(());
}

async fn get_all_homies(db_pool: &SqlitePool) -> Result<Vec<Homie>, sqlx::Error> {
    let homies = sqlx::query_file_as!(Homie, "src/sql/get_all_homies.sql")
        .fetch_all(db_pool)
        .await?;
    return Ok(homies);
}

fn get_db_url() -> String {
    dotenv::dotenv().ok();
    return std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
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
    return Ok(recent_meals);
}

async fn get_home_homies(homies: &[Homie]) -> Vec<String> {
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
        return vec![];
    } else {
        println!("Homies selected: {:?}", chosen);
    }
    let home_homies = chosen
        .iter()
        .map(|&index| {
            return homies_names[index].to_string();
        })
        .collect();
    return home_homies;
}

async fn get_user_input_homies_favorites(
    db_pool: &SqlitePool,
    homies: &[Homie],
    recipes: &[Recipe],
) -> Result<(), sqlx::Error> {
    let selected_homie_idx = Select::new()
        .with_prompt("Who's favorites are you adding?")
        .items(
            &homies
                .iter()
                .map(|h| {
                    return h.name.as_str();
                })
                .collect::<Vec<&str>>(),
        )
        .interact()
        .unwrap();
    let current_homie = &homies[selected_homie_idx];

    let homies_favorites = query_file_as!(
        HomiesFavorite,
        "src/sql/get_homies_favorites.sql",
        current_homie.id
    )
    .fetch_all(db_pool)
    .await?;

    let homies_favorites_ids = homies_favorites
        .iter()
        .map(|hf| {
            return hf.recipe_id;
        })
        .collect::<Vec<i64>>();

    let is_favorite_map: Vec<bool> = recipes
        .iter()
        .map(|x| {
            return homies_favorites_ids.contains(&x.id);
        })
        .collect();

    let input = MultiSelect::new()
        .with_prompt("What are {}'s favorites?")
        .items(
            &recipes
                .iter()
                .map(|r| {
                    return r.name.as_str();
                })
                .collect::<Vec<&str>>(),
        )
        .defaults(&is_favorite_map)
        .interact()
        .unwrap();

    let new_favorites = input
        .iter()
        .map(|&index| {
            return recipes[index].id;
        })
        .collect::<Vec<i64>>();

    query_file!("src/sql/delete_homies_favorites.sql", current_homie.id)
        .execute(db_pool)
        .await?;

    for recipe_id in new_favorites.iter() {
        query_file!(
            "src/sql/insert_homies_favorites.sql",
            current_homie.id,
            recipe_id
        )
        .execute(db_pool)
        .await?;
    }
    return Ok(());
}

async fn setup(db_pool: &SqlitePool) {
    let mut input = Input::<String>::new()
        .with_prompt("Enter homie's name")
        .default("".into())
        .interact_text()
        .unwrap();

    while !input.is_empty() {
        println!("Adding homie: {}", input);
        add_homie(db_pool, &input).await.unwrap();
        input = Input::<String>::new()
            .with_prompt("Add another homie? (leave blank to finish)")
            .default("".into())
            .interact()
            .unwrap();

        println!("Added homie: {}", input);
    }

    let all_homies = get_all_homies(db_pool).await.unwrap();
    let recipes = get_all_recipes(db_pool).await;
    get_user_input_homies_favorites(db_pool, &all_homies, &recipes)
        .await
        .unwrap();
}

async fn get_all_recipes(db_pool: &SqlitePool) -> Vec<Recipe> {
    let recipes = sqlx::query_file_as!(Recipe, "src/sql/get_all_recipes.sql")
        .fetch_all(db_pool)
        .await
        .unwrap();
    println!("Recipes: {:?}", recipes);
    return recipes;
}

fn check_if_file_exists(path: &str) -> bool {
    return std::path::Path::new(path).exists();
}

async fn add_recipe(db_pool: &SqlitePool, name: &str) -> Result<(), sqlx::Error> {
    sqlx::query_file!("src/sql/insert_recipe.sql", name)
        .execute(db_pool)
        .await?;
    return Ok(());
}

async fn get_favorites_for_home_homies(
    db_pool: &SqlitePool,
    home_homies: &[Homie],
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
    return Ok(recipes);
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let mut is_setup = false;
    let config_file_path = "./.shitty_lunch_picker.config";
    let mut db_url = String::new();

    if !check_if_file_exists(config_file_path) {
        println!("Config file doesn't exist");
        let mut file = File::create(config_file_path).unwrap();
        db_url = Input::<String>::new()
            .with_prompt("Enter database url")
            .default("./.shitty_lunch_picker.db".into())
            .interact()
            .unwrap();
        file.write_all(db_url.as_bytes()).unwrap();
    }

    if db_url.is_empty() {
        db_url = fs::read_to_string(config_file_path).expect("Failed to read config file");
    }
    if !check_if_file_exists(&db_url) {
        println!("Database file doesn't exist");
        File::create(&db_url).unwrap();
    } else {
        is_setup = true;
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    if !is_setup {
        migrate!("./migrations").run(&pool).await?;
        setup(&pool).await;
    }

    let homies = get_all_homies(&pool).await?;
    let _home_homies = get_home_homies(&homies).await;

    let recent = get_recent_meals(&pool).await?;
    println!("recents: {:?}", recent);
    Ok(())
}
