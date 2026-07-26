use tauri::State;

use crate::domain::personne::{
    current_annee_scolaire, est_mineur, valider_date_naissance, CreatePersonne,
    CriteresRecherchePersonnes, Pagination, Personne, PersonneDetail, ResultatRecherchePersonnes,
    UpdatePersonne,
};
use crate::error::AppError;
use crate::infrastructure::db::AppState;
use crate::repositories::{AdhesionRepository, PersonneRepository};

#[tauri::command]
pub async fn creer_personne(
    state: State<'_, AppState>,
    input: CreatePersonne,
) -> Result<Personne, AppError> {
    valider_date_naissance(input.date_naissance)?;

    if est_mineur(input.date_naissance) {
        match input.responsable_id {
            None => {
                return Err(AppError::Validation(
                    "Un mineur doit avoir un responsable légal".into(),
                ))
            }
            Some(rid) => {
                let responsable = state
                    .personne_repo
                    .find_by_id(rid)
                    .await?
                    .ok_or(AppError::NotFound("Responsable introuvable".into()))?;
                if est_mineur(responsable.date_naissance) {
                    return Err(AppError::Validation(
                        "Le responsable ne peut pas être mineur".into(),
                    ));
                }
            }
        }
    }

    state.personne_repo.create(input).await
}

#[tauri::command]
pub async fn modifier_personne(
    state: State<'_, AppState>,
    id: i64,
    input: UpdatePersonne,
) -> Result<Personne, AppError> {
    valider_date_naissance(input.date_naissance)?;

    if est_mineur(input.date_naissance) {
        match input.responsable_id {
            None => {
                return Err(AppError::Validation(
                    "Un mineur doit avoir un responsable légal".into(),
                ))
            }
            Some(rid) => {
                let responsable = state
                    .personne_repo
                    .find_by_id(rid)
                    .await?
                    .ok_or(AppError::NotFound("Responsable introuvable".into()))?;
                if est_mineur(responsable.date_naissance) {
                    return Err(AppError::Validation(
                        "Le responsable ne peut pas être mineur".into(),
                    ));
                }
            }
        }
    }

    state.personne_repo.update(id, input).await
}

#[tauri::command]
pub async fn obtenir_personne(
    state: State<'_, AppState>,
    id: i64,
) -> Result<Option<Personne>, AppError> {
    state.personne_repo.find_by_id(id).await
}

#[tauri::command]
pub async fn obtenir_detail_personne(
    state: State<'_, AppState>,
    id: i64,
) -> Result<PersonneDetail, AppError> {
    let personne = state
        .personne_repo
        .find_by_id(id)
        .await?
        .ok_or(AppError::NotFound("Personne introuvable".into()))?;

    let adhesions = state.adhesion_repo.list_by_personne(id).await?;

    let annee_scolaire = current_annee_scolaire();
    let a_adhesion_annee_cours = adhesions.iter().any(|a| a.annee_scolaire == annee_scolaire);

    Ok(PersonneDetail {
        personne,
        adhesions,
        a_adhesion_annee_cours,
    })
}

#[tauri::command]
pub async fn rechercher_personnes(
    state: State<'_, AppState>,
    criteres: CriteresRecherchePersonnes,
    pagination: Pagination,
) -> Result<ResultatRecherchePersonnes, AppError> {
    state.personne_repo.rechercher(criteres, pagination).await
}
