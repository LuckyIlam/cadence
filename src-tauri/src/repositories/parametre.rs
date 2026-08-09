use async_trait::async_trait;

use crate::domain::parametre::ParametresPlanning;
use crate::error::AppError;
use crate::infrastructure::db::{DbTransaction, DeserializeRow, RowView};

#[async_trait]
pub trait ParametreRepository: Send + Sync {
    async fn obtenir_parametres_planning(&self) -> Result<ParametresPlanning, AppError>;
    async fn mettre_a_jour_plage_horaire_tx(
        &self,
        tx: &mut dyn DbTransaction,
        heure_ouverture: &str,
        heure_fermeture: &str,
        utilisateur: &str,
    ) -> Result<ParametresPlanning, AppError>;
}

impl DeserializeRow for ParametresPlanning {
    fn from_row(row: &dyn RowView) -> Result<Self, AppError> {
        Ok(ParametresPlanning {
            id: row.get_i64(0)?,
            heure_ouverture: row.get_str(1)?.to_string(),
            heure_fermeture: row.get_str(2)?.to_string(),
        })
    }
}
