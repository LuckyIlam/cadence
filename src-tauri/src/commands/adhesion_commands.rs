use tauri::State;

use crate::domain::adhesion::{Adhesion, CreateAdhesion, UpdateAdhesion};
use crate::error::AppError;
use crate::infrastructure::db::AppState;
use crate::repositories::AdhesionRepository;

#[tauri::command]
pub async fn creer_adhesion(
    state: State<'_, AppState>,
    utilisateur: String,
    input: CreateAdhesion,
) -> Result<Adhesion, AppError> {
    let utilisateur = crate::infrastructure::audit::verifier_utilisateur(&utilisateur)?;
    state.adhesion_repo.create(input, &utilisateur).await
}

#[tauri::command]
pub async fn modifier_adhesion(
    state: State<'_, AppState>,
    id: i64,
    utilisateur: String,
    input: UpdateAdhesion,
) -> Result<Adhesion, AppError> {
    let utilisateur = crate::infrastructure::audit::verifier_utilisateur(&utilisateur)?;
    state.adhesion_repo.update(id, input, &utilisateur).await
}

#[tauri::command]
pub async fn lister_adhesions_personne(
    state: State<'_, AppState>,
    personne_id: i64,
) -> Result<Vec<Adhesion>, AppError> {
    state.adhesion_repo.list_by_personne(personne_id).await
}
