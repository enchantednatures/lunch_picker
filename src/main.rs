#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]
#![allow(dead_code)]

use std::fs;
use std::fs::File;
use std::io::Write;

use dialoguer::{Input, MultiSelect};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{migrate, SqlitePool};

pub use models::db_rows::RecentMeal;
pub use models::db_rows::Recipe;

mod models;

struct Homie {
    pub id: i64,
    pub name: String,
}

async fn add_homie(db_pool: &SqlitePool, name: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO homies (name)
        VALUES ($1)
        "#,
        name
    )
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
       SELECT id as "id!", name as "name!", created_at as "created_at!"
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
}

fn check_if_file_exists(path: &str) -> bool {
    return std::path::Path::new(path).exists();
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
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
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    migrate!("./migrations").run(&pool).await?;

    setup(&pool).await;

    let homies = get_all_homies(&pool).await?;

    let _home_homies = get_home_homies(&homies).await;
    let recent = get_recent_meals(&pool).await?;
    println!("recents: {:?}", recent);
    Ok(())
}
