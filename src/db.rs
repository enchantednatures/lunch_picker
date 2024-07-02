use anyhow::Result;
use sqlx::Pool;
use sqlx::Sqlite;

pub trait Migrator {
    async fn migrate(&self) -> Result<()>;
}

impl Migrator for Pool<Sqlite> {
    #[tracing::instrument(skip(self))]
    async fn migrate(&self) -> Result<()> {
        Ok(sqlx::migrate!("./migrations").run(self).await?)
    }
}
