use async_trait::async_trait;
use sqlx::SqlitePool;

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

pub struct SqliteActiviteRepository {
    pub(crate) pool: SqlitePool,
}

impl SqliteActiviteRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ActiviteRepository for SqliteActiviteRepository {
    async fn create(&self, input: CreateActivite) -> Result<Activite, AppError> {
        let row = sqlx::query_as::<_, Activite>(
            "INSERT INTO activites (nom, description, capacite_max)
             VALUES (?, ?, ?)
             RETURNING *",
        )
        .bind(&input.nom)
        .bind(&input.description)
        .bind(input.capacite_max)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    async fn creer_avec_tarif(&self, input: CreateActivite) -> Result<Activite, AppError> {
        let annee_scolaire = input.annee_scolaire.clone();
        let tarif = input.tarif;

        let mut tx = self.pool.begin().await?;

        let activite = sqlx::query_as::<_, Activite>(
            "INSERT INTO activites (nom, description, capacite_max)
             VALUES (?, ?, ?)
             RETURNING *",
        )
        .bind(&input.nom)
        .bind(&input.description)
        .bind(input.capacite_max)
        .fetch_one(&mut *tx)
        .await?;

        if let Some(ref annee) = annee_scolaire {
            sqlx::query(
                "INSERT INTO tarifs_activite (activite_id, annee_scolaire, tarif)
                 VALUES (?, ?, ?)
                 ON CONFLICT(activite_id, annee_scolaire) DO UPDATE SET tarif = excluded.tarif",
            )
            .bind(activite.id)
            .bind(annee)
            .bind(tarif.unwrap_or(0.0))
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(activite)
    }

    async fn update(&self, id: i64, input: UpdateActivite) -> Result<Activite, AppError> {
        let row = sqlx::query_as::<_, Activite>(
            "UPDATE activites
             SET nom = ?, description = ?, capacite_max = ?
             WHERE id = ?
             RETURNING *",
        )
        .bind(&input.nom)
        .bind(&input.description)
        .bind(input.capacite_max)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Activite>, AppError> {
        let row = sqlx::query_as::<_, Activite>("SELECT * FROM activites WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row)
    }

    async fn upsert_tarif(&self, input: CreateTarifActivite) -> Result<TarifActivite, AppError> {
        let row = sqlx::query_as::<_, TarifActivite>(
            "INSERT INTO tarifs_activite (activite_id, annee_scolaire, tarif)
             VALUES (?, ?, ?)
             ON CONFLICT(activite_id, annee_scolaire)
             DO UPDATE SET tarif = excluded.tarif
             RETURNING *",
        )
        .bind(input.activite_id)
        .bind(&input.annee_scolaire)
        .bind(input.tarif)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    async fn get_tarif(
        &self,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<Option<TarifActivite>, AppError> {
        let row = sqlx::query_as::<_, TarifActivite>(
            "SELECT * FROM tarifs_activite WHERE activite_id = ? AND annee_scolaire = ?",
        )
        .bind(activite_id)
        .bind(annee_scolaire)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn ajouter_personne(
        &self,
        input: CreateLiaisonActivitePersonne,
    ) -> Result<LiaisonActivitePersonne, AppError> {
        let row = sqlx::query_as::<_, LiaisonActivitePersonne>(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)
             RETURNING *",
        )
        .bind(input.activite_id)
        .bind(input.personne_id)
        .bind(&input.annee_scolaire)
        .bind(&input.role)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    async fn retirer_personne(
        &self,
        activite_id: i64,
        personne_id: i64,
        annee_scolaire: &str,
    ) -> Result<(), AppError> {
        sqlx::query(
            "DELETE FROM activite_personnes WHERE activite_id = ? AND personne_id = ? AND annee_scolaire = ?",
        )
        .bind(activite_id)
        .bind(personne_id)
        .bind(annee_scolaire)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn compter_participants(
        &self,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<i64, AppError> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM activite_personnes
             WHERE activite_id = ? AND annee_scolaire = ? AND role = 'participant'",
        )
        .bind(activite_id)
        .bind(annee_scolaire)
        .fetch_one(&self.pool)
        .await?;

        Ok(count.0)
    }

    async fn trouver_liaison(
        &self,
        activite_id: i64,
        personne_id: i64,
        annee_scolaire: &str,
    ) -> Result<Option<LiaisonActivitePersonne>, AppError> {
        let row = sqlx::query_as::<_, LiaisonActivitePersonne>(
            "SELECT * FROM activite_personnes
             WHERE activite_id = ? AND personne_id = ? AND annee_scolaire = ?",
        )
        .bind(activite_id)
        .bind(personne_id)
        .bind(annee_scolaire)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn lister_encadrants(
        &self,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<Vec<PersonneActivite>, AppError> {
        let rows = sqlx::query_as::<_, PersonneActivite>(
            "SELECT pp.id, pp.nom, pp.prenom
             FROM activite_personnes ap
             JOIN personnes_physiques pp ON pp.id = ap.personne_id
             WHERE ap.activite_id = ? AND ap.annee_scolaire = ? AND ap.role = 'encadrant'
             ORDER BY pp.nom, pp.prenom",
        )
        .bind(activite_id)
        .bind(annee_scolaire)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn lister_participants(
        &self,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<Vec<PersonneActivite>, AppError> {
        let rows = sqlx::query_as::<_, PersonneActivite>(
            "SELECT pp.id, pp.nom, pp.prenom
             FROM activite_personnes ap
             JOIN personnes_physiques pp ON pp.id = ap.personne_id
             WHERE ap.activite_id = ? AND ap.annee_scolaire = ? AND ap.role = 'participant'
             ORDER BY pp.nom, pp.prenom",
        )
        .bind(activite_id)
        .bind(annee_scolaire)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn lister_activites_personne(
        &self,
        personne_id: i64,
    ) -> Result<Vec<ActivitePersonne>, AppError> {
        #[derive(Debug, Clone, sqlx::FromRow)]
        struct ActivitePersonneRow {
            id: i64,
            nom: String,
            description: Option<String>,
            capacite_max: Option<i64>,
            role: Role,
        }

        let rows = sqlx::query_as::<_, ActivitePersonneRow>(
            "SELECT a.id, a.nom, a.description, a.capacite_max, ap.role
             FROM activite_personnes ap
             JOIN activites a ON a.id = ap.activite_id
             WHERE ap.personne_id = ?
             ORDER BY a.nom",
        )
        .bind(personne_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| ActivitePersonne {
                activite: Activite {
                    id: r.id,
                    nom: r.nom,
                    description: r.description,
                    capacite_max: r.capacite_max,
                },
                role: r.role,
            })
            .collect())
    }

    async fn lister_annees_disponibles(&self) -> Result<Vec<String>, AppError> {
        let rows = sqlx::query_scalar::<_, String>(
            "SELECT DISTINCT annee_scolaire FROM tarifs_activite ORDER BY annee_scolaire DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn lister_activites_par_annee(
        &self,
        annee_scolaire: &str,
    ) -> Result<Vec<(Activite, Option<f64>, i64)>, AppError> {
        #[derive(Debug, Clone, sqlx::FromRow)]
        struct ActiviteAnneeRow {
            id: i64,
            nom: String,
            description: Option<String>,
            capacite_max: Option<i64>,
            tarif: Option<f64>,
            nb_participants: i64,
        }

        let rows = sqlx::query_as::<_, ActiviteAnneeRow>(
            "SELECT a.id, a.nom, a.description, a.capacite_max, ta.tarif,
                    (SELECT COUNT(*) FROM activite_personnes ap2
                     WHERE ap2.activite_id = a.id AND ap2.annee_scolaire = ? AND ap2.role = 'participant') AS nb_participants
             FROM activites a
             JOIN tarifs_activite ta ON ta.activite_id = a.id AND ta.annee_scolaire = ?
             ORDER BY a.nom",
        )
        .bind(annee_scolaire)
        .bind(annee_scolaire)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| {
                (
                    Activite {
                        id: r.id,
                        nom: r.nom,
                        description: r.description,
                        capacite_max: r.capacite_max,
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
    use sqlx::SqlitePool;

    async fn setup_db() -> SqlitePool {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
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

    fn repo(pool: SqlitePool) -> SqliteActiviteRepository {
        SqliteActiviteRepository::new(pool)
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
    async fn seed_activite(pool: &SqlitePool, nom: &str) -> Activite {
        SqliteActiviteRepository::new(pool.clone())
            .create(create_activite_input(nom))
            .await
            .expect("failed to seed activite")
    }

    async fn seed_personne(pool: &SqlitePool) -> i64 {
        sqlx::query_scalar::<_, i64>(
            "INSERT INTO personnes_physiques (nom, prenom, date_naissance)
             VALUES (?, ?, ?) RETURNING id",
        )
        .bind("Test")
        .bind("User")
        .bind("2000-01-15")
        .fetch_one(pool)
        .await
        .expect("failed to seed personne")
    }

    #[tokio::test]
    async fn test_create_activite() {
        let pool = setup_db().await;
        let r = repo(pool);
        let a = r.create(create_activite_input("Poterie")).await.unwrap();
        assert_eq!(a.nom, "Poterie");
        assert_eq!(a.id, 1);
    }

    #[tokio::test]
    async fn test_creer_avec_tarif_sans_tarif() {
        let pool = setup_db().await;
        let r = repo(pool);
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
        let pool = setup_db().await;
        let r = repo(pool);
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
        let pool = setup_db().await;
        let r = repo(pool);
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
        let pool = setup_db().await;
        let r = repo(pool);
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
        let pool = setup_db().await;
        let r = repo(pool);
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
        let pool = setup_db().await;
        let r = repo(pool);
        let a = r.create(create_activite_input("Poterie")).await.unwrap();
        let pid = seed_personne(&r.pool).await;

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
        let pool = setup_db().await;
        let r = repo(pool);
        let a = r.create(create_activite_input("Poterie")).await.unwrap();
        let pid = seed_personne(&r.pool).await;

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
        let pool = setup_db().await;
        let r = repo(pool);
        let a = r.create(create_activite_input("Poterie")).await.unwrap();
        let pid = seed_personne(&r.pool).await;

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
