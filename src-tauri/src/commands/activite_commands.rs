use tauri::State;

use crate::domain::activite::{
    valider_role, verifier_capacite_max, Activite, CreateActivite, CreateLiaisonActivitePersonne,
    CreateTarifActivite, DetailActivite, UpdateActivite,
};
use crate::error::AppError;
use crate::infrastructure::db::AppState;
use crate::repositories;

#[tauri::command]
pub async fn creer_activite(
    state: State<'_, AppState>,
    input: CreateActivite,
) -> Result<Activite, AppError> {
    if input.nom.trim().is_empty() {
        return Err(AppError::Validation(
            "Le nom de l'activité est requis".into(),
        ));
    }

    let annee_scolaire = input.annee_scolaire.clone();
    let tarif = input.tarif;

    let activite = repositories::activite_repo::create(&state.pool, input).await?;

    if let Some(ref annee) = annee_scolaire {
        repositories::activite_repo::upsert_tarif(
            &state.pool,
            CreateTarifActivite {
                activite_id: activite.id,
                annee_scolaire: annee.clone(),
                tarif: tarif.unwrap_or(0.0),
            },
        )
        .await?;
    }

    Ok(activite)
}

#[tauri::command]
pub async fn modifier_activite(
    state: State<'_, AppState>,
    id: i64,
    input: UpdateActivite,
) -> Result<Activite, AppError> {
    if input.nom.trim().is_empty() {
        return Err(AppError::Validation(
            "Le nom de l'activité est requis".into(),
        ));
    }
    repositories::activite_repo::update(&state.pool, id, input).await
}

#[tauri::command]
pub async fn obtenir_activite(
    state: State<'_, AppState>,
    id: i64,
) -> Result<Option<Activite>, AppError> {
    repositories::activite_repo::find_by_id(&state.pool, id).await
}

#[tauri::command]
pub async fn obtenir_detail_activite(
    state: State<'_, AppState>,
    id: i64,
    annee_scolaire: String,
) -> Result<DetailActivite, AppError> {
    let activite = repositories::activite_repo::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound("Activité introuvable".into()))?;

    let tarif = repositories::activite_repo::get_tarif(&state.pool, id, &annee_scolaire)
        .await?
        .map(|t| t.tarif);

    let encadrants =
        repositories::activite_repo::lister_encadrants(&state.pool, id, &annee_scolaire).await?;

    let participants =
        repositories::activite_repo::lister_participants(&state.pool, id, &annee_scolaire).await?;

    Ok(DetailActivite {
        activite,
        tarif,
        encadrants,
        participants,
    })
}

#[tauri::command]
pub async fn lister_activites(
    state: State<'_, AppState>,
    annee_scolaire: String,
) -> Result<Vec<(Activite, Option<f64>, i64)>, AppError> {
    repositories::activite_repo::lister_activites_par_annee(&state.pool, &annee_scolaire).await
}

#[tauri::command]
pub async fn definir_tarif_activite(
    state: State<'_, AppState>,
    input: CreateTarifActivite,
) -> Result<(), AppError> {
    repositories::activite_repo::upsert_tarif(&state.pool, input).await?;
    Ok(())
}

#[tauri::command]
pub async fn ajouter_personne_activite(
    state: State<'_, AppState>,
    input: CreateLiaisonActivitePersonne,
) -> Result<(), AppError> {
    valider_role(&input.role)?;

    let liaison_existante = repositories::activite_repo::trouver_liaison(
        &state.pool,
        input.activite_id,
        input.personne_id,
        &input.annee_scolaire,
    )
    .await?;

    if let Some(existing) = liaison_existante {
        if existing.role == input.role {
            return Err(AppError::Conflict(
                "Cette personne est déjà inscrite à cette activité avec ce rôle".into(),
            ));
        }
        return Err(AppError::Conflict(format!(
            "Cette personne est déjà {} pour cette activité, elle ne peut pas être {}",
            existing.role, input.role
        )));
    }

    if input.role == "participant" {
        let activite = repositories::activite_repo::find_by_id(&state.pool, input.activite_id)
            .await?
            .ok_or(AppError::NotFound("Activité introuvable".into()))?;

        let nb_participants = repositories::activite_repo::compter_participants(
            &state.pool,
            input.activite_id,
            &input.annee_scolaire,
        )
        .await?;

        verifier_capacite_max(nb_participants, activite.capacite_max)?;
    }

    if let Some(collision) = repositories::planning_repo::verifier_collision(
        &state.pool,
        input.personne_id,
        input.activite_id,
        &input.annee_scolaire,
    )
    .await?
    {
        return Err(AppError::Conflict(format!(
            "Conflit d'horaire avec l'activité '{}' : jour {} ({}), {}–{}.",
            collision.activite_conflit,
            collision.jour_semaine,
            crate::domain::planning::jour_semaine_texte(collision.jour_semaine),
            collision.heure_debut,
            collision.heure_fin,
        )));
    }

    repositories::activite_repo::ajouter_personne(&state.pool, input).await?;

    Ok(())
}

#[tauri::command]
pub async fn retirer_personne_activite(
    state: State<'_, AppState>,
    activite_id: i64,
    personne_id: i64,
    annee_scolaire: String,
) -> Result<(), AppError> {
    repositories::activite_repo::retirer_personne(
        &state.pool,
        activite_id,
        personne_id,
        &annee_scolaire,
    )
    .await
}

#[tauri::command]
pub async fn lister_annees_activites(state: State<'_, AppState>) -> Result<Vec<String>, AppError> {
    repositories::activite_repo::lister_annees_disponibles(&state.pool).await
}

#[tauri::command]
pub async fn lister_activites_personne(
    state: State<'_, AppState>,
    personne_id: i64,
) -> Result<Vec<crate::domain::activite::ActivitePersonne>, AppError> {
    repositories::activite_repo::lister_activites_personne(&state.pool, personne_id).await
}
