use libsql::Connection;

use crate::domain::activite::{
    verifier_capacite_max, Activite, ActivitePersonne, CreateActivite,
    CreateLiaisonActivitePersonne, CreateTarifActivite, DetailActivite, Role, UpdateActivite,
};
use crate::domain::planning::format_conflit_plage;
use crate::error::AppError;
use crate::repositories::{ActiviteRepository, PlanningRepository};

pub struct ActiviteService<'a, R: ActiviteRepository, P: PlanningRepository> {
    activite_repo: &'a R,
    planning_repo: &'a P,
    conn: Connection,
}

impl<'a, R: ActiviteRepository, P: PlanningRepository> ActiviteService<'a, R, P> {
    pub fn new(activite_repo: &'a R, planning_repo: &'a P, conn: Connection) -> Self {
        Self {
            activite_repo,
            planning_repo,
            conn,
        }
    }

    pub async fn creer(
        &self,
        utilisateur: &str,
        input: CreateActivite,
    ) -> Result<Activite, AppError> {
        if input.nom.trim().is_empty() {
            return Err(AppError::Validation(
                "Le nom de l'activité est requis".into(),
            ));
        }

        self.activite_repo
            .creer_avec_tarif(input, utilisateur)
            .await
    }

    pub async fn modifier(
        &self,
        utilisateur: &str,
        id: i64,
        input: UpdateActivite,
    ) -> Result<Activite, AppError> {
        if input.nom.trim().is_empty() {
            return Err(AppError::Validation(
                "Le nom de l'activité est requis".into(),
            ));
        }
        self.activite_repo.update(id, input, utilisateur).await
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

    pub async fn definir_tarif(
        &self,
        utilisateur: &str,
        input: CreateTarifActivite,
    ) -> Result<(), AppError> {
        self.activite_repo.upsert_tarif(input, utilisateur).await?;
        Ok(())
    }

    async fn verifier_liaison_existante_tx(
        &self,
        tx: &mut libsql::Transaction,
        activite_id: i64,
        personne_id: i64,
        annee_scolaire: &str,
        role: &Role,
    ) -> Result<(), AppError> {
        let existing = self
            .activite_repo
            .trouver_liaison_tx(tx, activite_id, personne_id, annee_scolaire)
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

    async fn verifier_capacite_tx(
        &self,
        tx: &mut libsql::Transaction,
        activite_id: i64,
        annee_scolaire: &str,
        role: &Role,
    ) -> Result<(), AppError> {
        if *role != Role::Participant {
            return Ok(());
        }
        let activite = self
            .activite_repo
            .find_by_id_tx(tx, activite_id)
            .await?
            .ok_or(AppError::NotFound("Activité introuvable".into()))?;

        let nb_participants = self
            .activite_repo
            .compter_participants_tx(tx, activite_id, annee_scolaire)
            .await?;

        verifier_capacite_max(nb_participants, activite.capacite_max).map_err(AppError::Validation)
    }

    async fn verifier_collision_planning_tx(
        &self,
        tx: &mut libsql::Transaction,
        personne_id: i64,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<(), AppError> {
        if let Some(collision) = self
            .planning_repo
            .verifier_collision_tx(tx, personne_id, activite_id, annee_scolaire)
            .await?
        {
            return Err(AppError::Conflict(format!(
                "Conflit d'horaire avec l'activité '{}' : {}.",
                collision.activite_conflit,
                format_conflit_plage(
                    collision.jour_semaine,
                    &collision.heure_debut,
                    &collision.heure_fin,
                ),
            )));
        }
        Ok(())
    }

    pub async fn ajouter_personne(
        &self,
        utilisateur: &str,
        input: CreateLiaisonActivitePersonne,
    ) -> Result<(), AppError> {
        // BEGIN IMMEDIATE : acquiert le verrou d'écriture dès le début pour que
        // les vérifications et l'insertion soient atomiques (pas de TOCTOU entre
        // deux utilisateurs en mode multi).
        let mut tx = self
            .conn
            .transaction_with_behavior(libsql::TransactionBehavior::Immediate)
            .await?;

        self.verifier_liaison_existante_tx(
            &mut tx,
            input.activite_id,
            input.personne_id,
            &input.annee_scolaire,
            &input.role,
        )
        .await?;

        self.verifier_capacite_tx(
            &mut tx,
            input.activite_id,
            &input.annee_scolaire,
            &input.role,
        )
        .await?;

        self.verifier_collision_planning_tx(
            &mut tx,
            input.personne_id,
            input.activite_id,
            &input.annee_scolaire,
        )
        .await?;

        self.activite_repo
            .ajouter_personne_tx(&mut tx, input, utilisateur)
            .await?;

        tx.commit().await?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Mutex;

    use crate::domain::activite::{LiaisonActivitePersonne, TarifActivite};
    use crate::domain::planning::{Collision, PlanningCreneau};

    struct MockActiviteRepository {
        activites: Mutex<Vec<Activite>>,
        liaisons: Mutex<Vec<LiaisonActivitePersonne>>,
        capacite_max: Mutex<Option<i64>>,
    }

    impl MockActiviteRepository {
        fn new() -> Self {
            Self {
                activites: Mutex::new(Vec::new()),
                liaisons: Mutex::new(Vec::new()),
                capacite_max: Mutex::new(None),
            }
        }

        fn avec_capacite(capacite_max: i64) -> Self {
            Self {
                activites: Mutex::new(Vec::new()),
                liaisons: Mutex::new(Vec::new()),
                capacite_max: Mutex::new(Some(capacite_max)),
            }
        }
    }

    #[async_trait]
    impl ActiviteRepository for MockActiviteRepository {
        #[allow(dead_code)]
        async fn create(
            &self,
            input: CreateActivite,
            _utilisateur: &str,
        ) -> Result<Activite, AppError> {
            let id = self.activites.lock().unwrap().len() as i64 + 1;
            let a = Activite {
                id,
                nom: input.nom,
                description: input.description,
                capacite_max: *self.capacite_max.lock().unwrap(),
                version: 1,
            };
            self.activites.lock().unwrap().push(a.clone());
            Ok(a)
        }

        async fn creer_avec_tarif(
            &self,
            input: CreateActivite,
            utilisateur: &str,
        ) -> Result<Activite, AppError> {
            self.create(input, utilisateur).await
        }

        async fn update(
            &self,
            id: i64,
            input: UpdateActivite,
            _utilisateur: &str,
        ) -> Result<Activite, AppError> {
            let mut activites = self.activites.lock().unwrap();
            let a = activites
                .iter_mut()
                .find(|a| a.id == id)
                .ok_or(AppError::NotFound("Activité introuvable".into()))?;
            a.nom = input.nom;
            a.description = input.description;
            a.capacite_max = input.capacite_max;
            Ok(a.clone())
        }

        async fn find_by_id(&self, id: i64) -> Result<Option<Activite>, AppError> {
            Ok(self
                .activites
                .lock()
                .unwrap()
                .iter()
                .find(|a| a.id == id)
                .cloned())
        }

        async fn find_by_id_tx(
            &self,
            _tx: &mut libsql::Transaction,
            id: i64,
        ) -> Result<Option<Activite>, AppError> {
            self.find_by_id(id).await
        }

        async fn upsert_tarif(
            &self,
            _input: CreateTarifActivite,
            _utilisateur: &str,
        ) -> Result<TarifActivite, AppError> {
            unimplemented!()
        }

        async fn get_tarif(
            &self,
            _activite_id: i64,
            _annee_scolaire: &str,
        ) -> Result<Option<TarifActivite>, AppError> {
            Ok(None)
        }

        async fn ajouter_personne(
            &self,
            input: CreateLiaisonActivitePersonne,
            _utilisateur: &str,
        ) -> Result<LiaisonActivitePersonne, AppError> {
            let liaison = LiaisonActivitePersonne {
                activite_id: input.activite_id,
                personne_id: input.personne_id,
                annee_scolaire: input.annee_scolaire,
                role: input.role,
            };
            self.liaisons.lock().unwrap().push(liaison.clone());
            Ok(liaison)
        }

        async fn ajouter_personne_tx(
            &self,
            _tx: &mut libsql::Transaction,
            input: CreateLiaisonActivitePersonne,
            utilisateur: &str,
        ) -> Result<LiaisonActivitePersonne, AppError> {
            self.ajouter_personne(input, utilisateur).await
        }

        async fn retirer_personne(
            &self,
            _activite_id: i64,
            _personne_id: i64,
            _annee_scolaire: &str,
        ) -> Result<(), AppError> {
            unimplemented!()
        }

        async fn compter_participants(
            &self,
            activite_id: i64,
            annee_scolaire: &str,
        ) -> Result<i64, AppError> {
            let count = self
                .liaisons
                .lock()
                .unwrap()
                .iter()
                .filter(|l| {
                    l.activite_id == activite_id
                        && l.annee_scolaire == annee_scolaire
                        && l.role == Role::Participant
                })
                .count();
            Ok(count as i64)
        }

        async fn compter_participants_tx(
            &self,
            _tx: &mut libsql::Transaction,
            activite_id: i64,
            annee_scolaire: &str,
        ) -> Result<i64, AppError> {
            self.compter_participants(activite_id, annee_scolaire).await
        }

        async fn trouver_liaison(
            &self,
            activite_id: i64,
            personne_id: i64,
            annee_scolaire: &str,
        ) -> Result<Option<LiaisonActivitePersonne>, AppError> {
            Ok(self
                .liaisons
                .lock()
                .unwrap()
                .iter()
                .find(|l| {
                    l.activite_id == activite_id
                        && l.personne_id == personne_id
                        && l.annee_scolaire == annee_scolaire
                })
                .cloned())
        }

        async fn trouver_liaison_tx(
            &self,
            _tx: &mut libsql::Transaction,
            activite_id: i64,
            personne_id: i64,
            annee_scolaire: &str,
        ) -> Result<Option<LiaisonActivitePersonne>, AppError> {
            self.trouver_liaison(activite_id, personne_id, annee_scolaire)
                .await
        }

        async fn lister_encadrants(
            &self,
            _activite_id: i64,
            _annee_scolaire: &str,
        ) -> Result<Vec<crate::domain::activite::PersonneActivite>, AppError> {
            unimplemented!()
        }

        async fn lister_participants(
            &self,
            _activite_id: i64,
            _annee_scolaire: &str,
        ) -> Result<Vec<crate::domain::activite::PersonneActivite>, AppError> {
            unimplemented!()
        }

        async fn lister_activites_personne(
            &self,
            _personne_id: i64,
        ) -> Result<Vec<ActivitePersonne>, AppError> {
            unimplemented!()
        }

        async fn lister_annees_disponibles(&self) -> Result<Vec<String>, AppError> {
            unimplemented!()
        }

        async fn lister_activites_par_annee(
            &self,
            _annee_scolaire: &str,
        ) -> Result<Vec<(Activite, Option<f64>, i64)>, AppError> {
            unimplemented!()
        }
    }

    struct MockPlanningRepository {
        collision: Mutex<Option<Collision>>,
    }

    impl MockPlanningRepository {
        fn new() -> Self {
            Self {
                collision: Mutex::new(None),
            }
        }

        fn avec_collision() -> Self {
            Self {
                collision: Mutex::new(Some(Collision {
                    activite_conflit: "Poterie".into(),
                    jour_semaine: 2,
                    heure_debut: "14:00".into(),
                    heure_fin: "16:00".into(),
                })),
            }
        }
    }

    #[async_trait]
    impl PlanningRepository for MockPlanningRepository {
        async fn creer_creneau(
            &self,
            _input: crate::domain::planning::CreateCreneau,
            _utilisateur: &str,
        ) -> Result<crate::domain::planning::CreneauActivite, AppError> {
            unimplemented!()
        }

        async fn supprimer_creneau(&self, _id: i64) -> Result<(), AppError> {
            unimplemented!()
        }

        async fn modifier_creneau(
            &self,
            _id: i64,
            _input: crate::domain::planning::CreateCreneau,
            _version: i64,
            _utilisateur: &str,
        ) -> Result<crate::domain::planning::CreneauActivite, AppError> {
            unimplemented!()
        }

        async fn lister_creneaux(
            &self,
            _activite_id: i64,
            _annee_scolaire: &str,
        ) -> Result<Vec<crate::domain::planning::CreneauActivite>, AppError> {
            unimplemented!()
        }

        async fn lister_tous_creneaux(
            &self,
        ) -> Result<Vec<crate::domain::planning::CreneauActivite>, AppError> {
            unimplemented!()
        }

        async fn lister_creneaux_hors_plage(
            &self,
            _heure_ouverture: &str,
            _heure_fermeture: &str,
        ) -> Result<Vec<crate::domain::planning::CreneauHorsPlage>, AppError> {
            unimplemented!()
        }

        async fn lister_inscriptions(
            &self,
        ) -> Result<Vec<crate::domain::planning::Inscription>, AppError> {
            unimplemented!()
        }

        async fn supprimer_creneau_tx(
            &self,
            _tx: &mut libsql::Transaction,
            _id: i64,
        ) -> Result<(), AppError> {
            unimplemented!()
        }

        async fn deplacer_creneau_tx(
            &self,
            _tx: &mut libsql::Transaction,
            _id: i64,
            _heure_debut: &str,
            _heure_fin: &str,
            _utilisateur: &str,
        ) -> Result<crate::domain::planning::CreneauActivite, AppError> {
            unimplemented!()
        }

        async fn creer_creneau_tx(
            &self,
            _tx: &mut libsql::Transaction,
            _input: crate::domain::planning::CreateCreneau,
            _utilisateur: &str,
        ) -> Result<crate::domain::planning::CreneauActivite, AppError> {
            unimplemented!()
        }

        async fn modifier_creneau_tx(
            &self,
            _tx: &mut libsql::Transaction,
            _id: i64,
            _input: crate::domain::planning::CreateCreneau,
            _version: i64,
            _utilisateur: &str,
        ) -> Result<crate::domain::planning::CreneauActivite, AppError> {
            unimplemented!()
        }

        async fn ajouter_semaine_banalisee(
            &self,
            _input: crate::domain::planning::CreateSemaineBanalisee,
            _utilisateur: &str,
        ) -> Result<crate::domain::planning::SemaineBanalisee, AppError> {
            unimplemented!()
        }

        async fn supprimer_semaine_banalisee(&self, _id: i64) -> Result<(), AppError> {
            unimplemented!()
        }

        async fn lister_semaines_banalisees(
            &self,
            _activite_id: i64,
        ) -> Result<Vec<crate::domain::planning::SemaineBanalisee>, AppError> {
            unimplemented!()
        }

        async fn verifier_conflit_creneaux(
            &self,
            _activite_id: i64,
            _annee_scolaire: &str,
            _jour_semaine: i64,
            _heure_debut: &str,
            _heure_fin: &str,
            _exclure_id: Option<i64>,
        ) -> Result<Vec<crate::domain::planning::CreneauActivite>, AppError> {
            unimplemented!()
        }

        async fn verifier_conflit_creneaux_tx(
            &self,
            _tx: &mut libsql::Transaction,
            _activite_id: i64,
            _annee_scolaire: &str,
            _jour_semaine: i64,
            _heure_debut: &str,
            _heure_fin: &str,
            _exclure_id: Option<i64>,
        ) -> Result<Vec<crate::domain::planning::CreneauActivite>, AppError> {
            unimplemented!()
        }

        async fn compter_inscrits_activite(
            &self,
            _activite_id: i64,
            _annee_scolaire: &str,
        ) -> Result<i64, AppError> {
            unimplemented!()
        }

        async fn compter_inscrits_activite_tx(
            &self,
            _tx: &mut libsql::Transaction,
            _activite_id: i64,
            _annee_scolaire: &str,
        ) -> Result<i64, AppError> {
            unimplemented!()
        }

        async fn verifier_collision(
            &self,
            _personne_id: i64,
            _activite_id: i64,
            _annee_scolaire: &str,
        ) -> Result<Option<Collision>, AppError> {
            Ok(self.collision.lock().unwrap().clone())
        }

        async fn verifier_collision_tx(
            &self,
            _tx: &mut libsql::Transaction,
            _personne_id: i64,
            _activite_id: i64,
            _annee_scolaire: &str,
        ) -> Result<Option<Collision>, AppError> {
            Ok(self.collision.lock().unwrap().clone())
        }

        async fn lister_creneaux_tx(
            &self,
            _tx: &mut libsql::Transaction,
            _activite_id: i64,
            _annee_scolaire: &str,
        ) -> Result<Vec<crate::domain::planning::CreneauActivite>, AppError> {
            unimplemented!()
        }

        async fn planning_personne_semaine(
            &self,
            _personne_id: i64,
            _date_lundi: &str,
            _annee_scolaire: &str,
        ) -> Result<Vec<PlanningCreneau>, AppError> {
            unimplemented!()
        }
    }

    async fn make_conn() -> Connection {
        libsql::Builder::new_local(":memory:")
            .build()
            .await
            .expect("failed to create test db")
            .connect()
            .expect("failed to connect test db")
    }

    fn make_service<'a>(
        activite_repo: &'a MockActiviteRepository,
        planning_repo: &'a MockPlanningRepository,
        conn: Connection,
    ) -> ActiviteService<'a, MockActiviteRepository, MockPlanningRepository> {
        ActiviteService::new(activite_repo, planning_repo, conn)
    }

    #[tokio::test]
    async fn test_ajouter_personne_valide_cree_liaison() {
        let repo = MockActiviteRepository::new();
        let planning = MockPlanningRepository::new();
        let service = make_service(&repo, &planning, make_conn().await);

        let activite = repo
            .create(
                CreateActivite {
                    nom: "Poterie".into(),
                    description: None,
                    capacite_max: None,
                    annee_scolaire: None,
                    tarif: None,
                },
                "alice",
            )
            .await
            .unwrap();

        let result = service
            .ajouter_personne(
                "alice",
                CreateLiaisonActivitePersonne {
                    activite_id: activite.id,
                    personne_id: 1,
                    annee_scolaire: "2025-2026".into(),
                    role: Role::Participant,
                },
            )
            .await;

        assert!(result.is_ok());
        let liaisons = repo.liaisons.lock().unwrap();
        assert_eq!(liaisons.len(), 1);
        assert_eq!(liaisons[0].personne_id, 1);
    }

    #[tokio::test]
    async fn test_ajouter_personne_avec_liaison_existante_retourne_erreur() {
        let repo = MockActiviteRepository::new();
        let planning = MockPlanningRepository::new();
        let service = make_service(&repo, &planning, make_conn().await);

        let activite = repo
            .create(
                CreateActivite {
                    nom: "Poterie".into(),
                    description: None,
                    capacite_max: None,
                    annee_scolaire: None,
                    tarif: None,
                },
                "alice",
            )
            .await
            .unwrap();

        let input = CreateLiaisonActivitePersonne {
            activite_id: activite.id,
            personne_id: 1,
            annee_scolaire: "2025-2026".into(),
            role: Role::Participant,
        };

        service
            .ajouter_personne("alice", input.clone())
            .await
            .unwrap();

        let result = service.ajouter_personne("alice", input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Conflict(msg) => assert!(msg.contains("déjà inscrite")),
            _ => panic!("expected Conflict error"),
        }
    }

    #[tokio::test]
    async fn test_ajouter_personne_capacite_atteinte_retourne_erreur() {
        let repo = MockActiviteRepository::avec_capacite(1);
        let planning = MockPlanningRepository::new();
        let service = make_service(&repo, &planning, make_conn().await);

        let activite = repo
            .create(
                CreateActivite {
                    nom: "Poterie".into(),
                    description: None,
                    capacite_max: None,
                    annee_scolaire: None,
                    tarif: None,
                },
                "alice",
            )
            .await
            .unwrap();

        service
            .ajouter_personne(
                "alice",
                CreateLiaisonActivitePersonne {
                    activite_id: activite.id,
                    personne_id: 1,
                    annee_scolaire: "2025-2026".into(),
                    role: Role::Participant,
                },
            )
            .await
            .unwrap();

        let result = service
            .ajouter_personne(
                "alice",
                CreateLiaisonActivitePersonne {
                    activite_id: activite.id,
                    personne_id: 2,
                    annee_scolaire: "2025-2026".into(),
                    role: Role::Participant,
                },
            )
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Validation(msg) => assert!(msg.contains("Capacité")),
            _ => panic!("expected Validation error"),
        }
    }

    #[tokio::test]
    async fn test_ajouter_personne_avec_collision_planning_retourne_erreur() {
        let repo = MockActiviteRepository::new();
        let planning = MockPlanningRepository::avec_collision();
        let service = make_service(&repo, &planning, make_conn().await);

        let activite = repo
            .create(
                CreateActivite {
                    nom: "Poterie".into(),
                    description: None,
                    capacite_max: None,
                    annee_scolaire: None,
                    tarif: None,
                },
                "alice",
            )
            .await
            .unwrap();

        let result = service
            .ajouter_personne(
                "alice",
                CreateLiaisonActivitePersonne {
                    activite_id: activite.id,
                    personne_id: 1,
                    annee_scolaire: "2025-2026".into(),
                    role: Role::Participant,
                },
            )
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Conflict(msg) => assert!(msg.contains("Conflit")),
            _ => panic!("expected Conflict error"),
        }
    }
}
