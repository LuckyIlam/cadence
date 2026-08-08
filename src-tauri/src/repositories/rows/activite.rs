use crate::domain::activite::Role;
use crate::error::AppError;
use crate::infrastructure::db::{DeserializeRow, RowView};
use crate::repositories::rows::role_from_row;

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

pub struct ActivitePersonneRow {
    pub id: i64,
    pub nom: String,
    pub description: Option<String>,
    pub capacite_max: Option<i64>,
    pub version: i64,
    pub role: Role,
}

impl DeserializeRow for ActivitePersonneRow {
    fn from_row(row: &dyn RowView) -> Result<Self, AppError> {
        Ok(ActivitePersonneRow {
            id: row.get_i64(0)?,
            nom: row.get_str(1)?.to_string(),
            description: row.get_opt_str(2)?.map(String::from),
            capacite_max: row.get_opt_i64(3)?,
            version: row.get_i64(4)?,
            role: role_from_row(row, 5)?,
        })
    }
}

pub struct AnneeRow {
    pub annee_scolaire: String,
}

impl DeserializeRow for AnneeRow {
    fn from_row(row: &dyn RowView) -> Result<Self, AppError> {
        Ok(AnneeRow {
            annee_scolaire: row.get_str(0)?.to_string(),
        })
    }
}

pub struct ActiviteAnneeRow {
    pub id: i64,
    pub nom: String,
    pub description: Option<String>,
    pub capacite_max: Option<i64>,
    pub version: i64,
    pub tarif: Option<f64>,
    pub nb_participants: i64,
}

impl DeserializeRow for ActiviteAnneeRow {
    fn from_row(row: &dyn RowView) -> Result<Self, AppError> {
        Ok(ActiviteAnneeRow {
            id: row.get_i64(0)?,
            nom: row.get_str(1)?.to_string(),
            description: row.get_opt_str(2)?.map(String::from),
            capacite_max: row.get_opt_i64(3)?,
            version: row.get_i64(4)?,
            tarif: row.get_opt_f64(5)?,
            nb_participants: row.get_i64(6)?,
        })
    }
}
