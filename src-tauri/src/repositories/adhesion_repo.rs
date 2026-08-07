use async_trait::async_trait;
use libsql::Connection;

use crate::domain::adhesion::{Adhesion, CreateAdhesion, UpdateAdhesion};
use crate::error::AppError;
use crate::infrastructure::hrana_guard;

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
    pub(crate) conn: Connection,
}

impl LibsqlAdhesionRepository {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl AdhesionRepository for LibsqlAdhesionRepository {
    async fn create(&self, input: CreateAdhesion, utilisateur: &str) -> Result<Adhesion, AppError> {
        let maintenant = crate::infrastructure::audit::maintenant_utc();
        let mut rows = hrana_guard::query_avec_retry(
            &self.conn,
            "INSERT INTO adhesions (personne_id, annee_scolaire, reglee, note_paiement, modifie_par, modifie_le)
                 VALUES (?, ?, ?, ?, ?, ?)
                 RETURNING id, personne_id, annee_scolaire, reglee, note_paiement, version",
            libsql::params![
                input.personne_id,
                input.annee_scolaire,
                input.reglee,
                input.note_paiement,
                utilisateur,
                maintenant
            ],
        )
        .await?;

        let row = rows
            .next()
            .await?
            .ok_or(AppError::NotFound("Adhésion introuvable".into()))?;
        let valeur = libsql::de::from_row::<Adhesion>(&row)?;
        hrana_guard::vider_cursor(&mut rows).await?;
        Ok(valeur)
    }

    async fn update(
        &self,
        id: i64,
        input: UpdateAdhesion,
        utilisateur: &str,
    ) -> Result<Adhesion, AppError> {
        let maintenant = crate::infrastructure::audit::maintenant_utc();
        let affected = hrana_guard::execute_avec_retry(
            &self.conn,
            "UPDATE adhesions
                 SET reglee = ?, note_paiement = ?, modifie_par = ?, modifie_le = ?, version = version + 1
                 WHERE id = ? AND version = ?",
            libsql::params![
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
            let mut existe_rows = hrana_guard::query_avec_retry(
                &self.conn,
                "SELECT id FROM adhesions WHERE id = ?",
                libsql::params![id],
            )
            .await?;
            let existe = existe_rows.next().await?.is_some();
            hrana_guard::vider_cursor(&mut existe_rows).await?;
            if existe {
                return Err(AppError::Conflict(
                    crate::infrastructure::audit::MESSAGE_CONFLIT.to_string(),
                ));
            }
            return Err(AppError::NotFound("Adhésion introuvable".into()));
        }
        let mut rows = hrana_guard::query_avec_retry(
            &self.conn,
            "SELECT id, personne_id, annee_scolaire, reglee, note_paiement, version
                 FROM adhesions WHERE id = ?",
            libsql::params![id],
        )
        .await?;
        let row = rows
            .next()
            .await?
            .ok_or(AppError::NotFound("Adhésion introuvable".into()))?;
        let valeur = libsql::de::from_row::<Adhesion>(&row)?;
        hrana_guard::vider_cursor(&mut rows).await?;
        Ok(valeur)
    }

    async fn list_by_personne(&self, personne_id: i64) -> Result<Vec<Adhesion>, AppError> {
        let mut rows = hrana_guard::query_avec_retry(
            &self.conn,
            "SELECT id, personne_id, annee_scolaire, reglee, note_paiement, version
                 FROM adhesions WHERE personne_id = ? ORDER BY annee_scolaire DESC",
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
        let conn = setup_db().await;
        seed_personne(&conn).await;
        let r = repo(conn.clone());

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
        let conn = setup_db().await;
        seed_personne(&conn).await;
        let r = repo(conn.clone());

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
        let conn = setup_db().await;
        seed_personne(&conn).await;
        let r = repo(conn.clone());

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
        let conn = setup_db().await;
        seed_personne(&conn).await;
        let r = repo(conn);

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
