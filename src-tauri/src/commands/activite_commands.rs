use tauri::State;

use crate::domain::activite::{
    Activite, ActivitePersonne, CreateActivite, CreateLiaisonActivitePersonne, CreateTarifActivite,
    DetailActivite, UpdateActivite,
};
use crate::error::AppError;
use crate::infrastructure::db::AppState;
use crate::services::ActiviteService;

#[tauri::command]
pub async fn creer_activite(
    state: State<'_, AppState>,
    utilisateur: String,
    input: CreateActivite,
) -> Result<Activite, AppError> {
    let utilisateur = crate::infrastructure::audit::verifier_utilisateur(&utilisateur)?;
    let service = ActiviteService::new(&state.activite_repo, &state.planning_repo);
    service.creer(&utilisateur, input).await
}

#[tauri::command]
pub async fn modifier_activite(
    state: State<'_, AppState>,
    id: i64,
    utilisateur: String,
    input: UpdateActivite,
) -> Result<Activite, AppError> {
    let utilisateur = crate::infrastructure::audit::verifier_utilisateur(&utilisateur)?;
    let service = ActiviteService::new(&state.activite_repo, &state.planning_repo);
    service.modifier(&utilisateur, id, input).await
}

#[tauri::command]
pub async fn obtenir_activite(
    state: State<'_, AppState>,
    id: i64,
) -> Result<Option<Activite>, AppError> {
    let service = ActiviteService::new(&state.activite_repo, &state.planning_repo);
    service.obtenir(id).await
}

#[tauri::command]
pub async fn obtenir_detail_activite(
    state: State<'_, AppState>,
    id: i64,
    annee_scolaire: String,
) -> Result<DetailActivite, AppError> {
    let service = ActiviteService::new(&state.activite_repo, &state.planning_repo);
    service.obtenir_detail(id, &annee_scolaire).await
}

#[tauri::command]
pub async fn lister_activites(
    state: State<'_, AppState>,
    annee_scolaire: String,
) -> Result<Vec<(Activite, Option<f64>, i64)>, AppError> {
    let service = ActiviteService::new(&state.activite_repo, &state.planning_repo);
    service.lister_activites(&annee_scolaire).await
}

#[tauri::command]
pub async fn definir_tarif_activite(
    state: State<'_, AppState>,
    utilisateur: String,
    input: CreateTarifActivite,
) -> Result<(), AppError> {
    let utilisateur = crate::infrastructure::audit::verifier_utilisateur(&utilisateur)?;
    let service = ActiviteService::new(&state.activite_repo, &state.planning_repo);
    service.definir_tarif(&utilisateur, input).await
}

#[tauri::command]
pub async fn ajouter_personne_activite(
    state: State<'_, AppState>,
    utilisateur: String,
    input: CreateLiaisonActivitePersonne,
) -> Result<(), AppError> {
    let utilisateur = crate::infrastructure::audit::verifier_utilisateur(&utilisateur)?;
    let service = ActiviteService::new(&state.activite_repo, &state.planning_repo);
    service.ajouter_personne(&utilisateur, input).await
}

#[tauri::command]
pub async fn retirer_personne_activite(
    state: State<'_, AppState>,
    activite_id: i64,
    personne_id: i64,
    annee_scolaire: String,
) -> Result<(), AppError> {
    let service = ActiviteService::new(&state.activite_repo, &state.planning_repo);
    service
        .retirer_personne(activite_id, personne_id, &annee_scolaire)
        .await
}

#[tauri::command]
pub async fn lister_annees_activites(state: State<'_, AppState>) -> Result<Vec<String>, AppError> {
    let service = ActiviteService::new(&state.activite_repo, &state.planning_repo);
    service.lister_annees().await
}

#[tauri::command]
pub async fn lister_activites_personne(
    state: State<'_, AppState>,
    personne_id: i64,
) -> Result<Vec<ActivitePersonne>, AppError> {
    let service = ActiviteService::new(&state.activite_repo, &state.planning_repo);
    service.lister_activites_personne(personne_id).await
}
