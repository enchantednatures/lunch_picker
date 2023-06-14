use sqlx::sqlite::SqlitePool;

use crate::domain::Homie;

pub struct Repository {
    db_pool: SqlitePool,
}

impl Repository {
    pub fn new(db_pool: SqlitePool) -> Self {
        Self { db_pool }
    }

    pub async fn create_homie(&self, name: &str) -> Result<i64, sqlx::Error> {
        sqlx::query("INSERT INTO homies (name) VALUES (?)")
            .bind(name)
            .execute(&self.db_pool)
            .await
            .map(|res| res.last_insert_rowid())
    }

    pub async fn create_recipe(&self, name: &str) -> Result<i64, sqlx::Error> {
        let recipe_id = sqlx::query("INSERT INTO recipes (name) VALUES (?)")
            .bind(name)
            .execute(&self.db_pool)
            .await
            .map(|res| res.last_insert_rowid());
        recipe_id
    }

    pub async fn create_homies_favorite(
        &self,
        homie_id: i64,
        recipe_id: i64,
    ) -> Result<i64, sqlx::Error> {
        let homies_favorite_id =
            sqlx::query("INSERT INTO homies_favorites (homie_id, recipe_id) VALUES (?, ?)")
                .bind(homie_id)
                .bind(recipe_id)
                .execute(&self.db_pool)
                .await
                .map(|res| res.last_insert_rowid());
        homies_favorite_id
    }

    pub async fn get_homie_by_id(&self, id: i64) -> Result<Homie, sqlx::Error> {
        let homie = sqlx::query_as!(Homie, "SELECT * FROM homies WHERE id = ?", id)
            .fetch_one(&self.db_pool)
            .await?;
        Ok(homie)
    }
}
