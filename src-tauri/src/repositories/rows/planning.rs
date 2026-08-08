use crate::domain::activite::Role;
use crate::error::AppError;
use crate::infrastructure::db::{DeserializeRow, RowView};
use crate::repositories::rows::role_from_row;

#[derive(Debug, Clone)]
pub struct CompteurRow {
    pub count: i64,
}

impl DeserializeRow for CompteurRow {
    fn from_row(row: &dyn RowView) -> Result<Self, AppError> {
        Ok(CompteurRow {
            count: row.get_i64(0)?,
        })
    }
}

pub struct AutreActiviteRow {
    pub activite_id: i64,
}

impl DeserializeRow for AutreActiviteRow {
    fn from_row(row: &dyn RowView) -> Result<Self, AppError> {
        Ok(AutreActiviteRow {
            activite_id: row.get_i64(0)?,
        })
    }
}

pub struct NomActiviteRow {
    pub nom: String,
}

impl DeserializeRow for NomActiviteRow {
    fn from_row(row: &dyn RowView) -> Result<Self, AppError> {
        Ok(NomActiviteRow {
            nom: row.get_str(0)?.to_string(),
        })
    }
}

pub struct IdRow {
    #[allow(dead_code)]
    pub id: i64,
}

impl DeserializeRow for IdRow {
    fn from_row(row: &dyn RowView) -> Result<Self, AppError> {
        Ok(IdRow {
            id: row.get_i64(0)?,
        })
    }
}

pub struct ActiviteCreneauRow {
    pub activite_id: i64,
    pub nom: String,
    pub description: Option<String>,
    pub capacite_max: Option<i64>,
    pub activite_version: i64,
    pub creneau_id: i64,
    pub jour_semaine: i64,
    pub heure_debut: String,
    pub heure_fin: String,
    pub annee_scolaire: String,
    pub creneau_version: i64,
    pub role: Role,
}

impl DeserializeRow for ActiviteCreneauRow {
    fn from_row(row: &dyn RowView) -> Result<Self, AppError> {
        Ok(ActiviteCreneauRow {
            activite_id: row.get_i64(0)?,
            nom: row.get_str(1)?.to_string(),
            description: row.get_opt_str(2)?.map(String::from),
            capacite_max: row.get_opt_i64(3)?,
            activite_version: row.get_i64(4)?,
            creneau_id: row.get_i64(5)?,
            jour_semaine: row.get_i64(6)?,
            heure_debut: row.get_str(7)?.to_string(),
            heure_fin: row.get_str(8)?.to_string(),
            annee_scolaire: row.get_str(9)?.to_string(),
            creneau_version: row.get_i64(10)?,
            role: role_from_row(row, 11)?,
        })
    }
}
