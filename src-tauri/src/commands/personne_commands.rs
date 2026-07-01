use tauri::State;

use crate::domain::personne::{
    current_annee_scolaire, est_mineur, valider_date_naissance, CreatePersonne, Personne,
    PersonneDetail, UpdatePersonne,
};
use crate::infrastructure::db::AppState;
use crate::repositories;

#[tauri::command]
pub async fn creer_personne(
    state: State<'_, AppState>,
    input: CreatePersonne,
) -> Result<Personne, String> {
    valider_date_naissance(input.date_naissance)?;

    if est_mineur(input.date_naissance) {
        match input.responsable_id {
            None => return Err("Un mineur doit avoir un responsable légal".to_string()),
            Some(rid) => {
                let responsable = repositories::personne_repo::find_by_id(&state.pool, rid)
                    .await
                    .map_err(|e| e.to_string())?
                    .ok_or("Responsable introuvable")?;
                if est_mineur(responsable.date_naissance) {
                    return Err("Le responsable ne peut pas être mineur".to_string());
                }
            }
        }
    }

    repositories::personne_repo::create(&state.pool, input)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn modifier_personne(
    state: State<'_, AppState>,
    id: i64,
    input: UpdatePersonne,
) -> Result<Personne, String> {
    valider_date_naissance(input.date_naissance)?;

    if est_mineur(input.date_naissance) {
        match input.responsable_id {
            None => return Err("Un mineur doit avoir un responsable légal".to_string()),
            Some(rid) => {
                let responsable = repositories::personne_repo::find_by_id(&state.pool, rid)
                    .await
                    .map_err(|e| e.to_string())?
                    .ok_or("Responsable introuvable")?;
                if est_mineur(responsable.date_naissance) {
                    return Err("Le responsable ne peut pas être mineur".to_string());
                }
            }
        }
    }

    repositories::personne_repo::update(&state.pool, id, input)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn obtenir_personne(
    state: State<'_, AppState>,
    id: i64,
) -> Result<Option<Personne>, String> {
    repositories::personne_repo::find_by_id(&state.pool, id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn obtenir_detail_personne(
    state: State<'_, AppState>,
    id: i64,
) -> Result<PersonneDetail, String> {
    let personne = repositories::personne_repo::find_by_id(&state.pool, id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Personne introuvable".to_string())?;

    let adhesions = repositories::adhesion_repo::list_by_personne(&state.pool, id)
        .await
        .map_err(|e| e.to_string())?;

    let annee_scolaire = current_annee_scolaire();
    let a_adhesion_annee_cours = adhesions.iter().any(|a| a.annee_scolaire == annee_scolaire);

    Ok(PersonneDetail {
        personne,
        adhesions,
        a_adhesion_annee_cours,
    })
}

#[tauri::command]
pub async fn lister_personnes(state: State<'_, AppState>) -> Result<Vec<Personne>, String> {
    repositories::personne_repo::list_all(&state.pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rechercher_personnes(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<Personne>, String> {
    repositories::personne_repo::search(&state.pool, &query)
        .await
        .map_err(|e| e.to_string())
}
