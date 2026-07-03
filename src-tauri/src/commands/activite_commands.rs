use tauri::State;

use crate::domain::activite::{
    valider_role, verifier_capacite_max, Activite, CreateActivite, CreateLiaisonActivitePersonne,
    CreateTarifActivite, DetailActivite, UpdateActivite,
};
use crate::infrastructure::db::AppState;
use crate::repositories;

#[tauri::command]
pub async fn creer_activite(
    state: State<'_, AppState>,
    input: CreateActivite,
) -> Result<Activite, String> {
    if input.nom.trim().is_empty() {
        return Err("Le nom de l'activité est requis".to_string());
    }

    let annee_scolaire = input.annee_scolaire.clone();
    let tarif = input.tarif;

    let activite = repositories::activite_repo::create(&state.pool, input)
        .await
        .map_err(|e| e.to_string())?;

    if let Some(ref annee) = annee_scolaire {
        repositories::activite_repo::upsert_tarif(
            &state.pool,
            CreateTarifActivite {
                activite_id: activite.id,
                annee_scolaire: annee.clone(),
                tarif: tarif.unwrap_or(0.0),
            },
        )
        .await
        .map_err(|e| e.to_string())?;
    }

    Ok(activite)
}

#[tauri::command]
pub async fn modifier_activite(
    state: State<'_, AppState>,
    id: i64,
    input: UpdateActivite,
) -> Result<Activite, String> {
    if input.nom.trim().is_empty() {
        return Err("Le nom de l'activité est requis".to_string());
    }
    repositories::activite_repo::update(&state.pool, id, input)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn obtenir_activite(
    state: State<'_, AppState>,
    id: i64,
) -> Result<Option<Activite>, String> {
    repositories::activite_repo::find_by_id(&state.pool, id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn obtenir_detail_activite(
    state: State<'_, AppState>,
    id: i64,
    annee_scolaire: String,
) -> Result<DetailActivite, String> {
    let activite = repositories::activite_repo::find_by_id(&state.pool, id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Activité introuvable".to_string())?;

    let tarif = repositories::activite_repo::get_tarif(&state.pool, id, &annee_scolaire)
        .await
        .map_err(|e| e.to_string())?
        .map(|t| t.tarif);

    let encadrants =
        repositories::activite_repo::lister_encadrants(&state.pool, id, &annee_scolaire)
            .await
            .map_err(|e| e.to_string())?;

    let participants =
        repositories::activite_repo::lister_participants(&state.pool, id, &annee_scolaire)
            .await
            .map_err(|e| e.to_string())?;

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
) -> Result<Vec<(Activite, Option<f64>, i64)>, String> {
    repositories::activite_repo::lister_activites_par_annee(&state.pool, &annee_scolaire)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn definir_tarif_activite(
    state: State<'_, AppState>,
    input: CreateTarifActivite,
) -> Result<(), String> {
    repositories::activite_repo::upsert_tarif(&state.pool, input)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn ajouter_personne_activite(
    state: State<'_, AppState>,
    input: CreateLiaisonActivitePersonne,
) -> Result<(), String> {
    valider_role(&input.role)?;

    let liaison_existante = repositories::activite_repo::trouver_liaison(
        &state.pool,
        input.activite_id,
        input.personne_id,
        &input.annee_scolaire,
    )
    .await
    .map_err(|e| e.to_string())?;

    if let Some(existing) = liaison_existante {
        if existing.role == input.role {
            return Err(
                "Cette personne est déjà inscrite à cette activité avec ce rôle".to_string(),
            );
        }
        return Err(format!(
            "Cette personne est déjà {} pour cette activité, elle ne peut pas être {}",
            existing.role, input.role
        ));
    }

    if input.role == "participant" {
        let activite = repositories::activite_repo::find_by_id(&state.pool, input.activite_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Activité introuvable".to_string())?;

        let nb_participants = repositories::activite_repo::compter_participants(
            &state.pool,
            input.activite_id,
            &input.annee_scolaire,
        )
        .await
        .map_err(|e| e.to_string())?;

        verifier_capacite_max(nb_participants, activite.capacite_max)?;
    }

    repositories::activite_repo::ajouter_personne(&state.pool, input)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn retirer_personne_activite(
    state: State<'_, AppState>,
    activite_id: i64,
    personne_id: i64,
    annee_scolaire: String,
) -> Result<(), String> {
    repositories::activite_repo::retirer_personne(
        &state.pool,
        activite_id,
        personne_id,
        &annee_scolaire,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn lister_annees_activites(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    repositories::activite_repo::lister_annees_disponibles(&state.pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn lister_activites_personne(
    state: State<'_, AppState>,
    personne_id: i64,
) -> Result<Vec<crate::domain::activite::ActivitePersonne>, String> {
    repositories::activite_repo::lister_activites_personne(&state.pool, personne_id)
        .await
        .map_err(|e| e.to_string())
}
