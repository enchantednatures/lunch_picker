#![allow(dead_code)]

use std::fs;
use std::fs::File;
use std::io::Write;

use anyhow::Result;
use dialoguer::Input;
use rand::prelude::SliceRandom;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{migrate, SqlitePool};

use cli::setup;
use common::domain::HomiesFavorite;
use common::{
    check_if_file_exists, get_all_homies, get_favorites_for_home_homie, get_most_favorited_recipes,
    get_recipe, CONFIG_FILE,
};

struct LunchDecider<'a> {
    pool: &'a SqlitePool,
}

impl<'a> LunchDecider<'a> {
    fn new(pool: &'a SqlitePool) -> Self {
        LunchDecider { pool }
    }
    async fn setup(&self) -> Result<()> {
        migrate!("./../migrations").run(self.pool).await?;
        setup(self.pool).await;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
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

    let lunch_decider = LunchDecider::new(&pool);
    if !is_setup {
        let _setup_result = lunch_decider.setup().await;
    }

    let homies = get_all_homies(&pool).await?;
    let home_homies = cli::get_home_homies(&homies).await;

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
