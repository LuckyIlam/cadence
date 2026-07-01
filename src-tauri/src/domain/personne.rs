use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

use super::adhesion::Adhesion;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Personne {
    pub id: i64,
    pub nom: String,
    pub prenom: String,
    pub date_naissance: NaiveDate,
    pub email: Option<String>,
    pub telephone: Option<String>,
    pub responsable_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePersonne {
    pub nom: String,
    pub prenom: String,
    pub date_naissance: NaiveDate,
    pub email: Option<String>,
    pub telephone: Option<String>,
    pub responsable_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePersonne {
    pub nom: String,
    pub prenom: String,
    pub date_naissance: NaiveDate,
    pub email: Option<String>,
    pub telephone: Option<String>,
    pub responsable_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonneDetail {
    pub personne: Personne,
    pub adhesions: Vec<Adhesion>,
    pub a_adhesion_annee_cours: bool,
}

/// Retourne l'année scolaire courante au format "YYYY-YYYY".
/// Septembre (mois >= 9) démarre une nouvelle année scolaire.
pub fn current_annee_scolaire() -> String {
    annee_scolaire_from_date(chrono::Local::now().naive_local().date())
}

/// Calcule l'année scolaire à partir d'une date donnée.
/// Septembre (mois >= 9) démarre une nouvelle année scolaire.
pub fn annee_scolaire_from_date(date: NaiveDate) -> String {
    let y = date.year();
    if date.month() >= 9 {
        format!("{}-{}", y, y + 1)
    } else {
        format!("{}-{}", y - 1, y)
    }
}

pub fn age_from_date_naissance(date_naissance: NaiveDate) -> i32 {
    let today = chrono::Local::now().naive_local().date();
    let mut age = today.year() - date_naissance.year();
    if today.ordinal() < date_naissance.ordinal() {
        age -= 1;
    }
    age
}

pub fn est_mineur(date_naissance: NaiveDate) -> bool {
    age_from_date_naissance(date_naissance) < 18
}

pub fn valider_date_naissance(date_naissance: NaiveDate) -> Result<(), String> {
    let min_date = chrono::NaiveDate::from_ymd_opt(1920, 1, 1).ok_or("Date minimale invalide")?;
    let aujourd_hui = chrono::Local::now().naive_local().date();

    if date_naissance < min_date {
        return Err("La date de naissance doit être après 1920".to_string());
    }
    if date_naissance > aujourd_hui {
        return Err("La date de naissance ne peut pas être dans le futur".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_annee_scolaire_janvier() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        assert_eq!(annee_scolaire_from_date(date), "2025-2026");
    }

    #[test]
    fn test_annee_scolaire_aout() {
        let date = NaiveDate::from_ymd_opt(2026, 8, 31).unwrap();
        assert_eq!(annee_scolaire_from_date(date), "2025-2026");
    }

    #[test]
    fn test_annee_scolaire_septembre() {
        let date = NaiveDate::from_ymd_opt(2026, 9, 1).unwrap();
        assert_eq!(annee_scolaire_from_date(date), "2026-2027");
    }

    #[test]
    fn test_annee_scolaire_decembre() {
        let date = NaiveDate::from_ymd_opt(2026, 12, 25).unwrap();
        assert_eq!(annee_scolaire_from_date(date), "2026-2027");
    }

    #[test]
    fn test_annee_scolaire_annee_complete() {
        // Juillet 2026 → 2025-2026
        let date = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
        assert_eq!(annee_scolaire_from_date(date), "2025-2026");
    }

    #[test]
    fn test_current_annee_scolaire_returns_valid_format() {
        let result = current_annee_scolaire();
        // Vérifie le format "YYYY-YYYY" avec un écart de 1 an
        assert!(result.len() == 9, "Format should be YYYY-YYYY");
        let parts: Vec<&str> = result.split('-').collect();
        assert_eq!(parts.len(), 2);
        let annee1: i32 = parts[0].parse().unwrap();
        let annee2: i32 = parts[1].parse().unwrap();
        assert_eq!(annee2 - annee1, 1);
    }
}
