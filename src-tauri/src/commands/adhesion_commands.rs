use tauri::State;

use crate::domain::adhesion::{Adhesion, CreateAdhesion, UpdateAdhesion};
use crate::infrastructure::db::AppState;
use crate::repositories;

#[tauri::command]
pub async fn creer_adhesion(
    state: State<'_, AppState>,
    input: CreateAdhesion,
) -> Result<Adhesion, String> {
    repositories::adhesion_repo::create(&state.pool, input)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn modifier_adhesion(
    state: State<'_, AppState>,
    id: i64,
    input: UpdateAdhesion,
) -> Result<Adhesion, String> {
    repositories::adhesion_repo::update(&state.pool, id, input)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn lister_adhesions_personne(
    state: State<'_, AppState>,
    personne_id: i64,
) -> Result<Vec<Adhesion>, String> {
    repositories::adhesion_repo::list_by_personne(&state.pool, personne_id)
        .await
        .map_err(|e| e.to_string())
}
