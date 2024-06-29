use anyhow::Result;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, MultiSelect, Select};
use std::fmt::Debug;

use crate::features::{create_homie, get_all_homies, CreateHomie, GetAllHomies, Homie, Restaurant};
use crate::user::UserId;

#[tracing::instrument(name = "Add Homie", skip(db_pool))]
pub async fn add_homies<U, T>(user_id: U, db_pool: &T) -> Result<Vec<Homie>>
where
    U: Into<UserId> + Debug,
    T: CreateHomie + GetAllHomies,
{
    let user_id: UserId = user_id.into();
    let mut input = Input::<String>::new()
        .with_prompt("Enter homie's name")
        .default("".into())
        .interact_text()
        .unwrap();

    while !input.is_empty() {
        println!("Adding homie: {}", input);
        create_homie(input, *user_id.as_i32(), db_pool)
            .await
            .unwrap();
        input = Input::<String>::new()
            .with_prompt("Add another homie? (leave blank to finish)")
            .default("".into())
            .interact()
            .unwrap();

        println!("Added homie: {}", input);
    }

    Ok(get_all_homies(*user_id.as_i32(), db_pool).await?)
}

#[tracing::instrument(name = "User Selects Home Homies", skip(homies))]
pub async fn get_home_homies(homies: &[Homie]) -> Vec<&Homie> {
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
        // .with_theme(&dialoguer::theme::))
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

#[tracing::instrument(name = "User Selects Restarant From List", skip(restaurants))]
pub async fn select_restaurant(restaurants: &[Restaurant]) -> &Restaurant {
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
