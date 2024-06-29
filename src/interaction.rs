use dialoguer::{MultiSelect, Select};

use crate::features::{Homie, Restaurant};

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
