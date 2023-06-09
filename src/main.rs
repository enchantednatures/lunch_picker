#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]
#![allow(dead_code)]

use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;

use dialoguer::{Input, MultiSelect, Select};
use rand::seq::SliceRandom;
use sqlx::{migrate, Pool, query_file, query_file_as, Sqlite, SqlitePool};
use sqlx::sqlite::SqlitePoolOptions;

use models::db_rows::Homie;
use models::db_rows::HomiesFavorite;
use models::db_rows::RecentMeal;
use models::db_rows::Recipe;

mod models;

async fn add_homie(db_pool: &SqlitePool, name: &str) -> Result<(), sqlx::Error> {
    query_file!("src/sql/insert_homie.sql", name)
        .execute(db_pool)
        .await?;
    return Ok(());
}

async fn get_all_homies(db_pool: &SqlitePool) -> Result<Vec<Homie>, sqlx::Error> {
    let homies = query_file_as!(Homie, "src/sql/get_all_homies.sql")
        .fetch_all(db_pool)
        .await?;
    return Ok(homies);
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

async fn get_home_homies(homies: &[Homie]) -> Vec<&Homie> {
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
    let home_homies = chosen
        .iter()
        .map(|&index| {
            return &homies[index];
        })
        .collect();
    return home_homies;
}

async fn get_user_input_homies_favorites(
    db_pool: &SqlitePool,
    homies: &[Homie],
    recipes: &[Recipe],
) -> Result<(), sqlx::Error> {
    let mut c = true;
    while c {
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
        let input = Select::new()
            .with_prompt("Add another favorite?")
            .items(&["Yes", "No"])
            .default(1)
            .interact()
            .unwrap();
        if input == 1 {
            c = false;
        }
    }
    return Ok(());
}

async fn setup_foods(db_pool: &SqlitePool) {
    let mut input = Input::<String>::new()
        .with_prompt("Enter food name")
        .default("".into())
        .interact_text()
        .unwrap();

    while !input.is_empty() {
        println!("Adding food: {}", input);
        add_recipe(db_pool, &input).await.unwrap();
        input = Input::<String>::new()
            .with_prompt("Enter food name")
            .default("".into())
            .interact_text()
            .unwrap();
    }
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
    setup_foods(db_pool).await;

    let all_homies = get_all_homies(db_pool).await.unwrap();
    let recipes = get_all_recipes(db_pool).await;
    get_user_input_homies_favorites(db_pool, &all_homies, &recipes)
        .await
        .unwrap();
}

async fn get_all_recipes(db_pool: &SqlitePool) -> Vec<Recipe> {
    let recipes = query_file_as!(Recipe, "src/sql/get_all_recipes.sql")
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
    query_file!("src/sql/insert_recipe.sql", name)
        .execute(db_pool)
        .await?;
    return Ok(());
}

async fn get_favorites_for_home_homie(
    db_pool: &SqlitePool,
    homie: &Homie,
) -> Result<Vec<HomiesFavorite>, sqlx::Error> {
    let homies_favorites =
        query_file_as!(HomiesFavorite, "src/sql/get_homies_favorites.sql", homie.id)
            .fetch_all(db_pool)
            .await?;
    return Ok(homies_favorites);
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
    let home_homies = get_home_homies(&homies).await;

    let mut all_homies_favorites = Vec::<HomiesFavorite>::new();
    for home_homie in home_homies.iter() {
        let homies_favorites = get_favorites_for_home_homie(&pool, home_homie).await?;
        all_homies_favorites.extend(homies_favorites);
    }

    let most_favorited_recipes = get_most_favorited_recipes(&all_homies_favorites).await;
    println!("Home homies: {:?}", home_homies);
    let mut rng = rand::thread_rng();
    let random_recipe_id = most_favorited_recipes.choose(&mut rng).unwrap();
    let random_recipe = get_recipe(&pool, *random_recipe_id).await;
    println!("{:?}", random_recipe);
    Ok(())
}

async fn get_recipe(pool: &Pool<Sqlite>, recipe_id: &i64) -> Recipe {
    let recipe = sqlx::query_as!(
        Recipe,
        "SELECT id, name FROM recipes WHERE id = ?",
        recipe_id
    )
        .fetch_one(pool)
        .await
        .unwrap();
    println!("{:?}", recipe);
    return recipe;
}

async fn get_most_favorited_recipes(homies_favorites: &[HomiesFavorite]) -> Vec<&i64> {
    let mut recipe_counts = HashMap::<&i64, u32>::new();
    for homies_favorite in homies_favorites.iter() {
        let recipe = &homies_favorite.recipe_id;
        let count = recipe_counts.entry(recipe).or_insert(0);
        *count += 1;
    }
    let mut recipe_counts = recipe_counts.into_iter().collect::<Vec<(&i64, u32)>>();
    recipe_counts.sort_by(|a, b| return b.1.cmp(&a.1););
    let max_favorite = recipe_counts.iter().fold(0, |acc, (_, count)| {
        if acc < *count {
            return *count;
        }
        return acc;
    });
    let most_favorited_recipes = recipe_counts
        .iter()
        .filter(|(_, count)| {
            return &max_favorite == count;
        })
        .map(|(recipe, _)| {
            return *recipe;
        })
        .collect();
    return most_favorited_recipes;
}
