use sqlx::SqlitePool;

use crate::domain::planning::{
    Collision, CreateCreneau, CreateSemaineBanalisee, CreneauActivite, PlanningCreneau,
    SemaineBanalisee,
};

pub async fn creer_creneau(
    pool: &SqlitePool,
    input: CreateCreneau,
) -> Result<CreneauActivite, sqlx::Error> {
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
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn supprimer_creneau(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM creneaux_activite WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn modifier_creneau(
    pool: &SqlitePool,
    id: i64,
    input: CreateCreneau,
) -> Result<CreneauActivite, sqlx::Error> {
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
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn lister_creneaux(
    pool: &SqlitePool,
    activite_id: i64,
    annee_scolaire: &str,
) -> Result<Vec<CreneauActivite>, sqlx::Error> {
    let rows = sqlx::query_as::<_, CreneauActivite>(
        "SELECT * FROM creneaux_activite
         WHERE activite_id = ? AND annee_scolaire = ?
         ORDER BY jour_semaine, heure_debut",
    )
    .bind(activite_id)
    .bind(annee_scolaire)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn ajouter_semaine_banalisee(
    pool: &SqlitePool,
    input: CreateSemaineBanalisee,
) -> Result<SemaineBanalisee, sqlx::Error> {
    let row = sqlx::query_as::<_, SemaineBanalisee>(
        "INSERT INTO semaines_banalisees (activite_id, date_debut, motif, annee_scolaire)
         VALUES (?, ?, ?, ?)
         RETURNING *",
    )
    .bind(input.activite_id)
    .bind(&input.date_debut)
    .bind(&input.motif)
    .bind(&input.annee_scolaire)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn supprimer_semaine_banalisee(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM semaines_banalisees WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn lister_semaines_banalisees(
    pool: &SqlitePool,
    activite_id: i64,
) -> Result<Vec<SemaineBanalisee>, sqlx::Error> {
    let rows = sqlx::query_as::<_, SemaineBanalisee>(
        "SELECT * FROM semaines_banalisees
         WHERE activite_id = ?
         ORDER BY date_debut",
    )
    .bind(activite_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn compter_inscrits_activite(
    pool: &SqlitePool,
    activite_id: i64,
    annee_scolaire: &str,
) -> Result<i64, sqlx::Error> {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM activite_personnes
         WHERE activite_id = ? AND annee_scolaire = ?",
    )
    .bind(activite_id)
    .bind(annee_scolaire)
    .fetch_one(pool)
    .await?;

    Ok(count.0)
}

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
    role: String,
}

pub async fn verifier_collision(
    pool: &SqlitePool,
    personne_id: i64,
    activite_id: i64,
    annee_scolaire: &str,
) -> Result<Option<Collision>, sqlx::Error> {
    let creneaux_cibles = lister_creneaux(pool, activite_id, annee_scolaire).await?;
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
    .fetch_all(pool)
    .await?;

    for autre_id in autres_activites {
        let creneaux_autre = lister_creneaux(pool, autre_id, annee_scolaire).await?;
        for cible in &creneaux_cibles {
            for autre in &creneaux_autre {
                if cible.jour_semaine == autre.jour_semaine
                    && cible.heure_debut < autre.heure_fin
                    && cible.heure_fin > autre.heure_debut
                {
                    let nom =
                        sqlx::query_scalar::<_, String>("SELECT nom FROM activites WHERE id = ?")
                            .bind(autre_id)
                            .fetch_one(pool)
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

pub async fn planning_personne_semaine(
    pool: &SqlitePool,
    personne_id: i64,
    date_lundi: &str,
    annee_scolaire: &str,
) -> Result<Vec<PlanningCreneau>, sqlx::Error> {
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
    .fetch_all(pool)
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

#[cfg(test)]
mod tests {
    use super::*;
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

        let c = creer_creneau(
            &pool,
            CreateCreneau {
                activite_id,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
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

        creer_creneau(
            &pool,
            CreateCreneau {
                activite_id,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        creer_creneau(
            &pool,
            CreateCreneau {
                activite_id,
                jour_semaine: 3,
                heure_debut: "10:00".to_string(),
                heure_fin: "12:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        let list = lister_creneaux(&pool, activite_id, "2025-2026")
            .await
            .unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].jour_semaine, 1); // trié par jour
        assert_eq!(list[1].jour_semaine, 3);
    }

    #[tokio::test]
    async fn test_lister_creneaux_autre_annee() {
        let pool = setup_db().await;
        let activite_id = seed_activite(&pool, "Poterie").await;

        creer_creneau(
            &pool,
            CreateCreneau {
                activite_id,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2024-2025".to_string(),
            },
        )
        .await
        .unwrap();

        let list = lister_creneaux(&pool, activite_id, "2025-2026")
            .await
            .unwrap();
        assert_eq!(list.len(), 0);
    }

    #[tokio::test]
    async fn test_supprimer_creneau() {
        let pool = setup_db().await;
        let activite_id = seed_activite(&pool, "Poterie").await;

        let c = creer_creneau(
            &pool,
            CreateCreneau {
                activite_id,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        supprimer_creneau(&pool, c.id).await.unwrap();

        let list = lister_creneaux(&pool, activite_id, "2025-2026")
            .await
            .unwrap();
        assert_eq!(list.len(), 0);
    }

    #[tokio::test]
    async fn test_modifier_creneau() {
        let pool = setup_db().await;
        let activite_id = seed_activite(&pool, "Poterie").await;

        let c = creer_creneau(
            &pool,
            CreateCreneau {
                activite_id,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        let updated = modifier_creneau(
            &pool,
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

        let sb = ajouter_semaine_banalisee(
            &pool,
            CreateSemaineBanalisee {
                activite_id,
                date_debut: "2025-12-22".to_string(),
                motif: Some("Vacances de Noël".to_string()),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        assert_eq!(sb.date_debut, "2025-12-22");
        assert_eq!(sb.motif, Some("Vacances de Noël".to_string()));
    }

    #[tokio::test]
    async fn test_ajouter_semaine_banalisee_sans_motif() {
        let pool = setup_db().await;
        let activite_id = seed_activite(&pool, "Poterie").await;

        let sb = ajouter_semaine_banalisee(
            &pool,
            CreateSemaineBanalisee {
                activite_id,
                date_debut: "2025-12-22".to_string(),
                motif: None,
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        assert_eq!(sb.motif, None);
    }

    #[tokio::test]
    async fn test_lister_semaines_banalisees() {
        let pool = setup_db().await;
        let activite_id = seed_activite(&pool, "Poterie").await;

        ajouter_semaine_banalisee(
            &pool,
            CreateSemaineBanalisee {
                activite_id,
                date_debut: "2025-12-22".to_string(),
                motif: Some("Noël".to_string()),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        ajouter_semaine_banalisee(
            &pool,
            CreateSemaineBanalisee {
                activite_id,
                date_debut: "2026-02-23".to_string(),
                motif: Some("Hiver".to_string()),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        let list = lister_semaines_banalisees(&pool, activite_id)
            .await
            .unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].date_debut, "2025-12-22");
        assert_eq!(list[1].date_debut, "2026-02-23");
    }

    #[tokio::test]
    async fn test_supprimer_semaine_banalisee() {
        let pool = setup_db().await;
        let activite_id = seed_activite(&pool, "Poterie").await;

        let sb = ajouter_semaine_banalisee(
            &pool,
            CreateSemaineBanalisee {
                activite_id,
                date_debut: "2025-12-22".to_string(),
                motif: None,
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        supprimer_semaine_banalisee(&pool, sb.id).await.unwrap();

        let list = lister_semaines_banalisees(&pool, activite_id)
            .await
            .unwrap();
        assert_eq!(list.len(), 0);
    }

    #[tokio::test]
    async fn test_compter_inscrits_activite() {
        let pool = setup_db().await;
        let activite_id = seed_activite(&pool, "Poterie").await;
        let pid = seed_personne(&pool).await;

        let count = compter_inscrits_activite(&pool, activite_id, "2025-2026")
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
        .bind("participant")
        .execute(&pool)
        .await
        .unwrap();

        let count = compter_inscrits_activite(&pool, activite_id, "2025-2026")
            .await
            .unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_verifier_collision_pas_de_conflit() {
        let pool = setup_db().await;
        let a1 = seed_activite(&pool, "Poterie").await;
        let a2 = seed_activite(&pool, "Théâtre").await;
        let pid = seed_personne(&pool).await;

        creer_creneau(
            &pool,
            CreateCreneau {
                activite_id: a1,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        creer_creneau(
            &pool,
            CreateCreneau {
                activite_id: a2,
                jour_semaine: 3,
                heure_debut: "10:00".to_string(),
                heure_fin: "12:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)",
        )
        .bind(a1)
        .bind(pid)
        .bind("2025-2026")
        .bind("participant")
        .execute(&pool)
        .await
        .unwrap();

        let collision = verifier_collision(&pool, pid, a2, "2025-2026")
            .await
            .unwrap();
        assert!(collision.is_none());
    }

    #[tokio::test]
    async fn test_verifier_collision_conflit() {
        let pool = setup_db().await;
        let a1 = seed_activite(&pool, "Poterie").await;
        let a2 = seed_activite(&pool, "Théâtre").await;
        let pid = seed_personne(&pool).await;

        creer_creneau(
            &pool,
            CreateCreneau {
                activite_id: a1,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        creer_creneau(
            &pool,
            CreateCreneau {
                activite_id: a2,
                jour_semaine: 1,
                heure_debut: "15:00".to_string(),
                heure_fin: "17:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)",
        )
        .bind(a1)
        .bind(pid)
        .bind("2025-2026")
        .bind("encadrant")
        .execute(&pool)
        .await
        .unwrap();

        let collision = verifier_collision(&pool, pid, a2, "2025-2026")
            .await
            .unwrap();
        assert!(collision.is_some());
        let c = collision.unwrap();
        assert!(c.activite_conflit.contains("Poterie"));
    }

    #[tokio::test]
    async fn test_verifier_collision_meme_activite_ignoree() {
        let pool = setup_db().await;
        let a1 = seed_activite(&pool, "Poterie").await;
        let pid = seed_personne(&pool).await;

        creer_creneau(
            &pool,
            CreateCreneau {
                activite_id: a1,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)",
        )
        .bind(a1)
        .bind(pid)
        .bind("2025-2026")
        .bind("participant")
        .execute(&pool)
        .await
        .unwrap();

        let collision = verifier_collision(&pool, pid, a1, "2025-2026")
            .await
            .unwrap();
        assert!(collision.is_none());
    }

    #[tokio::test]
    async fn test_planning_personne_semaine() {
        let pool = setup_db().await;
        let a1 = seed_activite(&pool, "Poterie").await;
        let a2 = seed_activite(&pool, "Théâtre").await;
        let pid = seed_personne(&pool).await;

        creer_creneau(
            &pool,
            CreateCreneau {
                activite_id: a1,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        creer_creneau(
            &pool,
            CreateCreneau {
                activite_id: a2,
                jour_semaine: 3,
                heure_debut: "10:00".to_string(),
                heure_fin: "12:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)",
        )
        .bind(a1)
        .bind(pid)
        .bind("2025-2026")
        .bind("participant")
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)",
        )
        .bind(a2)
        .bind(pid)
        .bind("2025-2026")
        .bind("encadrant")
        .execute(&pool)
        .await
        .unwrap();

        let planning = planning_personne_semaine(&pool, pid, "2025-09-01", "2025-2026")
            .await
            .unwrap();
        assert_eq!(planning.len(), 2);
        assert_eq!(planning[0].role, "participant");
        assert_eq!(planning[1].role, "encadrant");
    }

    #[tokio::test]
    async fn test_planning_personne_semaine_banalisee_exclue() {
        let pool = setup_db().await;
        let a1 = seed_activite(&pool, "Poterie").await;
        let pid = seed_personne(&pool).await;

        creer_creneau(
            &pool,
            CreateCreneau {
                activite_id: a1,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        ajouter_semaine_banalisee(
            &pool,
            CreateSemaineBanalisee {
                activite_id: a1,
                date_debut: "2025-12-22".to_string(),
                motif: Some("Noël".to_string()),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)",
        )
        .bind(a1)
        .bind(pid)
        .bind("2025-2026")
        .bind("participant")
        .execute(&pool)
        .await
        .unwrap();

        // Semaine banalisée : planning vide
        let planning = planning_personne_semaine(&pool, pid, "2025-12-22", "2025-2026")
            .await
            .unwrap();
        assert_eq!(planning.len(), 0);

        // Autre semaine : planning normal
        let planning = planning_personne_semaine(&pool, pid, "2025-09-01", "2025-2026")
            .await
            .unwrap();
        assert_eq!(planning.len(), 1);
    }

    #[tokio::test]
    async fn test_planning_personne_semaine_aucune_activite() {
        let pool = setup_db().await;
        let pid = seed_personne(&pool).await;

        let planning = planning_personne_semaine(&pool, pid, "2025-09-01", "2025-2026")
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

        creer_creneau(
            &pool,
            CreateCreneau {
                activite_id: a1,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        creer_creneau(
            &pool,
            CreateCreneau {
                activite_id: a2,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)",
        )
        .bind(a1)
        .bind(pid)
        .bind("2025-2026")
        .bind("participant")
        .execute(&pool)
        .await
        .unwrap();

        let collision = verifier_collision(&pool, pid, a2, "2025-2026")
            .await
            .unwrap();
        assert!(collision.is_some());
    }

    #[tokio::test]
    async fn test_verifier_collision_contenant_contenu() {
        let pool = setup_db().await;
        let a1 = seed_activite(&pool, "Poterie").await;
        let a2 = seed_activite(&pool, "Théâtre").await;
        let pid = seed_personne(&pool).await;

        creer_creneau(
            &pool,
            CreateCreneau {
                activite_id: a1,
                jour_semaine: 1,
                heure_debut: "10:00".to_string(),
                heure_fin: "18:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        creer_creneau(
            &pool,
            CreateCreneau {
                activite_id: a2,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)",
        )
        .bind(a1)
        .bind(pid)
        .bind("2025-2026")
        .bind("encadrant")
        .execute(&pool)
        .await
        .unwrap();

        let collision = verifier_collision(&pool, pid, a2, "2025-2026")
            .await
            .unwrap();
        assert!(collision.is_some());
    }

    #[tokio::test]
    async fn test_verifier_collision_adjacent_no_overlap() {
        let pool = setup_db().await;
        let a1 = seed_activite(&pool, "Poterie").await;
        let a2 = seed_activite(&pool, "Théâtre").await;
        let pid = seed_personne(&pool).await;

        creer_creneau(
            &pool,
            CreateCreneau {
                activite_id: a1,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        creer_creneau(
            &pool,
            CreateCreneau {
                activite_id: a2,
                jour_semaine: 1,
                heure_debut: "16:00".to_string(),
                heure_fin: "18:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)",
        )
        .bind(a1)
        .bind(pid)
        .bind("2025-2026")
        .bind("participant")
        .execute(&pool)
        .await
        .unwrap();

        let collision = verifier_collision(&pool, pid, a2, "2025-2026")
            .await
            .unwrap();
        assert!(collision.is_none());
    }

    #[tokio::test]
    async fn test_verifier_collision_activite_sans_creneaux() {
        let pool = setup_db().await;
        let a1 = seed_activite(&pool, "Poterie").await;
        let a2 = seed_activite(&pool, "Théâtre").await;
        let pid = seed_personne(&pool).await;

        // a1 has no creneaux
        creer_creneau(
            &pool,
            CreateCreneau {
                activite_id: a2,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)",
        )
        .bind(a1)
        .bind(pid)
        .bind("2025-2026")
        .bind("participant")
        .execute(&pool)
        .await
        .unwrap();

        let collision = verifier_collision(&pool, pid, a2, "2025-2026")
            .await
            .unwrap();
        assert!(collision.is_none());
    }

    #[tokio::test]
    async fn test_verifier_collision_personne_sans_activite() {
        let pool = setup_db().await;
        let a1 = seed_activite(&pool, "Poterie").await;
        let pid = seed_personne(&pool).await;

        creer_creneau(
            &pool,
            CreateCreneau {
                activite_id: a1,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        // Personne is not enrolled in any activity
        let collision = verifier_collision(&pool, pid, a1, "2025-2026")
            .await
            .unwrap();
        assert!(collision.is_none());
    }

    #[tokio::test]
    async fn test_compter_inscrits_encadrant_et_participant() {
        let pool = setup_db().await;
        let activite_id = seed_activite(&pool, "Poterie").await;
        let pid1 = seed_personne(&pool).await;
        let pid2 = seed_personne(&pool).await;

        sqlx::query(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)",
        )
        .bind(activite_id)
        .bind(pid1)
        .bind("2025-2026")
        .bind("encadrant")
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)",
        )
        .bind(activite_id)
        .bind(pid2)
        .bind("2025-2026")
        .bind("participant")
        .execute(&pool)
        .await
        .unwrap();

        let count = compter_inscrits_activite(&pool, activite_id, "2025-2026")
            .await
            .unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_compter_inscrits_autre_annee() {
        let pool = setup_db().await;
        let activite_id = seed_activite(&pool, "Poterie").await;
        let pid = seed_personne(&pool).await;

        sqlx::query(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)",
        )
        .bind(activite_id)
        .bind(pid)
        .bind("2024-2025")
        .bind("participant")
        .execute(&pool)
        .await
        .unwrap();

        let count = compter_inscrits_activite(&pool, activite_id, "2025-2026")
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

        creer_creneau(
            &pool,
            CreateCreneau {
                activite_id: a1,
                jour_semaine: 1,
                heure_debut: "16:00".to_string(),
                heure_fin: "18:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        creer_creneau(
            &pool,
            CreateCreneau {
                activite_id: a2,
                jour_semaine: 1,
                heure_debut: "10:00".to_string(),
                heure_fin: "12:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)",
        )
        .bind(a1)
        .bind(pid)
        .bind("2025-2026")
        .bind("participant")
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)",
        )
        .bind(a2)
        .bind(pid)
        .bind("2025-2026")
        .bind("participant")
        .execute(&pool)
        .await
        .unwrap();

        let planning = planning_personne_semaine(&pool, pid, "2025-09-01", "2025-2026")
            .await
            .unwrap();
        assert_eq!(planning.len(), 2);
        // Trie par heure_debut : 10:00 avant 16:00
        assert_eq!(planning[0].creneau.heure_debut, "10:00");
        assert_eq!(planning[1].creneau.heure_debut, "16:00");
    }

    #[tokio::test]
    async fn test_creer_creneau_plusieurs_activites() {
        let pool = setup_db().await;
        let a1 = seed_activite(&pool, "Poterie").await;
        let a2 = seed_activite(&pool, "Théâtre").await;

        let c1 = creer_creneau(
            &pool,
            CreateCreneau {
                activite_id: a1,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        let c2 = creer_creneau(
            &pool,
            CreateCreneau {
                activite_id: a2,
                jour_semaine: 3,
                heure_debut: "10:00".to_string(),
                heure_fin: "12:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
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

        let sb1 = ajouter_semaine_banalisee(
            &pool,
            CreateSemaineBanalisee {
                activite_id: a1,
                date_debut: "2025-12-22".to_string(),
                motif: Some("Noël".to_string()),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        let sb2 = ajouter_semaine_banalisee(
            &pool,
            CreateSemaineBanalisee {
                activite_id: a2,
                date_debut: "2025-12-22".to_string(),
                motif: Some("Noël".to_string()),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        assert_eq!(sb1.date_debut, sb2.date_debut);
        assert_ne!(sb1.id, sb2.id);
    }

    #[tokio::test]
    async fn test_lister_creneaux_tri_par_jour_puis_heure() {
        let pool = setup_db().await;
        let activite_id = seed_activite(&pool, "Poterie").await;

        creer_creneau(
            &pool,
            CreateCreneau {
                activite_id,
                jour_semaine: 3,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        creer_creneau(
            &pool,
            CreateCreneau {
                activite_id,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        // Insert out of order to verify sorting
        let list = lister_creneaux(&pool, activite_id, "2025-2026")
            .await
            .unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].jour_semaine, 1);
        assert_eq!(list[1].jour_semaine, 3);
    }

    #[tokio::test]
    async fn test_modifier_creneau_inexistant() {
        let pool = setup_db().await;

        let result = modifier_creneau(
            &pool,
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

        let result = supprimer_creneau(&pool, 99999).await;
        assert!(result.is_ok()); // DELETE on non-existent row is not an error in SQLite
    }

    #[tokio::test]
    async fn test_planning_personne_activite_sans_creneaux() {
        let pool = setup_db().await;
        let a1 = seed_activite(&pool, "Poterie").await;
        let pid = seed_personne(&pool).await;

        // Personne is enrolled but activity has no creneaux
        sqlx::query(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)",
        )
        .bind(a1)
        .bind(pid)
        .bind("2025-2026")
        .bind("participant")
        .execute(&pool)
        .await
        .unwrap();

        let planning = planning_personne_semaine(&pool, pid, "2025-09-01", "2025-2026")
            .await
            .unwrap();
        assert_eq!(planning.len(), 0);
    }

    #[tokio::test]
    async fn test_semaine_banalisee_meme_activite_deux_dates() {
        let pool = setup_db().await;
        let activite_id = seed_activite(&pool, "Poterie").await;

        let _sb1 = ajouter_semaine_banalisee(
            &pool,
            CreateSemaineBanalisee {
                activite_id,
                date_debut: "2025-12-22".to_string(),
                motif: None,
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        let _sb2 = ajouter_semaine_banalisee(
            &pool,
            CreateSemaineBanalisee {
                activite_id,
                date_debut: "2025-12-29".to_string(),
                motif: None,
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        let list = lister_semaines_banalisees(&pool, activite_id)
            .await
            .unwrap();
        assert_eq!(list.len(), 2);
    }
}
