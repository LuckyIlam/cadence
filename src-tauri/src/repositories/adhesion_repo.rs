use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::adhesion::{Adhesion, CreateAdhesion, UpdateAdhesion};
use crate::error::AppError;
use crate::infrastructure::db::{Db, DbExt, DeserializeRow, RowView};

#[async_trait]
pub trait AdhesionRepository: Send + Sync {
    async fn create(&self, input: CreateAdhesion, utilisateur: &str) -> Result<Adhesion, AppError>;
    async fn update(
        &self,
        id: i64,
        input: UpdateAdhesion,
        utilisateur: &str,
    ) -> Result<Adhesion, AppError>;
    async fn list_by_personne(&self, personne_id: i64) -> Result<Vec<Adhesion>, AppError>;
}

pub struct LibsqlAdhesionRepository {
    db: Arc<dyn Db>,
}

impl LibsqlAdhesionRepository {
    pub fn new(db: Arc<dyn Db>) -> Self {
        Self { db }
    }
}

impl DeserializeRow for Adhesion {
    fn from_row(row: &dyn RowView) -> Result<Self, AppError> {
        Ok(Adhesion {
            id: row.get_i64(0)?,
            personne_id: row.get_i64(1)?,
            annee_scolaire: row.get_str(2)?.to_string(),
            reglee: row.get_bool(3)?,
            note_paiement: row.get_opt_str(4)?.map(String::from),
            version: row.get_i64(5)?,
        })
    }
}

struct IdRow {
    #[allow(dead_code)]
    id: i64,
}

impl DeserializeRow for IdRow {
    fn from_row(row: &dyn RowView) -> Result<Self, AppError> {
        Ok(IdRow {
            id: row.get_i64(0)?,
        })
    }
}

#[async_trait]
impl AdhesionRepository for LibsqlAdhesionRepository {
    async fn create(&self, input: CreateAdhesion, utilisateur: &str) -> Result<Adhesion, AppError> {
        let maintenant = crate::infrastructure::audit::maintenant_utc();
        self.db
            .fetch_one(
                "INSERT INTO adhesions (personne_id, annee_scolaire, reglee, note_paiement, modifie_par, modifie_le)
                 VALUES (?, ?, ?, ?, ?, ?)
                 RETURNING id, personne_id, annee_scolaire, reglee, note_paiement, version",
                crate::params![
                    input.personne_id,
                    input.annee_scolaire,
                    input.reglee,
                    input.note_paiement,
                    utilisateur,
                    maintenant
                ],
            )
            .await
    }

    async fn update(
        &self,
        id: i64,
        input: UpdateAdhesion,
        utilisateur: &str,
    ) -> Result<Adhesion, AppError> {
        let maintenant = crate::infrastructure::audit::maintenant_utc();
        let affected = self
            .db
            .execute(
                "UPDATE adhesions
                 SET reglee = ?, note_paiement = ?, modifie_par = ?, modifie_le = ?, version = version + 1
                 WHERE id = ? AND version = ?",
                crate::params![
                    input.reglee,
                    input.note_paiement,
                    utilisateur,
                    maintenant,
                    id,
                    input.version
                ],
            )
            .await?;
        if affected == 0 {
            let existe = self
                .db
                .fetch_optional::<IdRow>(
                    "SELECT id FROM adhesions WHERE id = ?",
                    crate::params![id],
                )
                .await?
                .is_some();
            if existe {
                return Err(AppError::Conflict(
                    crate::infrastructure::audit::MESSAGE_CONFLIT.to_string(),
                ));
            }
            return Err(AppError::NotFound("Adhésion introuvable".into()));
        }
        self.db
            .fetch_one(
                "SELECT id, personne_id, annee_scolaire, reglee, note_paiement, version
                 FROM adhesions WHERE id = ?",
                crate::params![id],
            )
            .await
    }

    async fn list_by_personne(&self, personne_id: i64) -> Result<Vec<Adhesion>, AppError> {
        self.db
            .fetch_all(
                "SELECT id, personne_id, annee_scolaire, reglee, note_paiement, version
                 FROM adhesions WHERE personne_id = ? ORDER BY annee_scolaire DESC",
                crate::params![personne_id],
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

    async fn seed_personne(db: &dyn Db) -> i64 {
        db.execute(
            "INSERT INTO personnes_physiques (nom, prenom, date_naissance)
             VALUES ('Test', 'User', '2000-01-15')",
            crate::params![],
        )
        .await
        .expect("failed to seed personne");
        1
    }

    fn repo(db: Arc<dyn Db>) -> LibsqlAdhesionRepository {
        LibsqlAdhesionRepository::new(db)
    }

    #[tokio::test]
    async fn test_create_adhesion() {
        let db = setup_db().await;
        seed_personne(db.as_ref()).await;
        let r = repo(db);

        let a = r
            .create(
                CreateAdhesion {
                    personne_id: 1,
                    annee_scolaire: "2025-2026".into(),
                    reglee: true,
                    note_paiement: None,
                },
                "alice",
            )
            .await
            .unwrap();
        assert_eq!(a.personne_id, 1);
        assert_eq!(a.annee_scolaire, "2025-2026");
        assert!(a.reglee);
        assert_eq!(a.version, 1);
    }

    #[tokio::test]
    async fn test_list_by_personne() {
        let db = setup_db().await;
        seed_personne(db.as_ref()).await;
        let r = repo(db.clone());

        r.create(
            CreateAdhesion {
                personne_id: 1,
                annee_scolaire: "2024-2025".into(),
                reglee: false,
                note_paiement: None,
            },
            "alice",
        )
        .await
        .unwrap();
        r.create(
            CreateAdhesion {
                personne_id: 1,
                annee_scolaire: "2025-2026".into(),
                reglee: true,
                note_paiement: Some("chèque".into()),
            },
            "alice",
        )
        .await
        .unwrap();

        let list = r.list_by_personne(1).await.unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].annee_scolaire, "2025-2026");
    }

    #[tokio::test]
    async fn test_update_adhesion() {
        let db = setup_db().await;
        seed_personne(db.as_ref()).await;
        let r = repo(db.clone());

        let a = r
            .create(
                CreateAdhesion {
                    personne_id: 1,
                    annee_scolaire: "2025-2026".into(),
                    reglee: false,
                    note_paiement: None,
                },
                "alice",
            )
            .await
            .unwrap();

        let updated = r
            .update(
                a.id,
                UpdateAdhesion {
                    reglee: true,
                    note_paiement: Some("espèces".into()),
                    version: a.version,
                },
                "bob",
            )
            .await
            .unwrap();

        assert!(updated.reglee);
        assert_eq!(updated.note_paiement.as_deref(), Some("espèces"));
        assert_eq!(updated.version, a.version + 1);
    }

    #[tokio::test]
    async fn test_update_adhesion_version_obsolete_conflit() {
        let db = setup_db().await;
        seed_personne(db.as_ref()).await;
        let r = repo(db.clone());

        let a = r
            .create(
                CreateAdhesion {
                    personne_id: 1,
                    annee_scolaire: "2025-2026".into(),
                    reglee: false,
                    note_paiement: None,
                },
                "alice",
            )
            .await
            .unwrap();

        r.update(
            a.id,
            UpdateAdhesion {
                reglee: true,
                note_paiement: None,
                version: a.version,
            },
            "bob",
        )
        .await
        .unwrap();

        let err = r
            .update(
                a.id,
                UpdateAdhesion {
                    reglee: true,
                    note_paiement: Some("carte".into()),
                    version: a.version,
                },
                "carol",
            )
            .await
            .unwrap_err();

        assert!(matches!(err, AppError::Conflict(_)));
    }

    #[tokio::test]
    async fn test_update_adhesion_inexistante_not_found() {
        let db = setup_db().await;
        seed_personne(db.as_ref()).await;
        let r = repo(db);

        let err = r
            .update(
                999,
                UpdateAdhesion {
                    reglee: true,
                    note_paiement: None,
                    version: 1,
                },
                "alice",
            )
            .await
            .unwrap_err();

        assert!(matches!(err, AppError::NotFound(_)));
    }
}
