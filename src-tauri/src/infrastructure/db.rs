use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;

use crate::repositories::{
    SqliteActiviteRepository, SqliteAdhesionRepository, SqliteParametreRepository,
    SqlitePersonneRepository, SqlitePlanningRepository,
};

pub struct AppState {
    pub pool: SqlitePool,
    pub personne_repo: SqlitePersonneRepository,
    pub activite_repo: SqliteActiviteRepository,
    pub adhesion_repo: SqliteAdhesionRepository,
    pub planning_repo: SqlitePlanningRepository,
    pub param_repo: SqliteParametreRepository,
}

pub async fn init_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

pub fn init_app_state(pool: SqlitePool) -> AppState {
    AppState {
        personne_repo: SqlitePersonneRepository::new(pool.clone()),
        activite_repo: SqliteActiviteRepository::new(pool.clone()),
        adhesion_repo: SqliteAdhesionRepository::new(pool.clone()),
        planning_repo: SqlitePlanningRepository::new(pool.clone()),
        param_repo: SqliteParametreRepository::new(pool.clone()),
        pool,
    }
}
