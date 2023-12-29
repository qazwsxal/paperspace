use crate::config::SqliteConfig;
use sqlx::{
    self,
    migrate::{MigrateError, Migrator},
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    SqlitePool,
};
pub mod queries;

static MIG: Migrator = sqlx::migrate!("./migrations/");


pub async fn init(config: &SqliteConfig) -> Result<SqlitePool, MigrateError> {
    let c_opts = SqliteConnectOptions::new()
        .filename(&config.db_path)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal)
        .create_if_missing(true)
        .read_only(config.read_only);
    // Specifiy higher max connections, we're using Wal, so writes don't lock reads.
    let pool = SqlitePoolOptions::new()
        .max_connections(config.max_connections)
        .connect_lazy_with(c_opts);
    MIG.run(&pool).await?;
    Ok(pool)
}
