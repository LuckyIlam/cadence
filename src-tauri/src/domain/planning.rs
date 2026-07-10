use chrono::Datelike;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CreneauActivite {
    pub id: i64,
    pub activite_id: i64,
    pub jour_semaine: i64,
    pub heure_debut: String,
    pub heure_fin: String,
    pub annee_scolaire: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCreneau {
    pub activite_id: i64,
    pub jour_semaine: i64,
    pub heure_debut: String,
    pub heure_fin: String,
    pub annee_scolaire: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SemaineBanalisee {
    pub id: i64,
    pub activite_id: i64,
    pub date_debut: String,
    pub motif: Option<String>,
    pub annee_scolaire: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSemaineBanalisee {
    pub activite_id: i64,
    pub date_debut: String,
    pub motif: Option<String>,
    pub annee_scolaire: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningCreneau {
    pub creneau: CreneauActivite,
    pub activite: super::activite::Activite,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collision {
    pub activite_conflit: String,
    pub jour_semaine: i64,
    pub heure_debut: String,
    pub heure_fin: String,
}

pub fn valider_jour_semaine(jour: i64) -> Result<(), String> {
    match jour {
        1..=7 => Ok(()),
        _ => Err(format!(
            "Jour de semaine invalide : {}. Doit être entre 1 (lundi) et 7 (dimanche)",
            jour
        )),
    }
}

pub fn valider_heure(heure: &str) -> Result<(), String> {
    let parts: Vec<&str> = heure.split(':').collect();
    if parts.len() != 2 {
        return Err(format!(
            "Format d'heure invalide : '{}'. Attendu HH:MM",
            heure
        ));
    }
    let h: u32 = parts[0]
        .parse()
        .map_err(|_| format!("Heure invalide : '{}'", heure))?;
    let m: u32 = parts[1]
        .parse()
        .map_err(|_| format!("Minutes invalides : '{}'", heure))?;
    if h > 23 || m > 59 {
        return Err(format!(
            "Heure invalide : '{}'. Les heures vont de 00:00 à 23:59",
            heure
        ));
    }
    Ok(())
}

pub fn valider_creneau(input: &CreateCreneau) -> Result<(), String> {
    valider_jour_semaine(input.jour_semaine)?;
    valider_heure(&input.heure_debut)?;
    valider_heure(&input.heure_fin)?;
    if input.heure_debut >= input.heure_fin {
        return Err(format!(
            "L'heure de début ({}) doit être avant l'heure de fin ({})",
            input.heure_debut, input.heure_fin
        ));
    }
    Ok(())
}

pub fn jour_semaine_texte(jour: i64) -> &'static str {
    match jour {
        1 => "Lundi",
        2 => "Mardi",
        3 => "Mercredi",
        4 => "Jeudi",
        5 => "Vendredi",
        6 => "Samedi",
        7 => "Dimanche",
        _ => "Inconnu",
    }
}

pub fn est_lundi(date: &str) -> Result<(), String> {
    let d = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map_err(|e| format!("Format de date invalide : '{}'. {}", date, e))?;
    // Weekday::num_days_from_monday returns 0 for Monday
    if d.weekday().num_days_from_monday() != 0 {
        return Err(format!(
            "La date '{}' n'est pas un lundi. La date de début doit être un lundi.",
            date
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valider_jour_semaine_ok() {
        for j in 1..=7 {
            assert!(valider_jour_semaine(j).is_ok());
        }
    }

    #[test]
    fn test_valider_jour_semaine_invalide() {
        assert!(valider_jour_semaine(0).is_err());
        assert!(valider_jour_semaine(8).is_err());
        assert!(valider_jour_semaine(-1).is_err());
    }

    #[test]
    fn test_valider_heure_ok() {
        assert!(valider_heure("08:00").is_ok());
        assert!(valider_heure("14:30").is_ok());
        assert!(valider_heure("23:59").is_ok());
    }

    #[test]
    fn test_valider_heure_format_invalide() {
        assert!(valider_heure("14").is_err());
        assert!(valider_heure("14:00:00").is_err());
        assert!(valider_heure("").is_err());
    }

    #[test]
    fn test_valider_heure_valeurs_invalides() {
        assert!(valider_heure("24:00").is_err());
        assert!(valider_heure("14:60").is_err());
        assert!(valider_heure("abc:def").is_err());
    }

    #[test]
    fn test_valider_creneau_ok() {
        let input = CreateCreneau {
            activite_id: 1,
            jour_semaine: 1,
            heure_debut: "14:00".to_string(),
            heure_fin: "16:00".to_string(),
            annee_scolaire: "2025-2026".to_string(),
        };
        assert!(valider_creneau(&input).is_ok());
    }

    #[test]
    fn test_valider_creneau_debut_apres_fin() {
        let input = CreateCreneau {
            activite_id: 1,
            jour_semaine: 1,
            heure_debut: "16:00".to_string(),
            heure_fin: "14:00".to_string(),
            annee_scolaire: "2025-2026".to_string(),
        };
        assert!(valider_creneau(&input).is_err());
    }

    #[test]
    fn test_valider_creneau_debut_egal_fin() {
        let input = CreateCreneau {
            activite_id: 1,
            jour_semaine: 1,
            heure_debut: "14:00".to_string(),
            heure_fin: "14:00".to_string(),
            annee_scolaire: "2025-2026".to_string(),
        };
        assert!(valider_creneau(&input).is_err());
    }

    #[test]
    fn test_est_lundi_ok() {
        // 2025-09-01 is a Monday
        assert!(est_lundi("2025-09-01").is_ok());
        // 2025-09-08 is a Monday
        assert!(est_lundi("2025-09-08").is_ok());
    }

    #[test]
    fn test_est_lundi_mardi() {
        // 2025-09-02 is a Tuesday
        assert!(est_lundi("2025-09-02").is_err());
    }

    #[test]
    fn test_est_lundi_format_invalide() {
        assert!(est_lundi("01-09-2025").is_err());
        assert!(est_lundi("2025/09/01").is_err());
        assert!(est_lundi("abc").is_err());
    }

    #[test]
    fn test_jour_semaine_texte_all_days() {
        assert_eq!(jour_semaine_texte(1), "Lundi");
        assert_eq!(jour_semaine_texte(2), "Mardi");
        assert_eq!(jour_semaine_texte(3), "Mercredi");
        assert_eq!(jour_semaine_texte(4), "Jeudi");
        assert_eq!(jour_semaine_texte(5), "Vendredi");
        assert_eq!(jour_semaine_texte(6), "Samedi");
        assert_eq!(jour_semaine_texte(7), "Dimanche");
    }

    #[test]
    fn test_jour_semaine_texte_invalide() {
        assert_eq!(jour_semaine_texte(0), "Inconnu");
        assert_eq!(jour_semaine_texte(8), "Inconnu");
        assert_eq!(jour_semaine_texte(-1), "Inconnu");
    }

    #[test]
    fn test_est_lundi_all_weekdays() {
        // 2025-09-01 = Monday (ok)
        assert!(est_lundi("2025-09-01").is_ok());
        // 2025-09-02 = Tuesday
        assert!(est_lundi("2025-09-02").is_err());
        // 2025-09-03 = Wednesday
        assert!(est_lundi("2025-09-03").is_err());
        // 2025-09-04 = Thursday
        assert!(est_lundi("2025-09-04").is_err());
        // 2025-09-05 = Friday
        assert!(est_lundi("2025-09-05").is_err());
        // 2025-09-06 = Saturday
        assert!(est_lundi("2025-09-06").is_err());
        // 2025-09-07 = Sunday
        assert!(est_lundi("2025-09-07").is_err());
    }

    #[test]
    fn test_est_lundi_message() {
        let err = est_lundi("2025-09-02").unwrap_err();
        assert!(err.contains("n'est pas un lundi"));
    }

    #[test]
    fn test_valider_heure_message_format() {
        let err = valider_heure("14").unwrap_err();
        assert!(err.contains("Attendu HH:MM"));
    }

    #[test]
    fn test_valider_heure_message_hors_limite() {
        let err = valider_heure("24:00").unwrap_err();
        assert!(err.contains("00:00 à 23:59"));
    }

    #[test]
    fn test_valider_jour_semaine_message_invalide() {
        let err = valider_jour_semaine(0).unwrap_err();
        assert!(err.contains("entre 1 (lundi) et 7 (dimanche)"));
    }

    #[test]
    fn test_valider_creneau_message_debut_apres_fin() {
        let input = CreateCreneau {
            activite_id: 1,
            jour_semaine: 1,
            heure_debut: "16:00".to_string(),
            heure_fin: "14:00".to_string(),
            annee_scolaire: "2025-2026".to_string(),
        };
        let err = valider_creneau(&input).unwrap_err();
        assert!(err.contains("doit être avant"));
    }

    #[test]
    fn test_valider_creneau_jour_invalide_propage_erreur() {
        let input = CreateCreneau {
            activite_id: 1,
            jour_semaine: 0,
            heure_debut: "14:00".to_string(),
            heure_fin: "16:00".to_string(),
            annee_scolaire: "2025-2026".to_string(),
        };
        let err = valider_creneau(&input).unwrap_err();
        assert!(err.contains("Jour de semaine invalide"));
    }

    #[test]
    fn test_valider_creneau_heure_invalide_propage_erreur() {
        let input = CreateCreneau {
            activite_id: 1,
            jour_semaine: 1,
            heure_debut: "not_a_time".to_string(),
            heure_fin: "16:00".to_string(),
            annee_scolaire: "2025-2026".to_string(),
        };
        let err = valider_creneau(&input).unwrap_err();
        assert!(err.contains("Format d'heure invalide"));
    }

    #[test]
    fn test_valider_heure_limites() {
        assert!(valider_heure("00:00").is_ok());
        assert!(valider_heure("23:59").is_ok());
        assert!(valider_heure("00:01").is_ok());
    }
}
