#![allow(dead_code)]

use anyhow::Result;
use clap::Parser;
use lunch_picker::cli_args::CliArgs;
use lunch_picker::cli_args::Command;
use lunch_picker::cli_args::Homies;
use lunch_picker::db::Migrator;
use lunch_picker::features::create_homie::create_homie;
use sqlx::sqlite::SqlitePoolOptions;
use tracing_bunyan_formatter::BunyanFormattingLayer;
use tracing_bunyan_formatter::JsonStorageLayer;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Registry;

#[tokio::main]
async fn main() -> Result<()> {
    let args = CliArgs::parse();

    let formatting_layer = BunyanFormattingLayer::new("Lunch".into(), std::io::stdout);

    let subscriber = Registry::default()
        .with(JsonStorageLayer)
        .with(EnvFilter::new("info"))
        .with(formatting_layer);

    tracing::subscriber::set_global_default(subscriber).unwrap();

    dbg!(&args);

    let db_path = "./lunch.db";
    let db = SqlitePoolOptions::new().connect(db_path).await?;

    db.migrate().await?;

    match args.command {
        Some(cmd) => match cmd {
            Command::Homies(homie_command) => match homie_command {
                Homies::Add { homies_name } => {
                    println!("Adding homie {}", homies_name);
                    let homie = create_homie(homies_name, &db).await?;
                    dbg!(&homie);
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
                _ => println!("Recipe command"),
            },
        },
        None => println!("No command provided"),
    }

    db.close().await;

    Ok(())
}
