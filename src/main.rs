use anyhow::Result;
use clap::Parser;
use lunch_picker::add_homies_favorite_restaurants_interactive;
use lunch_picker::add_homies_interactive;
use lunch_picker::add_restaurants_interactive;
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
use lunch_picker::*;
use sqlx::migrate::MigrateDatabase;
use std::fs;
// use lunch_picker::features::create_recipe;
use lunch_picker::features::create_restaurant;
use lunch_picker::features::get_all_homies;
use lunch_picker::features::get_candidate_restaurants;
use lunch_picker::features::remove_homies_favorite_restaurant;
use lunch_picker::features::Homie;
use lunch_picker::get_home_homies;
use lunch_picker::select_restaurant;
use opentelemetry::trace::TraceError;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::runtime;
use opentelemetry_sdk::trace::config;
use opentelemetry_sdk::Resource;

use sqlx::sqlite::SqlitePoolOptions;
use sqlx::Pool;

use sqlx::Sqlite;
use tracing::event;
use tracing::Instrument;
use tracing::Level;
use tracing_subscriber::prelude::*;
use tracing_subscriber::Registry;

const CLI_USER_ID: i32 = 1;

// trait HomiePaging: Iterator<Item = Vec<Homie>> {
//     fn get_next(&mut self) -> Option<Vec<Homie>>;
//     fn get_previous(&mut self) -> Option<Vec<Homie>>;
// }

pub(crate) fn init_tracer() -> Result<opentelemetry_sdk::trace::Tracer, TraceError> {
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317"),
        )
        .with_trace_config(config().with_resource(Resource::new(vec![KeyValue::new(
            "service.name",
            "lunch_picker.cli",
        )])))
        .install_batch(runtime::Tokio)
}

struct AppState {
    db: Pool<Sqlite>,
}

impl AppState {
    fn new(db: Pool<Sqlite>) -> Self {
        Self { db }
    }

    #[tracing::instrument(name = "User Interaction", skip(self))]
    async fn work(&self) -> Result<()> {
        let mut homies: Vec<Homie> = get_all_homies(1, &self.db).await?;
        if homies.is_empty() {
            event!(Level::ERROR, "No homies found");
            homies = add_homies_interactive(CLI_USER_ID, &self.db).await?;
            add_restaurants_interactive(CLI_USER_ID, &self.db).await?;
        }

        let home_homies = get_home_homies(&homies).await?;
        let mut restaurants = get_candidate_restaurants(home_homies.clone(), 1, &self.db).await?;
        if restaurants.is_empty() {
            event!(Level::ERROR, "No candidate restaurants found");
            add_restaurants_interactive(CLI_USER_ID, &self.db).await?;
            restaurants = get_candidate_restaurants(home_homies.clone(), 1, &self.db).await?;
        }

        if restaurants.is_empty() {
            event!(
                Level::ERROR,
                "User did not add any restaurants that produced candidates"
            );
            add_restaurants_interactive(CLI_USER_ID, &self.db).await?;
            restaurants = get_candidate_restaurants(home_homies.clone(), 1, &self.db).await?;
        }

        let selected = select_restaurant(&restaurants).await?;

        event!(
            Level::INFO,
            name = "Selected restaurant",
            restaurant_name = selected.name.as_str()
        );

        add_recent_restaurant_for_homies(home_homies, selected.id, CLI_USER_ID, &self.db).await?;

        Ok(())
    }
}

impl Drop for AppState {
    fn drop(&mut self) {
        futures::executor::block_on(self.db.close());
        opentelemetry::global::shutdown_tracer_provider();
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = CliArgs::parse();

    // check if a file has been given
    // if no file has been given use the default
    // if the default file doesn't exist then prompt the user for setting up
    let mut config_file = dirs::home_dir().expect("No home");
    config_file.push(".config/local/lunch.json");

    let prefix = &config_file.parent().unwrap();
    fs::create_dir_all(prefix).unwrap();
    let settings = match config_file.exists() {
        true => {
            let config_file = config_file.to_str().ok_or(ConfigError::UnableToParsePath)?;
            let settings = std::fs::read_to_string(config_file)?;
            serde_json::from_str(&settings)?
        }
        false => {
            let settings = user_setup()?;
            fs::write(config_file, serde_json::to_string_pretty(&settings)?)?;
            settings
        }
    };

    if settings.telemetry_enabled {
        let tracer = init_tracer()?;

        // Create a tracing layer with the configured tracer
        let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
        // Use the tracing subscriber `Registry`, or any other subscriber
        // that impls `LookupSpan`
        let subscriber = Registry::default().with(telemetry);

        // Trace executed code
        tracing::subscriber::set_global_default(subscriber).unwrap();
    }

    let database_url = std::env::var("DATABASE_URL").unwrap_or(settings.database_url);
    if !sqlx::Sqlite::database_exists(&database_url).await? {
        sqlx::Sqlite::create_database(&database_url).await?;
    }

    let db = SqlitePoolOptions::new()
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
                    AddRestaurant::Delete {
                        homie_name,
                        restaurant_name,
                    } => {
                        remove_homies_favorite_restaurant(
                            homie_name,
                            restaurant_name,
                            CLI_USER_ID,
                            &app_state.db,
                        )
                        .await?
                    }
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
                    AddRestaurant::Delete {
                        homie_name,
                        restaurant_name,
                    } => {
                        remove_homies_favorite_restaurant(
                            homie_name,
                            restaurant_name,
                            CLI_USER_ID,
                            &app_state.db,
                        )
                        .await?
                    } // _ => println!("Restaurant command"),
                },
                Homies::Interactive => {
                    add_homies_favorite_restaurants_interactive(CLI_USER_ID, &app_state.db).await?;
                }
            },

            Command::Restaurants(restaurant_command) => match restaurant_command {
                Restaurants::Add { restaurant_name } => {
                    create_restaurant(restaurant_name, CLI_USER_ID, &app_state.db).await?;
                }
                Restaurants::Delete { restaurant_name: _ } => todo!(),
                Restaurants::Rename {
                    restaurant_name: _,
                    updated_name: _,
                } => todo!(),
            },
            Command::Recipes(recipe_command) => match recipe_command {
                Recipes::Add { recipe_name } => {
                    // _ = create_recipe(recipe_name, 1, &app_state.db).await?;
                }
                _ => println!("Recipe command"),
            },
            Command::Pick => app_state.work().await?,
        },
        None => app_state.work().await?,
    }

    // app_state.db.close().await;
    // opentelemetry::global::shutdown_tracer_provider();
    Ok(())
}
