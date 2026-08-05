use tauri::State;

use crate::domain::parametre::valider_creneau_dans_plage;
use crate::domain::planning::{
    est_lundi, valider_creneau, CreateCreneau, CreateSemaineBanalisee, CreneauActivite,
    PlanningCreneau, SemaineBanalisee,
};
use crate::error::AppError;
use crate::infrastructure::db::AppState;
use crate::repositories::{ActiviteRepository, ParametreRepository, PlanningRepository};

#[tauri::command]
pub async fn ajouter_creneau(
    state: State<'_, AppState>,
    utilisateur: String,
    input: CreateCreneau,
) -> Result<CreneauActivite, AppError> {
    let utilisateur = crate::infrastructure::audit::verifier_utilisateur(&utilisateur)?;
    valider_creneau(&input)?;
    valider_creneau_dans_plage_global(&state, &input).await?;

    let _activite = state
        .activite_repo
        .find_by_id(input.activite_id)
        .await?
        .ok_or(AppError::NotFound("Activité introuvable".into()))?;

    let conflits = state
        .planning_repo
        .verifier_conflit_creneaux(
            input.activite_id,
            &input.annee_scolaire,
            input.jour_semaine,
            &input.heure_debut,
            &input.heure_fin,
            None,
        )
        .await?;

    if !conflits.is_empty() {
        let c = &conflits[0];
        return Err(AppError::Conflict(format!(
            "Conflit d'horaire avec un créneau existant : jour {} ({}), {}–{}.",
            c.jour_semaine,
            crate::domain::planning::jour_semaine_texte(c.jour_semaine),
            c.heure_debut,
            c.heure_fin,
        )));
    }

    state.planning_repo.creer_creneau(input, &utilisateur).await
}

/// Vérifie qu'un créneau est entièrement compris dans la plage horaire d'ouverture globale
/// configurée pour les activités (paramètres de l'application).
async fn valider_creneau_dans_plage_global(
    state: &State<'_, AppState>,
    input: &CreateCreneau,
) -> Result<(), AppError> {
    let params = state.param_repo.obtenir_parametres_planning().await?;
    valider_creneau_dans_plage(input, &params.heure_ouverture, &params.heure_fermeture)
        .map_err(AppError::Validation)
}

#[tauri::command]
pub async fn supprimer_creneau(
    state: State<'_, AppState>,
    id: i64,
    activite_id: i64,
    annee_scolaire: String,
) -> Result<(), AppError> {
    let nb = state
        .planning_repo
        .compter_inscrits_activite(activite_id, &annee_scolaire)
        .await?;

    if nb > 0 {
        return Err(AppError::Validation(
            "Impossible de supprimer un créneau : des personnes sont inscrites à cette activité pour cette année. Retirez d'abord les inscrits.".into()
        ));
    }

    state.planning_repo.supprimer_creneau(id).await
}

#[tauri::command]
pub async fn modifier_creneau(
    state: State<'_, AppState>,
    id: i64,
    utilisateur: String,
    input: CreateCreneau,
    version: i64,
) -> Result<CreneauActivite, AppError> {
    let utilisateur = crate::infrastructure::audit::verifier_utilisateur(&utilisateur)?;
    valider_creneau(&input)?;
    valider_creneau_dans_plage_global(&state, &input).await?;

    let nb = state
        .planning_repo
        .compter_inscrits_activite(input.activite_id, &input.annee_scolaire)
        .await?;

    if nb > 0 {
        return Err(AppError::Validation(
            "Impossible de modifier un créneau : des personnes sont inscrites à cette activité pour cette année. Retirez d'abord les inscrits.".into()
        ));
    }

    let conflits = state
        .planning_repo
        .verifier_conflit_creneaux(
            input.activite_id,
            &input.annee_scolaire,
            input.jour_semaine,
            &input.heure_debut,
            &input.heure_fin,
            Some(id),
        )
        .await?;

    if !conflits.is_empty() {
        let c = &conflits[0];
        return Err(AppError::Conflict(format!(
            "Conflit d'horaire avec un créneau existant : jour {} ({}), {}–{}.",
            c.jour_semaine,
            crate::domain::planning::jour_semaine_texte(c.jour_semaine),
            c.heure_debut,
            c.heure_fin,
        )));
    }

    state
        .planning_repo
        .modifier_creneau(id, input, version, &utilisateur)
        .await
}

#[tauri::command]
pub async fn lister_creneaux(
    state: State<'_, AppState>,
    activite_id: i64,
    annee_scolaire: String,
) -> Result<Vec<CreneauActivite>, AppError> {
    state
        .planning_repo
        .lister_creneaux(activite_id, &annee_scolaire)
        .await
}

#[tauri::command]
pub async fn ajouter_semaine_banalisee(
    state: State<'_, AppState>,
    utilisateur: String,
    input: CreateSemaineBanalisee,
) -> Result<SemaineBanalisee, AppError> {
    let utilisateur = crate::infrastructure::audit::verifier_utilisateur(&utilisateur)?;
    est_lundi(&input.date_debut)?;

    state
        .planning_repo
        .ajouter_semaine_banalisee(input, &utilisateur)
        .await
}

#[tauri::command]
pub async fn supprimer_semaine_banalisee(
    state: State<'_, AppState>,
    id: i64,
) -> Result<(), AppError> {
    state.planning_repo.supprimer_semaine_banalisee(id).await
}

#[tauri::command]
pub async fn lister_semaines_banalisees(
    state: State<'_, AppState>,
    activite_id: i64,
) -> Result<Vec<SemaineBanalisee>, AppError> {
    state
        .planning_repo
        .lister_semaines_banalisees(activite_id)
        .await
}

#[tauri::command]
pub async fn planning_personne(
    state: State<'_, AppState>,
    personne_id: i64,
    date_lundi: String,
    annee_scolaire: String,
) -> Result<Vec<PlanningCreneau>, AppError> {
    state
        .planning_repo
        .planning_personne_semaine(personne_id, &date_lundi, &annee_scolaire)
        .await
}

#[tauri::command]
pub async fn verifier_collision(
    state: State<'_, AppState>,
    personne_id: i64,
    activite_id: i64,
    annee_scolaire: String,
) -> Result<Option<crate::domain::planning::Collision>, AppError> {
    state
        .planning_repo
        .verifier_collision(personne_id, activite_id, &annee_scolaire)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::activite::Role;
    use crate::domain::planning::{CreateCreneau, CreateSemaineBanalisee};
    use crate::infrastructure::db::{init_app_state, AppState};
    use libsql::Connection;
    use tauri::Manager;

    async fn setup_app() -> (tauri::App<tauri::test::MockRuntime>, Connection) {
        let app = tauri::test::mock_app();
        let conn = libsql::Builder::new_local(":memory:")
            .build()
            .await
            .expect("failed to create test db")
            .connect()
            .expect("failed to connect test db");
        crate::infrastructure::migrations::cadence_migrations(&conn)
            .await
            .expect("failed to run migrations");
        app.manage(init_app_state(conn.clone()));
        (app, conn)
    }

    async fn seed_activite(conn: &Connection, nom: &str) -> i64 {
        let mut rows = conn
            .query(
                "INSERT INTO activites (nom, description, capacite_max) VALUES (?, ?, ?) RETURNING id",
                libsql::params![nom, None::<String>, None::<i64>],
            )
            .await
            .expect("failed to seed activite");
        let row = rows.next().await.expect("no row").expect("no row");
        libsql::de::from_row::<crate::infrastructure::db::IdRow>(&row)
            .expect("failed to read activite")
            .id
    }

    async fn seed_personne(conn: &Connection) -> i64 {
        let mut rows = conn
            .query(
                "INSERT INTO personnes_physiques (nom, prenom, date_naissance) VALUES (?, ?, ?) RETURNING id",
                libsql::params!["Test", "User", "2000-01-15"],
            )
            .await
            .expect("failed to seed personne");
        let row = rows.next().await.expect("no row").expect("no row");
        libsql::de::from_row::<crate::infrastructure::db::IdRow>(&row)
            .expect("failed to read personne")
            .id
    }

    async fn seed_inscrit(conn: &Connection, activite_id: i64, personne_id: i64, annee: &str) {
        conn.execute(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)",
            libsql::params![
                activite_id,
                personne_id,
                annee,
                Role::Participant.to_string()
            ],
        )
        .await
        .expect("failed to seed inscrit");
    }

    // ── ajouter_creneau ──

    #[tokio::test]
    async fn test_ajouter_creneau_ok() {
        let (app, pool) = setup_app().await;
        let a = seed_activite(&pool, "Poterie").await;

        let result = ajouter_creneau(
            app.state::<AppState>(),
            "alice".to_string(),
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
    async fn test_ajouter_creneau_avant_ouverture_refuse() {
        let (app, pool) = setup_app().await;
        let a = seed_activite(&pool, "Poterie").await;

        let err = ajouter_creneau(
            app.state::<AppState>(),
            "alice".to_string(),
            CreateCreneau {
                activite_id: a,
                jour_semaine: 1,
                heure_debut: "07:00".to_string(),
                heure_fin: "09:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .expect_err("créneau avant l'ouverture refusé");

        assert!(err.to_string().contains("avant l'ouverture"));
    }

    #[tokio::test]
    async fn test_ajouter_creneau_apres_fermeture_refuse() {
        let (app, pool) = setup_app().await;
        let a = seed_activite(&pool, "Poterie").await;

        let err = ajouter_creneau(
            app.state::<AppState>(),
            "alice".to_string(),
            CreateCreneau {
                activite_id: a,
                jour_semaine: 1,
                heure_debut: "18:00".to_string(),
                heure_fin: "21:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .expect_err("créneau après la fermeture refusé");

        assert!(err.to_string().contains("après la fermeture"));
    }

    #[tokio::test]
    async fn test_ajouter_creneau_aux_bornes_accepte() {
        let (app, pool) = setup_app().await;
        let a = seed_activite(&pool, "Poterie").await;

        let c = ajouter_creneau(
            app.state::<AppState>(),
            "alice".to_string(),
            CreateCreneau {
                activite_id: a,
                jour_semaine: 1,
                heure_debut: "08:00".to_string(),
                heure_fin: "20:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .expect("créneau aux bornes de la plage accepté");

        assert_eq!(c.heure_debut, "08:00");
        assert_eq!(c.heure_fin, "20:00");
    }

    #[tokio::test]
    async fn test_ajouter_creneau_activite_inexistante() {
        let (app, _pool) = setup_app().await;

        let result = ajouter_creneau(
            app.state::<AppState>(),
            "alice".to_string(),
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
        assert_eq!(err.to_string(), "Activité introuvable");
    }

    #[tokio::test]
    async fn test_ajouter_creneau_validation_heure() {
        let (app, _pool) = setup_app().await;

        let result = ajouter_creneau(
            app.state::<AppState>(),
            "alice".to_string(),
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
    async fn test_ajouter_creneau_doublon_exact() {
        let (app, pool) = setup_app().await;
        let a = seed_activite(&pool, "Poterie").await;

        ajouter_creneau(
            app.state::<AppState>(),
            "alice".to_string(),
            CreateCreneau {
                activite_id: a,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .expect("premier ajout OK");

        let err = ajouter_creneau(
            app.state::<AppState>(),
            "alice".to_string(),
            CreateCreneau {
                activite_id: a,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .expect_err("doublon refusé");

        assert!(err.to_string().contains("Conflit d'horaire"));
    }

    #[tokio::test]
    async fn test_ajouter_creneau_chevauchement() {
        let (app, pool) = setup_app().await;
        let a = seed_activite(&pool, "Poterie").await;

        ajouter_creneau(
            app.state::<AppState>(),
            "alice".to_string(),
            CreateCreneau {
                activite_id: a,
                jour_semaine: 1,
                heure_debut: "10:00".to_string(),
                heure_fin: "12:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .expect("premier ajout OK");

        let err = ajouter_creneau(
            app.state::<AppState>(),
            "alice".to_string(),
            CreateCreneau {
                activite_id: a,
                jour_semaine: 1,
                heure_debut: "11:00".to_string(),
                heure_fin: "13:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .expect_err("chevauchement refusé");

        assert!(err.to_string().contains("Conflit d'horaire"));
    }

    #[tokio::test]
    async fn test_ajouter_creneau_adjacent_accepte() {
        let (app, pool) = setup_app().await;
        let a = seed_activite(&pool, "Poterie").await;

        ajouter_creneau(
            app.state::<AppState>(),
            "alice".to_string(),
            CreateCreneau {
                activite_id: a,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .expect("premier créneau OK");

        let c = ajouter_creneau(
            app.state::<AppState>(),
            "alice".to_string(),
            CreateCreneau {
                activite_id: a,
                jour_semaine: 1,
                heure_debut: "16:00".to_string(),
                heure_fin: "18:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .expect("créneau adjacent accepté");

        assert_eq!(c.heure_debut, "16:00");
    }

    #[tokio::test]
    async fn test_ajouter_creneau_meme_jour_horaires_differents() {
        let (app, pool) = setup_app().await;
        let a = seed_activite(&pool, "Poterie").await;

        ajouter_creneau(
            app.state::<AppState>(),
            "alice".to_string(),
            CreateCreneau {
                activite_id: a,
                jour_semaine: 1,
                heure_debut: "08:00".to_string(),
                heure_fin: "10:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .expect("premier créneau OK");

        let c = ajouter_creneau(
            app.state::<AppState>(),
            "alice".to_string(),
            CreateCreneau {
                activite_id: a,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .expect("deuxième créneau même jour OK");

        assert_eq!(c.heure_debut, "14:00");
    }

    #[tokio::test]
    async fn test_ajouter_creneau_meme_heure_autre_activite() {
        let (app, pool) = setup_app().await;
        let a1 = seed_activite(&pool, "Poterie").await;
        let a2 = seed_activite(&pool, "Théâtre").await;

        ajouter_creneau(
            app.state::<AppState>(),
            "alice".to_string(),
            CreateCreneau {
                activite_id: a1,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .expect("premier OK");

        let c = ajouter_creneau(
            app.state::<AppState>(),
            "alice".to_string(),
            CreateCreneau {
                activite_id: a2,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .expect("même créneau autre activité OK");

        assert_eq!(c.activite_id, a2);
    }

    #[tokio::test]
    async fn test_ajouter_creneau_validation_jour() {
        let (app, _pool) = setup_app().await;

        let result = ajouter_creneau(
            app.state::<AppState>(),
            "alice".to_string(),
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

        let c = app
            .state::<AppState>()
            .planning_repo
            .creer_creneau(
                CreateCreneau {
                    activite_id: a,
                    jour_semaine: 1,
                    heure_debut: "14:00".to_string(),
                    heure_fin: "16:00".to_string(),
                    annee_scolaire: "2025-2026".to_string(),
                },
                "test",
            )
            .await
            .unwrap();

        let result =
            supprimer_creneau(app.state::<AppState>(), c.id, a, "2025-2026".to_string()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_supprimer_creneau_avec_inscrits() {
        let (app, pool) = setup_app().await;
        let a = seed_activite(&pool, "Poterie").await;
        let p = seed_personne(&pool).await;

        let c = app
            .state::<AppState>()
            .planning_repo
            .creer_creneau(
                CreateCreneau {
                    activite_id: a,
                    jour_semaine: 1,
                    heure_debut: "14:00".to_string(),
                    heure_fin: "16:00".to_string(),
                    annee_scolaire: "2025-2026".to_string(),
                },
                "test",
            )
            .await
            .unwrap();

        seed_inscrit(&pool, a, p, "2025-2026").await;

        let err = supprimer_creneau(app.state::<AppState>(), c.id, a, "2025-2026".to_string())
            .await
            .expect_err("devrait être bloqué");
        assert!(err.to_string().contains("Impossible de supprimer"));
    }

    // ── modifier_creneau ──

    #[tokio::test]
    async fn test_modifier_creneau_ok() {
        let (app, pool) = setup_app().await;
        let a = seed_activite(&pool, "Poterie").await;

        let c = app
            .state::<AppState>()
            .planning_repo
            .creer_creneau(
                CreateCreneau {
                    activite_id: a,
                    jour_semaine: 1,
                    heure_debut: "14:00".to_string(),
                    heure_fin: "16:00".to_string(),
                    annee_scolaire: "2025-2026".to_string(),
                },
                "test",
            )
            .await
            .unwrap();

        let updated = modifier_creneau(
            app.state::<AppState>(),
            c.id,
            "alice".to_string(),
            CreateCreneau {
                activite_id: a,
                jour_semaine: 3,
                heure_debut: "10:00".to_string(),
                heure_fin: "12:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
            c.version,
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

        let c = app
            .state::<AppState>()
            .planning_repo
            .creer_creneau(
                CreateCreneau {
                    activite_id: a,
                    jour_semaine: 1,
                    heure_debut: "14:00".to_string(),
                    heure_fin: "16:00".to_string(),
                    annee_scolaire: "2025-2026".to_string(),
                },
                "test",
            )
            .await
            .unwrap();

        seed_inscrit(&pool, a, p, "2025-2026").await;

        let err = modifier_creneau(
            app.state::<AppState>(),
            c.id,
            "alice".to_string(),
            CreateCreneau {
                activite_id: a,
                jour_semaine: 3,
                heure_debut: "10:00".to_string(),
                heure_fin: "12:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
            c.version,
        )
        .await
        .expect_err("devrait être bloqué");
        assert!(err.to_string().contains("Impossible de modifier"));
    }

    #[tokio::test]
    async fn test_modifier_creneau_conflit() {
        let (app, pool) = setup_app().await;
        let a = seed_activite(&pool, "Poterie").await;

        let c1 = app
            .state::<AppState>()
            .planning_repo
            .creer_creneau(
                CreateCreneau {
                    activite_id: a,
                    jour_semaine: 1,
                    heure_debut: "14:00".to_string(),
                    heure_fin: "16:00".to_string(),
                    annee_scolaire: "2025-2026".to_string(),
                },
                "test",
            )
            .await
            .unwrap();

        let _c2 = app
            .state::<AppState>()
            .planning_repo
            .creer_creneau(
                CreateCreneau {
                    activite_id: a,
                    jour_semaine: 1,
                    heure_debut: "10:00".to_string(),
                    heure_fin: "12:00".to_string(),
                    annee_scolaire: "2025-2026".to_string(),
                },
                "test",
            )
            .await
            .unwrap();

        let err = modifier_creneau(
            app.state::<AppState>(),
            c1.id,
            "alice".to_string(),
            CreateCreneau {
                activite_id: a,
                jour_semaine: 1,
                heure_debut: "11:00".to_string(),
                heure_fin: "13:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
            c1.version,
        )
        .await
        .expect_err("conflit refusé");

        assert!(err.to_string().contains("Conflit d'horaire"));
    }

    #[tokio::test]
    async fn test_modifier_creneau_vers_sans_conflit() {
        let (app, pool) = setup_app().await;
        let a = seed_activite(&pool, "Poterie").await;

        let c1 = app
            .state::<AppState>()
            .planning_repo
            .creer_creneau(
                CreateCreneau {
                    activite_id: a,
                    jour_semaine: 1,
                    heure_debut: "14:00".to_string(),
                    heure_fin: "16:00".to_string(),
                    annee_scolaire: "2025-2026".to_string(),
                },
                "test",
            )
            .await
            .unwrap();

        let _c2 = app
            .state::<AppState>()
            .planning_repo
            .creer_creneau(
                CreateCreneau {
                    activite_id: a,
                    jour_semaine: 3,
                    heure_debut: "10:00".to_string(),
                    heure_fin: "12:00".to_string(),
                    annee_scolaire: "2025-2026".to_string(),
                },
                "test",
            )
            .await
            .unwrap();

        let updated = modifier_creneau(
            app.state::<AppState>(),
            c1.id,
            "alice".to_string(),
            CreateCreneau {
                activite_id: a,
                jour_semaine: 1,
                heure_debut: "16:00".to_string(),
                heure_fin: "18:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
            c1.version,
        )
        .await
        .expect("modification sans conflit OK");

        assert_eq!(updated.heure_debut, "16:00");
        assert_eq!(updated.heure_fin, "18:00");
    }

    // ── lister_creneaux ──

    #[tokio::test]
    async fn test_lister_creneaux_ok() {
        let (app, pool) = setup_app().await;
        let a = seed_activite(&pool, "Poterie").await;

        app.state::<AppState>()
            .planning_repo
            .creer_creneau(
                CreateCreneau {
                    activite_id: a,
                    jour_semaine: 1,
                    heure_debut: "14:00".to_string(),
                    heure_fin: "16:00".to_string(),
                    annee_scolaire: "2025-2026".to_string(),
                },
                "test",
            )
            .await
            .unwrap();

        let list = lister_creneaux(app.state::<AppState>(), a, "2025-2026".to_string())
            .await
            .expect("lister devrait réussir");

        assert_eq!(list.len(), 1);
    }

    #[tokio::test]
    async fn test_lister_creneaux_vide() {
        let (app, _pool) = setup_app().await;

        let list = lister_creneaux(app.state::<AppState>(), 1, "2025-2026".to_string())
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
            app.state::<AppState>(),
            "alice".to_string(),
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
            app.state::<AppState>(),
            "alice".to_string(),
            CreateSemaineBanalisee {
                activite_id: a,
                date_debut: "2025-12-23".to_string(),
                motif: None,
                annee_scolaire: "2025-2026".to_string(),
            },
        )
        .await
        .expect_err("devrait échouer");
        assert!(err.to_string().contains("lundi"));
    }

    // ── supprimer_semaine_banalisee ──

    #[tokio::test]
    async fn test_supprimer_semaine_banalisee_ok() {
        let (app, pool) = setup_app().await;
        let a = seed_activite(&pool, "Poterie").await;

        let sb = app
            .state::<AppState>()
            .planning_repo
            .ajouter_semaine_banalisee(
                CreateSemaineBanalisee {
                    activite_id: a,
                    date_debut: "2025-12-22".to_string(),
                    motif: None,
                    annee_scolaire: "2025-2026".to_string(),
                },
                "test",
            )
            .await
            .unwrap();

        let result = supprimer_semaine_banalisee(app.state::<AppState>(), sb.id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_supprimer_semaine_banalisee_inexistante() {
        let (app, _pool) = setup_app().await;

        let result = supprimer_semaine_banalisee(app.state::<AppState>(), 99999).await;
        assert!(result.is_ok());
    }

    // ── lister_semaines_banalisees ──

    #[tokio::test]
    async fn test_lister_semaines_banalisees_ok() {
        let (app, pool) = setup_app().await;
        let a = seed_activite(&pool, "Poterie").await;

        app.state::<AppState>()
            .planning_repo
            .ajouter_semaine_banalisee(
                CreateSemaineBanalisee {
                    activite_id: a,
                    date_debut: "2025-12-22".to_string(),
                    motif: None,
                    annee_scolaire: "2025-2026".to_string(),
                },
                "test",
            )
            .await
            .unwrap();

        let list = lister_semaines_banalisees(app.state::<AppState>(), a)
            .await
            .expect("lister devrait réussir");

        assert_eq!(list.len(), 1);
    }

    #[tokio::test]
    async fn test_lister_semaines_banalisees_vide() {
        let (app, _pool) = setup_app().await;

        let list = lister_semaines_banalisees(app.state::<AppState>(), 1)
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

        app.state::<AppState>()
            .planning_repo
            .creer_creneau(
                CreateCreneau {
                    activite_id: a,
                    jour_semaine: 1,
                    heure_debut: "14:00".to_string(),
                    heure_fin: "16:00".to_string(),
                    annee_scolaire: "2025-2026".to_string(),
                },
                "test",
            )
            .await
            .unwrap();

        seed_inscrit(&pool, a, p, "2025-2026").await;

        let planning = planning_personne(
            app.state::<AppState>(),
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
            app.state::<AppState>(),
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

        app.state::<AppState>()
            .planning_repo
            .creer_creneau(
                CreateCreneau {
                    activite_id: a1,
                    jour_semaine: 1,
                    heure_debut: "14:00".to_string(),
                    heure_fin: "16:00".to_string(),
                    annee_scolaire: "2025-2026".to_string(),
                },
                "test",
            )
            .await
            .unwrap();

        app.state::<AppState>()
            .planning_repo
            .creer_creneau(
                CreateCreneau {
                    activite_id: a2,
                    jour_semaine: 1,
                    heure_debut: "15:00".to_string(),
                    heure_fin: "17:00".to_string(),
                    annee_scolaire: "2025-2026".to_string(),
                },
                "test",
            )
            .await
            .unwrap();

        seed_inscrit(&pool, a1, p, "2025-2026").await;

        let collision = verifier_collision(app.state::<AppState>(), p, a2, "2025-2026".to_string())
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

        app.state::<AppState>()
            .planning_repo
            .creer_creneau(
                CreateCreneau {
                    activite_id: a1,
                    jour_semaine: 1,
                    heure_debut: "14:00".to_string(),
                    heure_fin: "16:00".to_string(),
                    annee_scolaire: "2025-2026".to_string(),
                },
                "test",
            )
            .await
            .unwrap();

        app.state::<AppState>()
            .planning_repo
            .creer_creneau(
                CreateCreneau {
                    activite_id: a2,
                    jour_semaine: 3,
                    heure_debut: "14:00".to_string(),
                    heure_fin: "16:00".to_string(),
                    annee_scolaire: "2025-2026".to_string(),
                },
                "test",
            )
            .await
            .unwrap();

        seed_inscrit(&pool, a1, p, "2025-2026").await;

        let collision = verifier_collision(app.state::<AppState>(), p, a2, "2025-2026".to_string())
            .await
            .expect("verifier devrait réussir");
        assert!(collision.is_none());
    }

    #[tokio::test]
    async fn test_verifier_collision_meme_activite() {
        let (app, pool) = setup_app().await;
        let a = seed_activite(&pool, "Poterie").await;
        let p = seed_personne(&pool).await;

        app.state::<AppState>()
            .planning_repo
            .creer_creneau(
                CreateCreneau {
                    activite_id: a,
                    jour_semaine: 1,
                    heure_debut: "14:00".to_string(),
                    heure_fin: "16:00".to_string(),
                    annee_scolaire: "2025-2026".to_string(),
                },
                "test",
            )
            .await
            .unwrap();

        seed_inscrit(&pool, a, p, "2025-2026").await;

        let collision = verifier_collision(app.state::<AppState>(), p, a, "2025-2026".to_string())
            .await
            .expect("verifier devrait réussir");
        assert!(collision.is_none());
    }
}
