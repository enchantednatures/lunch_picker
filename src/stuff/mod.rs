use std::fs;
use std::fs::File;
use std::io::Write;

use axum::http::Request;
use dialoguer::Input;
use rand::prelude::SliceRandom;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{migrate, SqlitePool};

use tower_http::request_id::MakeRequestId;
use tower_http::request_id::RequestId;

use crate::api::CreateRecipeRequest;
use crate::models::db_rows::HomiesFavorite;
use crate::{algorithms, data_access, user_input, CONFIG_FILE};

async fn setup_foods(db_pool: &SqlitePool) {
    let mut input = Input::<String>::new()
        .with_prompt("Enter food name")
        .default("".into())
        .interact_text()
        .unwrap();

    while !input.is_empty() {
        println!("Adding food: {}", input);
        data_access::add_recipe(db_pool, &input).await.unwrap();
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
        data_access::add_homie(db_pool, &input).await.unwrap();
        input = Input::<String>::new()
            .with_prompt("Add another homie? (leave blank to finish)")
            .default("".into())
            .interact()
            .unwrap();

        println!("Added homie: {}", input);
    }
    setup_foods(db_pool).await;

    let all_homies = data_access::get_all_homies(db_pool).await.unwrap();
    let recipes = data_access::get_all_recipes(db_pool).await;
    user_input::get_user_input_homies_favorites(db_pool, &all_homies, &recipes)
        .await
        .unwrap();
}

pub fn check_if_file_exists(path: &str) -> bool {
    return std::path::Path::new(path).exists();
}

impl MakeRequestId for CreateRecipeRequest {
    fn make_request_id<B>(&mut self, request: &Request<B>) -> Option<RequestId> {
        let request_id = request
            .headers()
            .get("x-request-id")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse().ok())
            .unwrap();

        Some(RequestId::new(request_id))
    }
}

// #[tokio::main]
async fn cli_main() -> Result<(), sqlx::Error> {
    let mut is_setup = false;

    let mut db_url = String::new();

    if !check_if_file_exists(CONFIG_FILE) {
        println!("Config file doesn't exist");
        let mut file = File::create(CONFIG_FILE).unwrap();
        db_url = Input::<String>::new()
            .with_prompt("Enter database url")
            .default("./.shitty_lunch_picker.db".into())
            .interact()
            .unwrap();
        file.write_all(db_url.as_bytes()).unwrap();
    }

    if db_url.is_empty() {
        db_url = fs::read_to_string(CONFIG_FILE).expect("Failed to read config file");
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

    let homies = data_access::get_all_homies(&pool).await?;
    let home_homies = data_access::get_home_homies(&homies).await;

    let mut all_homies_favorites = Vec::<HomiesFavorite>::new();
    for home_homie in home_homies.iter() {
        let homies_favorites = data_access::get_favorites_for_home_homie(&pool, home_homie).await?;
        all_homies_favorites.extend(homies_favorites);
    }

    let most_favorited_recipes =
        algorithms::get_most_favorited_recipes(&all_homies_favorites).await;
    println!("Home homies: {:?}", home_homies);
    let mut rng = rand::thread_rng();
    let random_recipe_id = most_favorited_recipes.choose(&mut rng).unwrap();
    let random_recipe = data_access::get_recipe(&pool, random_recipe_id).await;
    println!("{:?}", random_recipe);
    Ok(())
}
