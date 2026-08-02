use serde::{Deserialize, Serialize};

use super::planning::{valider_heure, CreateCreneau};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ParametresPlanning {
    pub id: i64,
    pub heure_ouverture: String,
    pub heure_fermeture: String,
}

/// Valide une plage horaire d'ouverture : formats HH:MM valides et ouverture avant fermeture.
pub fn valider_plage_horaire(heure_ouverture: &str, heure_fermeture: &str) -> Result<(), String> {
    valider_heure(heure_ouverture)?;
    valider_heure(heure_fermeture)?;
    if heure_ouverture >= heure_fermeture {
        return Err(format!(
            "Plage horaire invalide : l'heure d'ouverture ({}) doit être avant l'heure de fermeture ({}).",
            heure_ouverture, heure_fermeture
        ));
    }
    Ok(())
}

/// Vérifie qu'un créneau est entièrement compris dans la plage d'ouverture de l'activité.
pub fn valider_creneau_dans_plage(
    creneau: &CreateCreneau,
    heure_ouverture: &str,
    heure_fermeture: &str,
) -> Result<(), String> {
    if (creneau.heure_debut.as_str()) < heure_ouverture {
        return Err(format!(
            "L'heure de début ({}) est avant l'ouverture de l'activité ({}). Les créneaux doivent être compris entre {} et {}.",
            creneau.heure_debut, heure_ouverture, heure_ouverture, heure_fermeture
        ));
    }
    if (creneau.heure_fin.as_str()) > heure_fermeture {
        return Err(format!(
            "L'heure de fin ({}) est après la fermeture de l'activité ({}). Les créneaux doivent être compris entre {} et {}.",
            creneau.heure_fin, heure_fermeture, heure_ouverture, heure_fermeture
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn creneau(debut: &str, fin: &str) -> CreateCreneau {
        CreateCreneau {
            activite_id: 1,
            jour_semaine: 1,
            heure_debut: debut.to_string(),
            heure_fin: fin.to_string(),
            annee_scolaire: "2025-2026".to_string(),
        }
    }

    #[test]
    fn test_valider_plage_horaire_ok() {
        assert!(valider_plage_horaire("08:00", "20:00").is_ok());
        assert!(valider_plage_horaire("09:30", "12:00").is_ok());
    }

    #[test]
    fn test_valider_plage_horaire_ouverture_apres_fermeture() {
        let err = valider_plage_horaire("20:00", "08:00").unwrap_err();
        assert!(err.contains("avant l'heure de fermeture"));
    }

    #[test]
    fn test_valider_plage_horaire_egales() {
        assert!(valider_plage_horaire("08:00", "08:00").is_err());
    }

    #[test]
    fn test_valider_plage_horaire_format_invalide() {
        assert!(valider_plage_horaire("8am", "20:00").is_err());
        assert!(valider_plage_horaire("08:00", "25:00").is_err());
    }

    #[test]
    fn test_valider_creneau_dans_plage_ok() {
        assert!(valider_creneau_dans_plage(&creneau("08:00", "20:00"), "08:00", "20:00").is_ok());
        assert!(valider_creneau_dans_plage(&creneau("09:00", "18:00"), "08:00", "20:00").is_ok());
    }

    #[test]
    fn test_valider_creneau_debut_avant_ouverture() {
        let err =
            valider_creneau_dans_plage(&creneau("07:00", "09:00"), "08:00", "20:00").unwrap_err();
        assert!(err.contains("avant l'ouverture"));
    }

    #[test]
    fn test_valider_creneau_fin_apres_fermeture() {
        let err =
            valider_creneau_dans_plage(&creneau("18:00", "21:00"), "08:00", "20:00").unwrap_err();
        assert!(err.contains("après la fermeture"));
    }
}
