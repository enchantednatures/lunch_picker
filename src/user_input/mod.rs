use dialoguer::{MultiSelect, Select};
use sqlx::{query_file, query_file_as, SqlitePool};

use crate::models::db_rows::{Homie, HomiesFavorite, Recipe};

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
        let homies_favorites = query_file_as!(
            HomiesFavorite,
            "src/sql/get_homies_favorites.sql",
            current_homie.id
        )
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
