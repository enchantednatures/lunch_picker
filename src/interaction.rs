use anyhow::Result;
use dialoguer::theme::ColorfulTheme;
use std::collections::HashSet;
use std::fmt::Debug;

use dialoguer::{Input, MultiSelect, Select};

use crate::features::{
    add_homies_favorite_restaurant, create_homie, create_restaurant, get_all_homies,
    get_all_restaurants, get_homies_favorite_restaurants, remove_homies_favorite_restaurant,
    AddFavoriteRestaurantToHomie, CreateHomie, CreateRestaurant, GetAllHomies, GetAllRestaurants,
    GetHomiesFavoriteRestaurants, Homie, RemoveFavoriteRestaurantFromHomie, Restaurant,
};
use crate::user::UserId;
use crate::Settings;

#[tracing::instrument(name = "User Setup")]
pub fn user_setup() -> Result<Settings> {
    let mut default_path = dirs::home_dir().expect("config dir not set");
    default_path.push(".config/state/lunch.db");
    let path = default_path
        .to_str()
        .expect("unable to set default database path");
    let database_url = Input::<String>::new()
        .with_prompt("Enter the database URL")
        .default(format!("sqlite:{}", path))
        .interact_text()?;
    let enable_telemetry = match Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Enable telemetry?")
        .items(&["Yes", "No"])
        .default(1)
        .interact()?
    {
        0 => true,
        1 => false,
        _ => unreachable!(),
    };
    Ok(Settings::new(database_url, enable_telemetry))
}

#[tracing::instrument(name = "User Adds Restaurants Interactively", skip(db))]
pub async fn add_restaurants_interactive<T>(
    user_id: impl Into<UserId> + Debug,
    db: &T,
) -> Result<()>
where
    T: CreateRestaurant
        + AddFavoriteRestaurantToHomie
        + GetAllHomies
        + GetAllRestaurants
        + RemoveFavoriteRestaurantFromHomie
        + GetHomiesFavoriteRestaurants,
{
    let mut input = Input::<String>::new()
        .with_prompt("Enter a restaurant name")
        .default("".into())
        .interact_text()?;

    let user_id = user_id.into();
    while !input.is_empty() {
        println!("Adding restaurant: {}", input);
        match create_restaurant(input, user_id, db).await {
            Ok(_) => Ok(()),
            Err(e) => match e {
                crate::features::CreateRestaurantError::InvalidName { name } => {
                    println!("Invalid name: {}", name);
                    Ok(())
                }
                crate::features::CreateRestaurantError::RestaurantAlreadyExists { name } => {
                    println!("Restaurant already exists: {}", name);
                    Ok(())
                }
                _ => Err(e),
            },
        }?;
        input = Input::<String>::new()
            .with_prompt("Add another restaurant? (leave blank to finish)")
            .default("".into())
            .interact()?;
    }
    let mut input = 0;
    while input != 1 {
        add_homies_favorite_restaurants_interactive(user_id, db).await?;
        input = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Continue adding favorites to homies?")
            .items(&["Yes", "No"])
            .interact()?;
        // input = Input::<String>::new()
        //     .with_prompt("Continue adding favorites to homies?")
        //     .default("n".into())
        //     .interact_text()?;
    }
    Ok(())
}

#[tracing::instrument(name = "User Adds Restaurants Interactively", skip(db))]
pub async fn add_homies_favorite_restaurants_interactive<T>(
    user_id: impl Into<UserId> + Debug,
    db: &T,
) -> Result<()>
where
    T: AddFavoriteRestaurantToHomie
        + GetAllHomies
        + GetAllRestaurants
        + RemoveFavoriteRestaurantFromHomie
        + GetHomiesFavoriteRestaurants,
{
    let user_id = user_id.into();
    let homies = get_all_homies(user_id, db).await?;
    let selected_home = select_homie(&homies)?;
    let restaurants = get_all_restaurants(user_id, db).await?;
    let favorited = get_homies_favorite_restaurants(user_id, selected_home.id, db).await?;
    let favorited_ids: HashSet<_> = favorited.iter().map(|r| r.id).collect();
    let mut pre_select = vec![];

    for restaurant in restaurants.iter() {
        if favorited_ids.contains(&restaurant.id) {
            pre_select.push(true);
        } else {
            pre_select.push(false);
        }
    }

    let restaurant_names = restaurants
        .iter()
        // .filter(|r| !favorited_ids.contains(&r.id))
        .map(|h| {
            return h.name.as_str();
        })
        .collect::<Vec<&str>>();

    let chosen = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "Which Restaurants are {}'s favorites?",
            selected_home.name.as_str()
        ))
        .items(&restaurant_names)
        .defaults(&pre_select)
        .interact()?;

    let homies_new_favorites: Vec<_> = chosen
        .iter()
        .map(|&index| &restaurants[index])
        .filter(|r| !favorited_ids.contains(&r.id))
        .collect();

    let new_favorite_ids: HashSet<_> = chosen
        .iter()
        .map(|&index| &restaurants[index])
        .map(|r| r.id)
        .collect();

    for restaurant in homies_new_favorites {
        add_homies_favorite_restaurant(
            selected_home.name.as_str().to_string(),
            restaurant.name.as_str().to_string(),
            user_id,
            db,
        )
        .await?;
    }

    let homies_removed_favorites = favorited
        .iter()
        .filter(|r| !new_favorite_ids.contains(&r.id));
    for removed in homies_removed_favorites {
        remove_homies_favorite_restaurant(
            selected_home.name.as_str().to_string(),
            removed.name.as_str().to_string(),
            user_id,
            db,
        )
        .await?;
    }

    Ok(())
}

#[tracing::instrument(name = "User Selects Home Homies", skip(homies))]
pub async fn get_favorite_restaurants(homies: &[Homie]) -> Result<Vec<&Homie>> {
    if homies.is_empty() {
        tracing::error!("No homies found");
        panic!();
    }
    let homies_names = homies
        .iter()
        .map(|h| {
            return h.name.as_str();
        })
        .collect::<Vec<&str>>();

    let chosen = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Who's home?")
        .items(&homies_names)
        .report(true)
        .interact()?;
    if chosen.is_empty() {
        return Ok(homies.iter().collect());
    }
    let home_homies = chosen.iter().map(|&index| &homies[index]).collect();
    Ok(home_homies)
}

#[tracing::instrument(name = "User selects a Homie to add favorites to")]
pub fn select_homie(homies: &Vec<Homie>) -> Result<&Homie> {
    let homies_names = homies
        .iter()
        .map(|h| {
            return h.name.as_str();
        })
        .collect::<Vec<&str>>();

    let chosen = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Which homie would you like to add favorites to?")
        .items(&homies_names)
        .interact()?;

    Ok(&homies[chosen])
}

#[tracing::instrument(name = "User Adds Homies Interactively", skip(db))]
pub async fn add_homies_interactive<T>(
    user_id: impl Into<UserId> + Debug,
    db: &T,
) -> Result<Vec<Homie>>
where
    T: CreateHomie + GetAllHomies,
{
    let mut input = Input::<String>::new()
        .with_prompt("Enter homie's name")
        .default("".into())
        .interact_text()?;

    let user_id = user_id.into();
    while !input.is_empty() {
        println!("Adding homie: {}", input);
        create_homie(input, user_id, db).await?;
        input = Input::<String>::new()
            .with_prompt("Add another homie? (leave blank to finish)")
            .default("".into())
            .interact()?;

        println!("Added homie: {}", input);
    }

    Ok(get_all_homies(user_id, db).await?)
}

#[tracing::instrument(name = "User Selects Home Homies", skip(homies))]
pub async fn get_home_homies(homies: &[Homie]) -> Result<Vec<&Homie>> {
    if homies.is_empty() {
        tracing::error!("No homies found");
        panic!();
    }
    let homies_names = homies
        .iter()
        .map(|h| {
            return h.name.as_str();
        })
        .collect::<Vec<&str>>();

    let chosen = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Who's home?")
        .items(&homies_names)
        .interact()?;
    if chosen.is_empty() {
        println!("No homies selected");
        return Ok(homies.iter().collect());
    }
    let home_homies = chosen.iter().map(|&index| &homies[index]).collect();
    Ok(home_homies)
}

#[tracing::instrument(name = "User Selects Restarant From List", skip(restaurants))]
pub async fn select_restaurant(restaurants: &[Restaurant]) -> Result<&Restaurant> {
    let restaurant_names = restaurants
        .iter()
        .map(|h| {
            return h.name.as_str();
        })
        .collect::<Vec<&str>>();
    let chosen = Select::new()
        .with_prompt("where would you like to eat?")
        .items(&restaurant_names)
        .interact()?;

    Ok(&restaurants[chosen])
}
