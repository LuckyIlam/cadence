use async_trait::async_trait;
use libsql::Connection;

use crate::domain::parametre::ParametresPlanning;
use crate::error::AppError;

#[async_trait]
pub trait ParametreRepository: Send + Sync {
    async fn obtenir_parametres_planning(&self) -> Result<ParametresPlanning, AppError>;
    async fn mettre_a_jour_plage_horaire_tx(
        &self,
        tx: &mut libsql::Transaction,
        heure_ouverture: &str,
        heure_fermeture: &str,
    ) -> Result<ParametresPlanning, AppError>;
}

pub struct LibsqlParametreRepository {
    pub(crate) conn: Connection,
}

impl LibsqlParametreRepository {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl ParametreRepository for LibsqlParametreRepository {
    async fn obtenir_parametres_planning(&self) -> Result<ParametresPlanning, AppError> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, heure_ouverture, heure_fermeture FROM parametres WHERE id = 1",
                libsql::params![],
            )
            .await?;

        let row = rows
            .next()
            .await?
            .ok_or(AppError::NotFound("Paramètres introuvables".into()))?;
        Ok(libsql::de::from_row::<ParametresPlanning>(&row)?)
    }

    async fn mettre_a_jour_plage_horaire_tx(
        &self,
        tx: &mut libsql::Transaction,
        heure_ouverture: &str,
        heure_fermeture: &str,
    ) -> Result<ParametresPlanning, AppError> {
        let mut rows = tx
            .query(
                "UPDATE parametres
                 SET heure_ouverture = ?, heure_fermeture = ?
                 WHERE id = 1
                 RETURNING id, heure_ouverture, heure_fermeture",
                libsql::params![heure_ouverture, heure_fermeture],
            )
            .await?;

        let row = rows
            .next()
            .await?
            .ok_or(AppError::NotFound("Paramètres introuvables".into()))?;
        Ok(libsql::de::from_row::<ParametresPlanning>(&row)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_db() -> Connection {
        let conn = libsql::Builder::new_local(":memory:")
            .build()
            .await
            .expect("failed to create test db")
            .connect()
            .expect("failed to connect test db");
        crate::infrastructure::migrations::cadence_migrations(&conn)
            .await
            .expect("failed to run migrations");
        conn
    }

    #[tokio::test]
    async fn test_obtenir_parametres_defaut() {
        let conn = setup_db().await;
        let r = LibsqlParametreRepository::new(conn);

        let params = r.obtenir_parametres_planning().await.unwrap();
        assert_eq!(params.heure_ouverture, "08:00");
        assert_eq!(params.heure_fermeture, "20:00");
    }

    #[tokio::test]
    async fn test_mettre_a_jour_plage_horaire() {
        let conn = setup_db().await;
        let r = LibsqlParametreRepository::new(conn.clone());

        let mut tx = conn.transaction().await.unwrap();
        let params = r
            .mettre_a_jour_plage_horaire_tx(&mut tx, "09:00", "18:00")
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
