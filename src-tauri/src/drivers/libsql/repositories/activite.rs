use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::activite::{
    Activite, ActivitePersonne, CreateActivite, CreateLiaisonActivitePersonne, CreateTarifActivite,
    LiaisonActivitePersonne, PersonneActivite, TarifActivite, UpdateActivite,
};
use crate::error::AppError;
use crate::infrastructure::db::{Db, DbExt, DbTransaction, DbTransactionExt};
use crate::repositories::rows::activite::{
    ActiviteAnneeRow, ActivitePersonneRow, AnneeRow, CompteurRow,
};
use crate::repositories::ActiviteRepository;

pub struct LibsqlActiviteRepository {
    db: Arc<dyn Db>,
}

impl LibsqlActiviteRepository {
    pub fn new(db: Arc<dyn Db>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ActiviteRepository for LibsqlActiviteRepository {
    async fn create(&self, input: CreateActivite, utilisateur: &str) -> Result<Activite, AppError> {
        let maintenant = crate::infrastructure::audit::maintenant_utc();
        self.db
            .fetch_one(
                "INSERT INTO activites (nom, description, capacite_max, modifie_par, modifie_le)
                 VALUES (?, ?, ?, ?, ?)
                 RETURNING id, nom, description, capacite_max, version",
                crate::params![
                    input.nom,
                    input.description,
                    input.capacite_max,
                    utilisateur,
                    maintenant
                ],
            )
            .await
    }

    async fn creer_avec_tarif(
        &self,
        input: CreateActivite,
        utilisateur: &str,
    ) -> Result<Activite, AppError> {
        let annee_scolaire = input.annee_scolaire.clone();
        let tarif = input.tarif;
        let maintenant = crate::infrastructure::audit::maintenant_utc();

        let tx = self.db.begin().await?;

        let activite = {
            let valeur = tx
                .fetch_one::<Activite>(
                    "INSERT INTO activites (nom, description, capacite_max, modifie_par, modifie_le)
                     VALUES (?, ?, ?, ?, ?)
                     RETURNING id, nom, description, capacite_max, version",
                    crate::params![input.nom, input.description, input.capacite_max, utilisateur, maintenant.clone()],
                )
                .await?;
            valeur
        };

        if let Some(annee) = annee_scolaire {
            tx.execute(
                "INSERT INTO tarifs_activite (activite_id, annee_scolaire, tarif, modifie_par, modifie_le)
                 VALUES (?, ?, ?, ?, ?)
                 ON CONFLICT(activite_id, annee_scolaire) DO UPDATE SET
                     tarif = excluded.tarif,
                     modifie_par = excluded.modifie_par,
                     modifie_le = excluded.modifie_le",
                crate::params![activite.id, annee, tarif.unwrap_or(0.0), utilisateur, maintenant],
            )
            .await?;
        }

        tx.commit().await?;
        Ok(activite)
    }

    async fn update(
        &self,
        id: i64,
        input: UpdateActivite,
        utilisateur: &str,
    ) -> Result<Activite, AppError> {
        let maintenant = crate::infrastructure::audit::maintenant_utc();
        let affected = self
            .db
            .execute(
                "UPDATE activites
                 SET nom = ?, description = ?, capacite_max = ?, modifie_par = ?, modifie_le = ?, version = version + 1
                 WHERE id = ? AND version = ?",
                crate::params![
                    input.nom,
                    input.description,
                    input.capacite_max,
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
            return Err(AppError::NotFound("Activité introuvable".into()));
        }
        self.find_by_id(id)
            .await?
            .ok_or(AppError::NotFound("Activité introuvable".into()))
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Activite>, AppError> {
        self.db
            .fetch_optional(
                "SELECT id, nom, description, capacite_max, version FROM activites WHERE id = ?",
                crate::params![id],
            )
            .await
    }

    async fn find_by_id_tx(
        &self,
        tx: &mut dyn DbTransaction,
        id: i64,
    ) -> Result<Option<Activite>, AppError> {
        tx.fetch_optional(
            "SELECT id, nom, description, capacite_max, version FROM activites WHERE id = ?",
            crate::params![id],
        )
        .await
    }

    async fn upsert_tarif(
        &self,
        input: CreateTarifActivite,
        utilisateur: &str,
    ) -> Result<TarifActivite, AppError> {
        let maintenant = crate::infrastructure::audit::maintenant_utc();
        self.db
            .fetch_one(
                "INSERT INTO tarifs_activite (activite_id, annee_scolaire, tarif, modifie_par, modifie_le)
                 VALUES (?, ?, ?, ?, ?)
                 ON CONFLICT(activite_id, annee_scolaire)
                 DO UPDATE SET
                     tarif = excluded.tarif,
                     modifie_par = excluded.modifie_par,
                     modifie_le = excluded.modifie_le
                 RETURNING activite_id, annee_scolaire, tarif",
                crate::params![
                    input.activite_id,
                    input.annee_scolaire,
                    input.tarif,
                    utilisateur,
                    maintenant
                ],
            )
            .await
    }

    async fn get_tarif(
        &self,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<Option<TarifActivite>, AppError> {
        self.db
            .fetch_optional(
                "SELECT activite_id, annee_scolaire, tarif FROM tarifs_activite WHERE activite_id = ? AND annee_scolaire = ?",
                crate::params![activite_id, annee_scolaire],
            )
            .await
    }

    async fn ajouter_personne(
        &self,
        input: CreateLiaisonActivitePersonne,
        utilisateur: &str,
    ) -> Result<LiaisonActivitePersonne, AppError> {
        let maintenant = crate::infrastructure::audit::maintenant_utc();
        self.db
            .fetch_one(
                "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role, modifie_par, modifie_le)
                 VALUES (?, ?, ?, ?, ?, ?)
                 RETURNING activite_id, personne_id, annee_scolaire, role",
                crate::params![
                    input.activite_id,
                    input.personne_id,
                    input.annee_scolaire,
                    input.role.to_string(),
                    utilisateur,
                    maintenant
                ],
            )
            .await
    }

    async fn ajouter_personne_tx(
        &self,
        tx: &mut dyn DbTransaction,
        input: CreateLiaisonActivitePersonne,
        utilisateur: &str,
    ) -> Result<LiaisonActivitePersonne, AppError> {
        let maintenant = crate::infrastructure::audit::maintenant_utc();
        tx.fetch_one(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role, modifie_par, modifie_le)
             VALUES (?, ?, ?, ?, ?, ?)
             RETURNING activite_id, personne_id, annee_scolaire, role",
            crate::params![
                input.activite_id,
                input.personne_id,
                input.annee_scolaire,
                input.role.to_string(),
                utilisateur,
                maintenant
            ],
        )
        .await
    }

    async fn retirer_personne(
        &self,
        activite_id: i64,
        personne_id: i64,
        annee_scolaire: &str,
    ) -> Result<(), AppError> {
        self.db
            .execute(
                "DELETE FROM activite_personnes WHERE activite_id = ? AND personne_id = ? AND annee_scolaire = ?",
                crate::params![activite_id, personne_id, annee_scolaire],
            )
            .await?;

        Ok(())
    }

    async fn compter_participants(
        &self,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<i64, AppError> {
        let compteur = self
            .db
            .fetch_optional::<CompteurRow>(
                "SELECT COUNT(*) AS count FROM activite_personnes
                 WHERE activite_id = ? AND annee_scolaire = ? AND role = 'participant'",
                crate::params![activite_id, annee_scolaire],
            )
            .await?
            .ok_or_else(|| AppError::Database("Aucune ligne de comptage".into()))?;
        Ok(compteur.count)
    }

    async fn compter_participants_tx(
        &self,
        tx: &mut dyn DbTransaction,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<i64, AppError> {
        let compteur = tx
            .fetch_optional::<CompteurRow>(
                "SELECT COUNT(*) AS count FROM activite_personnes
                 WHERE activite_id = ? AND annee_scolaire = ? AND role = 'participant'",
                crate::params![activite_id, annee_scolaire],
            )
            .await?
            .ok_or_else(|| AppError::Database("Aucune ligne de comptage".into()))?;
        Ok(compteur.count)
    }

    async fn trouver_liaison(
        &self,
        activite_id: i64,
        personne_id: i64,
        annee_scolaire: &str,
    ) -> Result<Option<LiaisonActivitePersonne>, AppError> {
        self.db
            .fetch_optional(
                "SELECT activite_id, personne_id, annee_scolaire, role FROM activite_personnes
                 WHERE activite_id = ? AND personne_id = ? AND annee_scolaire = ?",
                crate::params![activite_id, personne_id, annee_scolaire],
            )
            .await
    }

    async fn trouver_liaison_tx(
        &self,
        tx: &mut dyn DbTransaction,
        activite_id: i64,
        personne_id: i64,
        annee_scolaire: &str,
    ) -> Result<Option<LiaisonActivitePersonne>, AppError> {
        tx.fetch_optional(
            "SELECT activite_id, personne_id, annee_scolaire, role FROM activite_personnes
             WHERE activite_id = ? AND personne_id = ? AND annee_scolaire = ?",
            crate::params![activite_id, personne_id, annee_scolaire],
        )
        .await
    }

    async fn lister_encadrants(
        &self,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<Vec<PersonneActivite>, AppError> {
        self.db
            .fetch_all(
                "SELECT pp.id, pp.nom, pp.prenom
                 FROM activite_personnes ap
                 JOIN personnes_physiques pp ON pp.id = ap.personne_id
                 WHERE ap.activite_id = ? AND ap.annee_scolaire = ? AND ap.role = 'encadrant'
                 ORDER BY pp.nom, pp.prenom",
                crate::params![activite_id, annee_scolaire],
            )
            .await
    }

    async fn lister_participants(
        &self,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<Vec<PersonneActivite>, AppError> {
        self.db
            .fetch_all(
                "SELECT pp.id, pp.nom, pp.prenom
                 FROM activite_personnes ap
                 JOIN personnes_physiques pp ON pp.id = ap.personne_id
                 WHERE ap.activite_id = ? AND ap.annee_scolaire = ? AND ap.role = 'participant'
                 ORDER BY pp.nom, pp.prenom",
                crate::params![activite_id, annee_scolaire],
            )
            .await
    }

    async fn lister_activites_personne(
        &self,
        personne_id: i64,
    ) -> Result<Vec<ActivitePersonne>, AppError> {
        let lignes = self
            .db
            .fetch_all::<ActivitePersonneRow>(
                "SELECT a.id, a.nom, a.description, a.capacite_max, a.version, ap.role
                 FROM activite_personnes ap
                 JOIN activites a ON a.id = ap.activite_id
                 WHERE ap.personne_id = ?
                 ORDER BY a.nom",
                crate::params![personne_id],
            )
            .await?;

        Ok(lignes
            .into_iter()
            .map(|r| ActivitePersonne {
                activite: Activite {
                    id: r.id,
                    nom: r.nom,
                    description: r.description,
                    capacite_max: r.capacite_max,
                    version: r.version,
                },
                role: r.role,
            })
            .collect())
    }

    async fn lister_annees_disponibles(&self) -> Result<Vec<String>, AppError> {
        let lignes = self
            .db
            .fetch_all::<AnneeRow>(
                "SELECT DISTINCT annee_scolaire FROM tarifs_activite ORDER BY annee_scolaire DESC",
                crate::params![],
            )
            .await?;

        Ok(lignes.into_iter().map(|r| r.annee_scolaire).collect())
    }

    async fn lister_activites_par_annee(
        &self,
        annee_scolaire: &str,
    ) -> Result<Vec<(Activite, Option<f64>, i64)>, AppError> {
        let lignes = self
            .db
            .fetch_all::<ActiviteAnneeRow>(
                "SELECT a.id, a.nom, a.description, a.capacite_max, a.version, ta.tarif,
                        (SELECT COUNT(*) FROM activite_personnes ap2
                         WHERE ap2.activite_id = a.id AND ap2.annee_scolaire = ? AND ap2.role = 'participant') AS nb_participants
                 FROM activites a
                 JOIN tarifs_activite ta ON ta.activite_id = a.id AND ta.annee_scolaire = ?
                 ORDER BY a.nom",
                crate::params![annee_scolaire, annee_scolaire],
            )
            .await?;

        Ok(lignes
            .into_iter()
            .map(|r| {
                (
                    Activite {
                        id: r.id,
                        nom: r.nom,
                        description: r.description,
                        capacite_max: r.capacite_max,
                        version: r.version,
                    },
                    r.tarif,
                    r.nb_participants,
                )
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::activite::CreateActivite;
    use crate::domain::activite::Role;
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

    fn repo(db: Arc<dyn Db>) -> LibsqlActiviteRepository {
        LibsqlActiviteRepository::new(db)
    }

    fn create_activite_input(nom: &str) -> CreateActivite {
        CreateActivite {
            nom: nom.to_string(),
            description: None,
            capacite_max: None,
            annee_scolaire: None,
            tarif: None,
        }
    }

    #[allow(dead_code)]
    async fn seed_activite(db: &Arc<dyn Db>, nom: &str) -> Activite {
        LibsqlActiviteRepository::new(db.clone())
            .create(create_activite_input(nom), "test")
            .await
            .expect("failed to seed activite")
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

    #[tokio::test]
    async fn test_create_activite() {
        let db = setup_db().await;
        let r = repo(db);
        let a = r
            .create(create_activite_input("Poterie"), "alice")
            .await
            .unwrap();
        assert_eq!(a.nom, "Poterie");
        assert_eq!(a.id, 1);
        assert_eq!(a.version, 1);
    }

    #[tokio::test]
    async fn test_creer_avec_tarif_sans_tarif() {
        let db = setup_db().await;
        let r = repo(db);
        let input = CreateActivite {
            nom: "Théâtre".into(),
            description: None,
            capacite_max: None,
            annee_scolaire: None,
            tarif: None,
        };
        let a = r.creer_avec_tarif(input, "alice").await.unwrap();
        assert_eq!(a.nom, "Théâtre");
    }

    #[tokio::test]
    async fn test_creer_avec_tarif_avec_tarif() {
        let db = setup_db().await;
        let r = repo(db);
        let input = CreateActivite {
            nom: "Poterie".into(),
            description: None,
            capacite_max: None,
            annee_scolaire: Some("2025-2026".into()),
            tarif: Some(200.0),
        };
        let a = r.creer_avec_tarif(input, "alice").await.unwrap();
        assert_eq!(a.nom, "Poterie");

        let tarif = r.get_tarif(a.id, "2025-2026").await.unwrap();
        assert!(tarif.is_some());
        assert_eq!(tarif.unwrap().tarif, 200.0);
    }

    #[tokio::test]
    async fn test_creer_avec_tarif_sans_annee_n_insere_pas_tarif() {
        let db = setup_db().await;
        let r = repo(db);
        let input = CreateActivite {
            nom: "Danse".into(),
            description: None,
            capacite_max: None,
            annee_scolaire: None,
            tarif: Some(150.0),
        };
        let a = r.creer_avec_tarif(input, "alice").await.unwrap();
        assert_eq!(a.nom, "Danse");

        let tarif = r.get_tarif(a.id, "2025-2026").await.unwrap();
        assert!(tarif.is_none());
    }

    #[tokio::test]
    async fn test_liste_activites_par_annee() {
        let db = setup_db().await;
        let r = repo(db);
        let a = r
            .create(create_activite_input("Poterie"), "alice")
            .await
            .unwrap();

        r.upsert_tarif(
            CreateTarifActivite {
                activite_id: a.id,
                annee_scolaire: "2025-2026".into(),
                tarif: 200.0,
            },
            "alice",
        )
        .await
        .unwrap();

        let list = r.lister_activites_par_annee("2025-2026").await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].0.nom, "Poterie");
    }

    #[tokio::test]
    async fn test_tarif_upsert() {
        let db = setup_db().await;
        let r = repo(db);
        let a = r
            .create(create_activite_input("Poterie"), "alice")
            .await
            .unwrap();

        let t = r
            .upsert_tarif(
                CreateTarifActivite {
                    activite_id: a.id,
                    annee_scolaire: "2025-2026".into(),
                    tarif: 200.0,
                },
                "alice",
            )
            .await
            .unwrap();
        assert_eq!(t.tarif, 200.0);

        let t2 = r
            .upsert_tarif(
                CreateTarifActivite {
                    activite_id: a.id,
                    annee_scolaire: "2025-2026".into(),
                    tarif: 220.0,
                },
                "bob",
            )
            .await
            .unwrap();
        assert_eq!(t2.tarif, 220.0);
    }

    #[tokio::test]
    async fn test_ajouter_personne() {
        let db = setup_db().await;
        let r = repo(db.clone());
        let a = r
            .create(create_activite_input("Poterie"), "alice")
            .await
            .unwrap();
        let pid = seed_personne(r.db.as_ref()).await;

        let liaison = r
            .ajouter_personne(
                CreateLiaisonActivitePersonne {
                    activite_id: a.id,
                    personne_id: pid,
                    annee_scolaire: "2025-2026".into(),
                    role: Role::Participant,
                },
                "alice",
            )
            .await
            .unwrap();
        assert_eq!(liaison.role, Role::Participant);

        let participants = r.lister_participants(a.id, "2025-2026").await.unwrap();
        assert_eq!(participants.len(), 1);
    }

    #[tokio::test]
    async fn test_lister_activites_personne() {
        let db = setup_db().await;
        let r = repo(db.clone());
        let a = r
            .create(create_activite_input("Poterie"), "alice")
            .await
            .unwrap();
        let pid = seed_personne(r.db.as_ref()).await;

        r.ajouter_personne(
            CreateLiaisonActivitePersonne {
                activite_id: a.id,
                personne_id: pid,
                annee_scolaire: "2025-2026".into(),
                role: Role::Participant,
            },
            "alice",
        )
        .await
        .unwrap();

        let list = r.lister_activites_personne(pid).await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].activite.id, a.id);
        assert_eq!(list[0].role, Role::Participant);
    }

    #[tokio::test]
    async fn test_retirer_personne() {
        let db = setup_db().await;
        let r = repo(db.clone());
        let a = r
            .create(create_activite_input("Poterie"), "alice")
            .await
            .unwrap();
        let pid = seed_personne(r.db.as_ref()).await;

        r.ajouter_personne(
            CreateLiaisonActivitePersonne {
                activite_id: a.id,
                personne_id: pid,
                annee_scolaire: "2025-2026".into(),
                role: Role::Participant,
            },
            "alice",
        )
        .await
        .unwrap();

        r.retirer_personne(a.id, pid, "2025-2026").await.unwrap();

        let participants = r.lister_participants(a.id, "2025-2026").await.unwrap();
        assert_eq!(participants.len(), 0);
    }

    #[tokio::test]
    async fn test_update_activite_version_obsolete_conflit() {
        let db = setup_db().await;
        let r = repo(db);
        let a = r
            .create(create_activite_input("Poterie"), "alice")
            .await
            .unwrap();

        r.update(
            a.id,
            UpdateActivite {
                nom: "Poterie avancée".into(),
                description: None,
                capacite_max: None,
                version: a.version,
            },
            "bob",
        )
        .await
        .unwrap();

        let err = r
            .update(
                a.id,
                UpdateActivite {
                    nom: "Encore un nom".into(),
                    description: None,
                    capacite_max: None,
                    version: a.version,
                },
                "carol",
            )
            .await
            .unwrap_err();

        assert!(matches!(err, AppError::Conflict(_)));
    }

    #[tokio::test]
    async fn test_update_activite_inexistante_not_found() {
        let db = setup_db().await;
        let r = repo(db);

        let err = r
            .update(
                999,
                UpdateActivite {
                    nom: "X".into(),
                    description: None,
                    capacite_max: None,
                    version: 1,
                },
                "alice",
            )
            .await
            .unwrap_err();

        assert!(matches!(err, AppError::NotFound(_)));
    }
}
