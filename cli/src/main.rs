#[allow(dead_code)]
use std::fs;
use std::fs::File;
use std::io::Write;

use dialoguer::Input;

use cli::{
    check_if_file_exists, get_all_homies, get_favorites_for_home_homie, get_home_homies,
    get_most_favorited_recipes, get_recipe, setup, CONFIG_FILE,
};
use common::domain::HomiesFavorite;
use rand::prelude::SliceRandom;
use sqlx::migrate;
use sqlx::sqlite::SqlitePoolOptions;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
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
        migrate!("./../migrations").run(&pool).await?;
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
    let random_recipe = get_recipe(&pool, random_recipe_id).await;
    println!("{:?}", random_recipe);
    Ok(())
}
