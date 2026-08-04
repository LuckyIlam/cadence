use async_trait::async_trait;
use libsql::Connection;

use crate::domain::activite::{
    Activite, ActivitePersonne, CreateActivite, CreateLiaisonActivitePersonne, CreateTarifActivite,
    LiaisonActivitePersonne, PersonneActivite, Role, TarifActivite, UpdateActivite,
};
use crate::error::AppError;

#[async_trait]
pub trait ActiviteRepository: Send + Sync {
    #[allow(dead_code)]
    async fn create(&self, input: CreateActivite) -> Result<Activite, AppError>;
    async fn creer_avec_tarif(&self, input: CreateActivite) -> Result<Activite, AppError>;
    async fn update(&self, id: i64, input: UpdateActivite) -> Result<Activite, AppError>;
    async fn find_by_id(&self, id: i64) -> Result<Option<Activite>, AppError>;
    async fn upsert_tarif(&self, input: CreateTarifActivite) -> Result<TarifActivite, AppError>;
    async fn get_tarif(
        &self,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<Option<TarifActivite>, AppError>;
    async fn ajouter_personne(
        &self,
        input: CreateLiaisonActivitePersonne,
    ) -> Result<LiaisonActivitePersonne, AppError>;
    async fn retirer_personne(
        &self,
        activite_id: i64,
        personne_id: i64,
        annee_scolaire: &str,
    ) -> Result<(), AppError>;
    async fn compter_participants(
        &self,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<i64, AppError>;
    async fn trouver_liaison(
        &self,
        activite_id: i64,
        personne_id: i64,
        annee_scolaire: &str,
    ) -> Result<Option<LiaisonActivitePersonne>, AppError>;
    async fn lister_encadrants(
        &self,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<Vec<PersonneActivite>, AppError>;
    async fn lister_participants(
        &self,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<Vec<PersonneActivite>, AppError>;
    async fn lister_activites_personne(
        &self,
        personne_id: i64,
    ) -> Result<Vec<ActivitePersonne>, AppError>;
    async fn lister_annees_disponibles(&self) -> Result<Vec<String>, AppError>;
    async fn lister_activites_par_annee(
        &self,
        annee_scolaire: &str,
    ) -> Result<Vec<(Activite, Option<f64>, i64)>, AppError>;
}

pub struct LibsqlActiviteRepository {
    pub(crate) conn: Connection,
}

impl LibsqlActiviteRepository {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl ActiviteRepository for LibsqlActiviteRepository {
    async fn create(&self, input: CreateActivite) -> Result<Activite, AppError> {
        let mut rows = self
            .conn
            .query(
                "INSERT INTO activites (nom, description, capacite_max)
                 VALUES (?, ?, ?)
                 RETURNING *",
                libsql::params![input.nom, input.description, input.capacite_max],
            )
            .await?;

        let row = rows
            .next()
            .await?
            .ok_or(AppError::NotFound("Activité introuvable".into()))?;
        Ok(libsql::de::from_row::<Activite>(&row)?)
    }

    async fn creer_avec_tarif(&self, input: CreateActivite) -> Result<Activite, AppError> {
        let annee_scolaire = input.annee_scolaire.clone();
        let tarif = input.tarif;

        let tx = self.conn.transaction().await?;

        let activite = {
            let mut rows = tx
                .query(
                    "INSERT INTO activites (nom, description, capacite_max)
                     VALUES (?, ?, ?)
                     RETURNING *",
                    libsql::params![input.nom, input.description, input.capacite_max],
                )
                .await?;

            let row = rows
                .next()
                .await?
                .ok_or(AppError::NotFound("Activité introuvable".into()))?;
            libsql::de::from_row::<Activite>(&row)?
        };

        if let Some(annee) = annee_scolaire {
            tx.execute(
                "INSERT INTO tarifs_activite (activite_id, annee_scolaire, tarif)
                 VALUES (?, ?, ?)
                 ON CONFLICT(activite_id, annee_scolaire) DO UPDATE SET tarif = excluded.tarif",
                libsql::params![activite.id, annee, tarif.unwrap_or(0.0)],
            )
            .await?;
        }

        tx.commit().await?;
        Ok(activite)
    }

    async fn update(&self, id: i64, input: UpdateActivite) -> Result<Activite, AppError> {
        let mut rows = self
            .conn
            .query(
                "UPDATE activites
                 SET nom = ?, description = ?, capacite_max = ?
                 WHERE id = ?
                 RETURNING *",
                libsql::params![input.nom, input.description, input.capacite_max, id],
            )
            .await?;

        let row = rows
            .next()
            .await?
            .ok_or(AppError::NotFound("Activité introuvable".into()))?;
        Ok(libsql::de::from_row::<Activite>(&row)?)
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Activite>, AppError> {
        let mut rows = self
            .conn
            .query("SELECT * FROM activites WHERE id = ?", libsql::params![id])
            .await?;

        match rows.next().await? {
            Some(row) => Ok(Some(libsql::de::from_row::<Activite>(&row)?)),
            None => Ok(None),
        }
    }

    async fn upsert_tarif(&self, input: CreateTarifActivite) -> Result<TarifActivite, AppError> {
        let mut rows = self
            .conn
            .query(
                "INSERT INTO tarifs_activite (activite_id, annee_scolaire, tarif)
                 VALUES (?, ?, ?)
                 ON CONFLICT(activite_id, annee_scolaire)
                 DO UPDATE SET tarif = excluded.tarif
                 RETURNING *",
                libsql::params![input.activite_id, input.annee_scolaire, input.tarif],
            )
            .await?;

        let row = rows
            .next()
            .await?
            .ok_or(AppError::NotFound("Tarif introuvable".into()))?;
        Ok(libsql::de::from_row::<TarifActivite>(&row)?)
    }

    async fn get_tarif(
        &self,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<Option<TarifActivite>, AppError> {
        let mut rows = self
            .conn
            .query(
                "SELECT * FROM tarifs_activite WHERE activite_id = ? AND annee_scolaire = ?",
                libsql::params![activite_id, annee_scolaire],
            )
            .await?;

        match rows.next().await? {
            Some(row) => Ok(Some(libsql::de::from_row::<TarifActivite>(&row)?)),
            None => Ok(None),
        }
    }

    async fn ajouter_personne(
        &self,
        input: CreateLiaisonActivitePersonne,
    ) -> Result<LiaisonActivitePersonne, AppError> {
        let mut rows = self
            .conn
            .query(
                "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
                 VALUES (?, ?, ?, ?)
                 RETURNING *",
                libsql::params![
                    input.activite_id,
                    input.personne_id,
                    input.annee_scolaire,
                    input.role.to_string()
                ],
            )
            .await?;

        let row = rows
            .next()
            .await?
            .ok_or(AppError::NotFound("Inscription introuvable".into()))?;
        Ok(libsql::de::from_row::<LiaisonActivitePersonne>(&row)?)
    }

    async fn retirer_personne(
        &self,
        activite_id: i64,
        personne_id: i64,
        annee_scolaire: &str,
    ) -> Result<(), AppError> {
        self.conn
            .execute(
                "DELETE FROM activite_personnes WHERE activite_id = ? AND personne_id = ? AND annee_scolaire = ?",
                libsql::params![activite_id, personne_id, annee_scolaire],
            )
            .await?;

        Ok(())
    }

    async fn compter_participants(
        &self,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<i64, AppError> {
        #[derive(Debug, Clone, serde::Deserialize)]
        struct CompteurRow {
            count: i64,
        }

        let mut rows = self
            .conn
            .query(
                "SELECT COUNT(*) AS count FROM activite_personnes
                 WHERE activite_id = ? AND annee_scolaire = ? AND role = 'participant'",
                libsql::params![activite_id, annee_scolaire],
            )
            .await?;

        let row = rows
            .next()
            .await?
            .ok_or(AppError::Database("Aucune ligne de comptage".into()))?;
        Ok(libsql::de::from_row::<CompteurRow>(&row)?.count)
    }

    async fn trouver_liaison(
        &self,
        activite_id: i64,
        personne_id: i64,
        annee_scolaire: &str,
    ) -> Result<Option<LiaisonActivitePersonne>, AppError> {
        let mut rows = self
            .conn
            .query(
                "SELECT * FROM activite_personnes
                 WHERE activite_id = ? AND personne_id = ? AND annee_scolaire = ?",
                libsql::params![activite_id, personne_id, annee_scolaire],
            )
            .await?;

        match rows.next().await? {
            Some(row) => Ok(Some(libsql::de::from_row::<LiaisonActivitePersonne>(&row)?)),
            None => Ok(None),
        }
    }

    async fn lister_encadrants(
        &self,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<Vec<PersonneActivite>, AppError> {
        let mut rows = self
            .conn
            .query(
                "SELECT pp.id, pp.nom, pp.prenom
                 FROM activite_personnes ap
                 JOIN personnes_physiques pp ON pp.id = ap.personne_id
                 WHERE ap.activite_id = ? AND ap.annee_scolaire = ? AND ap.role = 'encadrant'
                 ORDER BY pp.nom, pp.prenom",
                libsql::params![activite_id, annee_scolaire],
            )
            .await?;

        let mut donnees = Vec::new();
        while let Some(row) = rows.next().await? {
            donnees.push(libsql::de::from_row::<PersonneActivite>(&row)?);
        }

        Ok(donnees)
    }

    async fn lister_participants(
        &self,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<Vec<PersonneActivite>, AppError> {
        let mut rows = self
            .conn
            .query(
                "SELECT pp.id, pp.nom, pp.prenom
                 FROM activite_personnes ap
                 JOIN personnes_physiques pp ON pp.id = ap.personne_id
                 WHERE ap.activite_id = ? AND ap.annee_scolaire = ? AND ap.role = 'participant'
                 ORDER BY pp.nom, pp.prenom",
                libsql::params![activite_id, annee_scolaire],
            )
            .await?;

        let mut donnees = Vec::new();
        while let Some(row) = rows.next().await? {
            donnees.push(libsql::de::from_row::<PersonneActivite>(&row)?);
        }

        Ok(donnees)
    }

    async fn lister_activites_personne(
        &self,
        personne_id: i64,
    ) -> Result<Vec<ActivitePersonne>, AppError> {
        #[derive(Debug, Clone, serde::Deserialize)]
        struct ActivitePersonneRow {
            id: i64,
            nom: String,
            description: Option<String>,
            capacite_max: Option<i64>,
            role: Role,
        }

        let mut rows = self
            .conn
            .query(
                "SELECT a.id, a.nom, a.description, a.capacite_max, ap.role
                 FROM activite_personnes ap
                 JOIN activites a ON a.id = ap.activite_id
                 WHERE ap.personne_id = ?
                 ORDER BY a.nom",
                libsql::params![personne_id],
            )
            .await?;

        let mut donnees = Vec::new();
        while let Some(row) = rows.next().await? {
            let r = libsql::de::from_row::<ActivitePersonneRow>(&row)?;
            donnees.push(ActivitePersonne {
                activite: Activite {
                    id: r.id,
                    nom: r.nom,
                    description: r.description,
                    capacite_max: r.capacite_max,
                },
                role: r.role,
            });
        }

        Ok(donnees)
    }

    async fn lister_annees_disponibles(&self) -> Result<Vec<String>, AppError> {
        #[derive(Debug, Clone, serde::Deserialize)]
        struct AnneeRow {
            annee_scolaire: String,
        }

        let mut rows = self
            .conn
            .query(
                "SELECT DISTINCT annee_scolaire FROM tarifs_activite ORDER BY annee_scolaire DESC",
                libsql::params![],
            )
            .await?;

        let mut donnees = Vec::new();
        while let Some(row) = rows.next().await? {
            donnees.push(libsql::de::from_row::<AnneeRow>(&row)?.annee_scolaire);
        }

        Ok(donnees)
    }

    async fn lister_activites_par_annee(
        &self,
        annee_scolaire: &str,
    ) -> Result<Vec<(Activite, Option<f64>, i64)>, AppError> {
        #[derive(Debug, Clone, serde::Deserialize)]
        struct ActiviteAnneeRow {
            id: i64,
            nom: String,
            description: Option<String>,
            capacite_max: Option<i64>,
            tarif: Option<f64>,
            nb_participants: i64,
        }

        let mut rows = self
            .conn
            .query(
                "SELECT a.id, a.nom, a.description, a.capacite_max, ta.tarif,
                        (SELECT COUNT(*) FROM activite_personnes ap2
                         WHERE ap2.activite_id = a.id AND ap2.annee_scolaire = ? AND ap2.role = 'participant') AS nb_participants
                 FROM activites a
                 JOIN tarifs_activite ta ON ta.activite_id = a.id AND ta.annee_scolaire = ?
                 ORDER BY a.nom",
                libsql::params![annee_scolaire, annee_scolaire],
            )
            .await?;

        let mut donnees = Vec::new();
        while let Some(row) = rows.next().await? {
            let r = libsql::de::from_row::<ActiviteAnneeRow>(&row)?;
            donnees.push((
                Activite {
                    id: r.id,
                    nom: r.nom,
                    description: r.description,
                    capacite_max: r.capacite_max,
                },
                r.tarif,
                r.nb_participants,
            ));
        }

        Ok(donnees)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::activite::CreateActivite;
    use crate::domain::activite::Role;

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

    fn repo(conn: Connection) -> LibsqlActiviteRepository {
        LibsqlActiviteRepository::new(conn)
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
    async fn seed_activite(conn: &Connection, nom: &str) -> Activite {
        LibsqlActiviteRepository::new(conn.clone())
            .create(create_activite_input(nom))
            .await
            .expect("failed to seed activite")
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

    #[tokio::test]
    async fn test_create_activite() {
        let conn = setup_db().await;
        let r = repo(conn);
        let a = r.create(create_activite_input("Poterie")).await.unwrap();
        assert_eq!(a.nom, "Poterie");
        assert_eq!(a.id, 1);
    }

    #[tokio::test]
    async fn test_creer_avec_tarif_sans_tarif() {
        let conn = setup_db().await;
        let r = repo(conn);
        let input = CreateActivite {
            nom: "Théâtre".into(),
            description: None,
            capacite_max: None,
            annee_scolaire: None,
            tarif: None,
        };
        let a = r.creer_avec_tarif(input).await.unwrap();
        assert_eq!(a.nom, "Théâtre");
    }

    #[tokio::test]
    async fn test_creer_avec_tarif_avec_tarif() {
        let conn = setup_db().await;
        let r = repo(conn);
        let input = CreateActivite {
            nom: "Poterie".into(),
            description: None,
            capacite_max: None,
            annee_scolaire: Some("2025-2026".into()),
            tarif: Some(200.0),
        };
        let a = r.creer_avec_tarif(input).await.unwrap();
        assert_eq!(a.nom, "Poterie");

        let tarif = r.get_tarif(a.id, "2025-2026").await.unwrap();
        assert!(tarif.is_some());
        assert_eq!(tarif.unwrap().tarif, 200.0);
    }

    #[tokio::test]
    async fn test_creer_avec_tarif_sans_annee_n_insere_pas_tarif() {
        let conn = setup_db().await;
        let r = repo(conn);
        let input = CreateActivite {
            nom: "Danse".into(),
            description: None,
            capacite_max: None,
            annee_scolaire: None,
            tarif: Some(150.0),
        };
        let a = r.creer_avec_tarif(input).await.unwrap();
        assert_eq!(a.nom, "Danse");

        let tarif = r.get_tarif(a.id, "2025-2026").await.unwrap();
        assert!(tarif.is_none());
    }

    #[tokio::test]
    async fn test_liste_activites_par_annee() {
        let conn = setup_db().await;
        let r = repo(conn);
        let a = r.create(create_activite_input("Poterie")).await.unwrap();

        r.upsert_tarif(CreateTarifActivite {
            activite_id: a.id,
            annee_scolaire: "2025-2026".into(),
            tarif: 200.0,
        })
        .await
        .unwrap();

        let list = r.lister_activites_par_annee("2025-2026").await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].0.nom, "Poterie");
    }

    #[tokio::test]
    async fn test_tarif_upsert() {
        let conn = setup_db().await;
        let r = repo(conn);
        let a = r.create(create_activite_input("Poterie")).await.unwrap();

        let t = r
            .upsert_tarif(CreateTarifActivite {
                activite_id: a.id,
                annee_scolaire: "2025-2026".into(),
                tarif: 200.0,
            })
            .await
            .unwrap();
        assert_eq!(t.tarif, 200.0);

        let t2 = r
            .upsert_tarif(CreateTarifActivite {
                activite_id: a.id,
                annee_scolaire: "2025-2026".into(),
                tarif: 220.0,
            })
            .await
            .unwrap();
        assert_eq!(t2.tarif, 220.0);
    }

    #[tokio::test]
    async fn test_ajouter_personne() {
        let conn = setup_db().await;
        let r = repo(conn.clone());
        let a = r.create(create_activite_input("Poterie")).await.unwrap();
        let pid = seed_personne(&r.conn).await;

        let liaison = r
            .ajouter_personne(CreateLiaisonActivitePersonne {
                activite_id: a.id,
                personne_id: pid,
                annee_scolaire: "2025-2026".into(),
                role: Role::Participant,
            })
            .await
            .unwrap();
        assert_eq!(liaison.role, Role::Participant);

        let participants = r.lister_participants(a.id, "2025-2026").await.unwrap();
        assert_eq!(participants.len(), 1);
    }

    #[tokio::test]
    async fn test_lister_activites_personne() {
        let conn = setup_db().await;
        let r = repo(conn.clone());
        let a = r.create(create_activite_input("Poterie")).await.unwrap();
        let pid = seed_personne(&r.conn).await;

        r.ajouter_personne(CreateLiaisonActivitePersonne {
            activite_id: a.id,
            personne_id: pid,
            annee_scolaire: "2025-2026".into(),
            role: Role::Participant,
        })
        .await
        .unwrap();

        let list = r.lister_activites_personne(pid).await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].activite.id, a.id);
        assert_eq!(list[0].role, Role::Participant);
    }

    #[tokio::test]
    async fn test_retirer_personne() {
        let conn = setup_db().await;
        let r = repo(conn.clone());
        let a = r.create(create_activite_input("Poterie")).await.unwrap();
        let pid = seed_personne(&r.conn).await;

        r.ajouter_personne(CreateLiaisonActivitePersonne {
            activite_id: a.id,
            personne_id: pid,
            annee_scolaire: "2025-2026".into(),
            role: Role::Participant,
        })
        .await
        .unwrap();

        r.retirer_personne(a.id, pid, "2025-2026").await.unwrap();

        let participants = r.lister_participants(a.id, "2025-2026").await.unwrap();
        assert_eq!(participants.len(), 0);
    }
}
