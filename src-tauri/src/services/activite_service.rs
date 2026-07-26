use crate::domain::activite::{
    verifier_capacite_max, Activite, ActivitePersonne, CreateActivite,
    CreateLiaisonActivitePersonne, CreateTarifActivite, DetailActivite, Role, UpdateActivite,
};
use crate::domain::planning::jour_semaine_texte;
use crate::error::AppError;
use crate::repositories::{ActiviteRepository, PlanningRepository};

pub struct ActiviteService<'a, R: ActiviteRepository, P: PlanningRepository> {
    activite_repo: &'a R,
    planning_repo: &'a P,
}

impl<'a, R: ActiviteRepository, P: PlanningRepository> ActiviteService<'a, R, P> {
    pub fn new(activite_repo: &'a R, planning_repo: &'a P) -> Self {
        Self {
            activite_repo,
            planning_repo,
        }
    }

    pub async fn creer(&self, input: CreateActivite) -> Result<Activite, AppError> {
        if input.nom.trim().is_empty() {
            return Err(AppError::Validation(
                "Le nom de l'activité est requis".into(),
            ));
        }

        self.activite_repo.creer_avec_tarif(input).await
    }

    pub async fn modifier(&self, id: i64, input: UpdateActivite) -> Result<Activite, AppError> {
        if input.nom.trim().is_empty() {
            return Err(AppError::Validation(
                "Le nom de l'activité est requis".into(),
            ));
        }
        self.activite_repo.update(id, input).await
    }

    pub async fn obtenir(&self, id: i64) -> Result<Option<Activite>, AppError> {
        self.activite_repo.find_by_id(id).await
    }

    pub async fn obtenir_detail(
        &self,
        id: i64,
        annee_scolaire: &str,
    ) -> Result<DetailActivite, AppError> {
        let activite = self
            .activite_repo
            .find_by_id(id)
            .await?
            .ok_or(AppError::NotFound("Activité introuvable".into()))?;

        let tarif = self
            .activite_repo
            .get_tarif(id, annee_scolaire)
            .await?
            .map(|t| t.tarif);

        let encadrants = self
            .activite_repo
            .lister_encadrants(id, annee_scolaire)
            .await?;

        let participants = self
            .activite_repo
            .lister_participants(id, annee_scolaire)
            .await?;

        Ok(DetailActivite {
            activite,
            tarif,
            encadrants,
            participants,
        })
    }

    pub async fn lister_activites(
        &self,
        annee_scolaire: &str,
    ) -> Result<Vec<(Activite, Option<f64>, i64)>, AppError> {
        self.activite_repo
            .lister_activites_par_annee(annee_scolaire)
            .await
    }

    pub async fn definir_tarif(&self, input: CreateTarifActivite) -> Result<(), AppError> {
        self.activite_repo.upsert_tarif(input).await?;
        Ok(())
    }

    async fn verifier_liaison_existante(
        &self,
        activite_id: i64,
        personne_id: i64,
        annee_scolaire: &str,
        role: &Role,
    ) -> Result<(), AppError> {
        let existing = self
            .activite_repo
            .trouver_liaison(activite_id, personne_id, annee_scolaire)
            .await?;

        match existing {
            None => Ok(()),
            Some(l) if &l.role == role => Err(AppError::Conflict(
                "Cette personne est déjà inscrite à cette activité avec ce rôle".into(),
            )),
            Some(l) => Err(AppError::Conflict(format!(
                "Cette personne est déjà {} pour cette activité, elle ne peut pas être {}",
                l.role, role
            ))),
        }
    }

    async fn verifier_capacite(
        &self,
        activite_id: i64,
        annee_scolaire: &str,
        role: &Role,
    ) -> Result<(), AppError> {
        if *role != Role::Participant {
            return Ok(());
        }
        let activite = self
            .activite_repo
            .find_by_id(activite_id)
            .await?
            .ok_or(AppError::NotFound("Activité introuvable".into()))?;

        let nb_participants = self
            .activite_repo
            .compter_participants(activite_id, annee_scolaire)
            .await?;

        verifier_capacite_max(nb_participants, activite.capacite_max).map_err(AppError::Validation)
    }

    async fn verifier_collision_planning(
        &self,
        personne_id: i64,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<(), AppError> {
        if let Some(collision) = self
            .planning_repo
            .verifier_collision(personne_id, activite_id, annee_scolaire)
            .await?
        {
            return Err(AppError::Conflict(format!(
                "Conflit d'horaire avec l'activité '{}' : jour {} ({}), {}–{}.",
                collision.activite_conflit,
                collision.jour_semaine,
                jour_semaine_texte(collision.jour_semaine),
                collision.heure_debut,
                collision.heure_fin,
            )));
        }
        Ok(())
    }

    pub async fn ajouter_personne(
        &self,
        input: CreateLiaisonActivitePersonne,
    ) -> Result<(), AppError> {
        self.verifier_liaison_existante(
            input.activite_id,
            input.personne_id,
            &input.annee_scolaire,
            &input.role,
        )
        .await?;

        self.verifier_capacite(input.activite_id, &input.annee_scolaire, &input.role)
            .await?;

        self.verifier_collision_planning(
            input.personne_id,
            input.activite_id,
            &input.annee_scolaire,
        )
        .await?;

        self.activite_repo.ajouter_personne(input).await?;
        Ok(())
    }

    pub async fn retirer_personne(
        &self,
        activite_id: i64,
        personne_id: i64,
        annee_scolaire: &str,
    ) -> Result<(), AppError> {
        self.activite_repo
            .retirer_personne(activite_id, personne_id, annee_scolaire)
            .await
    }

    pub async fn lister_annees(&self) -> Result<Vec<String>, AppError> {
        self.activite_repo.lister_annees_disponibles().await
    }

    pub async fn lister_activites_personne(
        &self,
        personne_id: i64,
    ) -> Result<Vec<ActivitePersonne>, AppError> {
        self.activite_repo
            .lister_activites_personne(personne_id)
            .await
    }
}
