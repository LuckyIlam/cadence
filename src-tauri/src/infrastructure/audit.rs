use crate::error::AppError;

pub const MESSAGE_CONFLIT: &str = "Fiche modifiée entre-temps, rechargez-la";

pub fn maintenant_utc() -> String {
    chrono::Utc::now().to_rfc3339()
}

pub fn verifier_utilisateur(utilisateur: &str) -> Result<String, AppError> {
    let utilisateur = utilisateur.trim().to_string();
    if utilisateur.is_empty() {
        return Err(AppError::Validation(
            "Le nom d'utilisateur est requis pour cette écriture".to_string(),
        ));
    }
    Ok(utilisateur)
}
