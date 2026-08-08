use async_trait::async_trait;

use crate::domain::activite::{
    Activite, ActivitePersonne, CreateActivite, CreateLiaisonActivitePersonne, CreateTarifActivite,
    LiaisonActivitePersonne, PersonneActivite, TarifActivite, UpdateActivite,
};
use crate::error::AppError;
use crate::infrastructure::db::{DbTransaction, DeserializeRow, RowView};
use crate::repositories::rows::role_from_row;

#[async_trait]
pub trait ActiviteRepository: Send + Sync {
    #[allow(dead_code)]
    async fn create(&self, input: CreateActivite, utilisateur: &str) -> Result<Activite, AppError>;
    async fn creer_avec_tarif(
        &self,
        input: CreateActivite,
        utilisateur: &str,
    ) -> Result<Activite, AppError>;
    async fn update(
        &self,
        id: i64,
        input: UpdateActivite,
        utilisateur: &str,
    ) -> Result<Activite, AppError>;
    async fn find_by_id(&self, id: i64) -> Result<Option<Activite>, AppError>;
    async fn upsert_tarif(
        &self,
        input: CreateTarifActivite,
        utilisateur: &str,
    ) -> Result<TarifActivite, AppError>;
    async fn get_tarif(
        &self,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<Option<TarifActivite>, AppError>;
    #[allow(dead_code)]
    async fn ajouter_personne(
        &self,
        input: CreateLiaisonActivitePersonne,
        utilisateur: &str,
    ) -> Result<LiaisonActivitePersonne, AppError>;
    async fn retirer_personne(
        &self,
        activite_id: i64,
        personne_id: i64,
        annee_scolaire: &str,
    ) -> Result<(), AppError>;
    #[allow(dead_code)]
    async fn compter_participants(
        &self,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<i64, AppError>;
    #[allow(dead_code)]
    async fn trouver_liaison(
        &self,
        activite_id: i64,
        personne_id: i64,
        annee_scolaire: &str,
    ) -> Result<Option<LiaisonActivitePersonne>, AppError>;
    async fn trouver_liaison_tx(
        &self,
        tx: &mut dyn DbTransaction,
        activite_id: i64,
        personne_id: i64,
        annee_scolaire: &str,
    ) -> Result<Option<LiaisonActivitePersonne>, AppError>;
    async fn find_by_id_tx(
        &self,
        tx: &mut dyn DbTransaction,
        id: i64,
    ) -> Result<Option<Activite>, AppError>;
    async fn compter_participants_tx(
        &self,
        tx: &mut dyn DbTransaction,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<i64, AppError>;
    async fn ajouter_personne_tx(
        &self,
        tx: &mut dyn DbTransaction,
        input: CreateLiaisonActivitePersonne,
        utilisateur: &str,
    ) -> Result<LiaisonActivitePersonne, AppError>;
    async fn lister_encadrants(
        &self,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<Vec<PersonneActivite>, AppError>;
    async fn lister_participants(
        &self,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<Vec<PersonneActivite>, AppError>;
    async fn lister_activites_personne(
        &self,
        personne_id: i64,
    ) -> Result<Vec<ActivitePersonne>, AppError>;
    async fn lister_annees_disponibles(&self) -> Result<Vec<String>, AppError>;
    async fn lister_activites_par_annee(
        &self,
        annee_scolaire: &str,
    ) -> Result<Vec<(Activite, Option<f64>, i64)>, AppError>;
}

impl DeserializeRow for Activite {
    fn from_row(row: &dyn RowView) -> Result<Self, AppError> {
        Ok(Activite {
            id: row.get_i64(0)?,
            nom: row.get_str(1)?.to_string(),
            description: row.get_opt_str(2)?.map(String::from),
            capacite_max: row.get_opt_i64(3)?,
            version: row.get_i64(4)?,
        })
    }
}

impl DeserializeRow for TarifActivite {
    fn from_row(row: &dyn RowView) -> Result<Self, AppError> {
        Ok(TarifActivite {
            activite_id: row.get_i64(0)?,
            annee_scolaire: row.get_str(1)?.to_string(),
            tarif: row.get_f64(2)?,
        })
    }
}

impl DeserializeRow for LiaisonActivitePersonne {
    fn from_row(row: &dyn RowView) -> Result<Self, AppError> {
        Ok(LiaisonActivitePersonne {
            activite_id: row.get_i64(0)?,
            personne_id: row.get_i64(1)?,
            annee_scolaire: row.get_str(2)?.to_string(),
            role: role_from_row(row, 3)?,
        })
    }
}

impl DeserializeRow for PersonneActivite {
    fn from_row(row: &dyn RowView) -> Result<Self, AppError> {
        Ok(PersonneActivite {
            id: row.get_i64(0)?,
            nom: row.get_str(1)?.to_string(),
            prenom: row.get_str(2)?.to_string(),
        })
    }
}
