use anyhow::Result;
use sqlx::Pool;
#[cfg(feature = "postgres")]
use sqlx::Postgres;
#[cfg(feature = "sqlite")]
use sqlx::Sqlite;

pub trait Migrator {
    async fn migrate(&self) -> Result<()>;
}

#[cfg(feature = "postgres")]
impl Migrator for Pool<Postgres> {
    #[tracing::instrument(skip(self))]
    async fn migrate(&self) -> Result<()> {
        Ok(sqlx::migrate!("./migrations/postgres").run(self).await?)
    }
}

#[cfg(feature = "sqlite")]
impl Migrator for Pool<Sqlite> {
    #[tracing::instrument(skip(self))]
    async fn migrate(&self) -> Result<()> {
        Ok(sqlx::migrate!("./migrations/sqlite").run(self).await?)
    }
}
