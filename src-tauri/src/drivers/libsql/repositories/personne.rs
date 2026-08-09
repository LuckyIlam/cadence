use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::personne::{
    CreatePersonne, CriteresRecherchePersonnes, Pagination, Personne, ResultatRecherchePersonnes,
    UpdatePersonne,
};
use crate::error::AppError;
use crate::infrastructure::db::{Db, DbExt, DbValue};
use crate::repositories::rows::personne::TotalRow;
use crate::repositories::PersonneRepository;

pub struct LibsqlPersonneRepository {
    db: Arc<dyn Db>,
}

impl LibsqlPersonneRepository {
    pub fn new(db: Arc<dyn Db>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl PersonneRepository for LibsqlPersonneRepository {
    async fn create(&self, input: CreatePersonne, utilisateur: &str) -> Result<Personne, AppError> {
        let maintenant = crate::infrastructure::audit::maintenant_utc();
        self.db
            .fetch_one(
                "INSERT INTO personnes_physiques (nom, prenom, date_naissance, email, telephone, responsable_id, modifie_par, modifie_le)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                 RETURNING id, nom, prenom, date_naissance, email, telephone, responsable_id, version",
                crate::params![
                    input.nom,
                    input.prenom,
                    input.date_naissance.to_string(),
                    input.email,
                    input.telephone,
                    input.responsable_id,
                    utilisateur,
                    maintenant
                ],
            )
            .await
    }

    async fn update(
        &self,
        id: i64,
        input: UpdatePersonne,
        utilisateur: &str,
    ) -> Result<Personne, AppError> {
        let maintenant = crate::infrastructure::audit::maintenant_utc();
        let affected = self
            .db
            .execute(
                "UPDATE personnes_physiques
                 SET nom = ?, prenom = ?, date_naissance = ?, email = ?, telephone = ?, responsable_id = ?,
                     modifie_par = ?, modifie_le = ?, version = version + 1
                 WHERE id = ? AND version = ?",
                crate::params![
                    input.nom,
                    input.prenom,
                    input.date_naissance.to_string(),
                    input.email,
                    input.telephone,
                    input.responsable_id,
                    utilisateur,
                    maintenant,
                    id,
                    input.version
                ],
            )
            .await?;
        if affected == 0 {
            if self.find_by_id(id).await?.is_some() {
                return Err(AppError::Conflict(
                    crate::infrastructure::audit::MESSAGE_CONFLIT.to_string(),
                ));
            }
            return Err(AppError::NotFound("Personne introuvable".into()));
        }
        self.find_by_id(id)
            .await?
            .ok_or(AppError::NotFound("Personne introuvable".into()))
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Personne>, AppError> {
        self.db
            .fetch_optional(
                "SELECT id, nom, prenom, date_naissance, email, telephone, responsable_id, version
                 FROM personnes_physiques WHERE id = ?",
                crate::params![id],
            )
            .await
    }

    async fn rechercher(
        &self,
        criteres: CriteresRecherchePersonnes,
        pagination: Pagination,
    ) -> Result<ResultatRecherchePersonnes, AppError> {
        let annee_scolaire = crate::domain::personne::current_annee_scolaire();
        let pattern = criteres.texte_libre.as_ref().map(|t| format!("%{}%", t));

        let mut conditions: Vec<String> = Vec::new();

        if criteres.texte_libre.is_some() {
            let cols = ["pp.nom", "pp.prenom", "pp.email", "pp.telephone"];
            let ors: Vec<String> = cols
                .iter()
                .map(|c| format!("LOWER({}) LIKE LOWER(?)", c))
                .collect();
            conditions.push(format!("({})", ors.join(" OR ")));
        }

        if criteres.adherent_uniquement {
            conditions.push(
                "EXISTS (SELECT 1 FROM adhesions a WHERE a.personne_id = pp.id AND a.annee_scolaire = ?)"
                    .to_string(),
            );
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!(" WHERE 1=1 AND {}", conditions.join(" AND "))
        };

        // --- count ---
        let count_sql = format!(
            "SELECT COUNT(*) AS count FROM personnes_physiques pp{}",
            where_clause
        );

        let mut count_params: Vec<DbValue> = Vec::new();
        if let Some(ref p) = pattern {
            for _ in 0..4 {
                count_params.push(DbValue::from(p.clone()));
            }
        }
        if criteres.adherent_uniquement {
            count_params.push(DbValue::from(annee_scolaire.clone()));
        }

        let total = self
            .db
            .fetch_optional::<TotalRow>(&count_sql, count_params)
            .await?
            .ok_or_else(|| AppError::Database("Aucune ligne de comptage".into()))?
            .count;

        // --- data ---
        let offset = if pagination.par_page > 0 {
            (pagination.page - 1) * pagination.par_page
        } else {
            0
        };

        let data_sql = if pagination.par_page > 0 {
            format!(
                "SELECT pp.id, pp.nom, pp.prenom, pp.date_naissance, pp.email, pp.telephone, pp.responsable_id, pp.version
                 FROM personnes_physiques pp{} ORDER BY pp.nom, pp.prenom LIMIT ? OFFSET ?",
                where_clause
            )
        } else {
            format!(
                "SELECT pp.id, pp.nom, pp.prenom, pp.date_naissance, pp.email, pp.telephone, pp.responsable_id, pp.version
                 FROM personnes_physiques pp{} ORDER BY pp.nom, pp.prenom",
                where_clause
            )
        };

        let mut data_params: Vec<DbValue> = Vec::new();
        if let Some(ref p) = pattern {
            for _ in 0..4 {
                data_params.push(DbValue::from(p.clone()));
            }
        }
        if criteres.adherent_uniquement {
            data_params.push(DbValue::from(annee_scolaire));
        }
        if pagination.par_page > 0 {
            data_params.push(DbValue::from(pagination.par_page as i64));
            data_params.push(DbValue::from(offset as i64));
        }

        let donnees = self
            .db
            .fetch_all::<Personne>(&data_sql, data_params)
            .await?;

        let pages = if pagination.par_page > 0 {
            (total as f64 / pagination.par_page as f64).ceil() as u32
        } else {
            1
        };

        Ok(ResultatRecherchePersonnes {
            donnees,
            total: total as u32,
            page: pagination.page,
            pages,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::drivers::libsql::db::LibsqlDb;
    use crate::infrastructure::db::{DeserializeRow, RowView};

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

    fn repo(db: Arc<dyn Db>) -> LibsqlPersonneRepository {
        LibsqlPersonneRepository::new(db)
    }

    async fn seed_personne(
        db: &dyn Db,
        nom: &str,
        prenom: &str,
        email: Option<&str>,
        telephone: Option<&str>,
    ) -> Personne {
        db.fetch_one(
            "INSERT INTO personnes_physiques (nom, prenom, date_naissance, email, telephone)
             VALUES (?, ?, ?, ?, ?) RETURNING id, nom, prenom, date_naissance, email, telephone, responsable_id, version",
            crate::params![nom, prenom, "2000-01-15", email, telephone],
        )
        .await
        .expect("failed to seed personne")
    }

    async fn seed_adhesion(db: &dyn Db, personne_id: i64, annee_scolaire: &str) {
        db.execute(
            "INSERT INTO adhesions (personne_id, annee_scolaire, reglee)
             VALUES (?, ?, 1)",
            crate::params![personne_id, annee_scolaire],
        )
        .await
        .expect("failed to seed adhesion");
    }

    #[derive(Debug, Clone)]
    struct AuditRow {
        modifie_par: String,
        modifie_le: String,
    }

    impl DeserializeRow for AuditRow {
        fn from_row(row: &dyn RowView) -> Result<Self, AppError> {
            Ok(AuditRow {
                modifie_par: row.get_str(0)?.to_string(),
                modifie_le: row.get_str(1)?.to_string(),
            })
        }
    }

    #[derive(Debug, Clone)]
    struct ModifieParRow {
        modifie_par: String,
    }

    impl DeserializeRow for ModifieParRow {
        fn from_row(row: &dyn RowView) -> Result<Self, AppError> {
            Ok(ModifieParRow {
                modifie_par: row.get_str(0)?.to_string(),
            })
        }
    }

    #[tokio::test]
    async fn test_texte_libre_cherche_nom() {
        let db = setup_db().await;
        seed_personne(db.as_ref(), "Dupont", "Jean", None, None).await;
        seed_personne(db.as_ref(), "Martin", "Alice", None, None).await;

        let r = repo(db);
        let resultat = r
            .rechercher(
                CriteresRecherchePersonnes {
                    texte_libre: Some("dup".into()),
                    adherent_uniquement: false,
                },
                Pagination {
                    page: 1,
                    par_page: 20,
                },
            )
            .await
            .unwrap();

        assert_eq!(resultat.total, 1);
        assert_eq!(resultat.donnees.len(), 1);
        assert_eq!(resultat.donnees[0].nom, "Dupont");
    }

    #[tokio::test]
    async fn test_texte_libre_cherche_prenom() {
        let db = setup_db().await;
        seed_personne(db.as_ref(), "Dupont", "Jean", None, None).await;
        seed_personne(db.as_ref(), "Martin", "Jeanne", None, None).await;
        seed_personne(db.as_ref(), "Durand", "Pierre", None, None).await;

        let r = repo(db);
        let resultat = r
            .rechercher(
                CriteresRecherchePersonnes {
                    texte_libre: Some("jean".into()),
                    adherent_uniquement: false,
                },
                Pagination {
                    page: 1,
                    par_page: 20,
                },
            )
            .await
            .unwrap();

        assert_eq!(resultat.total, 2);
    }

    #[tokio::test]
    async fn test_texte_libre_cherche_email() {
        let db = setup_db().await;
        seed_personne(
            db.as_ref(),
            "Dupont",
            "Jean",
            Some("jean@example.com"),
            None,
        )
        .await;
        seed_personne(
            db.as_ref(),
            "Martin",
            "Alice",
            Some("alice@gmail.com"),
            None,
        )
        .await;

        let r = repo(db);
        let resultat = r
            .rechercher(
                CriteresRecherchePersonnes {
                    texte_libre: Some("gmail".into()),
                    adherent_uniquement: false,
                },
                Pagination {
                    page: 1,
                    par_page: 20,
                },
            )
            .await
            .unwrap();

        assert_eq!(resultat.total, 1);
        assert_eq!(resultat.donnees[0].nom, "Martin");
    }

    #[tokio::test]
    async fn test_texte_libre_cherche_telephone() {
        let db = setup_db().await;
        seed_personne(db.as_ref(), "Dupont", "Jean", None, Some("0612345678")).await;
        seed_personne(db.as_ref(), "Martin", "Alice", None, Some("0798765432")).await;

        let r = repo(db);
        let resultat = r
            .rechercher(
                CriteresRecherchePersonnes {
                    texte_libre: Some("0612".into()),
                    adherent_uniquement: false,
                },
                Pagination {
                    page: 1,
                    par_page: 20,
                },
            )
            .await
            .unwrap();

        assert_eq!(resultat.total, 1);
        assert_eq!(resultat.donnees[0].nom, "Dupont");
    }

    #[tokio::test]
    async fn test_sans_criteres() {
        let db = setup_db().await;
        seed_personne(db.as_ref(), "C", "X", None, None).await;
        seed_personne(db.as_ref(), "A", "Y", None, None).await;
        seed_personne(db.as_ref(), "B", "Z", None, None).await;

        let r = repo(db);
        let resultat = r
            .rechercher(
                CriteresRecherchePersonnes {
                    texte_libre: None,
                    adherent_uniquement: false,
                },
                Pagination {
                    page: 1,
                    par_page: 20,
                },
            )
            .await
            .unwrap();

        assert_eq!(resultat.total, 3);
        assert_eq!(resultat.donnees.len(), 3);
        assert_eq!(resultat.pages, 1);
    }

    #[tokio::test]
    async fn test_aucun_resultat() {
        let db = setup_db().await;
        seed_personne(db.as_ref(), "Dupont", "Jean", None, None).await;

        let r = repo(db);
        let resultat = r
            .rechercher(
                CriteresRecherchePersonnes {
                    texte_libre: Some("xyzzzzz".into()),
                    adherent_uniquement: false,
                },
                Pagination {
                    page: 1,
                    par_page: 20,
                },
            )
            .await
            .unwrap();

        assert_eq!(resultat.total, 0);
        assert_eq!(resultat.donnees.len(), 0);
        assert_eq!(resultat.pages, 0);
    }

    #[tokio::test]
    async fn test_pagination_page_1() {
        let db = setup_db().await;
        for i in 0..25 {
            seed_personne(db.as_ref(), &format!("Nom{:02}", i), "Prenom", None, None).await;
        }

        let r = repo(db);
        let resultat = r
            .rechercher(
                CriteresRecherchePersonnes {
                    texte_libre: None,
                    adherent_uniquement: false,
                },
                Pagination {
                    page: 1,
                    par_page: 20,
                },
            )
            .await
            .unwrap();

        assert_eq!(resultat.total, 25);
        assert_eq!(resultat.donnees.len(), 20);
        assert_eq!(resultat.page, 1);
        assert_eq!(resultat.pages, 2);
    }

    #[tokio::test]
    async fn test_pagination_page_2() {
        let db = setup_db().await;
        for i in 0..25 {
            seed_personne(db.as_ref(), &format!("Nom{:02}", i), "Prenom", None, None).await;
        }

        let r = repo(db);
        let resultat = r
            .rechercher(
                CriteresRecherchePersonnes {
                    texte_libre: None,
                    adherent_uniquement: false,
                },
                Pagination {
                    page: 2,
                    par_page: 20,
                },
            )
            .await
            .unwrap();

        assert_eq!(resultat.total, 25);
        assert_eq!(resultat.donnees.len(), 5);
        assert_eq!(resultat.page, 2);
        assert_eq!(resultat.pages, 2);
    }

    #[tokio::test]
    async fn test_pagination_par_page_0() {
        let db = setup_db().await;
        for i in 0..25 {
            seed_personne(db.as_ref(), &format!("Nom{:02}", i), "Prenom", None, None).await;
        }

        let r = repo(db);
        let resultat = r
            .rechercher(
                CriteresRecherchePersonnes {
                    texte_libre: None,
                    adherent_uniquement: false,
                },
                Pagination {
                    page: 1,
                    par_page: 0,
                },
            )
            .await
            .unwrap();

        assert_eq!(resultat.total, 25);
        assert_eq!(resultat.donnees.len(), 25);
        assert_eq!(resultat.pages, 1);
    }

    #[tokio::test]
    async fn test_adherent_uniquement() {
        let db = setup_db().await;
        let p1 = seed_personne(db.as_ref(), "Dupont", "Jean", None, None).await;
        let _p2 = seed_personne(db.as_ref(), "Martin", "Alice", None, None).await;
        let p3 = seed_personne(db.as_ref(), "Durand", "Pierre", None, None).await;

        let annee = crate::domain::personne::current_annee_scolaire();
        seed_adhesion(db.as_ref(), p1.id, &annee).await;
        seed_adhesion(db.as_ref(), p3.id, &annee).await;

        let r = repo(db);
        let resultat = r
            .rechercher(
                CriteresRecherchePersonnes {
                    texte_libre: None,
                    adherent_uniquement: true,
                },
                Pagination {
                    page: 1,
                    par_page: 20,
                },
            )
            .await
            .unwrap();

        assert_eq!(resultat.total, 2);
        assert_eq!(resultat.donnees.len(), 2);
    }

    #[tokio::test]
    async fn test_texte_libre_et_adherent() {
        let db = setup_db().await;
        let p1 = seed_personne(db.as_ref(), "Dupont", "Jean", None, None).await;
        let _p2 = seed_personne(db.as_ref(), "Dupond", "Alice", None, None).await;

        let annee = crate::domain::personne::current_annee_scolaire();
        seed_adhesion(db.as_ref(), p1.id, &annee).await;

        let r = repo(db);
        let resultat = r
            .rechercher(
                CriteresRecherchePersonnes {
                    texte_libre: Some("dup".into()),
                    adherent_uniquement: true,
                },
                Pagination {
                    page: 1,
                    par_page: 20,
                },
            )
            .await
            .unwrap();

        assert_eq!(resultat.total, 1);
        assert_eq!(resultat.donnees[0].nom, "Dupont");
    }

    #[tokio::test]
    async fn test_casse_insensible() {
        let db = setup_db().await;
        seed_personne(db.as_ref(), "Dupont", "Jean", None, None).await;

        let r = repo(db);

        let resultat_min = r
            .rechercher(
                CriteresRecherchePersonnes {
                    texte_libre: Some("dup".into()),
                    adherent_uniquement: false,
                },
                Pagination {
                    page: 1,
                    par_page: 20,
                },
            )
            .await
            .unwrap();

        let resultat_maj = r
            .rechercher(
                CriteresRecherchePersonnes {
                    texte_libre: Some("DUP".into()),
                    adherent_uniquement: false,
                },
                Pagination {
                    page: 1,
                    par_page: 20,
                },
            )
            .await
            .unwrap();

        assert_eq!(resultat_min.total, 1);
        assert_eq!(resultat_maj.total, 1);
    }

    #[tokio::test]
    async fn test_create_enregistre_audit() {
        let db = setup_db().await;
        let r = repo(db);

        let creee = r
            .create(
                CreatePersonne {
                    nom: "Dupont".into(),
                    prenom: "Jean".into(),
                    date_naissance: chrono::NaiveDate::from_ymd_opt(2000, 1, 15).unwrap(),
                    email: None,
                    telephone: None,
                    responsable_id: None,
                },
                "alice",
            )
            .await
            .unwrap();

        assert_eq!(creee.version, 1);

        let audit: AuditRow =
            r.db.fetch_one(
                "SELECT modifie_par, modifie_le FROM personnes_physiques WHERE id = ?",
                crate::params![creee.id],
            )
            .await
            .unwrap();
        assert_eq!(audit.modifie_par, "alice");
        assert!(!audit.modifie_le.is_empty());
    }

    #[tokio::test]
    async fn test_update_enregistre_audit_et_incremente_version() {
        let db = setup_db().await;
        let r = repo(db);
        let creee = r
            .create(
                CreatePersonne {
                    nom: "Dupont".into(),
                    prenom: "Jean".into(),
                    date_naissance: chrono::NaiveDate::from_ymd_opt(2000, 1, 15).unwrap(),
                    email: None,
                    telephone: None,
                    responsable_id: None,
                },
                "alice",
            )
            .await
            .unwrap();

        let modifiee = r
            .update(
                creee.id,
                UpdatePersonne {
                    nom: "Dupont".into(),
                    prenom: "Jean-Marc".into(),
                    date_naissance: chrono::NaiveDate::from_ymd_opt(2000, 1, 15).unwrap(),
                    email: None,
                    telephone: None,
                    responsable_id: None,
                    version: creee.version,
                },
                "bob",
            )
            .await
            .unwrap();

        assert_eq!(modifiee.version, creee.version + 1);

        let audit: ModifieParRow =
            r.db.fetch_one(
                "SELECT modifie_par FROM personnes_physiques WHERE id = ?",
                crate::params![creee.id],
            )
            .await
            .unwrap();
        assert_eq!(audit.modifie_par, "bob");
    }

    #[tokio::test]
    async fn test_update_version_obsolete_conflit() {
        let db = setup_db().await;
        let r = repo(db);
        let creee = r
            .create(
                CreatePersonne {
                    nom: "Dupont".into(),
                    prenom: "Jean".into(),
                    date_naissance: chrono::NaiveDate::from_ymd_opt(2000, 1, 15).unwrap(),
                    email: None,
                    telephone: None,
                    responsable_id: None,
                },
                "alice",
            )
            .await
            .unwrap();

        r.update(
            creee.id,
            UpdatePersonne {
                nom: "Dupont".into(),
                prenom: "Jean-Marc".into(),
                date_naissance: chrono::NaiveDate::from_ymd_opt(2000, 1, 15).unwrap(),
                email: None,
                telephone: None,
                responsable_id: None,
                version: creee.version,
            },
            "bob",
        )
        .await
        .unwrap();

        let err = r
            .update(
                creee.id,
                UpdatePersonne {
                    nom: "Dupont".into(),
                    prenom: "Quelqu'un d'autre".into(),
                    date_naissance: chrono::NaiveDate::from_ymd_opt(2000, 1, 15).unwrap(),
                    email: None,
                    telephone: None,
                    responsable_id: None,
                    version: creee.version,
                },
                "carol",
            )
            .await
            .unwrap_err();

        assert!(matches!(err, AppError::Conflict(_)));
    }

    #[tokio::test]
    async fn test_update_personne_inexistante_not_found() {
        let db = setup_db().await;
        let r = repo(db);

        let err = r
            .update(
                999,
                UpdatePersonne {
                    nom: "X".into(),
                    prenom: "Y".into(),
                    date_naissance: chrono::NaiveDate::from_ymd_opt(2000, 1, 15).unwrap(),
                    email: None,
                    telephone: None,
                    responsable_id: None,
                    version: 1,
                },
                "alice",
            )
            .await
            .unwrap_err();

        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_version_transmise_et_audit_non_expose() {
        let db = setup_db().await;
        let r = repo(db);
        let creee = r
            .create(
                CreatePersonne {
                    nom: "Dupont".into(),
                    prenom: "Jean".into(),
                    date_naissance: chrono::NaiveDate::from_ymd_opt(2000, 1, 15).unwrap(),
                    email: None,
                    telephone: None,
                    responsable_id: None,
                },
                "alice",
            )
            .await
            .unwrap();

        let json = serde_json::to_string(&creee).unwrap();
        assert!(json.contains("version"));
        assert!(!json.contains("modifie_par"));
        assert!(!json.contains("modifie_le"));
        assert!(json.contains("Dupont"));
    }
}
