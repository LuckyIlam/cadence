use async_trait::async_trait;

use crate::domain::personne::{
    CreatePersonne, CriteresRecherchePersonnes, Pagination, Personne, ResultatRecherchePersonnes,
    UpdatePersonne,
};
use crate::error::AppError;
use crate::infrastructure::db::{DeserializeRow, RowView};

#[async_trait]
pub trait PersonneRepository: Send + Sync {
    async fn create(&self, input: CreatePersonne, utilisateur: &str) -> Result<Personne, AppError>;
    async fn update(
        &self,
        id: i64,
        input: UpdatePersonne,
        utilisateur: &str,
    ) -> Result<Personne, AppError>;
    async fn find_by_id(&self, id: i64) -> Result<Option<Personne>, AppError>;
    async fn rechercher(
        &self,
        criteres: CriteresRecherchePersonnes,
        pagination: Pagination,
    ) -> Result<ResultatRecherchePersonnes, AppError>;
}

impl DeserializeRow for Personne {
    fn from_row(row: &dyn RowView) -> Result<Self, AppError> {
        Ok(Personne {
            id: row.get_i64(0)?,
            nom: row.get_str(1)?.to_string(),
            prenom: row.get_str(2)?.to_string(),
            date_naissance: row.get_naive_date(3)?,
            email: row.get_opt_str(4)?.map(String::from),
            telephone: row.get_opt_str(5)?.map(String::from),
            responsable_id: row.get_opt_i64(6)?,
            version: row.get_i64(7)?,
        })
    }
}
