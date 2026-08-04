use async_trait::async_trait;
use libsql::Connection;

use crate::domain::adhesion::{Adhesion, CreateAdhesion, UpdateAdhesion};
use crate::error::AppError;

#[async_trait]
pub trait AdhesionRepository: Send + Sync {
    async fn create(&self, input: CreateAdhesion) -> Result<Adhesion, AppError>;
    async fn update(&self, id: i64, input: UpdateAdhesion) -> Result<Adhesion, AppError>;
    async fn list_by_personne(&self, personne_id: i64) -> Result<Vec<Adhesion>, AppError>;
}

pub struct LibsqlAdhesionRepository {
    pub(crate) conn: Connection,
}

impl LibsqlAdhesionRepository {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl AdhesionRepository for LibsqlAdhesionRepository {
    async fn create(&self, input: CreateAdhesion) -> Result<Adhesion, AppError> {
        let mut rows = self
            .conn
            .query(
                "INSERT INTO adhesions (personne_id, annee_scolaire, reglee, note_paiement)
                 VALUES (?, ?, ?, ?)
                 RETURNING *",
                libsql::params![
                    input.personne_id,
                    input.annee_scolaire,
                    input.reglee,
                    input.note_paiement
                ],
            )
            .await?;

        let row = rows
            .next()
            .await?
            .ok_or(AppError::NotFound("Adhésion introuvable".into()))?;
        Ok(libsql::de::from_row::<Adhesion>(&row)?)
    }

    async fn update(&self, id: i64, input: UpdateAdhesion) -> Result<Adhesion, AppError> {
        let mut rows = self
            .conn
            .query(
                "UPDATE adhesions
                 SET reglee = ?, note_paiement = ?
                 WHERE id = ?
                 RETURNING *",
                libsql::params![input.reglee, input.note_paiement, id],
            )
            .await?;

        let row = rows
            .next()
            .await?
            .ok_or(AppError::NotFound("Adhésion introuvable".into()))?;
        Ok(libsql::de::from_row::<Adhesion>(&row)?)
    }

    async fn list_by_personne(&self, personne_id: i64) -> Result<Vec<Adhesion>, AppError> {
        let mut rows = self
            .conn
            .query(
                "SELECT * FROM adhesions WHERE personne_id = ? ORDER BY annee_scolaire DESC",
                libsql::params![personne_id],
            )
            .await?;

        let mut donnees = Vec::new();
        while let Some(row) = rows.next().await? {
            donnees.push(libsql::de::from_row::<Adhesion>(&row)?);
        }

        Ok(donnees)
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

    async fn seed_personne(conn: &Connection) -> i64 {
        conn.execute(
            "INSERT INTO personnes_physiques (nom, prenom, date_naissance)
             VALUES ('Test', 'User', '2000-01-15')",
            libsql::params![],
        )
        .await
        .expect("failed to seed personne");
        1
    }

    fn repo(conn: Connection) -> LibsqlAdhesionRepository {
        LibsqlAdhesionRepository::new(conn)
    }

    #[tokio::test]
    async fn test_create_adhesion() {
        let conn = setup_db().await;
        seed_personne(&conn).await;
        let r = repo(conn);

        let a = r
            .create(CreateAdhesion {
                personne_id: 1,
                annee_scolaire: "2025-2026".into(),
                reglee: true,
                note_paiement: None,
            })
            .await
            .unwrap();
        assert_eq!(a.personne_id, 1);
        assert_eq!(a.annee_scolaire, "2025-2026");
        assert!(a.reglee);
    }

    #[tokio::test]
    async fn test_list_by_personne() {
        let conn = setup_db().await;
        seed_personne(&conn).await;
        let r = repo(conn.clone());

        r.create(CreateAdhesion {
            personne_id: 1,
            annee_scolaire: "2024-2025".into(),
            reglee: false,
            note_paiement: None,
        })
        .await
        .unwrap();
        r.create(CreateAdhesion {
            personne_id: 1,
            annee_scolaire: "2025-2026".into(),
            reglee: true,
            note_paiement: Some("chèque".into()),
        })
        .await
        .unwrap();

        let list = r.list_by_personne(1).await.unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].annee_scolaire, "2025-2026");
    }

    #[tokio::test]
    async fn test_update_adhesion() {
        let conn = setup_db().await;
        seed_personne(&conn).await;
        let r = repo(conn.clone());

        let a = r
            .create(CreateAdhesion {
                personne_id: 1,
                annee_scolaire: "2025-2026".into(),
                reglee: false,
                note_paiement: None,
            })
            .await
            .unwrap();

        let updated = r
            .update(
                a.id,
                UpdateAdhesion {
                    reglee: true,
                    note_paiement: Some("espèces".into()),
                },
            )
            .await
            .unwrap();

        assert!(updated.reglee);
        assert_eq!(updated.note_paiement.as_deref(), Some("espèces"));
    }
}
