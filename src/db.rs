use anyhow::Result;
use sqlx::Pool;
use sqlx::Postgres;

pub trait Migrator {
    async fn migrate(&self) -> Result<()>;
}

impl Migrator for Pool<Postgres> {
    #[tracing::instrument(skip(self))]
    async fn migrate(&self) -> Result<()> {
        Ok(sqlx::migrate!("./migrations").run(self).await?)
    }
}
