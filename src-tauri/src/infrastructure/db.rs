use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;

pub struct AppState {
    pub pool: SqlitePool,
}

pub async fn init_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
