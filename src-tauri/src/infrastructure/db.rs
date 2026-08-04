use std::path::Path;

use libsql::Connection;

use super::config::{ConnexionConfig, ModeConnexion};
use super::migrations::cadence_migrations;
use crate::error::AppError;
use crate::repositories::{
    LibsqlActiviteRepository, LibsqlAdhesionRepository, LibsqlParametreRepository,
    LibsqlPersonneRepository, LibsqlPlanningRepository,
};

pub struct AppState {
    pub conn: Connection,
    pub personne_repo: LibsqlPersonneRepository,
    pub activite_repo: LibsqlActiviteRepository,
    pub adhesion_repo: LibsqlAdhesionRepository,
    pub planning_repo: LibsqlPlanningRepository,
    pub param_repo: LibsqlParametreRepository,
}

pub async fn init_connection(
    config: &ConnexionConfig,
    app_dir: &Path,
) -> Result<Connection, AppError> {
    let database = match config.mode {
        ModeConnexion::Mono => {
            libsql::Builder::new_local(app_dir.join("cadence.db"))
                .build()
                .await?
        }
        ModeConnexion::Multi => {
            let url = config.url.clone().ok_or_else(|| {
                AppError::Validation("L'URL de la base distante est requise en mode multi".into())
            })?;
            let token = config.token.clone().ok_or_else(|| {
                AppError::Validation("La clé d'accès est requise en mode multi".into())
            })?;
            libsql::Builder::new_remote(url, token).build().await?
        }
    };

    let conn = database.connect()?;
    cadence_migrations(&conn).await?;

    Ok(conn)
}

pub fn init_app_state(conn: Connection) -> AppState {
    AppState {
        personne_repo: LibsqlPersonneRepository::new(conn.clone()),
        activite_repo: LibsqlActiviteRepository::new(conn.clone()),
        adhesion_repo: LibsqlAdhesionRepository::new(conn.clone()),
        planning_repo: LibsqlPlanningRepository::new(conn.clone()),
        param_repo: LibsqlParametreRepository::new(conn.clone()),
        conn,
    }
}
