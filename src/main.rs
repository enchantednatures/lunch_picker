use anyhow::Result;
use chrono::Local;
use clap::Parser;
use lunch_picker::add_homies_favorite_restaurants_interactive;
use lunch_picker::add_homies_interactive;
use lunch_picker::add_restaurants_interactive;
use lunch_picker::cli_args::AddRestaurant;
use lunch_picker::cli_args::CliArgs;
use lunch_picker::cli_args::Command;
use lunch_picker::cli_args::Homies;
use lunch_picker::cli_args::Pantry;
use lunch_picker::cli_args::Plan;
use lunch_picker::cli_args::Recipes;
use lunch_picker::cli_args::Restaurants;
use lunch_picker::cli_args::Template;
use lunch_picker::db::Migrator;
use lunch_picker::features::add_homies_favorite_restaurant;
use lunch_picker::features::add_recent_restaurant_for_homie;
use lunch_picker::features::add_recent_restaurant_for_homies;
use lunch_picker::features::create_homie;
use lunch_picker::features::create_recipe;
use lunch_picker::features::create_recipe_step;
use lunch_picker::features::create_restaurant;
use lunch_picker::features::delete_recipe;
use lunch_picker::features::get_all_homies;
use lunch_picker::features::get_candidate_restaurants;
use lunch_picker::features::get_recipe_by_name;
use lunch_picker::features::import_recipe_from_url;
use lunch_picker::features::list_recipes;
use lunch_picker::features::remove_homies_favorite_restaurant;
use lunch_picker::features::Homie;
use lunch_picker::get_home_homies;
use lunch_picker::select_pick_source;
use lunch_picker::select_recipe;
use lunch_picker::select_restaurant;
use lunch_picker::ConfigError;
use lunch_picker::user_setup;
use lunch_picker::features::add_recent_recipe_for_homies;
use lunch_picker::features::get_candidate_recipes;
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

use sqlx::migrate::MigrateDatabase;
use std::fs;

const CLI_USER_ID: i32 = 1;

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
        let mut homies: Vec<Homie> = get_all_homies(CLI_USER_ID, &self.db).await?;
        if homies.is_empty() {
            event!(Level::ERROR, "No homies found");
            homies = add_homies_interactive(CLI_USER_ID, &self.db).await?;
            add_restaurants_interactive(CLI_USER_ID, &self.db).await?;
        }

        let home_homies = get_home_homies(&homies).await?;
        let source = select_pick_source().await?;

        match source {
            "Restaurants" => {
                let mut restaurants = get_candidate_restaurants(home_homies.clone(), CLI_USER_ID, &self.db).await?;
                if restaurants.is_empty() {
                    event!(Level::ERROR, "No candidate restaurants found");
                    add_restaurants_interactive(CLI_USER_ID, &self.db).await?;
                    restaurants = get_candidate_restaurants(home_homies.clone(), CLI_USER_ID, &self.db).await?;
                }

                if restaurants.is_empty() {
                    event!(
                        Level::ERROR,
                        "User did not add any restaurants that produced candidates"
                    );
                    add_restaurants_interactive(CLI_USER_ID, &self.db).await?;
                    restaurants = get_candidate_restaurants(home_homies.clone(), CLI_USER_ID, &self.db).await?;
                }

                let selected = select_restaurant(&restaurants).await?;

                event!(
                    Level::INFO,
                    name = "Selected restaurant",
                    restaurant_name = selected.name.as_str()
                );

                add_recent_restaurant_for_homies(home_homies, selected.id, CLI_USER_ID, &self.db).await?;
            }
            "Recipes" => {
                let recipes = get_candidate_recipes(home_homies.clone(), CLI_USER_ID, &self.db).await?;
                if recipes.is_empty() {
                    event!(Level::ERROR, "No candidate recipes found");
                    println!("No recipes found. Add recipes via: lunch_picker recipes add <name>");
                    return Ok(());
                }

                let selected = select_recipe(&recipes).await?;

                event!(
                    Level::INFO,
                    name = "Selected recipe",
                    recipe_name = selected.name.as_str()
                );

                add_recent_recipe_for_homies(home_homies, selected.id, CLI_USER_ID, &self.db).await?;
            }
            _ => unreachable!(),
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = CliArgs::parse();

    let mut config_file = dirs::config_dir().expect("No config directory");
    config_file.push("lunch_picker");
    config_file.push("config.json");

    let old_config_file = {
        let mut p = dirs::home_dir().expect("No home");
        p.push(".config/local/lunch.json");
        p
    };
    if !config_file.exists() && old_config_file.exists() {
        if let Some(prefix) = config_file.parent() {
            fs::create_dir_all(prefix).unwrap();
        }
        fs::copy(&old_config_file, &config_file).unwrap();
    }

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
        let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
        let subscriber = Registry::default().with(telemetry);
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
                            CLI_USER_ID,
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
                            CLI_USER_ID,
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
                    }
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
                Recipes::Add {
                    recipe_name,
                    description,
                    prep_time,
                    cook_time,
                    servings,
                } => {
                    let recipe = create_recipe(
                        recipe_name,
                        CLI_USER_ID,
                        description,
                        prep_time,
                        cook_time,
                        servings,
                        &app_state.db,
                    )
                    .await?;
                    println!("Created recipe: {}", recipe.name.as_str());
                }
                Recipes::Delete { recipe_name } => {
                    let recipe = get_recipe_by_name(&recipe_name, CLI_USER_ID, &app_state.db).await?;
                    delete_recipe(recipe.id.as_i32(), CLI_USER_ID, &app_state.db).await?;
                    println!("Deleted recipe: {}", recipe_name);
                }
                Recipes::View { recipe_name } => {
                    let recipe = get_recipe_by_name(&recipe_name, CLI_USER_ID, &app_state.db).await?;
                    println!("Recipe: {}", recipe.name.as_str());
                    if let Some(desc) = recipe.description {
                        println!("Description: {}", desc);
                    }
                    if let Some(pt) = recipe.prep_time {
                        println!("Prep time: {} min", pt);
                    }
                    if let Some(ct) = recipe.cook_time {
                        println!("Cook time: {} min", ct);
                    }
                    if let Some(sv) = recipe.servings {
                        println!("Servings: {}", sv);
                    }
                }
                Recipes::List => {
                    let recipes = list_recipes(CLI_USER_ID, &app_state.db).await?;
                    if recipes.is_empty() {
                        println!("No recipes found");
                    } else {
                        for recipe in recipes {
                            println!("  - {}", recipe.name.as_str());
                        }
                    }
                }
                Recipes::Import { url } => {
                    match import_recipe_from_url(url, CLI_USER_ID, &app_state.db).await {
                        Ok(recipe) => println!("Imported recipe: {}", recipe.name.as_str()),
                        Err(e) => println!("Import failed: {}", e),
                    }
                }
                Recipes::AddStep {
                    recipe_name,
                    step_number,
                    instruction,
                } => {
                    let recipe = get_recipe_by_name(&recipe_name, CLI_USER_ID, &app_state.db).await?;
                    create_recipe_step(
                        recipe.id.as_i32(),
                        step_number,
                        instruction,
                        CLI_USER_ID,
                        &app_state.db,
                    )
                    .await?;
                    println!("Added step {} to recipe {}", step_number, recipe_name);
                }
                Recipes::AddIngredient {
                    recipe_name,
                    ingredient_name,
                    qty,
                    measure,
                } => {
                    let recipe = get_recipe_by_name(&recipe_name, CLI_USER_ID, &app_state.db).await?;
                    lunch_picker::features::add_recipe_ingredient(
                        recipe.id.as_i32(),
                        ingredient_name,
                        qty,
                        measure,
                        CLI_USER_ID,
                        &app_state.db,
                    )
                    .await?;
                    println!("Added ingredient to recipe {}", recipe_name);
                }
            },

            Command::Pantry(pantry_command) => match pantry_command {
                Pantry::Add {
                    ingredient_name,
                    qty,
                    measure,
                } => {
                    lunch_picker::features::add_pantry_ingredient(
                        ingredient_name,
                        qty,
                        measure,
                        CLI_USER_ID,
                        &app_state.db,
                    )
                    .await?;
                    println!("Added to pantry");
                }
                Pantry::Update {
                    ingredient_name,
                    qty,
                    measure,
                } => {
                    lunch_picker::features::update_pantry_ingredient(
                        ingredient_name,
                        qty,
                        measure,
                        CLI_USER_ID,
                        &app_state.db,
                    )
                    .await?;
                    println!("Updated pantry");
                }
                Pantry::Remove {
                    ingredient_name,
                    measure,
                } => {
                    if let Some(m) = measure {
                        lunch_picker::features::remove_pantry_ingredient(
                            ingredient_name,
                            Some(m),
                            CLI_USER_ID,
                            &app_state.db,
                        )
                        .await?;
                    } else {
                        lunch_picker::features::remove_pantry_ingredient(
                            ingredient_name,
                            None::<String>,
                            CLI_USER_ID,
                            &app_state.db,
                        )
                        .await?;
                    }
                    println!("Removed from pantry");
                }
                Pantry::List => {
                    let items = lunch_picker::features::list_pantry_ingredients(CLI_USER_ID, &app_state.db).await?;
                    if items.is_empty() {
                        println!("Pantry is empty");
                    } else {
                        for item in items {
                            println!(
                                "  - {}: {} {}",
                                item.ingredient_name.as_str(),
                                item.quantity,
                                item.measure.as_str()
                            );
                        }
                    }
                }
            },

            Command::Plan(plan_command) => match plan_command {
                Plan::Add { recipe, date, slot } => {
                    let recipe = get_recipe_by_name(&recipe, CLI_USER_ID, &app_state.db).await?;
                    let date = chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d")?;
                    let slot = match slot.as_str() {
                        "breakfast" => lunch_picker::features::MealSlot::Breakfast,
                        "lunch" => lunch_picker::features::MealSlot::Lunch,
                        "dinner" => lunch_picker::features::MealSlot::Dinner,
                        "snack" => lunch_picker::features::MealSlot::Snack,
                        _ => lunch_picker::features::MealSlot::Dinner,
                    };
                    lunch_picker::features::create_meal_plan_entry(
                        recipe.id.as_i32(),
                        date,
                        slot,
                        CLI_USER_ID,
                        &app_state.db,
                    )
                    .await?;
                    println!("Added to meal plan");
                }
                Plan::Remove { id } => {
                    lunch_picker::features::delete_meal_plan_entry(id, CLI_USER_ID, &app_state.db)
                        .await?;
                    println!("Removed from meal plan");
                }
                Plan::View { start, days } => {
                    let start_date = start
                        .map(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d"))
                        .transpose()?
                        .unwrap_or_else(|| Local::now().naive_local().date());
                    let end_date = start_date + chrono::Duration::days(days);
                    let entries = lunch_picker::features::list_meal_plan_entries(
                        CLI_USER_ID,
                        start_date,
                        end_date,
                        &app_state.db,
                    )
                    .await?;
                    if entries.is_empty() {
                        println!("No meal plan entries");
                    } else {
                        for entry in entries {
                            println!(
                                "  {} [{}]: {}",
                                entry.date,
                                entry.meal_slot.as_str(),
                                entry.recipe_name
                            );
                        }
                    }
                }
                Plan::Shop { start, days } => {
                    let start_date = start
                        .map(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d"))
                        .transpose()?
                        .unwrap_or_else(|| Local::now().naive_local().date());
                    let end_date = start_date + chrono::Duration::days(days);
                    let items = lunch_picker::features::generate_shopping_list(
                        CLI_USER_ID,
                        start_date,
                        end_date,
                        &app_state.db,
                    )
                    .await?;
                    if items.is_empty() {
                        println!("Shopping list is empty");
                    } else {
                        println!("Shopping list:");
                        for item in items {
                            println!(
                                "  - {}: {} {}",
                                item.ingredient_name, item.quantity_to_buy, item.measure
                            );
                        }
                    }
                }
            },

            Command::Template(template_command) => match template_command {
                Template::Add {
                    name,
                    day,
                    slot,
                    recipe,
                } => {
                    let recipe = get_recipe_by_name(&recipe, CLI_USER_ID, &app_state.db).await?;
                    let slot = match slot.as_str() {
                        "breakfast" => lunch_picker::features::MealSlot::Breakfast,
                        "lunch" => lunch_picker::features::MealSlot::Lunch,
                        "dinner" => lunch_picker::features::MealSlot::Dinner,
                        "snack" => lunch_picker::features::MealSlot::Snack,
                        _ => lunch_picker::features::MealSlot::Dinner,
                    };
                    lunch_picker::features::create_meal_plan_template(
                        name,
                        day,
                        slot,
                        recipe.id.as_i32(),
                        CLI_USER_ID,
                        &app_state.db,
                    )
                    .await?;
                    println!("Created template");
                }
                Template::Remove { id } => {
                    lunch_picker::features::delete_meal_plan_template(id, CLI_USER_ID, &app_state.db)
                        .await?;
                    println!("Removed template");
                }
                Template::List => {
                    let templates = lunch_picker::features::list_meal_plan_templates(CLI_USER_ID, &app_state.db)
                        .await?;
                    if templates.is_empty() {
                        println!("No templates");
                    } else {
                        for template in templates {
                            println!(
                                "  {} (day {} [{}]): {}",
                                template.name,
                                template.day_of_week,
                                template.meal_slot.as_str(),
                                template.recipe_name
                            );
                        }
                    }
                }
                Template::Apply { start, days } => {
                    let start_date = start
                        .map(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d"))
                        .transpose()?
                        .unwrap_or_else(|| Local::now().naive_local().date());
                    let end_date = start_date + chrono::Duration::days(days);
                    let applied = lunch_picker::features::apply_meal_plan_templates(
                        CLI_USER_ID,
                        start_date,
                        end_date,
                        &app_state.db,
                    )
                    .await?;
                    println!("Applied {} templates", applied.len());
                }
            },

            Command::Pick => app_state.work().await?,
        },
        None => app_state.work().await?,
    }

    app_state.db.close().await;
    opentelemetry::global::shutdown_tracer_provider();
    Ok(())
}
