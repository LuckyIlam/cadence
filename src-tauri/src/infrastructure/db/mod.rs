// PR 1 : le nouveau contrat DB est posé mais encore inutilisé en interne
// (consommé par les repositories en PR 2). `dead_code` est volontairement
// neutralisé sur tout le module jusqu'à cette adoption.
#![allow(dead_code)]

#[allow(clippy::module_inception)]
// db::db : nommage du design D1 (db/{db,params,row,transaction}.rs)
pub mod db;
pub mod params;
pub mod row;
pub mod transaction;

// Re-exports publics du nouveau contrat DB, consommés à partir de la PR 2
// (repositories derrière `dyn Db`). Non utilisés en interne dans cette PR :
// `unused_imports` est donc volontairement neutralisé ici.
#[allow(unused_imports)]
pub use db::{Db, DbExt};
#[allow(unused_imports)]
pub use params::{DbParams, DbValue, IntoParams, ToDbValue};
#[allow(unused_imports)]
pub use row::{DbRow, DeserializeRow, RowView};
#[allow(unused_imports)]
pub use transaction::{DbTransaction, DbTransactionExt};

use std::path::Path;
use std::sync::Arc;

use super::config::{ConnexionConfig, Driver, ModeConnexion};
use super::migrations::cadence_migrations;
use crate::drivers::libsql::db::LibsqlDb;
use crate::drivers::libsql::repositories::{
    LibsqlActiviteRepository, LibsqlAdhesionRepository, LibsqlParametreRepository,
    LibsqlPersonneRepository, LibsqlPlanningRepository,
};
use crate::error::AppError;

pub struct AppState {
    pub db: Arc<dyn Db>,
    pub personne_repo: LibsqlPersonneRepository,
    pub activite_repo: LibsqlActiviteRepository,
    pub adhesion_repo: LibsqlAdhesionRepository,
    pub planning_repo: LibsqlPlanningRepository,
    pub param_repo: LibsqlParametreRepository,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[allow(dead_code)]
pub struct IdRow {
    pub id: i64,
}

pub async fn init_connection(
    config: &ConnexionConfig,
    app_dir: &Path,
) -> Result<Arc<dyn Db>, AppError> {
    match config.driver {
        Driver::Sqlite => {}
        Driver::Postgres => {
            unimplemented!("Driver Postgres : prévu dans un change dédié (db-driver-abstraction)")
        }
        Driver::Mysql => {
            unimplemented!("Driver Mysql : prévu dans un change dédié (db-driver-abstraction)")
        }
    }

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

    // SQLite (et libsql) désactive l'application des clés étrangères par défaut
    // par connexion : sans ce pragma, les FOREIGN KEY des migrations ne sont que
    // documentaires. À activer explicitement hors transaction.
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .await
        .map_err(AppError::from)?;

    cadence_migrations(&conn).await?;

    Ok(Arc::new(LibsqlDb::new(conn)))
}

pub fn init_app_state(db: Arc<dyn Db>) -> AppState {
    AppState {
        db: db.clone(),
        personne_repo: LibsqlPersonneRepository::new(db.clone()),
        activite_repo: LibsqlActiviteRepository::new(db.clone()),
        adhesion_repo: LibsqlAdhesionRepository::new(db.clone()),
        planning_repo: LibsqlPlanningRepository::new(db.clone()),
        param_repo: LibsqlParametreRepository::new(db),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn local_conn() -> libsql::Connection {
        let database = libsql::Builder::new_local(":memory:")
            .build()
            .await
            .expect("base en mémoire");
        let conn = database.connect().expect("connexion");
        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .await
            .expect("pragma");
        cadence_migrations(&conn).await.expect("migrations");
        conn
    }

    fn est_erreur_foreign_key(err: &libsql::Error) -> bool {
        let m = err.to_string().to_lowercase();
        m.contains("foreign key") || m.contains("constraint failed")
    }

    #[tokio::test]
    async fn fk_refuse_adhesion_personne_inexistante() {
        let conn = local_conn().await;
        let err = conn
            .execute(
                "INSERT INTO adhesions (personne_id, annee_scolaire) VALUES (?, ?)",
                libsql::params![99999, "2025-2026"],
            )
            .await
            .expect_err("la clé étrangère doit bloquer l'insertion");
        assert!(est_erreur_foreign_key(&err));
    }

    #[tokio::test]
    async fn fk_refuse_liaison_personne_inexistante() {
        let conn = local_conn().await;
        conn.execute(
            "INSERT INTO activites (nom) VALUES (?)",
            libsql::params!["Poterie"],
        )
        .await
        .expect("activité insérée");
        let err = conn
            .execute(
                "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
                 VALUES (?, ?, ?, ?)",
                libsql::params![1, 99999, "2025-2026", "participant"],
            )
            .await
            .expect_err("la clé étrangère personne doit bloquer l'insertion");
        assert!(est_erreur_foreign_key(&err));
    }

    #[tokio::test]
    async fn fk_refuse_suppression_activite_referencee() {
        let conn = local_conn().await;
        conn.execute(
            "INSERT INTO activites (nom) VALUES (?)",
            libsql::params!["Poterie"],
        )
        .await
        .expect("activité insérée");
        conn.execute(
            "INSERT INTO creneaux_activite (activite_id, jour_semaine, heure_debut, heure_fin, annee_scolaire)
             VALUES (1, 1, '14:00', '16:00', '2025-2026')",
            libsql::params![],
        )
        .await
        .expect("créneau inséré");
        let err = conn
            .execute("DELETE FROM activites WHERE id = 1", libsql::params![])
            .await
            .expect_err("la clé étrangère doit bloquer la suppression");
        assert!(est_erreur_foreign_key(&err));
    }
}
