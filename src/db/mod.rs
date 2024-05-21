use anyhow::Result;
use sqlx::sqlite::SqlitePool;

use crate::domain::Homie;
use crate::Recipe;

pub trait HomieRepository {
    async fn create_homie(&self, name: &str) -> Result<i64>;
    async fn create_homies_favorite(&self, homie_id: i64, recipe_id: i64) -> Result<i64>;
    async fn get_homie_by_id(&self, id: i64) -> Result<Homie>;

     async fn create_recipe(&self, name: &str) -> Result<i64> ;

     async fn get_recipe_by_id(&self, id: i64) -> Result<Recipe> ;
}


impl HomieRepository for SqlitePool {

     async fn create_homie(&self, name: &str) -> Result<i64> {
        Ok(
            sqlx::query("INSERT INTO homies (name) VALUES (?)")
            .bind(name)
            .execute(self)
            .await?
            .last_insert_rowid()
        )
    }

     async fn create_recipe(&self, name: &str) -> Result<i64> {
        Ok(
        sqlx::query("INSERT INTO recipes (name) VALUES (?)")
            .bind(name)
            .execute(self)
            .await?
            .last_insert_rowid()
                    
        )
    }

     async fn create_homies_favorite(
        &self,
        homie_id: i64,
        recipe_id: i64,
    ) -> Result<i64> {
        Ok(
        sqlx::query("INSERT INTO homies_favorites (homie_id, recipe_id) VALUES (?, ?)")
            .bind(homie_id)
            .bind(recipe_id)
            .execute(self)
            .await
                ?
            .last_insert_rowid())
    }

     async fn get_homie_by_id(&self, id: i64) -> Result<Homie> {
        let homie = sqlx::query_as!(Homie, "SELECT * FROM homies WHERE id = ?", id)
            .fetch_one(self)
            .await?;
        Ok(homie)
    }

     async fn get_recipe_by_id(&self, id: i64) -> Result<Recipe> {
        Ok(sqlx::query_as!(Recipe, "SELECT * FROM homies WHERE id = ?", id)
            .fetch_one(self)
            .await?)
    }
}
