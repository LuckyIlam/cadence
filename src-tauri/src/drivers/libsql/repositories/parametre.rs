use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::parametre::ParametresPlanning;
use crate::error::AppError;
use crate::infrastructure::db::{
    Db, DbExt, DbTransaction, DbTransactionExt, DeserializeRow, RowView,
};

#[async_trait]
pub trait ParametreRepository: Send + Sync {
    async fn obtenir_parametres_planning(&self) -> Result<ParametresPlanning, AppError>;
    async fn mettre_a_jour_plage_horaire_tx(
        &self,
        tx: &mut dyn DbTransaction,
        heure_ouverture: &str,
        heure_fermeture: &str,
        utilisateur: &str,
    ) -> Result<ParametresPlanning, AppError>;
}

pub struct LibsqlParametreRepository {
    db: Arc<dyn Db>,
}

impl LibsqlParametreRepository {
    pub fn new(db: Arc<dyn Db>) -> Self {
        Self { db }
    }
}

impl DeserializeRow for ParametresPlanning {
    fn from_row(row: &dyn RowView) -> Result<Self, AppError> {
        Ok(ParametresPlanning {
            id: row.get_i64(0)?,
            heure_ouverture: row.get_str(1)?.to_string(),
            heure_fermeture: row.get_str(2)?.to_string(),
        })
    }
}

#[async_trait]
impl ParametreRepository for LibsqlParametreRepository {
    async fn obtenir_parametres_planning(&self) -> Result<ParametresPlanning, AppError> {
        self.db
            .fetch_one(
                "SELECT id, heure_ouverture, heure_fermeture FROM parametres WHERE id = 1",
                crate::params![],
            )
            .await
    }

    async fn mettre_a_jour_plage_horaire_tx(
        &self,
        tx: &mut dyn DbTransaction,
        heure_ouverture: &str,
        heure_fermeture: &str,
        utilisateur: &str,
    ) -> Result<ParametresPlanning, AppError> {
        let maintenant = crate::infrastructure::audit::maintenant_utc();
        tx.fetch_one(
            "UPDATE parametres
             SET heure_ouverture = ?, heure_fermeture = ?, modifie_par = ?, modifie_le = ?
             WHERE id = 1
             RETURNING id, heure_ouverture, heure_fermeture",
            crate::params![heure_ouverture, heure_fermeture, utilisateur, maintenant],
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::drivers::libsql::db::LibsqlDb;

    async fn setup_db() -> Arc<dyn Db> {
        let conn = libsql::Builder::new_local(":memory:")
            .build()
            .await
            .expect("failed to create test db")
            .connect()
            .expect("failed to connect test db");
        crate::infrastructure::migrations::cadence_migrations(&conn)
            .await
            .expect("failed to run migrations");
        Arc::new(LibsqlDb::new(conn))
    }

    #[tokio::test]
    async fn test_obtenir_parametres_defaut() {
        let db = setup_db().await;
        let r = LibsqlParametreRepository::new(db);

        let params = r.obtenir_parametres_planning().await.unwrap();
        assert_eq!(params.heure_ouverture, "08:00");
        assert_eq!(params.heure_fermeture, "20:00");
    }

    #[tokio::test]
    async fn test_mettre_a_jour_plage_horaire() {
        let db = setup_db().await;
        let r = LibsqlParametreRepository::new(db.clone());

        let mut tx = db.begin().await.unwrap();
        let params = r
            .mettre_a_jour_plage_horaire_tx(&mut *tx, "09:00", "18:00", "alice")
            .await
            .unwrap();
        tx.commit().await.unwrap();
        assert_eq!(params.heure_ouverture, "09:00");
        assert_eq!(params.heure_fermeture, "18:00");

        // La mise à jour persiste : relecture depuis la base
        let reread = r.obtenir_parametres_planning().await.unwrap();
        assert_eq!(reread.heure_ouverture, "09:00");
        assert_eq!(reread.heure_fermeture, "18:00");
    }
}
