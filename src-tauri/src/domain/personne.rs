use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

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
