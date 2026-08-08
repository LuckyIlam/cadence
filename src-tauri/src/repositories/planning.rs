use async_trait::async_trait;

use crate::domain::planning::{
    Collision, CreateCreneau, CreateSemaineBanalisee, CreneauActivite, CreneauHorsPlage,
    Inscription, PlanningCreneau, SemaineBanalisee,
};
use crate::error::AppError;
use crate::infrastructure::db::{DbTransaction, DeserializeRow, RowView};

#[async_trait]
pub trait PlanningRepository: Send + Sync {
    #[allow(dead_code)]
    async fn creer_creneau(
        &self,
        input: CreateCreneau,
        utilisateur: &str,
    ) -> Result<CreneauActivite, AppError>;
    #[allow(dead_code)]
    async fn supprimer_creneau(&self, id: i64) -> Result<(), AppError>;
    #[allow(dead_code)]
    async fn modifier_creneau(
        &self,
        id: i64,
        input: CreateCreneau,
        version: i64,
        utilisateur: &str,
    ) -> Result<CreneauActivite, AppError>;
    async fn lister_creneaux(
        &self,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<Vec<CreneauActivite>, AppError>;
    async fn lister_tous_creneaux(&self) -> Result<Vec<CreneauActivite>, AppError>;
    async fn lister_creneaux_hors_plage(
        &self,
        heure_ouverture: &str,
        heure_fermeture: &str,
    ) -> Result<Vec<CreneauHorsPlage>, AppError>;
    async fn lister_inscriptions(&self) -> Result<Vec<Inscription>, AppError>;
    async fn supprimer_creneau_tx(
        &self,
        tx: &mut dyn DbTransaction,
        id: i64,
    ) -> Result<(), AppError>;
    async fn deplacer_creneau_tx(
        &self,
        tx: &mut dyn DbTransaction,
        id: i64,
        heure_debut: &str,
        heure_fin: &str,
        utilisateur: &str,
    ) -> Result<CreneauActivite, AppError>;
    async fn creer_creneau_tx(
        &self,
        tx: &mut dyn DbTransaction,
        input: CreateCreneau,
        utilisateur: &str,
    ) -> Result<CreneauActivite, AppError>;
    async fn modifier_creneau_tx(
        &self,
        tx: &mut dyn DbTransaction,
        id: i64,
        input: CreateCreneau,
        version: i64,
        utilisateur: &str,
    ) -> Result<CreneauActivite, AppError>;
    async fn ajouter_semaine_banalisee(
        &self,
        input: CreateSemaineBanalisee,
        utilisateur: &str,
    ) -> Result<SemaineBanalisee, AppError>;
    async fn supprimer_semaine_banalisee(&self, id: i64) -> Result<(), AppError>;
    async fn lister_semaines_banalisees(
        &self,
        activite_id: i64,
    ) -> Result<Vec<SemaineBanalisee>, AppError>;
    #[allow(dead_code)]
    async fn verifier_conflit_creneaux(
        &self,
        activite_id: i64,
        annee_scolaire: &str,
        jour_semaine: i64,
        heure_debut: &str,
        heure_fin: &str,
        exclure_id: Option<i64>,
    ) -> Result<Vec<CreneauActivite>, AppError>;
    #[allow(clippy::too_many_arguments)]
    async fn verifier_conflit_creneaux_tx(
        &self,
        tx: &mut dyn DbTransaction,
        activite_id: i64,
        annee_scolaire: &str,
        jour_semaine: i64,
        heure_debut: &str,
        heure_fin: &str,
        exclure_id: Option<i64>,
    ) -> Result<Vec<CreneauActivite>, AppError>;
    #[allow(dead_code)]
    async fn compter_inscrits_activite(
        &self,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<i64, AppError>;
    async fn compter_inscrits_activite_tx(
        &self,
        tx: &mut dyn DbTransaction,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<i64, AppError>;
    async fn verifier_collision(
        &self,
        personne_id: i64,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<Option<Collision>, AppError>;
    async fn verifier_collision_tx(
        &self,
        tx: &mut dyn DbTransaction,
        personne_id: i64,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<Option<Collision>, AppError>;
    async fn lister_creneaux_tx(
        &self,
        tx: &mut dyn DbTransaction,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<Vec<CreneauActivite>, AppError>;
    async fn planning_personne_semaine(
        &self,
        personne_id: i64,
        date_lundi: &str,
        annee_scolaire: &str,
    ) -> Result<Vec<PlanningCreneau>, AppError>;
}

impl DeserializeRow for CreneauActivite {
    fn from_row(row: &dyn RowView) -> Result<Self, AppError> {
        Ok(CreneauActivite {
            id: row.get_i64(0)?,
            activite_id: row.get_i64(1)?,
            jour_semaine: row.get_i64(2)?,
            heure_debut: row.get_str(3)?.to_string(),
            heure_fin: row.get_str(4)?.to_string(),
            annee_scolaire: row.get_str(5)?.to_string(),
            version: row.get_i64(6)?,
        })
    }
}

impl DeserializeRow for SemaineBanalisee {
    fn from_row(row: &dyn RowView) -> Result<Self, AppError> {
        Ok(SemaineBanalisee {
            id: row.get_i64(0)?,
            activite_id: row.get_i64(1)?,
            date_debut: row.get_str(2)?.to_string(),
            motif: row.get_opt_str(3)?.map(String::from),
            annee_scolaire: row.get_str(4)?.to_string(),
        })
    }
}

impl DeserializeRow for CreneauHorsPlage {
    fn from_row(row: &dyn RowView) -> Result<Self, AppError> {
        Ok(CreneauHorsPlage {
            creneau_id: row.get_i64(0)?,
            activite_id: row.get_i64(1)?,
            activite_nom: row.get_str(2)?.to_string(),
            jour_semaine: row.get_i64(3)?,
            heure_debut: row.get_str(4)?.to_string(),
            heure_fin: row.get_str(5)?.to_string(),
            annee_scolaire: row.get_str(6)?.to_string(),
            nb_inscrits: row.get_i64(7)?,
        })
    }
}

impl DeserializeRow for Inscription {
    fn from_row(row: &dyn RowView) -> Result<Self, AppError> {
        Ok(Inscription {
            activite_id: row.get_i64(0)?,
            personne_id: row.get_i64(1)?,
            annee_scolaire: row.get_str(2)?.to_string(),
            activite_nom: row.get_str(3)?.to_string(),
        })
    }
}
