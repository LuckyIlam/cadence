use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::domain::parametre::ParametresPlanning;
use crate::error::AppError;

#[async_trait]
pub trait ParametreRepository: Send + Sync {
    async fn obtenir_parametres_planning(&self) -> Result<ParametresPlanning, AppError>;
    async fn mettre_a_jour_plage_horaire_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        heure_ouverture: &str,
        heure_fermeture: &str,
    ) -> Result<ParametresPlanning, AppError>;
}

pub struct SqliteParametreRepository {
    pub(crate) pool: SqlitePool,
}

impl SqliteParametreRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ParametreRepository for SqliteParametreRepository {
    async fn obtenir_parametres_planning(&self) -> Result<ParametresPlanning, AppError> {
        let row = sqlx::query_as::<_, ParametresPlanning>(
            "SELECT id, heure_ouverture, heure_fermeture FROM parametres WHERE id = 1",
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    async fn mettre_a_jour_plage_horaire_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        heure_ouverture: &str,
        heure_fermeture: &str,
    ) -> Result<ParametresPlanning, AppError> {
        let row = sqlx::query_as::<_, ParametresPlanning>(
            "UPDATE parametres
             SET heure_ouverture = ?, heure_fermeture = ?
             WHERE id = 1
             RETURNING id, heure_ouverture, heure_fermeture",
        )
        .bind(heure_ouverture)
        .bind(heure_fermeture)
        .fetch_one(&mut **tx)
        .await?;

        Ok(row)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_db() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("failed to create test pool");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("failed to run migrations");
        pool
    }

    #[tokio::test]
    async fn test_obtenir_parametres_defaut() {
        let pool = setup_db().await;
        let r = SqliteParametreRepository::new(pool);

        let params = r.obtenir_parametres_planning().await.unwrap();
        assert_eq!(params.heure_ouverture, "08:00");
        assert_eq!(params.heure_fermeture, "20:00");
    }

    #[tokio::test]
    async fn test_mettre_a_jour_plage_horaire() {
        let pool = setup_db().await;
        let r = SqliteParametreRepository::new(pool.clone());

        let mut tx = pool.begin().await.unwrap();
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
