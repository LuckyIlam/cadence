use tauri::State;

use crate::domain::parametre::{valider_plage_horaire, ImpactCreneau, ParametresPlanning};
use crate::error::AppError;
use crate::infrastructure::db::AppState;
use crate::services::ParametreService;

#[tauri::command]
pub async fn obtenir_parametres_planning(
    state: State<'_, AppState>,
) -> Result<ParametresPlanning, AppError> {
    let service =
        ParametreService::new(&state.param_repo, &state.planning_repo, state.conn.clone());
    service.obtenir_parametres().await
}

/// Calcule (sans modifier la base) les créneaux impactés par une modification de la plage horaire.
#[tauri::command]
pub async fn apercu_creneaux_hors_plage(
    state: State<'_, AppState>,
    heure_ouverture: String,
    heure_fermeture: String,
) -> Result<Vec<ImpactCreneau>, AppError> {
    let service =
        ParametreService::new(&state.param_repo, &state.planning_repo, state.conn.clone());
    service
        .apercu_impact_plage(&heure_ouverture, &heure_fermeture)
        .await
}

/// Modifie la plage horaire d'ouverture des activités.
///
/// `confirmer_suppression` doit être vrai lorsque l'utilisateur a validé l'avertissement :
/// la réduction peut alors supprimer des créneaux sans inscrits et déplacer les autres.
#[tauri::command]
pub async fn modifier_plage_horaire(
    state: State<'_, AppState>,
    heure_ouverture: String,
    heure_fermeture: String,
    confirmer_suppression: bool,
) -> Result<ParametresPlanning, AppError> {
    valider_plage_horaire(&heure_ouverture, &heure_fermeture).map_err(AppError::Validation)?;
    let service =
        ParametreService::new(&state.param_repo, &state.planning_repo, state.conn.clone());
    service
        .appliquer_plage(&heure_ouverture, &heure_fermeture, confirmer_suppression)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::parametre::ImpactAction;
    use crate::infrastructure::db::init_app_state;
    use crate::repositories::PlanningRepository;
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
                "INSERT INTO activites (nom, description, capacite_max)
                 VALUES (?, ?, ?) RETURNING *",
                libsql::params![nom, None::<String>, None::<i64>],
            )
            .await
            .expect("failed to seed activite");
        let row = rows.next().await.expect("no row").expect("no row");
        libsql::de::from_row::<crate::domain::activite::Activite>(&row)
            .expect("failed to read activite")
            .id
    }

    #[tokio::test]
    async fn test_obtenir_parametres_planning_defaut() {
        let (app, _pool) = setup_app().await;
        let params = obtenir_parametres_planning(app.state::<AppState>())
            .await
            .unwrap();
        assert_eq!(params.heure_ouverture, "08:00");
        assert_eq!(params.heure_fermeture, "20:00");
    }

    #[tokio::test]
    async fn test_modifier_plage_horaire_ok() {
        let (app, _pool) = setup_app().await;
        let params = modifier_plage_horaire(
            app.state::<AppState>(),
            "09:00".to_string(),
            "18:00".to_string(),
            true,
        )
        .await
        .unwrap();
        assert_eq!(params.heure_ouverture, "09:00");
        assert_eq!(params.heure_fermeture, "18:00");
    }

    #[tokio::test]
    async fn test_modifier_plage_horaire_invalide() {
        let (app, _pool) = setup_app().await;
        let err = modifier_plage_horaire(
            app.state::<AppState>(),
            "20:00".to_string(),
            "08:00".to_string(),
            true,
        )
        .await
        .unwrap_err();
        assert!(err.to_string().contains("avant l'heure de fermeture"));
    }

    #[tokio::test]
    async fn test_modifier_plage_horaire_reduction_sans_confirmation_refusee() {
        let (app, pool) = setup_app().await;
        let a = seed_activite(&pool, "Poterie").await;

        app.state::<AppState>()
            .planning_repo
            .creer_creneau(crate::domain::planning::CreateCreneau {
                activite_id: a,
                jour_semaine: 1,
                heure_debut: "07:00".to_string(),
                heure_fin: "09:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            })
            .await
            .unwrap();

        let err = modifier_plage_horaire(
            app.state::<AppState>(),
            "08:00".to_string(),
            "20:00".to_string(),
            false,
        )
        .await
        .expect_err("réduction impactant des créneaux refusée sans confirmation");

        assert!(err.to_string().contains("Confirmez"));
    }

    #[tokio::test]
    async fn test_modifier_plage_horaire_reduction_confirmee_supprime_creneau() {
        let (app, pool) = setup_app().await;
        let a = seed_activite(&pool, "Poterie").await;

        let c = app
            .state::<AppState>()
            .planning_repo
            .creer_creneau(crate::domain::planning::CreateCreneau {
                activite_id: a,
                jour_semaine: 1,
                heure_debut: "07:00".to_string(),
                heure_fin: "09:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            })
            .await
            .unwrap();

        let params = modifier_plage_horaire(
            app.state::<AppState>(),
            "08:00".to_string(),
            "20:00".to_string(),
            true,
        )
        .await
        .expect("réduction confirmée");

        assert_eq!(params.heure_ouverture, "08:00");
        assert_eq!(params.heure_fermeture, "20:00");

        let list = app
            .state::<AppState>()
            .planning_repo
            .lister_creneaux(a, "2025-2026")
            .await
            .unwrap();
        assert!(list.iter().all(|k| k.id != c.id));
    }

    #[tokio::test]
    async fn test_apercu_creneaux_hors_plage_ok() {
        let (app, pool) = setup_app().await;
        let a = seed_activite(&pool, "Poterie").await;

        app.state::<AppState>()
            .planning_repo
            .creer_creneau(crate::domain::planning::CreateCreneau {
                activite_id: a,
                jour_semaine: 1,
                heure_debut: "07:00".to_string(),
                heure_fin: "09:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            })
            .await
            .unwrap();

        let impacts = apercu_creneaux_hors_plage(
            app.state::<AppState>(),
            "08:00".to_string(),
            "20:00".to_string(),
        )
        .await
        .expect("apercu OK");

        assert_eq!(impacts.len(), 1);
        assert_eq!(impacts[0].creneau_id, 1);
        assert_eq!(impacts[0].activite_nom, "Poterie");
        assert!(matches!(impacts[0].action, ImpactAction::Supprime));
    }
}
