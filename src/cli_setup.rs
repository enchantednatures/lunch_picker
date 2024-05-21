#![allow(dead_code)]

use dialoguer::Input;
use dialoguer::{MultiSelect, Select};
use sqlx::{query_file, query_file_as, SqlitePool};


use common::domain::{Homie, HomiesFavorite, Recipe};
pub mod cli_args;


async fn setup_foods(db_pool: &SqlitePool) {
    let mut input = Input::<String>::new()
        .with_prompt("Enter food name")
        .default("".into())
        .interact_text()
        .unwrap();

    while !input.is_empty() {
        println!("Adding food: {}", input);
        common::add_recipe(db_pool, &input).await.unwrap();
        input = Input::<String>::new()
            .with_prompt("Enter food name")
            .default("".into())
            .interact_text()
            .unwrap();
    }
}

pub async fn setup(db_pool: &SqlitePool) {
    let mut input = Input::<String>::new()
        .with_prompt("Enter homie's name")
        .default("".into())
        .interact_text()
        .unwrap();

    while !input.is_empty() {
        println!("Adding homie: {}", input);
        common::add_homie(db_pool, &input).await.unwrap();
        input = Input::<String>::new()
            .with_prompt("Add another homie? (leave blank to finish)")
            .default("".into())
            .interact()
            .unwrap();

        println!("Added homie: {}", input);
    }
    setup_foods(db_pool).await;

    let all_homies = common::get_all_homies(db_pool).await.unwrap();
    let recipes = common::get_all_recipes(db_pool).await;
    get_user_input_homies_favorites(db_pool, &all_homies, &recipes)
        .await
        .unwrap();
}

pub async fn get_user_input_homies_favorites(
    db_pool: &SqlitePool,
    homies: &[Homie],
    recipes: &[Recipe],
) -> Result<(), sqlx::Error> {
    let mut c = true;
    while c {
        let selected_homie_idx = Select::new()
            .with_prompt("Who's favorites are you adding?")
            .items(
                &homies
                    .iter()
                    .map(|h| {
                        return h.name.as_str();
                    })
                    .collect::<Vec<&str>>(),
            )
            .interact()
            .unwrap();

        let current_homie = &homies[selected_homie_idx];
        let homie_id = current_homie.id;
        let homies_favorites =
            query_file_as!(HomiesFavorite, "src/sql/get_homies_favorites.sql", homie_id)
                .fetch_all(db_pool)
                .await?;

        let homies_favorites_ids = homies_favorites
            .iter()
            .map(|hf| hf.recipe_id)
            .collect::<Vec<i64>>();

        let is_favorite_map: Vec<bool> = recipes
            .iter()
            .map(|x| homies_favorites_ids.contains(&x.id))
            .collect();

        let input = MultiSelect::new()
            .with_prompt("What are {}'s favorites?")
            .items(
                &recipes
                    .iter()
                    .map(|r| {
                        return r.name.as_str();
                    })
                    .collect::<Vec<&str>>(),
            )
            .defaults(&is_favorite_map)
            .interact()
            .unwrap();

        let new_favorites = input
            .iter()
            .map(|&index| recipes[index].id)
            .collect::<Vec<i64>>();

        query_file!("src/sql/delete_homies_favorites.sql", current_homie.id)
            .execute(db_pool)
            .await?;

        for recipe_id in new_favorites.iter() {
            query_file!(
                "src/sql/insert_homies_favorites.sql",
                current_homie.id,
                recipe_id
            )
            .execute(db_pool)
            .await?;
        }
        let input = Select::new()
            .with_prompt("Add another favorite?")
            .items(&["Yes", "No"])
            .default(1)
            .interact()
            .unwrap();
        if input == 1 {
            c = false;
        }
    }
    Ok(())
}

pub async fn get_home_homies(homies: &[Homie]) -> Vec<&Homie> {
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
