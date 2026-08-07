use tauri::State;

use crate::domain::personne::{
    CreatePersonne, CriteresRecherchePersonnes, Pagination, Personne, PersonneDetail,
    ResultatRecherchePersonnes, UpdatePersonne,
};
use crate::error::AppError;
use crate::infrastructure::db::AppState;
use crate::services::PersonneService;

#[tauri::command]
pub async fn creer_personne(
    state: State<'_, AppState>,
    utilisateur: String,
    input: CreatePersonne,
) -> Result<Personne, AppError> {
    let utilisateur = crate::infrastructure::audit::verifier_utilisateur(&utilisateur)?;
    let service = PersonneService::new(&state.personne_repo, &state.adhesion_repo);
    service.creer(&utilisateur, input).await
}

#[tauri::command]
pub async fn modifier_personne(
    state: State<'_, AppState>,
    id: i64,
    utilisateur: String,
    input: UpdatePersonne,
) -> Result<Personne, AppError> {
    let utilisateur = crate::infrastructure::audit::verifier_utilisateur(&utilisateur)?;
    let service = PersonneService::new(&state.personne_repo, &state.adhesion_repo);
    service.modifier(&utilisateur, id, input).await
}

#[tauri::command]
pub async fn obtenir_personne(
    state: State<'_, AppState>,
    id: i64,
) -> Result<Option<Personne>, AppError> {
    let service = PersonneService::new(&state.personne_repo, &state.adhesion_repo);
    service.obtenir(id).await
}

#[tauri::command]
pub async fn obtenir_detail_personne(
    state: State<'_, AppState>,
    id: i64,
) -> Result<PersonneDetail, AppError> {
    let service = PersonneService::new(&state.personne_repo, &state.adhesion_repo);
    service.obtenir_detail(id).await
}

#[tauri::command]
pub async fn rechercher_personnes(
    state: State<'_, AppState>,
    criteres: CriteresRecherchePersonnes,
    pagination: Pagination,
) -> Result<ResultatRecherchePersonnes, AppError> {
    let service = PersonneService::new(&state.personne_repo, &state.adhesion_repo);
    service.rechercher(criteres, pagination).await
}
