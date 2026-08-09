use async_trait::async_trait;

use crate::domain::adhesion::{Adhesion, CreateAdhesion, UpdateAdhesion};
use crate::error::AppError;
use crate::infrastructure::db::{DeserializeRow, RowView};

#[async_trait]
pub trait AdhesionRepository: Send + Sync {
    async fn create(&self, input: CreateAdhesion, utilisateur: &str) -> Result<Adhesion, AppError>;
    async fn update(
        &self,
        id: i64,
        input: UpdateAdhesion,
        utilisateur: &str,
    ) -> Result<Adhesion, AppError>;
    async fn list_by_personne(&self, personne_id: i64) -> Result<Vec<Adhesion>, AppError>;
}

impl DeserializeRow for Adhesion {
    fn from_row(row: &dyn RowView) -> Result<Self, AppError> {
        Ok(Adhesion {
            id: row.get_i64(0)?,
            personne_id: row.get_i64(1)?,
            annee_scolaire: row.get_str(2)?.to_string(),
            reglee: row.get_bool(3)?,
            note_paiement: row.get_opt_str(4)?.map(String::from),
            version: row.get_i64(5)?,
        })
    }
}
