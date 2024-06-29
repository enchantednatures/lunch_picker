#![allow(dead_code)]

use anyhow::Result;
use clap::Parser;
use dialoguer::MultiSelect;
use dialoguer::Select;
use lunch_picker::cli_args::AddRestaurant;
use lunch_picker::cli_args::CliArgs;
use lunch_picker::cli_args::Command;
use lunch_picker::cli_args::Homies;
use lunch_picker::cli_args::Recipes;
use lunch_picker::cli_args::Restaurants;
use lunch_picker::db::Migrator;
use lunch_picker::features::add_homies_favorite_restaurant;
use lunch_picker::features::add_recent_restaurant_for_homie;
use lunch_picker::features::add_recent_restaurant_for_homies;
use lunch_picker::features::create_homie;
use lunch_picker::features::create_recipe;
use lunch_picker::features::create_restaurant;
use lunch_picker::features::get_all_homies;
use lunch_picker::features::get_candidate_restaurants;
use lunch_picker::features::Homie;
use lunch_picker::features::HomieId;
use lunch_picker::features::Restaurant;
use sqlx::postgres::PgPoolOptions;
use sqlx::Pool;
use sqlx::Postgres;
use tracing::Instrument;
use tracing_bunyan_formatter::BunyanFormattingLayer;
use tracing_bunyan_formatter::JsonStorageLayer;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Registry;

const CLI_USER_ID: i32 = 1;

// trait HomiePaging: Iterator<Item = Vec<Homie>> {
//     fn get_next(&mut self) -> Option<Vec<Homie>>;
//     fn get_previous(&mut self) -> Option<Vec<Homie>>;
// }

async fn select_restaurant(restaurants: &[Restaurant]) -> &Restaurant {
    let restaurant_names = restaurants
        .iter()
        .map(|h| {
            return h.name.as_str();
        })
        .collect::<Vec<&str>>();
    let chosen = Select::new()
        .with_prompt("where would you like to eat?")
        .items(&restaurant_names)
        .interact()
        .unwrap();

    &restaurants[chosen]
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
    let home_homies = chosen.iter().map(|&index| &homies[index]).collect();
    home_homies
}

struct AppState {
    db: Pool<Postgres>,
}

impl AppState {
    fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    async fn work(&self) -> Result<()> {
        let homies: Vec<Homie> = get_all_homies(1, &self.db).await?;
        let home_homies = get_home_homies(&homies).await;
        let hh: Vec<&HomieId> = home_homies.iter().map(|&x| &x.id).collect();
        let restaurants = get_candidate_restaurants(&hh, 1, &self.db).await?;
        dbg!(&restaurants);
        let selected = select_restaurant(&restaurants).await;
        println!("Selected restaurant: {}", selected.name.as_str());

        add_recent_restaurant_for_homies(hh, selected.id, CLI_USER_ID, &self.db).await?;

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
                Homies::Add(args) => {
                    println!("Adding homie {}", args.homies_name);
                    _ = create_homie(args.homies_name, CLI_USER_ID, &app_state.db).await?;
                }
                Homies::Delete { homies_name } => println!("Deleting homie {}", homies_name),
                Homies::Rename {
                    homies_name,
                    updated_name,
                } => println!(
                    "Updating homie {} with new name {}",
                    homies_name, updated_name
                ),
                Homies::Restaurants(restaurant_command) => match restaurant_command {
                    AddRestaurant::Add {
                        homie_name,
                        restaurant_name,
                    } => {
                        add_homies_favorite_restaurant(
                            homie_name.clone(),
                            restaurant_name.clone(),
                            1,
                            &app_state.db,
                        )
                        .await?;
                        println!(
                            "Added restaurant {} to homie {}",
                            restaurant_name, homie_name
                        )
                    }
                    _ => println!("Restaurant command"),
                },
                Homies::RecentRestaurant(restaurant_command) => match restaurant_command {
                    AddRestaurant::Add {
                        homie_name,
                        restaurant_name,
                    } => {
                        add_recent_restaurant_for_homie(
                            homie_name,
                            restaurant_name,
                            1,
                            &app_state.db,
                        )
                        .await?;
                    }
                    _ => println!("Restaurant command"),
                },
            },

            Command::Restaurants(restaurant_command) => match restaurant_command {
                Restaurants::Add { restaurant_name } => {
                    create_restaurant(restaurant_name, CLI_USER_ID, &app_state.db).await?;
                }
                Restaurants::Delete { restaurant_name } => todo!(),
                Restaurants::Rename {
                    restaurant_name,
                    updated_name,
                } => todo!(),
            },
            Command::Recipes(recipe_command) => match recipe_command {
                Recipes::Add { recipe_name } => {
                    _ = create_recipe(recipe_name, 1, &app_state.db).await?;
                }
                _ => println!("Recipe command"),
            },
            Command::Pick => app_state.work().await?,
        },
        None => app_state.work().await?,
    }

    app_state.db.close().await;

    Ok(())
}
