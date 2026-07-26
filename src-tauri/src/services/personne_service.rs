use chrono::NaiveDate;

use crate::domain::personne::{
    current_annee_scolaire, est_mineur, valider_date_naissance, CreatePersonne,
    CriteresRecherchePersonnes, Pagination, Personne, PersonneDetail, ResultatRecherchePersonnes,
    UpdatePersonne,
};
use crate::error::AppError;
use crate::repositories::{AdhesionRepository, PersonneRepository};

pub struct PersonneService<'a, R: PersonneRepository, A: AdhesionRepository> {
    personne_repo: &'a R,
    adhesion_repo: &'a A,
}

impl<'a, R: PersonneRepository, A: AdhesionRepository> PersonneService<'a, R, A> {
    pub fn new(personne_repo: &'a R, adhesion_repo: &'a A) -> Self {
        Self {
            personne_repo,
            adhesion_repo,
        }
    }

    async fn valider_responsable_legal(
        &self,
        date_naissance: NaiveDate,
        responsable_id: Option<i64>,
    ) -> Result<(), AppError> {
        if !est_mineur(date_naissance) {
            return Ok(());
        }
        let rid = responsable_id.ok_or(AppError::Validation(
            "Un mineur doit avoir un responsable légal".into(),
        ))?;
        let responsable = self
            .personne_repo
            .find_by_id(rid)
            .await?
            .ok_or(AppError::NotFound("Responsable introuvable".into()))?;
        if est_mineur(responsable.date_naissance) {
            return Err(AppError::Validation(
                "Le responsable ne peut pas être mineur".into(),
            ));
        }
        Ok(())
    }

    pub async fn creer(&self, input: CreatePersonne) -> Result<Personne, AppError> {
        valider_date_naissance(input.date_naissance)?;
        self.valider_responsable_legal(input.date_naissance, input.responsable_id)
            .await?;
        self.personne_repo.create(input).await
    }

    pub async fn modifier(&self, id: i64, input: UpdatePersonne) -> Result<Personne, AppError> {
        valider_date_naissance(input.date_naissance)?;
        self.valider_responsable_legal(input.date_naissance, input.responsable_id)
            .await?;
        self.personne_repo.update(id, input).await
    }

    pub async fn obtenir(&self, id: i64) -> Result<Option<Personne>, AppError> {
        self.personne_repo.find_by_id(id).await
    }

    pub async fn obtenir_detail(&self, id: i64) -> Result<PersonneDetail, AppError> {
        let personne = self
            .personne_repo
            .find_by_id(id)
            .await?
            .ok_or(AppError::NotFound("Personne introuvable".into()))?;

        let adhesions = self.adhesion_repo.list_by_personne(id).await?;

        let annee_scolaire = current_annee_scolaire();
        let a_adhesion_annee_cours = adhesions.iter().any(|a| a.annee_scolaire == annee_scolaire);

        Ok(PersonneDetail {
            personne,
            adhesions,
            a_adhesion_annee_cours,
        })
    }

    pub async fn rechercher(
        &self,
        criteres: CriteresRecherchePersonnes,
        pagination: Pagination,
    ) -> Result<ResultatRecherchePersonnes, AppError> {
        self.personne_repo.rechercher(criteres, pagination).await
    }
}
