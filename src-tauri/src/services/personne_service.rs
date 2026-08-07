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

    pub async fn creer(
        &self,
        utilisateur: &str,
        input: CreatePersonne,
    ) -> Result<Personne, AppError> {
        valider_date_naissance(input.date_naissance)?;
        self.valider_responsable_legal(input.date_naissance, input.responsable_id)
            .await?;
        self.personne_repo.create(input, utilisateur).await
    }

    pub async fn modifier(
        &self,
        utilisateur: &str,
        id: i64,
        input: UpdatePersonne,
    ) -> Result<Personne, AppError> {
        valider_date_naissance(input.date_naissance)?;
        self.valider_responsable_legal(input.date_naissance, input.responsable_id)
            .await?;
        self.personne_repo.update(id, input, utilisateur).await
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

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::{Mutex, OnceLock};

    static NEXT_ID: OnceLock<Mutex<i64>> = OnceLock::new();

    fn next_id() -> i64 {
        let lock = NEXT_ID.get_or_init(|| Mutex::new(1));
        let mut id = lock.lock().unwrap();
        let current = *id;
        *id += 1;
        current
    }

    fn date(ymd: &str) -> NaiveDate {
        NaiveDate::parse_from_str(ymd, "%Y-%m-%d").unwrap()
    }

    struct MockPersonneRepository {
        personnes: Mutex<Vec<Personne>>,
    }

    impl MockPersonneRepository {
        fn new() -> Self {
            Self {
                personnes: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl PersonneRepository for MockPersonneRepository {
        async fn create(
            &self,
            input: CreatePersonne,
            _utilisateur: &str,
        ) -> Result<Personne, AppError> {
            let p = Personne {
                id: next_id(),
                nom: input.nom,
                prenom: input.prenom,
                date_naissance: input.date_naissance,
                email: input.email,
                telephone: input.telephone,
                responsable_id: input.responsable_id,
                version: 1,
            };
            let id = p.id;
            self.personnes.lock().unwrap().push(p);
            Ok(self
                .personnes
                .lock()
                .unwrap()
                .iter()
                .find(|p| p.id == id)
                .unwrap()
                .clone())
        }

        async fn update(
            &self,
            id: i64,
            input: UpdatePersonne,
            _utilisateur: &str,
        ) -> Result<Personne, AppError> {
            let mut personnes = self.personnes.lock().unwrap();
            let p = personnes
                .iter_mut()
                .find(|p| p.id == id)
                .ok_or(AppError::NotFound("Personne introuvable".into()))?;
            p.nom = input.nom;
            p.prenom = input.prenom;
            p.date_naissance = input.date_naissance;
            p.email = input.email;
            p.telephone = input.telephone;
            p.responsable_id = input.responsable_id;
            Ok(p.clone())
        }

        async fn find_by_id(&self, id: i64) -> Result<Option<Personne>, AppError> {
            Ok(self
                .personnes
                .lock()
                .unwrap()
                .iter()
                .find(|p| p.id == id)
                .cloned())
        }

        async fn rechercher(
            &self,
            _criteres: CriteresRecherchePersonnes,
            _pagination: Pagination,
        ) -> Result<ResultatRecherchePersonnes, AppError> {
            let personnes = self.personnes.lock().unwrap();
            Ok(ResultatRecherchePersonnes {
                donnees: personnes.clone(),
                total: personnes.len() as u32,
                page: 1,
                pages: 1,
            })
        }
    }

    struct MockAdhesionRepository;

    #[async_trait]
    impl AdhesionRepository for MockAdhesionRepository {
        async fn create(
            &self,
            _input: crate::domain::adhesion::CreateAdhesion,
            _utilisateur: &str,
        ) -> Result<crate::domain::adhesion::Adhesion, AppError> {
            unreachable!("not used in personne service tests")
        }

        async fn update(
            &self,
            _id: i64,
            _input: crate::domain::adhesion::UpdateAdhesion,
            _utilisateur: &str,
        ) -> Result<crate::domain::adhesion::Adhesion, AppError> {
            unreachable!("not used in personne service tests")
        }

        async fn list_by_personne(
            &self,
            _personne_id: i64,
        ) -> Result<Vec<crate::domain::adhesion::Adhesion>, AppError> {
            Ok(Vec::new())
        }
    }

    fn make_service() -> PersonneService<'static, MockPersonneRepository, MockAdhesionRepository> {
        let repo = Box::new(MockPersonneRepository::new());
        let adhesion_repo = Box::new(MockAdhesionRepository);
        PersonneService::new(Box::leak(repo), Box::leak(adhesion_repo))
    }

    #[tokio::test]
    async fn test_creer_majeur_sans_responsable_cree() {
        let service = make_service();
        let result = service
            .creer(
                "alice",
                CreatePersonne {
                    nom: "Dupont".into(),
                    prenom: "Jean".into(),
                    date_naissance: date("1990-01-15"),
                    email: None,
                    telephone: None,
                    responsable_id: None,
                },
            )
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().nom, "Dupont");
    }

    #[tokio::test]
    async fn test_creer_mineur_sans_responsable_retourne_erreur() {
        let service = make_service();
        let result = service
            .creer(
                "alice",
                CreatePersonne {
                    nom: "Martin".into(),
                    prenom: "Lucas".into(),
                    date_naissance: date("2010-06-01"),
                    email: None,
                    telephone: None,
                    responsable_id: None,
                },
            )
            .await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Validation(msg) => assert!(msg.contains("mineur")),
            _ => panic!("expected Validation error"),
        }
    }

    #[tokio::test]
    async fn test_creer_mineur_avec_responsable_mineur_retourne_erreur() {
        let service = make_service();
        let responsable = service
            .personne_repo
            .create(
                CreatePersonne {
                    nom: "Petit".into(),
                    prenom: "Enfant".into(),
                    date_naissance: date("2012-03-10"),
                    email: None,
                    telephone: None,
                    responsable_id: None,
                },
                "alice",
            )
            .await
            .unwrap();

        let result = service
            .creer(
                "alice",
                CreatePersonne {
                    nom: "Martin".into(),
                    prenom: "Lucas".into(),
                    date_naissance: date("2010-06-01"),
                    email: None,
                    telephone: None,
                    responsable_id: Some(responsable.id),
                },
            )
            .await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Validation(msg) => {
                assert!(msg.contains("responsable") && msg.contains("mineur"))
            }
            _ => panic!("expected Validation error"),
        }
    }

    #[tokio::test]
    async fn test_creer_mineur_avec_responsable_majeur_cree() {
        let service = make_service();
        let responsable = service
            .personne_repo
            .create(
                CreatePersonne {
                    nom: "Dupont".into(),
                    prenom: "Adulte".into(),
                    date_naissance: date("1985-07-20"),
                    email: None,
                    telephone: None,
                    responsable_id: None,
                },
                "alice",
            )
            .await
            .unwrap();

        let result = service
            .creer(
                "alice",
                CreatePersonne {
                    nom: "Martin".into(),
                    prenom: "Lucas".into(),
                    date_naissance: date("2010-06-01"),
                    email: None,
                    telephone: None,
                    responsable_id: Some(responsable.id),
                },
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_obtenir_detail_personne() {
        let service = make_service();
        let p = service
            .personne_repo
            .create(
                CreatePersonne {
                    nom: "Durand".into(),
                    prenom: "Sophie".into(),
                    date_naissance: date("1992-11-03"),
                    email: None,
                    telephone: None,
                    responsable_id: None,
                },
                "alice",
            )
            .await
            .unwrap();

        let detail = service.obtenir_detail(p.id).await.unwrap();
        assert_eq!(detail.personne.nom, "Durand");
        assert!(detail.adhesions.is_empty());
    }
}
