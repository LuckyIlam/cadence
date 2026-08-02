use tauri::State;

use crate::domain::parametre::{valider_plage_horaire, ParametresPlanning};
use crate::error::AppError;
use crate::infrastructure::db::AppState;
use crate::repositories::ParametreRepository;

#[tauri::command]
pub async fn obtenir_parametres_planning(
    state: State<'_, AppState>,
) -> Result<ParametresPlanning, AppError> {
    state.param_repo.obtenir_parametres_planning().await
}

#[tauri::command]
pub async fn modifier_plage_horaire(
    state: State<'_, AppState>,
    heure_ouverture: String,
    heure_fermeture: String,
) -> Result<ParametresPlanning, AppError> {
    valider_plage_horaire(&heure_ouverture, &heure_fermeture).map_err(AppError::Validation)?;
    state
        .param_repo
        .mettre_a_jour_plage_horaire(&heure_ouverture, &heure_fermeture)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::db::init_app_state;
    use sqlx::SqlitePool;
    use tauri::Manager;

    async fn setup_app() -> (tauri::App<tauri::test::MockRuntime>, SqlitePool) {
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
        app.manage(init_app_state(pool.clone()));
        (app, pool)
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
        )
        .await
        .unwrap_err();
        assert!(err.to_string().contains("avant l'heure de fermeture"));
    }
}
