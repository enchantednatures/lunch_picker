#![allow(dead_code)]

use anyhow::Result;
use clap::Parser;
use dialoguer::MultiSelect;
use lunch_picker::cli_args::CliArgs;
use lunch_picker::cli_args::Command;
use lunch_picker::cli_args::Homies;
use lunch_picker::cli_args::Recipes;
use lunch_picker::db::Migrator;
use lunch_picker::features::create_homie::create_homie;
use lunch_picker::features::create_recipe::create_recipe;
use  lunch_picker::features::get_all_homies::get_all_homies;
use lunch_picker::models::Homie;
use sqlx::postgres::PgPoolOptions;
use sqlx::Pool;
use sqlx::Postgres;
use tracing::Instrument;
use tracing_bunyan_formatter::BunyanFormattingLayer;
use tracing_bunyan_formatter::JsonStorageLayer;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Registry;



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
    let home_homies = chosen.iter().map(|&index| &homies[index]).collect();
    home_homies
}

struct AppState {
    db: Pool<Postgres>
}

impl AppState {

    fn new(db: Pool<Postgres>) -> Self {
        Self {
            db
        }
    }

    async fn work(&self) -> Result<()> {
        let homies: Vec<Homie> = get_all_homies(&self.db, 1).await?;
        let home_homies = get_home_homies(&homies).await;
        Ok(())
    }

}

#[tokio::main]
async fn main() -> Result<()> {
    let args = CliArgs::parse();


    let subscriber = Registry::default()
        .with(JsonStorageLayer)
        .with(EnvFilter::new("info"))
        .with(BunyanFormattingLayer::new("Lunch".into(), std::io::stdout));

    tracing::subscriber::set_global_default(subscriber).unwrap();

    dbg!(&args);
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .instrument(tracing::info_span!("database connection"))
        .await
        .expect("can't connect to database");

    db.migrate().await?;

    let app_state = AppState::new(db);

    match args.command {
        Some(cmd) => match cmd {
            Command::Homies(homie_command) => match homie_command {
                Homies::Add { homies_name } => {
                    println!("Adding homie {}", homies_name);
                    _ = create_homie(homies_name, 1.into(), &app_state.db).await?;
                }
                Homies::Delete { homies_name } => println!("Deleting homie {}", homies_name),
                Homies::Rename {
                    homies_name,
                    updated_name,
                } => println!(
                    "Updating homie {} with new name {}",
                    homies_name, updated_name
                ),
            },
            Command::Recipes(recipe_command) => match recipe_command {
                Recipes::Add { recipe_name } => {
                    _ = create_recipe(recipe_name, 1.into(), &app_state.db).await?;
                }
                _ => println!("Recipe command"),
            },
        },
        None => app_state.work().await?,
    }

    app_state.db.close().await;

    Ok(())
}
