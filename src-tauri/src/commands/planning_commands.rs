use tauri::State;

use crate::domain::planning::{
    est_lundi, valider_creneau, CreateCreneau, CreateSemaineBanalisee, CreneauActivite,
    PlanningCreneau, SemaineBanalisee,
};
use crate::infrastructure::db::AppState;
use crate::repositories;

#[tauri::command]
pub async fn ajouter_creneau(
    state: State<'_, AppState>,
    input: CreateCreneau,
) -> Result<CreneauActivite, String> {
    valider_creneau(&input)?;

    let activite = repositories::activite_repo::find_by_id(&state.pool, input.activite_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Activité introuvable".to_string())?;
    let _ = activite;

    repositories::planning_repo::creer_creneau(&state.pool, input)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn supprimer_creneau(
    state: State<'_, AppState>,
    id: i64,
    activite_id: i64,
    annee_scolaire: String,
) -> Result<(), String> {
    let nb = repositories::planning_repo::compter_inscrits_activite(
        &state.pool,
        activite_id,
        &annee_scolaire,
    )
    .await
    .map_err(|e| e.to_string())?;

    if nb > 0 {
        return Err(
            "Impossible de supprimer un créneau : des personnes sont inscrites à cette activité pour cette année. Retirez d'abord les inscrits.".to_string()
        );
    }

    repositories::planning_repo::supprimer_creneau(&state.pool, id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn modifier_creneau(
    state: State<'_, AppState>,
    id: i64,
    input: CreateCreneau,
) -> Result<CreneauActivite, String> {
    valider_creneau(&input)?;

    let nb = repositories::planning_repo::compter_inscrits_activite(
        &state.pool,
        input.activite_id,
        &input.annee_scolaire,
    )
    .await
    .map_err(|e| e.to_string())?;

    if nb > 0 {
        return Err(
            "Impossible de modifier un créneau : des personnes sont inscrites à cette activité pour cette année. Retirez d'abord les inscrits.".to_string()
        );
    }

    repositories::planning_repo::modifier_creneau(&state.pool, id, input)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn lister_creneaux(
    state: State<'_, AppState>,
    activite_id: i64,
    annee_scolaire: String,
) -> Result<Vec<CreneauActivite>, String> {
    repositories::planning_repo::lister_creneaux(&state.pool, activite_id, &annee_scolaire)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ajouter_semaine_banalisee(
    state: State<'_, AppState>,
    input: CreateSemaineBanalisee,
) -> Result<SemaineBanalisee, String> {
    est_lundi(&input.date_debut)?;

    repositories::planning_repo::ajouter_semaine_banalisee(&state.pool, input)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn supprimer_semaine_banalisee(
    state: State<'_, AppState>,
    id: i64,
) -> Result<(), String> {
    repositories::planning_repo::supprimer_semaine_banalisee(&state.pool, id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn lister_semaines_banalisees(
    state: State<'_, AppState>,
    activite_id: i64,
) -> Result<Vec<SemaineBanalisee>, String> {
    repositories::planning_repo::lister_semaines_banalisees(&state.pool, activite_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn planning_personne(
    state: State<'_, AppState>,
    personne_id: i64,
    date_lundi: String,
    annee_scolaire: String,
) -> Result<Vec<PlanningCreneau>, String> {
    repositories::planning_repo::planning_personne_semaine(
        &state.pool,
        personne_id,
        &date_lundi,
        &annee_scolaire,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn verifier_collision(
    state: State<'_, AppState>,
    personne_id: i64,
    activite_id: i64,
    annee_scolaire: String,
) -> Result<Option<crate::domain::planning::Collision>, String> {
    repositories::planning_repo::verifier_collision(
        &state.pool,
        personne_id,
        activite_id,
        &annee_scolaire,
    )
    .await
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::planning::{CreateCreneau, CreateSemaineBanalisee};
    use crate::repositories::planning_repo::creer_creneau;
    use sqlx::SqlitePool;
    use tauri::test::MockRuntime;
    use tauri::Manager;

    async fn setup_app() -> (tauri::App<MockRuntime>, SqlitePool) {
        let app = tauri::test::mock_app();
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("failed to create test pool");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("failed to run migrations");
        app.manage(AppState { pool: pool.clone() });
        (app, pool)
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

    async fn seed_inscrit(pool: &SqlitePool, activite_id: i64, personne_id: i64, annee: &str) {
        sqlx::query(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)",
        )
        .bind(activite_id)
        .bind(personne_id)
        .bind(annee)
        .bind("participant")
        .execute(pool)
        .await
        .expect("failed to seed inscrit");
    }

    // ── ajouter_creneau ──

    #[tokio::test]
    async fn test_ajouter_creneau_ok() {
        let (app, pool) = setup_app().await;
        let a = seed_activite(&pool, "Poterie").await;

        let result = ajouter_creneau(
            app.state(),
            CreateCreneau {
                activite_id: a,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await;

        let c = result.expect("ajouter_creneau devrait réussir");
        assert_eq!(c.activite_id, a);
        assert_eq!(c.jour_semaine, 1);
        assert_eq!(c.heure_debut, "14:00");
    }

    #[tokio::test]
    async fn test_ajouter_creneau_activite_inexistante() {
        let (app, _pool) = setup_app().await;

        let result = ajouter_creneau(
            app.state(),
            CreateCreneau {
                activite_id: 99999,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await;

        let err = result.expect_err("devrait échouer");
        assert_eq!(err, "Activité introuvable");
    }

    #[tokio::test]
    async fn test_ajouter_creneau_validation_heure() {
        let (app, _pool) = setup_app().await;

        let result = ajouter_creneau(
            app.state(),
            CreateCreneau {
                activite_id: 1,
                jour_semaine: 1,
                heure_debut: "25:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_ajouter_creneau_validation_jour() {
        let (app, _pool) = setup_app().await;

        let result = ajouter_creneau(
            app.state(),
            CreateCreneau {
                activite_id: 1,
                jour_semaine: 0,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await;

        assert!(result.is_err());
    }

    // ── supprimer_creneau ──

    #[tokio::test]
    async fn test_supprimer_creneau_ok() {
        let (app, pool) = setup_app().await;
        let a = seed_activite(&pool, "Poterie").await;

        let c = creer_creneau(
            &pool,
            CreateCreneau {
                activite_id: a,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        let result = supprimer_creneau(app.state(), c.id, a, "2025-2026".to_string()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_supprimer_creneau_avec_inscrits() {
        let (app, pool) = setup_app().await;
        let a = seed_activite(&pool, "Poterie").await;
        let p = seed_personne(&pool).await;

        let c = creer_creneau(
            &pool,
            CreateCreneau {
                activite_id: a,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        seed_inscrit(&pool, a, p, "2025-2026").await;

        let err = supprimer_creneau(app.state(), c.id, a, "2025-2026".to_string())
            .await
            .expect_err("devrait être bloqué");
        assert!(err.contains("Impossible de supprimer"));
    }

    // ── modifier_creneau ──

    #[tokio::test]
    async fn test_modifier_creneau_ok() {
        let (app, pool) = setup_app().await;
        let a = seed_activite(&pool, "Poterie").await;

        let c = creer_creneau(
            &pool,
            CreateCreneau {
                activite_id: a,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        let updated = modifier_creneau(
            app.state(),
            c.id,
            CreateCreneau {
                activite_id: a,
                jour_semaine: 3,
                heure_debut: "10:00".to_string(),
                heure_fin: "12:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .expect("modifier devrait réussir");

        assert_eq!(updated.jour_semaine, 3);
        assert_eq!(updated.heure_debut, "10:00");
    }

    #[tokio::test]
    async fn test_modifier_creneau_avec_inscrits() {
        let (app, pool) = setup_app().await;
        let a = seed_activite(&pool, "Poterie").await;
        let p = seed_personne(&pool).await;

        let c = creer_creneau(
            &pool,
            CreateCreneau {
                activite_id: a,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        seed_inscrit(&pool, a, p, "2025-2026").await;

        let err = modifier_creneau(
            app.state(),
            c.id,
            CreateCreneau {
                activite_id: a,
                jour_semaine: 3,
                heure_debut: "10:00".to_string(),
                heure_fin: "12:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .expect_err("devrait être bloqué");
        assert!(err.contains("Impossible de modifier"));
    }

    // ── lister_creneaux ──

    #[tokio::test]
    async fn test_lister_creneaux_ok() {
        let (app, pool) = setup_app().await;
        let a = seed_activite(&pool, "Poterie").await;

        creer_creneau(
            &pool,
            CreateCreneau {
                activite_id: a,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        let list = lister_creneaux(app.state(), a, "2025-2026".to_string())
            .await
            .expect("lister devrait réussir");

        assert_eq!(list.len(), 1);
    }

    #[tokio::test]
    async fn test_lister_creneaux_vide() {
        let (app, _pool) = setup_app().await;

        let list = lister_creneaux(app.state(), 1, "2025-2026".to_string())
            .await
            .expect("lister devrait réussir");

        assert!(list.is_empty());
    }

    // ── ajouter_semaine_banalisee ──

    #[tokio::test]
    async fn test_ajouter_semaine_banalisee_ok() {
        let (app, pool) = setup_app().await;
        let a = seed_activite(&pool, "Poterie").await;

        let sb = ajouter_semaine_banalisee(
            app.state(),
            CreateSemaineBanalisee {
                activite_id: a,
                date_debut: "2025-12-22".to_string(),
                motif: Some("Noël".to_string()),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .expect("ajouter devrait réussir");

        assert_eq!(sb.activite_id, a);
        assert_eq!(sb.date_debut, "2025-12-22");
        assert_eq!(sb.motif, Some("Noël".to_string()));
    }

    #[tokio::test]
    async fn test_ajouter_semaine_banalisee_pas_lundi() {
        let (app, pool) = setup_app().await;
        let a = seed_activite(&pool, "Poterie").await;

        let err = ajouter_semaine_banalisee(
            app.state(),
            CreateSemaineBanalisee {
                activite_id: a,
                date_debut: "2025-12-23".to_string(), // mardi
                motif: None,
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .expect_err("devrait échouer");
        assert!(err.contains("lundi"));
    }

    // ── supprimer_semaine_banalisee ──

    #[tokio::test]
    async fn test_supprimer_semaine_banalisee_ok() {
        let (app, pool) = setup_app().await;
        let a = seed_activite(&pool, "Poterie").await;

        let sb = crate::repositories::planning_repo::ajouter_semaine_banalisee(
            &pool,
            CreateSemaineBanalisee {
                activite_id: a,
                date_debut: "2025-12-22".to_string(),
                motif: None,
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        let result = supprimer_semaine_banalisee(app.state(), sb.id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_supprimer_semaine_banalisee_inexistante() {
        let (app, _pool) = setup_app().await;

        let result = supprimer_semaine_banalisee(app.state(), 99999).await;
        assert!(result.is_ok()); // DELETE sur inexistant ne plante pas
    }

    // ── lister_semaines_banalisees ──

    #[tokio::test]
    async fn test_lister_semaines_banalisees_ok() {
        let (app, pool) = setup_app().await;
        let a = seed_activite(&pool, "Poterie").await;

        crate::repositories::planning_repo::ajouter_semaine_banalisee(
            &pool,
            CreateSemaineBanalisee {
                activite_id: a,
                date_debut: "2025-12-22".to_string(),
                motif: None,
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        let list = lister_semaines_banalisees(app.state(), a)
            .await
            .expect("lister devrait réussir");

        assert_eq!(list.len(), 1);
    }

    #[tokio::test]
    async fn test_lister_semaines_banalisees_vide() {
        let (app, _pool) = setup_app().await;

        let list = lister_semaines_banalisees(app.state(), 1)
            .await
            .expect("lister devrait réussir");

        assert!(list.is_empty());
    }

    // ── planning_personne ──

    #[tokio::test]
    async fn test_planning_personne_ok() {
        let (app, pool) = setup_app().await;
        let a = seed_activite(&pool, "Poterie").await;
        let p = seed_personne(&pool).await;

        creer_creneau(
            &pool,
            CreateCreneau {
                activite_id: a,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        seed_inscrit(&pool, a, p, "2025-2026").await;

        let planning = planning_personne(
            app.state(),
            p,
            "2025-09-01".to_string(),
            "2025-2026".to_string(),
        )
        .await
        .expect("planning devrait réussir");

        assert_eq!(planning.len(), 1);
    }

    #[tokio::test]
    async fn test_planning_personne_vide() {
        let (app, pool) = setup_app().await;
        let p = seed_personne(&pool).await;

        let planning = planning_personne(
            app.state(),
            p,
            "2025-09-01".to_string(),
            "2025-2026".to_string(),
        )
        .await
        .expect("planning devrait réussir");

        assert!(planning.is_empty());
    }

    // ── verifier_collision ──

    #[tokio::test]
    async fn test_verifier_collision_trouvee() {
        let (app, pool) = setup_app().await;
        let a1 = seed_activite(&pool, "Poterie").await;
        let a2 = seed_activite(&pool, "Théâtre").await;
        let p = seed_personne(&pool).await;

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

        seed_inscrit(&pool, a1, p, "2025-2026").await;

        let collision = verifier_collision(app.state(), p, a2, "2025-2026".to_string())
            .await
            .expect("verifier devrait réussir");
        assert!(collision.is_some());
    }

    #[tokio::test]
    async fn test_verifier_collision_aucune() {
        let (app, pool) = setup_app().await;
        let a1 = seed_activite(&pool, "Poterie").await;
        let a2 = seed_activite(&pool, "Théâtre").await;
        let p = seed_personne(&pool).await;

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
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        seed_inscrit(&pool, a1, p, "2025-2026").await;

        let collision = verifier_collision(app.state(), p, a2, "2025-2026".to_string())
            .await
            .expect("verifier devrait réussir");
        assert!(collision.is_none());
    }

    #[tokio::test]
    async fn test_verifier_collision_meme_activite() {
        let (app, pool) = setup_app().await;
        let a = seed_activite(&pool, "Poterie").await;
        let p = seed_personne(&pool).await;

        creer_creneau(
            &pool,
            CreateCreneau {
                activite_id: a,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .unwrap();

        seed_inscrit(&pool, a, p, "2025-2026").await;

        let collision = verifier_collision(app.state(), p, a, "2025-2026".to_string())
            .await
            .expect("verifier devrait réussir");
        assert!(collision.is_none()); // même activité ignorée
    }
}
