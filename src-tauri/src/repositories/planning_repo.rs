use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::domain::activite::Role;
use crate::domain::planning::{
    Collision, CreateCreneau, CreateSemaineBanalisee, CreneauActivite, CreneauHorsPlage,
    Inscription, PlanningCreneau, SemaineBanalisee,
};
use crate::error::AppError;

#[async_trait]
pub trait PlanningRepository: Send + Sync {
    async fn creer_creneau(&self, input: CreateCreneau) -> Result<CreneauActivite, AppError>;
    async fn supprimer_creneau(&self, id: i64) -> Result<(), AppError>;
    async fn modifier_creneau(
        &self,
        id: i64,
        input: CreateCreneau,
    ) -> Result<CreneauActivite, AppError>;
    async fn lister_creneaux(
        &self,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<Vec<CreneauActivite>, AppError>;
    async fn lister_tous_creneaux(&self) -> Result<Vec<CreneauActivite>, AppError>;
    async fn lister_creneaux_hors_plage(
        &self,
        heure_ouverture: &str,
        heure_fermeture: &str,
    ) -> Result<Vec<CreneauHorsPlage>, AppError>;
    async fn lister_inscriptions(&self) -> Result<Vec<Inscription>, AppError>;
    async fn supprimer_creneau_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        id: i64,
    ) -> Result<(), AppError>;
    async fn deplacer_creneau_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        id: i64,
        heure_debut: &str,
        heure_fin: &str,
    ) -> Result<CreneauActivite, AppError>;
    async fn ajouter_semaine_banalisee(
        &self,
        input: CreateSemaineBanalisee,
    ) -> Result<SemaineBanalisee, AppError>;
    async fn supprimer_semaine_banalisee(&self, id: i64) -> Result<(), AppError>;
    async fn lister_semaines_banalisees(
        &self,
        activite_id: i64,
    ) -> Result<Vec<SemaineBanalisee>, AppError>;
    async fn verifier_conflit_creneaux(
        &self,
        activite_id: i64,
        annee_scolaire: &str,
        jour_semaine: i64,
        heure_debut: &str,
        heure_fin: &str,
        exclure_id: Option<i64>,
    ) -> Result<Vec<CreneauActivite>, AppError>;
    async fn compter_inscrits_activite(
        &self,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<i64, AppError>;
    async fn verifier_collision(
        &self,
        personne_id: i64,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<Option<Collision>, AppError>;
    async fn planning_personne_semaine(
        &self,
        personne_id: i64,
        date_lundi: &str,
        annee_scolaire: &str,
    ) -> Result<Vec<PlanningCreneau>, AppError>;
}

pub struct SqlitePlanningRepository {
    pub(crate) pool: SqlitePool,
}

impl SqlitePlanningRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PlanningRepository for SqlitePlanningRepository {
    async fn creer_creneau(&self, input: CreateCreneau) -> Result<CreneauActivite, AppError> {
        let row = sqlx::query_as::<_, CreneauActivite>(
            "INSERT INTO creneaux_activite (activite_id, jour_semaine, heure_debut, heure_fin, annee_scolaire)
             VALUES (?, ?, ?, ?, ?)
             RETURNING *",
        )
        .bind(input.activite_id)
        .bind(input.jour_semaine)
        .bind(&input.heure_debut)
        .bind(&input.heure_fin)
        .bind(&input.annee_scolaire)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    async fn supprimer_creneau(&self, id: i64) -> Result<(), AppError> {
        sqlx::query("DELETE FROM creneaux_activite WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn modifier_creneau(
        &self,
        id: i64,
        input: CreateCreneau,
    ) -> Result<CreneauActivite, AppError> {
        let row = sqlx::query_as::<_, CreneauActivite>(
            "UPDATE creneaux_activite
             SET jour_semaine = ?, heure_debut = ?, heure_fin = ?
             WHERE id = ?
             RETURNING *",
        )
        .bind(input.jour_semaine)
        .bind(&input.heure_debut)
        .bind(&input.heure_fin)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    async fn lister_creneaux(
        &self,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<Vec<CreneauActivite>, AppError> {
        let rows = sqlx::query_as::<_, CreneauActivite>(
            "SELECT * FROM creneaux_activite
             WHERE activite_id = ? AND annee_scolaire = ?
             ORDER BY jour_semaine, heure_debut",
        )
        .bind(activite_id)
        .bind(annee_scolaire)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn lister_tous_creneaux(&self) -> Result<Vec<CreneauActivite>, AppError> {
        let rows =
            sqlx::query_as::<_, CreneauActivite>("SELECT * FROM creneaux_activite ORDER BY id")
                .fetch_all(&self.pool)
                .await?;

        Ok(rows)
    }

    async fn lister_creneaux_hors_plage(
        &self,
        heure_ouverture: &str,
        heure_fermeture: &str,
    ) -> Result<Vec<CreneauHorsPlage>, AppError> {
        let rows = sqlx::query_as::<_, CreneauHorsPlage>(
            "SELECT c.id AS creneau_id,
                    c.activite_id AS activite_id,
                    a.nom AS activite_nom,
                    c.jour_semaine AS jour_semaine,
                    c.heure_debut AS heure_debut,
                    c.heure_fin AS heure_fin,
                    c.annee_scolaire AS annee_scolaire,
                    (SELECT COUNT(*) FROM activite_personnes ap
                     WHERE ap.activite_id = c.activite_id
                       AND ap.annee_scolaire = c.annee_scolaire) AS nb_inscrits
             FROM creneaux_activite c
             JOIN activites a ON a.id = c.activite_id
             WHERE c.heure_debut < ? OR c.heure_fin > ?
             ORDER BY c.id",
        )
        .bind(heure_ouverture)
        .bind(heure_fermeture)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn lister_inscriptions(&self) -> Result<Vec<Inscription>, AppError> {
        let rows = sqlx::query_as::<_, Inscription>(
            "SELECT ap.activite_id AS activite_id,
                    ap.personne_id AS personne_id,
                    ap.annee_scolaire AS annee_scolaire,
                    a.nom AS activite_nom
             FROM activite_personnes ap
             JOIN activites a ON a.id = ap.activite_id
             ORDER BY ap.activite_id, ap.personne_id",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn supprimer_creneau_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        id: i64,
    ) -> Result<(), AppError> {
        sqlx::query("DELETE FROM creneaux_activite WHERE id = ?")
            .bind(id)
            .execute(&mut **tx)
            .await?;

        Ok(())
    }

    async fn deplacer_creneau_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        id: i64,
        heure_debut: &str,
        heure_fin: &str,
    ) -> Result<CreneauActivite, AppError> {
        let row = sqlx::query_as::<_, CreneauActivite>(
            "UPDATE creneaux_activite
             SET heure_debut = ?, heure_fin = ?
             WHERE id = ?
             RETURNING *",
        )
        .bind(heure_debut)
        .bind(heure_fin)
        .bind(id)
        .fetch_one(&mut **tx)
        .await?;

        Ok(row)
    }

    async fn ajouter_semaine_banalisee(
        &self,
        input: CreateSemaineBanalisee,
    ) -> Result<SemaineBanalisee, AppError> {
        let row = sqlx::query_as::<_, SemaineBanalisee>(
            "INSERT INTO semaines_banalisees (activite_id, date_debut, motif, annee_scolaire)
             VALUES (?, ?, ?, ?)
             RETURNING *",
        )
        .bind(input.activite_id)
        .bind(&input.date_debut)
        .bind(&input.motif)
        .bind(&input.annee_scolaire)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    async fn supprimer_semaine_banalisee(&self, id: i64) -> Result<(), AppError> {
        sqlx::query("DELETE FROM semaines_banalisees WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn lister_semaines_banalisees(
        &self,
        activite_id: i64,
    ) -> Result<Vec<SemaineBanalisee>, AppError> {
        let rows = sqlx::query_as::<_, SemaineBanalisee>(
            "SELECT * FROM semaines_banalisees
             WHERE activite_id = ?
             ORDER BY date_debut",
        )
        .bind(activite_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn verifier_conflit_creneaux(
        &self,
        activite_id: i64,
        annee_scolaire: &str,
        jour_semaine: i64,
        heure_debut: &str,
        heure_fin: &str,
        exclure_id: Option<i64>,
    ) -> Result<Vec<CreneauActivite>, AppError> {
        let rows = sqlx::query_as::<_, CreneauActivite>(
            "SELECT * FROM creneaux_activite
             WHERE activite_id = ?
               AND annee_scolaire = ?
               AND jour_semaine = ?
               AND heure_debut < ?
               AND heure_fin > ?
               AND (? IS NULL OR id != ?)",
        )
        .bind(activite_id)
        .bind(annee_scolaire)
        .bind(jour_semaine)
        .bind(heure_fin)
        .bind(heure_debut)
        .bind(exclure_id)
        .bind(exclure_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn compter_inscrits_activite(
        &self,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<i64, AppError> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM activite_personnes
             WHERE activite_id = ? AND annee_scolaire = ?",
        )
        .bind(activite_id)
        .bind(annee_scolaire)
        .fetch_one(&self.pool)
        .await?;

        Ok(count.0)
    }

    async fn verifier_collision(
        &self,
        personne_id: i64,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<Option<Collision>, AppError> {
        let creneaux_cibles = self.lister_creneaux(activite_id, annee_scolaire).await?;
        if creneaux_cibles.is_empty() {
            return Ok(None);
        }

        let autres_activites = sqlx::query_scalar::<_, i64>(
            "SELECT activite_id FROM activite_personnes
             WHERE personne_id = ? AND annee_scolaire = ? AND activite_id != ?",
        )
        .bind(personne_id)
        .bind(annee_scolaire)
        .bind(activite_id)
        .fetch_all(&self.pool)
        .await?;

        for autre_id in autres_activites {
            let creneaux_autre = self.lister_creneaux(autre_id, annee_scolaire).await?;
            for cible in &creneaux_cibles {
                for autre in &creneaux_autre {
                    if cible.jour_semaine == autre.jour_semaine
                        && cible.heure_debut < autre.heure_fin
                        && cible.heure_fin > autre.heure_debut
                    {
                        let nom = sqlx::query_scalar::<_, String>(
                            "SELECT nom FROM activites WHERE id = ?",
                        )
                        .bind(autre_id)
                        .fetch_one(&self.pool)
                        .await?;

                        return Ok(Some(Collision {
                            activite_conflit: nom,
                            jour_semaine: cible.jour_semaine,
                            heure_debut: cible.heure_debut.clone(),
                            heure_fin: cible.heure_fin.clone(),
                        }));
                    }
                }
            }
        }

        Ok(None)
    }

    async fn planning_personne_semaine(
        &self,
        personne_id: i64,
        date_lundi: &str,
        annee_scolaire: &str,
    ) -> Result<Vec<PlanningCreneau>, AppError> {
        #[derive(Debug, Clone, sqlx::FromRow)]
        struct ActiviteCreneauRow {
            activite_id: i64,
            nom: String,
            description: Option<String>,
            capacite_max: Option<i64>,
            creneau_id: i64,
            jour_semaine: i64,
            heure_debut: String,
            heure_fin: String,
            annee_scolaire: String,
            role: Role,
        }

        let rows = sqlx::query_as::<_, ActiviteCreneauRow>(
            "SELECT a.id AS activite_id, a.nom, a.description, a.capacite_max,
                    c.id AS creneau_id, c.jour_semaine, c.heure_debut, c.heure_fin, c.annee_scolaire,
                    ap.role
             FROM activite_personnes ap
             JOIN activites a ON a.id = ap.activite_id
             JOIN creneaux_activite c ON c.activite_id = a.id
             WHERE ap.personne_id = ?
               AND c.annee_scolaire = ?
               AND ap.annee_scolaire = ?
               AND NOT EXISTS (
                   SELECT 1 FROM semaines_banalisees sb
                   WHERE sb.activite_id = a.id AND sb.date_debut = ?
               )
             ORDER BY c.jour_semaine, c.heure_debut",
        )
        .bind(personne_id)
        .bind(annee_scolaire)
        .bind(annee_scolaire)
        .bind(date_lundi)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| PlanningCreneau {
                creneau: CreneauActivite {
                    id: r.creneau_id,
                    activite_id: r.activite_id,
                    jour_semaine: r.jour_semaine,
                    heure_debut: r.heure_debut,
                    heure_fin: r.heure_fin,
                    annee_scolaire: r.annee_scolaire,
                },
                activite: crate::domain::activite::Activite {
                    id: r.activite_id,
                    nom: r.nom,
                    description: r.description,
                    capacite_max: r.capacite_max,
                },
                role: r.role,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::activite::Role;
    use crate::domain::planning::{CreateCreneau, CreateSemaineBanalisee};
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

    fn repo(pool: SqlitePool) -> SqlitePlanningRepository {
        SqlitePlanningRepository::new(pool)
    }

    async fn seed_activite(pool: &SqlitePool, nom: &str) -> i64 {
        let row = sqlx::query_as::<_, crate::domain::activite::Activite>(
            "INSERT INTO activites (nom, description, capacite_max)
             VALUES (?, ?, ?) RETURNING *",
        )
        .bind(nom)
        .bind(None::<String>)
        .bind(None::<i64>)
        .fetch_one(pool)
        .await
        .expect("failed to seed activite");
        row.id
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
    async fn test_creer_creneau() {
        let pool = setup_db().await;
        let activite_id = seed_activite(&pool, "Poterie").await;
        let r = repo(pool.clone());

        let c = r
            .creer_creneau(CreateCreneau {
                activite_id,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            })
            .await
            .expect("failed to create creneau");

        assert_eq!(c.activite_id, activite_id);
        assert_eq!(c.jour_semaine, 1);
        assert_eq!(c.heure_debut, "14:00");
        assert_eq!(c.heure_fin, "16:00");
    }

    #[tokio::test]
    async fn test_lister_creneaux() {
        let pool = setup_db().await;
        let activite_id = seed_activite(&pool, "Poterie").await;
        let r = repo(pool.clone());

        r.creer_creneau(CreateCreneau {
            activite_id,
            jour_semaine: 1,
            heure_debut: "14:00".to_string(),
            heure_fin: "16:00".to_string(),
            annee_scolaire: "2025-2026".to_string(),
        })
        .await
        .unwrap();

        r.creer_creneau(CreateCreneau {
            activite_id,
            jour_semaine: 3,
            heure_debut: "10:00".to_string(),
            heure_fin: "12:00".to_string(),
            annee_scolaire: "2025-2026".to_string(),
        })
        .await
        .unwrap();

        let list = r.lister_creneaux(activite_id, "2025-2026").await.unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].jour_semaine, 1);
        assert_eq!(list[1].jour_semaine, 3);
    }

    #[tokio::test]
    async fn test_lister_creneaux_autre_annee() {
        let pool = setup_db().await;
        let activite_id = seed_activite(&pool, "Poterie").await;
        let r = repo(pool.clone());

        r.creer_creneau(CreateCreneau {
            activite_id,
            jour_semaine: 1,
            heure_debut: "14:00".to_string(),
            heure_fin: "16:00".to_string(),
            annee_scolaire: "2024-2025".to_string(),
        })
        .await
        .unwrap();

        let list = r.lister_creneaux(activite_id, "2025-2026").await.unwrap();
        assert_eq!(list.len(), 0);
    }

    #[tokio::test]
    async fn test_supprimer_creneau() {
        let pool = setup_db().await;
        let activite_id = seed_activite(&pool, "Poterie").await;
        let r = repo(pool.clone());

        let c = r
            .creer_creneau(CreateCreneau {
                activite_id,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            })
            .await
            .unwrap();

        r.supprimer_creneau(c.id).await.unwrap();

        let list = r.lister_creneaux(activite_id, "2025-2026").await.unwrap();
        assert_eq!(list.len(), 0);
    }

    #[tokio::test]
    async fn test_modifier_creneau() {
        let pool = setup_db().await;
        let activite_id = seed_activite(&pool, "Poterie").await;
        let r = repo(pool.clone());

        let c = r
            .creer_creneau(CreateCreneau {
                activite_id,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            })
            .await
            .unwrap();

        let updated = r
            .modifier_creneau(
                c.id,
                CreateCreneau {
                    activite_id,
                    jour_semaine: 2,
                    heure_debut: "09:00".to_string(),
                    heure_fin: "11:00".to_string(),
                    annee_scolaire: "2025-2026".to_string(),
                },
            )
            .await
            .unwrap();

        assert_eq!(updated.jour_semaine, 2);
        assert_eq!(updated.heure_debut, "09:00");
        assert_eq!(updated.heure_fin, "11:00");
    }

    #[tokio::test]
    async fn test_ajouter_semaine_banalisee() {
        let pool = setup_db().await;
        let activite_id = seed_activite(&pool, "Poterie").await;
        let r = repo(pool.clone());

        let sb = r
            .ajouter_semaine_banalisee(CreateSemaineBanalisee {
                activite_id,
                date_debut: "2025-12-22".to_string(),
                motif: Some("Vacances de Noël".to_string()),
                annee_scolaire: "2025-2026".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(sb.date_debut, "2025-12-22");
        assert_eq!(sb.motif, Some("Vacances de Noël".to_string()));
    }

    #[tokio::test]
    async fn test_ajouter_semaine_banalisee_sans_motif() {
        let pool = setup_db().await;
        let activite_id = seed_activite(&pool, "Poterie").await;
        let r = repo(pool.clone());

        let sb = r
            .ajouter_semaine_banalisee(CreateSemaineBanalisee {
                activite_id,
                date_debut: "2025-12-22".to_string(),
                motif: None,
                annee_scolaire: "2025-2026".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(sb.motif, None);
    }

    #[tokio::test]
    async fn test_lister_semaines_banalisees() {
        let pool = setup_db().await;
        let activite_id = seed_activite(&pool, "Poterie").await;
        let r = repo(pool.clone());

        r.ajouter_semaine_banalisee(CreateSemaineBanalisee {
            activite_id,
            date_debut: "2025-12-22".to_string(),
            motif: Some("Noël".to_string()),
            annee_scolaire: "2025-2026".to_string(),
        })
        .await
        .unwrap();

        r.ajouter_semaine_banalisee(CreateSemaineBanalisee {
            activite_id,
            date_debut: "2026-02-23".to_string(),
            motif: Some("Hiver".to_string()),
            annee_scolaire: "2025-2026".to_string(),
        })
        .await
        .unwrap();

        let list = r.lister_semaines_banalisees(activite_id).await.unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].date_debut, "2025-12-22");
        assert_eq!(list[1].date_debut, "2026-02-23");
    }

    #[tokio::test]
    async fn test_supprimer_semaine_banalisee() {
        let pool = setup_db().await;
        let activite_id = seed_activite(&pool, "Poterie").await;
        let r = repo(pool.clone());

        let sb = r
            .ajouter_semaine_banalisee(CreateSemaineBanalisee {
                activite_id,
                date_debut: "2025-12-22".to_string(),
                motif: None,
                annee_scolaire: "2025-2026".to_string(),
            })
            .await
            .unwrap();

        r.supprimer_semaine_banalisee(sb.id).await.unwrap();

        let list = r.lister_semaines_banalisees(activite_id).await.unwrap();
        assert_eq!(list.len(), 0);
    }

    #[tokio::test]
    async fn test_compter_inscrits_activite() {
        let pool = setup_db().await;
        let activite_id = seed_activite(&pool, "Poterie").await;
        let pid = seed_personne(&pool).await;
        let r = repo(pool.clone());

        let count = r
            .compter_inscrits_activite(activite_id, "2025-2026")
            .await
            .unwrap();
        assert_eq!(count, 0);

        sqlx::query(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)",
        )
        .bind(activite_id)
        .bind(pid)
        .bind("2025-2026")
        .bind(Role::Participant)
        .execute(&r.pool)
        .await
        .unwrap();

        let count = r
            .compter_inscrits_activite(activite_id, "2025-2026")
            .await
            .unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_verifier_conflit_creneaux_doublon() {
        let pool = setup_db().await;
        let a = seed_activite(&pool, "Poterie").await;
        let r = repo(pool.clone());

        r.creer_creneau(CreateCreneau {
            activite_id: a,
            jour_semaine: 1,
            heure_debut: "14:00".to_string(),
            heure_fin: "16:00".to_string(),
            annee_scolaire: "2025-2026".to_string(),
        })
        .await
        .unwrap();

        let conflits = r
            .verifier_conflit_creneaux(a, "2025-2026", 1, "14:00", "16:00", None)
            .await
            .unwrap();
        assert_eq!(conflits.len(), 1);
    }

    #[tokio::test]
    async fn test_verifier_conflit_creneaux_exclure_id() {
        let pool = setup_db().await;
        let a = seed_activite(&pool, "Poterie").await;
        let r = repo(pool.clone());

        let c = r
            .creer_creneau(CreateCreneau {
                activite_id: a,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            })
            .await
            .unwrap();

        let conflits = r
            .verifier_conflit_creneaux(a, "2025-2026", 1, "14:00", "16:00", Some(c.id))
            .await
            .unwrap();
        assert!(conflits.is_empty());
    }

    #[tokio::test]
    async fn test_verifier_conflit_creneaux_partiel() {
        let pool = setup_db().await;
        let a = seed_activite(&pool, "Poterie").await;
        let r = repo(pool.clone());

        r.creer_creneau(CreateCreneau {
            activite_id: a,
            jour_semaine: 1,
            heure_debut: "10:00".to_string(),
            heure_fin: "12:00".to_string(),
            annee_scolaire: "2025-2026".to_string(),
        })
        .await
        .unwrap();

        let conflits = r
            .verifier_conflit_creneaux(a, "2025-2026", 1, "11:00", "13:00", None)
            .await
            .unwrap();
        assert_eq!(conflits.len(), 1);
    }

    #[tokio::test]
    async fn test_verifier_conflit_creneaux_adjacent() {
        let pool = setup_db().await;
        let a = seed_activite(&pool, "Poterie").await;
        let r = repo(pool.clone());

        r.creer_creneau(CreateCreneau {
            activite_id: a,
            jour_semaine: 1,
            heure_debut: "14:00".to_string(),
            heure_fin: "16:00".to_string(),
            annee_scolaire: "2025-2026".to_string(),
        })
        .await
        .unwrap();

        let conflits = r
            .verifier_conflit_creneaux(a, "2025-2026", 1, "16:00", "18:00", None)
            .await
            .unwrap();
        assert!(conflits.is_empty());
    }

    #[tokio::test]
    async fn test_verifier_conflit_creneaux_autre_activite() {
        let pool = setup_db().await;
        let a1 = seed_activite(&pool, "Poterie").await;
        let a2 = seed_activite(&pool, "Théâtre").await;
        let r = repo(pool.clone());

        r.creer_creneau(CreateCreneau {
            activite_id: a1,
            jour_semaine: 1,
            heure_debut: "14:00".to_string(),
            heure_fin: "16:00".to_string(),
            annee_scolaire: "2025-2026".to_string(),
        })
        .await
        .unwrap();

        let conflits = r
            .verifier_conflit_creneaux(a2, "2025-2026", 1, "14:00", "16:00", None)
            .await
            .unwrap();
        assert!(conflits.is_empty());
    }

    #[tokio::test]
    async fn test_verifier_conflit_creneaux_autre_annee() {
        let pool = setup_db().await;
        let a = seed_activite(&pool, "Poterie").await;
        let r = repo(pool.clone());

        r.creer_creneau(CreateCreneau {
            activite_id: a,
            jour_semaine: 1,
            heure_debut: "14:00".to_string(),
            heure_fin: "16:00".to_string(),
            annee_scolaire: "2024-2025".to_string(),
        })
        .await
        .unwrap();

        let conflits = r
            .verifier_conflit_creneaux(a, "2025-2026", 1, "14:00", "16:00", None)
            .await
            .unwrap();
        assert!(conflits.is_empty());
    }

    #[tokio::test]
    async fn test_verifier_confrit_creneaux_autre_jour() {
        let pool = setup_db().await;
        let a = seed_activite(&pool, "Poterie").await;
        let r = repo(pool.clone());

        r.creer_creneau(CreateCreneau {
            activite_id: a,
            jour_semaine: 1,
            heure_debut: "14:00".to_string(),
            heure_fin: "16:00".to_string(),
            annee_scolaire: "2025-2026".to_string(),
        })
        .await
        .unwrap();

        let conflits = r
            .verifier_conflit_creneaux(a, "2025-2026", 2, "14:00", "16:00", None)
            .await
            .unwrap();
        assert!(conflits.is_empty());
    }

    #[tokio::test]
    async fn test_verifier_collision_pas_de_conflit() {
        let pool = setup_db().await;
        let a1 = seed_activite(&pool, "Poterie").await;
        let a2 = seed_activite(&pool, "Théâtre").await;
        let pid = seed_personne(&pool).await;
        let r = repo(pool.clone());

        r.creer_creneau(CreateCreneau {
            activite_id: a1,
            jour_semaine: 1,
            heure_debut: "14:00".to_string(),
            heure_fin: "16:00".to_string(),
            annee_scolaire: "2025-2026".to_string(),
        })
        .await
        .unwrap();

        r.creer_creneau(CreateCreneau {
            activite_id: a2,
            jour_semaine: 3,
            heure_debut: "10:00".to_string(),
            heure_fin: "12:00".to_string(),
            annee_scolaire: "2025-2026".to_string(),
        })
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)",
        )
        .bind(a1)
        .bind(pid)
        .bind("2025-2026")
        .bind(Role::Participant)
        .execute(&r.pool)
        .await
        .unwrap();

        let collision = r.verifier_collision(pid, a2, "2025-2026").await.unwrap();
        assert!(collision.is_none());
    }

    #[tokio::test]
    async fn test_verifier_collision_conflit() {
        let pool = setup_db().await;
        let a1 = seed_activite(&pool, "Poterie").await;
        let a2 = seed_activite(&pool, "Théâtre").await;
        let pid = seed_personne(&pool).await;
        let r = repo(pool.clone());

        r.creer_creneau(CreateCreneau {
            activite_id: a1,
            jour_semaine: 1,
            heure_debut: "14:00".to_string(),
            heure_fin: "16:00".to_string(),
            annee_scolaire: "2025-2026".to_string(),
        })
        .await
        .unwrap();

        r.creer_creneau(CreateCreneau {
            activite_id: a2,
            jour_semaine: 1,
            heure_debut: "15:00".to_string(),
            heure_fin: "17:00".to_string(),
            annee_scolaire: "2025-2026".to_string(),
        })
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)",
        )
        .bind(a1)
        .bind(pid)
        .bind("2025-2026")
        .bind(Role::Encadrant)
        .execute(&r.pool)
        .await
        .unwrap();

        let collision = r.verifier_collision(pid, a2, "2025-2026").await.unwrap();
        assert!(collision.is_some());
        let c = collision.unwrap();
        assert!(c.activite_conflit.contains("Poterie"));
    }

    #[tokio::test]
    async fn test_verifier_collision_meme_activite_ignoree() {
        let pool = setup_db().await;
        let a1 = seed_activite(&pool, "Poterie").await;
        let pid = seed_personne(&pool).await;
        let r = repo(pool.clone());

        r.creer_creneau(CreateCreneau {
            activite_id: a1,
            jour_semaine: 1,
            heure_debut: "14:00".to_string(),
            heure_fin: "16:00".to_string(),
            annee_scolaire: "2025-2026".to_string(),
        })
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)",
        )
        .bind(a1)
        .bind(pid)
        .bind("2025-2026")
        .bind(Role::Participant)
        .execute(&r.pool)
        .await
        .unwrap();

        let collision = r.verifier_collision(pid, a1, "2025-2026").await.unwrap();
        assert!(collision.is_none());
    }

    #[tokio::test]
    async fn test_planning_personne_semaine() {
        let pool = setup_db().await;
        let a1 = seed_activite(&pool, "Poterie").await;
        let a2 = seed_activite(&pool, "Théâtre").await;
        let pid = seed_personne(&pool).await;
        let r = repo(pool.clone());

        r.creer_creneau(CreateCreneau {
            activite_id: a1,
            jour_semaine: 1,
            heure_debut: "14:00".to_string(),
            heure_fin: "16:00".to_string(),
            annee_scolaire: "2025-2026".to_string(),
        })
        .await
        .unwrap();

        r.creer_creneau(CreateCreneau {
            activite_id: a2,
            jour_semaine: 3,
            heure_debut: "10:00".to_string(),
            heure_fin: "12:00".to_string(),
            annee_scolaire: "2025-2026".to_string(),
        })
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)",
        )
        .bind(a1)
        .bind(pid)
        .bind("2025-2026")
        .bind(Role::Participant)
        .execute(&r.pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)",
        )
        .bind(a2)
        .bind(pid)
        .bind("2025-2026")
        .bind(Role::Encadrant)
        .execute(&r.pool)
        .await
        .unwrap();

        let planning = r
            .planning_personne_semaine(pid, "2025-09-01", "2025-2026")
            .await
            .unwrap();
        assert_eq!(planning.len(), 2);
        assert_eq!(planning[0].role, Role::Participant);
        assert_eq!(planning[1].role, Role::Encadrant);
    }

    #[tokio::test]
    async fn test_planning_personne_semaine_banalisee_exclue() {
        let pool = setup_db().await;
        let a1 = seed_activite(&pool, "Poterie").await;
        let pid = seed_personne(&pool).await;
        let r = repo(pool.clone());

        r.creer_creneau(CreateCreneau {
            activite_id: a1,
            jour_semaine: 1,
            heure_debut: "14:00".to_string(),
            heure_fin: "16:00".to_string(),
            annee_scolaire: "2025-2026".to_string(),
        })
        .await
        .unwrap();

        r.ajouter_semaine_banalisee(CreateSemaineBanalisee {
            activite_id: a1,
            date_debut: "2025-12-22".to_string(),
            motif: Some("Noël".to_string()),
            annee_scolaire: "2025-2026".to_string(),
        })
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)",
        )
        .bind(a1)
        .bind(pid)
        .bind("2025-2026")
        .bind(Role::Participant)
        .execute(&r.pool)
        .await
        .unwrap();

        let planning = r
            .planning_personne_semaine(pid, "2025-12-22", "2025-2026")
            .await
            .unwrap();
        assert_eq!(planning.len(), 0);

        let planning = r
            .planning_personne_semaine(pid, "2025-09-01", "2025-2026")
            .await
            .unwrap();
        assert_eq!(planning.len(), 1);
    }

    #[tokio::test]
    async fn test_planning_personne_semaine_aucune_activite() {
        let pool = setup_db().await;
        let pid = seed_personne(&pool).await;
        let r = repo(pool.clone());

        let planning = r
            .planning_personne_semaine(pid, "2025-09-01", "2025-2026")
            .await
            .unwrap();
        assert_eq!(planning.len(), 0);
    }

    #[tokio::test]
    async fn test_verifier_collision_exact_overlap() {
        let pool = setup_db().await;
        let a1 = seed_activite(&pool, "Poterie").await;
        let a2 = seed_activite(&pool, "Théâtre").await;
        let pid = seed_personne(&pool).await;
        let r = repo(pool.clone());

        r.creer_creneau(CreateCreneau {
            activite_id: a1,
            jour_semaine: 1,
            heure_debut: "14:00".to_string(),
            heure_fin: "16:00".to_string(),
            annee_scolaire: "2025-2026".to_string(),
        })
        .await
        .unwrap();

        r.creer_creneau(CreateCreneau {
            activite_id: a2,
            jour_semaine: 1,
            heure_debut: "14:00".to_string(),
            heure_fin: "16:00".to_string(),
            annee_scolaire: "2025-2026".to_string(),
        })
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)",
        )
        .bind(a1)
        .bind(pid)
        .bind("2025-2026")
        .bind(Role::Participant)
        .execute(&r.pool)
        .await
        .unwrap();

        let collision = r.verifier_collision(pid, a2, "2025-2026").await.unwrap();
        assert!(collision.is_some());
    }

    #[tokio::test]
    async fn test_verifier_collision_contenant_contenu() {
        let pool = setup_db().await;
        let a1 = seed_activite(&pool, "Poterie").await;
        let a2 = seed_activite(&pool, "Théâtre").await;
        let pid = seed_personne(&pool).await;
        let r = repo(pool.clone());

        r.creer_creneau(CreateCreneau {
            activite_id: a1,
            jour_semaine: 1,
            heure_debut: "10:00".to_string(),
            heure_fin: "18:00".to_string(),
            annee_scolaire: "2025-2026".to_string(),
        })
        .await
        .unwrap();

        r.creer_creneau(CreateCreneau {
            activite_id: a2,
            jour_semaine: 1,
            heure_debut: "14:00".to_string(),
            heure_fin: "16:00".to_string(),
            annee_scolaire: "2025-2026".to_string(),
        })
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)",
        )
        .bind(a1)
        .bind(pid)
        .bind("2025-2026")
        .bind(Role::Encadrant)
        .execute(&r.pool)
        .await
        .unwrap();

        let collision = r.verifier_collision(pid, a2, "2025-2026").await.unwrap();
        assert!(collision.is_some());
    }

    #[tokio::test]
    async fn test_verifier_collision_adjacent_no_overlap() {
        let pool = setup_db().await;
        let a1 = seed_activite(&pool, "Poterie").await;
        let a2 = seed_activite(&pool, "Théâtre").await;
        let pid = seed_personne(&pool).await;
        let r = repo(pool.clone());

        r.creer_creneau(CreateCreneau {
            activite_id: a1,
            jour_semaine: 1,
            heure_debut: "14:00".to_string(),
            heure_fin: "16:00".to_string(),
            annee_scolaire: "2025-2026".to_string(),
        })
        .await
        .unwrap();

        r.creer_creneau(CreateCreneau {
            activite_id: a2,
            jour_semaine: 1,
            heure_debut: "16:00".to_string(),
            heure_fin: "18:00".to_string(),
            annee_scolaire: "2025-2026".to_string(),
        })
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)",
        )
        .bind(a1)
        .bind(pid)
        .bind("2025-2026")
        .bind(Role::Participant)
        .execute(&r.pool)
        .await
        .unwrap();

        let collision = r.verifier_collision(pid, a2, "2025-2026").await.unwrap();
        assert!(collision.is_none());
    }

    #[tokio::test]
    async fn test_verifier_collision_activite_sans_creneaux() {
        let pool = setup_db().await;
        let a1 = seed_activite(&pool, "Poterie").await;
        let a2 = seed_activite(&pool, "Théâtre").await;
        let pid = seed_personne(&pool).await;
        let r = repo(pool.clone());

        r.creer_creneau(CreateCreneau {
            activite_id: a2,
            jour_semaine: 1,
            heure_debut: "14:00".to_string(),
            heure_fin: "16:00".to_string(),
            annee_scolaire: "2025-2026".to_string(),
        })
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)",
        )
        .bind(a1)
        .bind(pid)
        .bind("2025-2026")
        .bind(Role::Participant)
        .execute(&r.pool)
        .await
        .unwrap();

        let collision = r.verifier_collision(pid, a2, "2025-2026").await.unwrap();
        assert!(collision.is_none());
    }

    #[tokio::test]
    async fn test_verifier_collision_personne_sans_activite() {
        let pool = setup_db().await;
        let a1 = seed_activite(&pool, "Poterie").await;
        let pid = seed_personne(&pool).await;
        let r = repo(pool.clone());

        r.creer_creneau(CreateCreneau {
            activite_id: a1,
            jour_semaine: 1,
            heure_debut: "14:00".to_string(),
            heure_fin: "16:00".to_string(),
            annee_scolaire: "2025-2026".to_string(),
        })
        .await
        .unwrap();

        let collision = r.verifier_collision(pid, a1, "2025-2026").await.unwrap();
        assert!(collision.is_none());
    }

    #[tokio::test]
    async fn test_compter_inscrits_encadrant_et_participant() {
        let pool = setup_db().await;
        let activite_id = seed_activite(&pool, "Poterie").await;
        let pid1 = seed_personne(&pool).await;
        let pid2 = seed_personne(&pool).await;
        let r = repo(pool.clone());

        sqlx::query(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)",
        )
        .bind(activite_id)
        .bind(pid1)
        .bind("2025-2026")
        .bind(Role::Encadrant)
        .execute(&r.pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)",
        )
        .bind(activite_id)
        .bind(pid2)
        .bind("2025-2026")
        .bind(Role::Participant)
        .execute(&r.pool)
        .await
        .unwrap();

        let count = r
            .compter_inscrits_activite(activite_id, "2025-2026")
            .await
            .unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_compter_inscrits_autre_annee() {
        let pool = setup_db().await;
        let activite_id = seed_activite(&pool, "Poterie").await;
        let pid = seed_personne(&pool).await;
        let r = repo(pool.clone());

        sqlx::query(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)",
        )
        .bind(activite_id)
        .bind(pid)
        .bind("2024-2025")
        .bind(Role::Participant)
        .execute(&r.pool)
        .await
        .unwrap();

        let count = r
            .compter_inscrits_activite(activite_id, "2025-2026")
            .await
            .unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_planning_personne_meme_jour_trie_par_heure() {
        let pool = setup_db().await;
        let a1 = seed_activite(&pool, "Poterie").await;
        let a2 = seed_activite(&pool, "Théâtre").await;
        let pid = seed_personne(&pool).await;
        let r = repo(pool.clone());

        r.creer_creneau(CreateCreneau {
            activite_id: a1,
            jour_semaine: 1,
            heure_debut: "16:00".to_string(),
            heure_fin: "18:00".to_string(),
            annee_scolaire: "2025-2026".to_string(),
        })
        .await
        .unwrap();

        r.creer_creneau(CreateCreneau {
            activite_id: a2,
            jour_semaine: 1,
            heure_debut: "10:00".to_string(),
            heure_fin: "12:00".to_string(),
            annee_scolaire: "2025-2026".to_string(),
        })
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)",
        )
        .bind(a1)
        .bind(pid)
        .bind("2025-2026")
        .bind(Role::Participant)
        .execute(&r.pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)",
        )
        .bind(a2)
        .bind(pid)
        .bind("2025-2026")
        .bind(Role::Participant)
        .execute(&r.pool)
        .await
        .unwrap();

        let planning = r
            .planning_personne_semaine(pid, "2025-09-01", "2025-2026")
            .await
            .unwrap();
        assert_eq!(planning.len(), 2);
        assert_eq!(planning[0].creneau.heure_debut, "10:00");
        assert_eq!(planning[1].creneau.heure_debut, "16:00");
    }

    #[tokio::test]
    async fn test_creer_creneau_plusieurs_activites() {
        let pool = setup_db().await;
        let a1 = seed_activite(&pool, "Poterie").await;
        let a2 = seed_activite(&pool, "Théâtre").await;
        let r = repo(pool.clone());

        let c1 = r
            .creer_creneau(CreateCreneau {
                activite_id: a1,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            })
            .await
            .unwrap();

        let c2 = r
            .creer_creneau(CreateCreneau {
                activite_id: a2,
                jour_semaine: 3,
                heure_debut: "10:00".to_string(),
                heure_fin: "12:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(c1.activite_id, a1);
        assert_eq!(c2.activite_id, a2);
    }

    #[tokio::test]
    async fn test_semaine_banalisee_meme_date_deux_activites() {
        let pool = setup_db().await;
        let a1 = seed_activite(&pool, "Poterie").await;
        let a2 = seed_activite(&pool, "Théâtre").await;
        let r = repo(pool.clone());

        let sb1 = r
            .ajouter_semaine_banalisee(CreateSemaineBanalisee {
                activite_id: a1,
                date_debut: "2025-12-22".to_string(),
                motif: Some("Noël".to_string()),
                annee_scolaire: "2025-2026".to_string(),
            })
            .await
            .unwrap();

        let sb2 = r
            .ajouter_semaine_banalisee(CreateSemaineBanalisee {
                activite_id: a2,
                date_debut: "2025-12-22".to_string(),
                motif: Some("Noël".to_string()),
                annee_scolaire: "2025-2026".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(sb1.date_debut, sb2.date_debut);
        assert_ne!(sb1.id, sb2.id);
    }

    #[tokio::test]
    async fn test_lister_creneaux_tri_par_jour_puis_heure() {
        let pool = setup_db().await;
        let activite_id = seed_activite(&pool, "Poterie").await;
        let r = repo(pool.clone());

        r.creer_creneau(CreateCreneau {
            activite_id,
            jour_semaine: 3,
            heure_debut: "14:00".to_string(),
            heure_fin: "16:00".to_string(),
            annee_scolaire: "2025-2026".to_string(),
        })
        .await
        .unwrap();

        r.creer_creneau(CreateCreneau {
            activite_id,
            jour_semaine: 1,
            heure_debut: "14:00".to_string(),
            heure_fin: "16:00".to_string(),
            annee_scolaire: "2025-2026".to_string(),
        })
        .await
        .unwrap();

        let list = r.lister_creneaux(activite_id, "2025-2026").await.unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].jour_semaine, 1);
        assert_eq!(list[1].jour_semaine, 3);
    }

    #[tokio::test]
    async fn test_modifier_creneau_inexistant() {
        let pool = setup_db().await;
        let r = repo(pool.clone());

        let result = r
            .modifier_creneau(
                99999,
                CreateCreneau {
                    activite_id: 1,
                    jour_semaine: 1,
                    heure_debut: "14:00".to_string(),
                    heure_fin: "16:00".to_string(),
                    annee_scolaire: "2025-2026".to_string(),
                },
            )
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_supprimer_creneau_inexistant() {
        let pool = setup_db().await;
        let r = repo(pool.clone());

        let result = r.supprimer_creneau(99999).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_planning_personne_activite_sans_creneaux() {
        let pool = setup_db().await;
        let a1 = seed_activite(&pool, "Poterie").await;
        let pid = seed_personne(&pool).await;
        let r = repo(pool.clone());

        sqlx::query(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)",
        )
        .bind(a1)
        .bind(pid)
        .bind("2025-2026")
        .bind(Role::Participant)
        .execute(&r.pool)
        .await
        .unwrap();

        let planning = r
            .planning_personne_semaine(pid, "2025-09-01", "2025-2026")
            .await
            .unwrap();
        assert_eq!(planning.len(), 0);
    }

    #[tokio::test]
    async fn test_semaine_banalisee_meme_activite_deux_dates() {
        let pool = setup_db().await;
        let activite_id = seed_activite(&pool, "Poterie").await;
        let r = repo(pool.clone());

        r.ajouter_semaine_banalisee(CreateSemaineBanalisee {
            activite_id,
            date_debut: "2025-12-22".to_string(),
            motif: None,
            annee_scolaire: "2025-2026".to_string(),
        })
        .await
        .unwrap();

        r.ajouter_semaine_banalisee(CreateSemaineBanalisee {
            activite_id,
            date_debut: "2025-12-29".to_string(),
            motif: None,
            annee_scolaire: "2025-2026".to_string(),
        })
        .await
        .unwrap();

        let list = r.lister_semaines_banalisees(activite_id).await.unwrap();
        assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn test_lister_creneaux_hors_plage_av_apres_partiel() {
        let pool = setup_db().await;
        let activite_id = seed_activite(&pool, "Poterie").await;
        let pid = seed_personne(&pool).await;
        let r = repo(pool.clone());

        let c1 = r
            .creer_creneau(CreateCreneau {
                activite_id,
                jour_semaine: 1,
                heure_debut: "07:00".to_string(),
                heure_fin: "08:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            })
            .await
            .unwrap();

        let c2 = r
            .creer_creneau(CreateCreneau {
                activite_id,
                jour_semaine: 1,
                heure_debut: "09:00".to_string(),
                heure_fin: "11:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            })
            .await
            .unwrap();

        let c3 = r
            .creer_creneau(CreateCreneau {
                activite_id,
                jour_semaine: 2,
                heure_debut: "19:00".to_string(),
                heure_fin: "21:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            })
            .await
            .unwrap();

        let c4 = r
            .creer_creneau(CreateCreneau {
                activite_id,
                jour_semaine: 3,
                heure_debut: "07:30".to_string(),
                heure_fin: "09:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            })
            .await
            .unwrap();

        sqlx::query(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)",
        )
        .bind(activite_id)
        .bind(pid)
        .bind("2025-2026")
        .bind(Role::Participant)
        .execute(&r.pool)
        .await
        .unwrap();

        let hors = r
            .lister_creneaux_hors_plage("08:00", "20:00")
            .await
            .unwrap();

        let ids: Vec<i64> = hors.iter().map(|h| h.creneau_id).collect();
        assert_eq!(ids, vec![c1.id, c3.id, c4.id]);
        assert!(!ids.contains(&c2.id));
        for h in &hors {
            assert_eq!(h.annee_scolaire, "2025-2026");
            assert_eq!(h.nb_inscrits, 1);
        }
    }

    #[tokio::test]
    async fn test_lister_creneaux_hors_plage_aucun() {
        let pool = setup_db().await;
        let activite_id = seed_activite(&pool, "Poterie").await;
        let r = repo(pool.clone());

        r.creer_creneau(CreateCreneau {
            activite_id,
            jour_semaine: 1,
            heure_debut: "09:00".to_string(),
            heure_fin: "11:00".to_string(),
            annee_scolaire: "2025-2026".to_string(),
        })
        .await
        .unwrap();

        let hors = r
            .lister_creneaux_hors_plage("08:00", "20:00")
            .await
            .unwrap();
        assert!(hors.is_empty());
    }

    #[tokio::test]
    async fn test_lister_tous_creneaux_sans_filtre() {
        let pool = setup_db().await;
        let a1 = seed_activite(&pool, "Poterie").await;
        let a2 = seed_activite(&pool, "Théâtre").await;
        let r = repo(pool.clone());

        let c1 = r
            .creer_creneau(CreateCreneau {
                activite_id: a1,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            })
            .await
            .unwrap();

        let c2 = r
            .creer_creneau(CreateCreneau {
                activite_id: a2,
                jour_semaine: 3,
                heure_debut: "10:00".to_string(),
                heure_fin: "12:00".to_string(),
                annee_scolaire: "2024-2025".to_string(),
            })
            .await
            .unwrap();

        let tous = r.lister_tous_creneaux().await.unwrap();
        assert_eq!(tous.len(), 2);
        assert_eq!(tous[0].id, c1.id);
        assert_eq!(tous[1].id, c2.id);
    }

    #[tokio::test]
    async fn test_supprimer_creneau_tx_commit_visible() {
        let pool = setup_db().await;
        let activite_id = seed_activite(&pool, "Poterie").await;
        let r = repo(pool.clone());

        let c = r
            .creer_creneau(CreateCreneau {
                activite_id,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            })
            .await
            .unwrap();

        let mut tx = pool.begin().await.unwrap();
        r.supprimer_creneau_tx(&mut tx, c.id).await.unwrap();
        tx.commit().await.unwrap();

        let list = r.lister_creneaux(activite_id, "2025-2026").await.unwrap();
        assert!(list.is_empty());
    }

    #[tokio::test]
    async fn test_supprimer_creneau_tx_rollback_sans_effet() {
        let pool = setup_db().await;
        let activite_id = seed_activite(&pool, "Poterie").await;
        let r = repo(pool.clone());

        let c = r
            .creer_creneau(CreateCreneau {
                activite_id,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            })
            .await
            .unwrap();

        let mut tx = pool.begin().await.unwrap();
        r.supprimer_creneau_tx(&mut tx, c.id).await.unwrap();
        tx.rollback().await.unwrap();

        let list = r.lister_creneaux(activite_id, "2025-2026").await.unwrap();
        assert_eq!(list.len(), 1);
    }

    #[tokio::test]
    async fn test_deplacer_creneau_tx_commit_visible() {
        let pool = setup_db().await;
        let activite_id = seed_activite(&pool, "Poterie").await;
        let r = repo(pool.clone());

        let c = r
            .creer_creneau(CreateCreneau {
                activite_id,
                jour_semaine: 1,
                heure_debut: "07:00".to_string(),
                heure_fin: "09:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            })
            .await
            .unwrap();

        let mut tx = pool.begin().await.unwrap();
        r.deplacer_creneau_tx(&mut tx, c.id, "09:00", "11:00")
            .await
            .unwrap();
        tx.commit().await.unwrap();

        let list = r.lister_creneaux(activite_id, "2025-2026").await.unwrap();
        assert_eq!(list[0].heure_debut, "09:00");
        assert_eq!(list[0].heure_fin, "11:00");
    }

    #[tokio::test]
    async fn test_deplacer_creneau_tx_rollback_sans_effet() {
        let pool = setup_db().await;
        let activite_id = seed_activite(&pool, "Poterie").await;
        let r = repo(pool.clone());

        let c = r
            .creer_creneau(CreateCreneau {
                activite_id,
                jour_semaine: 1,
                heure_debut: "07:00".to_string(),
                heure_fin: "09:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            })
            .await
            .unwrap();

        let mut tx = pool.begin().await.unwrap();
        r.deplacer_creneau_tx(&mut tx, c.id, "09:00", "11:00")
            .await
            .unwrap();
        tx.rollback().await.unwrap();

        let list = r.lister_creneaux(activite_id, "2025-2026").await.unwrap();
        assert_eq!(list[0].heure_debut, "07:00");
        assert_eq!(list[0].heure_fin, "09:00");
    }

    #[tokio::test]
    async fn test_lister_inscriptions_nom_joint() {
        let pool = setup_db().await;
        let a1 = seed_activite(&pool, "Poterie").await;
        let a2 = seed_activite(&pool, "Théâtre").await;
        let pid1 = seed_personne(&pool).await;
        let pid2 = seed_personne(&pool).await;
        let r = repo(pool.clone());

        sqlx::query(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)",
        )
        .bind(a1)
        .bind(pid1)
        .bind("2025-2026")
        .bind(Role::Participant)
        .execute(&r.pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)",
        )
        .bind(a1)
        .bind(pid2)
        .bind("2025-2026")
        .bind(Role::Encadrant)
        .execute(&r.pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)",
        )
        .bind(a2)
        .bind(pid1)
        .bind("2024-2025")
        .bind(Role::Participant)
        .execute(&r.pool)
        .await
        .unwrap();

        let inscrits = r.lister_inscriptions().await.unwrap();
        assert_eq!(inscrits.len(), 3);

        let poterie = inscrits
            .iter()
            .find(|i| i.activite_id == a1 && i.personne_id == pid1)
            .unwrap();
        assert_eq!(poterie.activite_nom, "Poterie");
        assert_eq!(poterie.annee_scolaire, "2025-2026");

        let theatre = inscrits.iter().find(|i| i.activite_id == a2).unwrap();
        assert_eq!(theatre.activite_nom, "Théâtre");
        assert_eq!(theatre.annee_scolaire, "2024-2025");
    }
}
