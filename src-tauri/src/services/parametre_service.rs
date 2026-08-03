use std::collections::HashMap;

use sqlx::SqlitePool;

use crate::domain::parametre::{
    trouver_place_deplacement, valider_plage_horaire, ImpactAction, ImpactCreneau,
    ParametresPlanning,
};
use crate::domain::planning::jour_semaine_texte;
use crate::error::AppError;
use crate::repositories::{ParametreRepository, PlanningRepository};

/// Service métier des paramètres de planning : gestion de la plage horaire d'ouverture
/// et du traitement des créneaux sortant de la plage lors d'une réduction.
pub struct ParametreService<'a, R: ParametreRepository, P: PlanningRepository> {
    param_repo: &'a R,
    planning_repo: &'a P,
    pool: SqlitePool,
}

impl<'a, R: ParametreRepository, P: PlanningRepository> ParametreService<'a, R, P> {
    pub fn new(param_repo: &'a R, planning_repo: &'a P, pool: SqlitePool) -> Self {
        Self {
            param_repo,
            planning_repo,
            pool,
        }
    }

    pub async fn obtenir_parametres(&self) -> Result<ParametresPlanning, AppError> {
        self.param_repo.obtenir_parametres_planning().await
    }

    /// Calcule l'impact d'une réduction de la plage horaire sur les créneaux existants, sans
    /// rien modifier en base.
    ///
    /// Règles :
    /// - créneau sortant de la plage **sans inscrits** → supprimé ;
    /// - créneau sortant avec **inscrits** → déplacé vers la place libre la plus proche,
    ///   sur le même jour, sans chevaucher les créneaux qui restent ni les créneaux déjà
    ///   déplacés de la même opération ;
    /// - aucune place libre → `DeplaceImpossible` (bloque la réduction).
    pub async fn apercu_impact_plage(
        &self,
        heure_ouverture: &str,
        heure_fermeture: &str,
    ) -> Result<Vec<ImpactCreneau>, AppError> {
        valider_plage_horaire(heure_ouverture, heure_fermeture).map_err(AppError::Validation)?;

        let hors_plage = self
            .planning_repo
            .lister_creneaux_hors_plage(heure_ouverture, heure_fermeture)
            .await?;
        if hors_plage.is_empty() {
            return Ok(Vec::new());
        }

        let tous = self.planning_repo.lister_tous_creneaux().await?;
        let hors_ids: std::collections::HashSet<i64> =
            hors_plage.iter().map(|h| h.creneau_id).collect();

        // Créneaux qui restent en place : ceux entièrement compris dans la plage.
        let mut fixes: HashMap<(String, i64, i64), Vec<(String, String)>> = HashMap::new();
        for c in &tous {
            if hors_ids.contains(&c.id) {
                continue;
            }
            fixes
                .entry((c.annee_scolaire.clone(), c.jour_semaine, c.activite_id))
                .or_default()
                .push((c.heure_debut.clone(), c.heure_fin.clone()));
        }

        let mut impacts: Vec<ImpactCreneau> = Vec::new();
        for hp in &hors_plage {
            let base = ImpactCreneau {
                creneau_id: hp.creneau_id,
                activite_id: hp.activite_id,
                activite_nom: hp.activite_nom.clone(),
                jour_semaine: hp.jour_semaine,
                heure_debut: hp.heure_debut.clone(),
                heure_fin: hp.heure_fin.clone(),
                annee_scolaire: hp.annee_scolaire.clone(),
                action: ImpactAction::Supprime,
                nouveau_debut: None,
                nouveau_fin: None,
                raison: None,
            };

            if hp.nb_inscrits == 0 {
                impacts.push(base);
                continue;
            }

            // Occupation : créneaux fixes + destinations déjà retenues pour le même
            // (année, jour) de la même activité (Option C : seuls les créneaux de la même
            // activité se gênent ; deux activités distinctes peuvent partager un créneau).
            let mut occupes = fixes
                .get(&(hp.annee_scolaire.clone(), hp.jour_semaine, hp.activite_id))
                .cloned()
                .unwrap_or_default();
            for imp in &impacts {
                if imp.action == ImpactAction::Deplace
                    && imp.annee_scolaire == hp.annee_scolaire
                    && imp.jour_semaine == hp.jour_semaine
                    && imp.activite_id == hp.activite_id
                {
                    if let (Some(d), Some(f)) = (&imp.nouveau_debut, &imp.nouveau_fin) {
                        occupes.push((d.clone(), f.clone()));
                    }
                }
            }

            match trouver_place_deplacement(
                &hp.heure_debut,
                &hp.heure_fin,
                heure_ouverture,
                heure_fermeture,
                &occupes,
            ) {
                Some((nouveau_debut, nouveau_fin)) => impacts.push(ImpactCreneau {
                    action: ImpactAction::Deplace,
                    nouveau_debut: Some(nouveau_debut),
                    nouveau_fin: Some(nouveau_fin),
                    ..base
                }),
                None => impacts.push(ImpactCreneau {
                    action: ImpactAction::DeplaceImpossible,
                    ..base
                }),
            }
        }

        // Contrôle adhérent : un adhérent d'un créneau déplacé ne doit pas se retrouver inscrit à
        // une autre activité (même année) dont un créneau — à l'état final — chevauche le nouvel
        // horaire (« elle pourrait très bien enchaîner les activités »). En cas de chevauchement,
        // le déplacement est marqué DeplaceImpossible et la réduction est bloquée.
        if impacts.iter().any(|i| i.action == ImpactAction::Deplace) {
            let inscriptions = self.planning_repo.lister_inscriptions().await?;

            // Positions finales : créneaux restants en place + destinations des créneaux déplacés ;
            // les créneaux supprimés sont retirés.
            let mut positions: HashMap<i64, (i64, String, String)> = HashMap::new();
            for c in &tous {
                positions.insert(
                    c.id,
                    (c.jour_semaine, c.heure_debut.clone(), c.heure_fin.clone()),
                );
            }
            for imp in &impacts {
                match imp.action {
                    ImpactAction::Supprime => {
                        positions.remove(&imp.creneau_id);
                    }
                    ImpactAction::Deplace => {
                        if let (Some(d), Some(f)) = (&imp.nouveau_debut, &imp.nouveau_fin) {
                            positions
                                .insert(imp.creneau_id, (imp.jour_semaine, d.clone(), f.clone()));
                        }
                    }
                    ImpactAction::DeplaceImpossible => {}
                }
            }

            let mut inscrits_activite: HashMap<(i64, String), Vec<i64>> = HashMap::new();
            let mut activites_personne: HashMap<(i64, String), Vec<i64>> = HashMap::new();
            let mut noms: HashMap<i64, String> = HashMap::new();
            for ins in &inscriptions {
                inscrits_activite
                    .entry((ins.activite_id, ins.annee_scolaire.clone()))
                    .or_default()
                    .push(ins.personne_id);
                activites_personne
                    .entry((ins.personne_id, ins.annee_scolaire.clone()))
                    .or_default()
                    .push(ins.activite_id);
                noms.entry(ins.activite_id)
                    .or_insert_with(|| ins.activite_nom.clone());
            }

            for imp in &mut impacts {
                if imp.action != ImpactAction::Deplace {
                    continue;
                }
                let (Some(nd), Some(nf)) = (&imp.nouveau_debut, &imp.nouveau_fin) else {
                    continue;
                };
                let inscrits = inscrits_activite
                    .get(&(imp.activite_id, imp.annee_scolaire.clone()))
                    .cloned()
                    .unwrap_or_default();
                for personne_id in inscrits {
                    let activites = activites_personne
                        .get(&(personne_id, imp.annee_scolaire.clone()))
                        .cloned()
                        .unwrap_or_default();
                    for autre_activite_id in activites {
                        if autre_activite_id == imp.activite_id {
                            continue;
                        }
                        for c in &tous {
                            if c.activite_id != autre_activite_id
                                || c.annee_scolaire != imp.annee_scolaire
                            {
                                continue;
                            }
                            let Some((jour, d, f)) = positions.get(&c.id) else {
                                continue;
                            };
                            if *jour == imp.jour_semaine
                                && nd.as_str() < f.as_str()
                                && nf.as_str() > d.as_str()
                            {
                                imp.action = ImpactAction::DeplaceImpossible;
                                imp.raison = Some(format!(
                                    "chevauche l'activité « {} » le {} ({}–{}) avec un adhérent déjà inscrit",
                                    noms.get(&autre_activite_id)
                                        .map(|s| s.as_str())
                                        .unwrap_or("?"),
                                    jour_semaine_texte(imp.jour_semaine),
                                    d,
                                    f,
                                ));
                                break;
                            }
                        }
                        if imp.action == ImpactAction::DeplaceImpossible {
                            break;
                        }
                    }
                    if imp.action == ImpactAction::DeplaceImpossible {
                        break;
                    }
                }
            }
        }

        Ok(impacts)
    }

    /// Applique une modification de la plage horaire, en transaction :
    /// suppressions des créneaux sans inscrits, déplacements des créneaux avec inscrits,
    /// puis mise à jour de la plage.
    ///
    /// Sans confirmation explicite (`confirmer_suppression`), toute modification qui impacte
    /// au moins un créneau est refusée. Une réduction est aussi refusée si un créneau avec
    /// inscrits ne peut pas être déplacé.
    pub async fn appliquer_plage(
        &self,
        heure_ouverture: &str,
        heure_fermeture: &str,
        confirmer_suppression: bool,
    ) -> Result<ParametresPlanning, AppError> {
        let impacts = self
            .apercu_impact_plage(heure_ouverture, heure_fermeture)
            .await?;

        if let Some(bloquant) = impacts
            .iter()
            .find(|i| i.action == ImpactAction::DeplaceImpossible)
        {
            let message = match &bloquant.raison {
                Some(raison) => format!(
                    "Impossible de réduire la plage horaire : le créneau {}–{} de l'activité « {} » (jour {}) ne peut pas être déplacé : {}.",
                    bloquant.heure_debut,
                    bloquant.heure_fin,
                    bloquant.activite_nom,
                    bloquant.jour_semaine,
                    raison,
                ),
                None => format!(
                    "Impossible de réduire la plage horaire : le créneau {}–{} de l'activité « {} » (jour {}) compte des inscrits et ne trouve aucune place libre dans la nouvelle plage. Élargissez la plage ou retirez d'abord les inscrits.",
                    bloquant.heure_debut,
                    bloquant.heure_fin,
                    bloquant.activite_nom,
                    bloquant.jour_semaine,
                ),
            };
            return Err(AppError::Conflict(message));
        }

        if !impacts.is_empty() && !confirmer_suppression {
            let nb_supprimes = impacts
                .iter()
                .filter(|i| i.action == ImpactAction::Supprime)
                .count();
            let nb_deplaces = impacts
                .iter()
                .filter(|i| i.action == ImpactAction::Deplace)
                .count();
            return Err(AppError::Conflict(format!(
                "La réduction de la plage horaire impacte {} créneau(x) ({} à supprimer, {} à déplacer). Confirmez pour appliquer.",
                impacts.len(),
                nb_supprimes,
                nb_deplaces
            )));
        }

        let mut tx = self.pool.begin().await?;

        for imp in &impacts {
            match imp.action {
                ImpactAction::Supprime => {
                    self.planning_repo
                        .supprimer_creneau_tx(&mut tx, imp.creneau_id)
                        .await?;
                }
                ImpactAction::Deplace => {
                    if let (Some(d), Some(f)) = (&imp.nouveau_debut, &imp.nouveau_fin) {
                        self.planning_repo
                            .deplacer_creneau_tx(&mut tx, imp.creneau_id, d, f)
                            .await?;
                    }
                }
                ImpactAction::DeplaceImpossible => {
                    // Refusé plus haut : ne doit pas arriver ici.
                    return Err(AppError::Conflict(
                        "Un créneau avec inscrits ne peut pas être déplacé".into(),
                    ));
                }
            }
        }

        let params = self
            .param_repo
            .mettre_a_jour_plage_horaire_tx(&mut tx, heure_ouverture, heure_fermeture)
            .await?;

        tx.commit().await?;
        Ok(params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Mutex;

    use crate::domain::planning::{
        CreateCreneau, CreateSemaineBanalisee, CreneauActivite, CreneauHorsPlage, Inscription,
    };
    use crate::domain::planning::{PlanningCreneau, SemaineBanalisee};

    struct MockParametreRepository {
        params: Mutex<ParametresPlanning>,
    }

    impl MockParametreRepository {
        fn new() -> Self {
            Self {
                params: Mutex::new(ParametresPlanning {
                    id: 1,
                    heure_ouverture: "08:00".to_string(),
                    heure_fermeture: "20:00".to_string(),
                }),
            }
        }
    }

    #[async_trait]
    impl ParametreRepository for MockParametreRepository {
        async fn obtenir_parametres_planning(&self) -> Result<ParametresPlanning, AppError> {
            Ok(self.params.lock().unwrap().clone())
        }

        async fn mettre_a_jour_plage_horaire_tx(
            &self,
            _tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
            heure_ouverture: &str,
            heure_fermeture: &str,
        ) -> Result<ParametresPlanning, AppError> {
            let mut params = self.params.lock().unwrap();
            params.heure_ouverture = heure_ouverture.to_string();
            params.heure_fermeture = heure_fermeture.to_string();
            Ok(params.clone())
        }
    }

    struct MockPlanningRepository {
        creneaux: Mutex<Vec<CreneauHorsPlage>>,
        inscriptions: Mutex<Vec<Inscription>>,
        suppressions: Mutex<Vec<i64>>,
        deplacements: Mutex<Vec<(i64, String, String)>>,
    }

    impl MockPlanningRepository {
        fn new(creneaux: Vec<CreneauHorsPlage>) -> Self {
            Self {
                creneaux: Mutex::new(creneaux),
                inscriptions: Mutex::new(Vec::new()),
                suppressions: Mutex::new(Vec::new()),
                deplacements: Mutex::new(Vec::new()),
            }
        }

        fn avec_inscriptions(
            creneaux: Vec<CreneauHorsPlage>,
            inscriptions: Vec<Inscription>,
        ) -> Self {
            Self {
                creneaux: Mutex::new(creneaux),
                inscriptions: Mutex::new(inscriptions),
                suppressions: Mutex::new(Vec::new()),
                deplacements: Mutex::new(Vec::new()),
            }
        }
    }

    fn hors_plage(
        id: i64,
        activite_id: i64,
        nom: &str,
        jour: i64,
        debut: &str,
        fin: &str,
        inscrits: i64,
    ) -> CreneauHorsPlage {
        CreneauHorsPlage {
            creneau_id: id,
            activite_id,
            activite_nom: nom.to_string(),
            jour_semaine: jour,
            heure_debut: debut.to_string(),
            heure_fin: fin.to_string(),
            annee_scolaire: "2025-2026".to_string(),
            nb_inscrits: inscrits,
        }
    }

    #[async_trait]
    impl PlanningRepository for MockPlanningRepository {
        async fn creer_creneau(&self, _input: CreateCreneau) -> Result<CreneauActivite, AppError> {
            unimplemented!()
        }

        async fn supprimer_creneau(&self, _id: i64) -> Result<(), AppError> {
            unimplemented!()
        }

        async fn modifier_creneau(
            &self,
            _id: i64,
            _input: CreateCreneau,
        ) -> Result<CreneauActivite, AppError> {
            unimplemented!()
        }

        async fn lister_creneaux(
            &self,
            _activite_id: i64,
            _annee_scolaire: &str,
        ) -> Result<Vec<CreneauActivite>, AppError> {
            unimplemented!()
        }

        async fn lister_tous_creneaux(&self) -> Result<Vec<CreneauActivite>, AppError> {
            Ok(self
                .creneaux
                .lock()
                .unwrap()
                .iter()
                .map(|h| CreneauActivite {
                    id: h.creneau_id,
                    activite_id: h.activite_id,
                    jour_semaine: h.jour_semaine,
                    heure_debut: h.heure_debut.clone(),
                    heure_fin: h.heure_fin.clone(),
                    annee_scolaire: h.annee_scolaire.clone(),
                })
                .collect())
        }

        async fn lister_creneaux_hors_plage(
            &self,
            heure_ouverture: &str,
            heure_fermeture: &str,
        ) -> Result<Vec<CreneauHorsPlage>, AppError> {
            let mut rows: Vec<CreneauHorsPlage> = self
                .creneaux
                .lock()
                .unwrap()
                .iter()
                .filter(|c| {
                    c.heure_debut.as_str() < heure_ouverture
                        || c.heure_fin.as_str() > heure_fermeture
                })
                .cloned()
                .collect();
            rows.sort_unstable_by_key(|c| c.creneau_id);
            Ok(rows)
        }

        async fn lister_inscriptions(&self) -> Result<Vec<Inscription>, AppError> {
            Ok(self.inscriptions.lock().unwrap().clone())
        }

        async fn supprimer_creneau_tx(
            &self,
            _tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
            id: i64,
        ) -> Result<(), AppError> {
            self.suppressions.lock().unwrap().push(id);
            self.creneaux.lock().unwrap().retain(|c| c.creneau_id != id);
            Ok(())
        }

        async fn deplacer_creneau_tx(
            &self,
            _tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
            id: i64,
            heure_debut: &str,
            heure_fin: &str,
        ) -> Result<CreneauActivite, AppError> {
            self.deplacements.lock().unwrap().push((
                id,
                heure_debut.to_string(),
                heure_fin.to_string(),
            ));
            let mut creneaux = self.creneaux.lock().unwrap();
            let c = creneaux
                .iter_mut()
                .find(|c| c.creneau_id == id)
                .ok_or(AppError::NotFound("Créneau introuvable".into()))?;
            c.heure_debut = heure_debut.to_string();
            c.heure_fin = heure_fin.to_string();
            Ok(CreneauActivite {
                id: c.creneau_id,
                activite_id: c.activite_id,
                jour_semaine: c.jour_semaine,
                heure_debut: c.heure_debut.clone(),
                heure_fin: c.heure_fin.clone(),
                annee_scolaire: c.annee_scolaire.clone(),
            })
        }

        async fn ajouter_semaine_banalisee(
            &self,
            _input: CreateSemaineBanalisee,
        ) -> Result<SemaineBanalisee, AppError> {
            unimplemented!()
        }

        async fn supprimer_semaine_banalisee(&self, _id: i64) -> Result<(), AppError> {
            unimplemented!()
        }

        async fn lister_semaines_banalisees(
            &self,
            _activite_id: i64,
        ) -> Result<Vec<SemaineBanalisee>, AppError> {
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
        ) -> Result<Vec<CreneauActivite>, AppError> {
            unimplemented!()
        }

        async fn compter_inscrits_activite(
            &self,
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
        ) -> Result<Option<crate::domain::planning::Collision>, AppError> {
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

    async fn make_pool() -> SqlitePool {
        SqlitePool::connect("sqlite::memory:")
            .await
            .expect("failed to create test pool")
    }

    fn make_service<'a, R: ParametreRepository, P: PlanningRepository>(
        param_repo: &'a R,
        planning_repo: &'a P,
        pool: SqlitePool,
    ) -> ParametreService<'a, R, P> {
        ParametreService::new(param_repo, planning_repo, pool)
    }

    #[tokio::test]
    async fn test_apercu_supprime_sans_inscrits_deplace_avec_inscrits() {
        let creneaux = vec![
            hors_plage(1, 1, "Poterie", 1, "07:00", "09:00", 0),
            hors_plage(2, 2, "Théâtre", 1, "19:00", "21:00", 2),
            hors_plage(3, 1, "Poterie", 1, "09:00", "11:00", 1),
        ];
        let param = MockParametreRepository::new();
        let planning = MockPlanningRepository::new(creneaux);
        let pool = make_pool().await;
        let service = make_service(&param, &planning, pool);

        let impacts = service
            .apercu_impact_plage("08:00", "20:00")
            .await
            .expect("apercu OK");

        assert_eq!(impacts.len(), 2);
        assert_eq!(impacts[0].creneau_id, 1);
        assert_eq!(impacts[0].action, ImpactAction::Supprime);
        assert_eq!(impacts[1].creneau_id, 2);
        assert_eq!(impacts[1].action, ImpactAction::Deplace);
        // 19:00-21:00 (120 min) déplacé au plus proche sans chevaucher 09:00-11:00 -> 18:00-20:00.
        assert_eq!(
            (
                impacts[1].nouveau_debut.as_deref(),
                impacts[1].nouveau_fin.as_deref()
            ),
            (Some("18:00"), Some("20:00"))
        );
    }

    #[tokio::test]
    async fn test_apercu_aucun_impact_si_creneaux_dans_plage() {
        let creneaux = vec![hors_plage(3, 1, "Poterie", 1, "09:00", "11:00", 1)];
        let param = MockParametreRepository::new();
        let planning = MockPlanningRepository::new(creneaux);
        let pool = make_pool().await;
        let service = make_service(&param, &planning, pool);

        let impacts = service
            .apercu_impact_plage("08:00", "20:00")
            .await
            .expect("apercu OK");
        assert!(impacts.is_empty());
    }

    #[tokio::test]
    async fn test_apercu_deplace_impossible_si_aucune_place() {
        let creneaux = vec![hors_plage(1, 1, "Poterie", 1, "08:00", "19:00", 3)];
        let param = MockParametreRepository::new();
        let planning = MockPlanningRepository::new(creneaux);
        let pool = make_pool().await;
        let service = make_service(&param, &planning, pool);

        let impacts = service
            .apercu_impact_plage("09:00", "18:00")
            .await
            .expect("apercu OK");

        assert_eq!(impacts.len(), 1);
        assert_eq!(impacts[0].action, ImpactAction::DeplaceImpossible);
    }

    #[tokio::test]
    async fn test_apercu_plage_invalide() {
        let param = MockParametreRepository::new();
        let planning = MockPlanningRepository::new(Vec::new());
        let pool = make_pool().await;
        let service = make_service(&param, &planning, pool);

        let err = service
            .apercu_impact_plage("20:00", "08:00")
            .await
            .expect_err("plage invalide");
        assert!(err.to_string().contains("avant l'heure de fermeture"));
    }

    #[tokio::test]
    async fn test_appliquer_sans_impact_sans_confirmation() {
        let creneaux = vec![hors_plage(3, 1, "Poterie", 1, "09:00", "11:00", 1)];
        let param = MockParametreRepository::new();
        let planning = MockPlanningRepository::new(creneaux);
        let pool = make_pool().await;
        let service = make_service(&param, &planning, pool);

        let params = service
            .appliquer_plage("08:00", "20:00", false)
            .await
            .expect("plage sans impact appliquée sans confirmation");
        assert_eq!(params.heure_ouverture, "08:00");
        assert_eq!(params.heure_fermeture, "20:00");
    }

    #[tokio::test]
    async fn test_appliquer_refuse_sans_confirmation() {
        let creneaux = vec![
            hors_plage(1, 1, "Poterie", 1, "07:00", "09:00", 0),
            hors_plage(2, 2, "Théâtre", 1, "19:00", "21:00", 2),
        ];
        let param = MockParametreRepository::new();
        let planning = MockPlanningRepository::new(creneaux);
        let pool = make_pool().await;
        let service = make_service(&param, &planning, pool);

        let err = service
            .appliquer_plage("08:00", "20:00", false)
            .await
            .expect_err("refusé sans confirmation");
        assert!(err.to_string().contains("Confirmez"));
    }

    #[tokio::test]
    async fn test_appliquer_avec_confirmation_supprime_et_deplace() {
        let creneaux = vec![
            hors_plage(1, 1, "Poterie", 1, "07:00", "09:00", 0),
            hors_plage(2, 2, "Théâtre", 1, "19:00", "21:00", 2),
            hors_plage(3, 1, "Poterie", 1, "09:00", "11:00", 1),
        ];
        let param = MockParametreRepository::new();
        let planning = MockPlanningRepository::new(creneaux);
        let pool = make_pool().await;
        let service = make_service(&param, &planning, pool);

        let params = service
            .appliquer_plage("08:00", "20:00", true)
            .await
            .expect("application OK");

        assert_eq!(params.heure_ouverture, "08:00");
        assert_eq!(params.heure_fermeture, "20:00");
        assert_eq!(*planning.suppressions.lock().unwrap(), vec![1]);
        assert_eq!(
            *planning.deplacements.lock().unwrap(),
            vec![(2, "18:00".to_string(), "20:00".to_string())]
        );
    }

    #[tokio::test]
    async fn test_appliquer_refuse_si_deplacement_impossible() {
        let creneaux = vec![hors_plage(1, 1, "Poterie", 1, "08:00", "19:00", 3)];
        let param = MockParametreRepository::new();
        let planning = MockPlanningRepository::new(creneaux);
        let pool = make_pool().await;
        let service = make_service(&param, &planning, pool);

        let err = service
            .appliquer_plage("09:00", "18:00", true)
            .await
            .expect_err("bloqué si aucun déplacement possible");
        assert!(err.to_string().contains("aucune place libre"));
    }

    fn inscription(activite_id: i64, personne_id: i64, nom: &str) -> Inscription {
        Inscription {
            activite_id,
            personne_id,
            annee_scolaire: "2025-2026".to_string(),
            activite_nom: nom.to_string(),
        }
    }

    #[tokio::test]
    async fn test_deplacement_ignore_creneaux_autres_activites() {
        let creneaux = vec![
            hors_plage(1, 1, "Poterie", 1, "08:30", "19:00", 1),
            hors_plage(2, 2, "Théâtre", 1, "07:00", "09:00", 2),
        ];
        let param = MockParametreRepository::new();
        let planning = MockPlanningRepository::new(creneaux);
        let pool = make_pool().await;
        let service = make_service(&param, &planning, pool);

        let impacts = service
            .apercu_impact_plage("08:00", "19:00")
            .await
            .expect("apercu OK");

        assert_eq!(impacts.len(), 1);
        assert_eq!(impacts[0].activite_nom, "Théâtre");
        assert_eq!(impacts[0].action, ImpactAction::Deplace);
        assert_eq!(
            (
                impacts[0].nouveau_debut.as_deref(),
                impacts[0].nouveau_fin.as_deref()
            ),
            (Some("08:00"), Some("10:00"))
        );
    }

    #[tokio::test]
    async fn test_deplacement_bloque_par_creneau_meme_activite() {
        let creneaux = vec![
            hors_plage(1, 1, "Poterie", 1, "08:30", "19:00", 3),
            hors_plage(2, 1, "Poterie", 1, "07:00", "09:00", 3),
        ];
        let param = MockParametreRepository::new();
        let planning = MockPlanningRepository::new(creneaux);
        let pool = make_pool().await;
        let service = make_service(&param, &planning, pool);

        let impacts = service
            .apercu_impact_plage("08:00", "19:00")
            .await
            .expect("apercu OK");

        assert_eq!(impacts.len(), 1);
        assert_eq!(impacts[0].activite_nom, "Poterie");
        assert_eq!(impacts[0].action, ImpactAction::DeplaceImpossible);
        assert!(impacts[0].raison.is_none());
    }

    #[tokio::test]
    async fn test_deplacement_refuse_si_adherent_deja_inscrit_autre_activite() {
        let creneaux = vec![
            hors_plage(1, 1, "Poterie", 1, "08:30", "10:30", 1),
            hors_plage(2, 2, "Théâtre", 1, "07:00", "09:00", 2),
        ];
        let inscrits = vec![inscription(1, 7, "Poterie"), inscription(2, 7, "Théâtre")];
        let param = MockParametreRepository::new();
        let planning = MockPlanningRepository::avec_inscriptions(creneaux, inscrits);
        let pool = make_pool().await;
        let service = make_service(&param, &planning, pool);

        let impacts = service
            .apercu_impact_plage("08:00", "19:00")
            .await
            .expect("apercu OK");

        assert_eq!(impacts.len(), 1);
        assert_eq!(impacts[0].activite_nom, "Théâtre");
        assert_eq!(impacts[0].action, ImpactAction::DeplaceImpossible);
        let raison = impacts[0].raison.as_deref().expect("raison présente");
        assert!(raison.contains("Poterie"));
    }

    #[tokio::test]
    async fn test_deplacement_autorise_si_adherent_autre_activite_sans_chevauchement() {
        let creneaux = vec![
            hors_plage(1, 1, "Poterie", 1, "10:00", "12:00", 1),
            hors_plage(2, 2, "Théâtre", 1, "07:00", "09:00", 2),
        ];
        let inscrits = vec![inscription(1, 7, "Poterie"), inscription(2, 7, "Théâtre")];
        let param = MockParametreRepository::new();
        let planning = MockPlanningRepository::avec_inscriptions(creneaux, inscrits);
        let pool = make_pool().await;
        let service = make_service(&param, &planning, pool);

        let impacts = service
            .apercu_impact_plage("08:00", "19:00")
            .await
            .expect("apercu OK");

        assert_eq!(impacts.len(), 1);
        assert_eq!(impacts[0].activite_nom, "Théâtre");
        assert_eq!(impacts[0].action, ImpactAction::Deplace);
        assert!(impacts[0].raison.is_none());
        assert_eq!(
            (
                impacts[0].nouveau_debut.as_deref(),
                impacts[0].nouveau_fin.as_deref()
            ),
            (Some("08:00"), Some("10:00"))
        );
    }

    #[tokio::test]
    async fn test_apercu_reduction_heure_debut_creneau_avec_inscrits() {
        let creneaux = vec![hors_plage(1, 1, "Poterie", 1, "08:00", "09:00", 2)];
        let param = MockParametreRepository::new();
        let planning = MockPlanningRepository::new(creneaux);
        let pool = make_pool().await;
        let service = make_service(&param, &planning, pool);

        let impacts = service
            .apercu_impact_plage("09:00", "20:00")
            .await
            .expect("apercu OK");

        assert_eq!(impacts.len(), 1);
        assert_eq!(impacts[0].action, ImpactAction::Deplace);
        assert_eq!(
            (
                impacts[0].nouveau_debut.as_deref(),
                impacts[0].nouveau_fin.as_deref()
            ),
            (Some("09:00"), Some("10:00"))
        );
    }

    #[tokio::test]
    async fn test_apercu_reduction_heure_debut_creneau_sans_inscrits() {
        let creneaux = vec![hors_plage(1, 1, "Poterie", 1, "08:00", "09:00", 0)];
        let param = MockParametreRepository::new();
        let planning = MockPlanningRepository::new(creneaux);
        let pool = make_pool().await;
        let service = make_service(&param, &planning, pool);

        let impacts = service
            .apercu_impact_plage("09:00", "20:00")
            .await
            .expect("apercu OK");

        assert_eq!(impacts.len(), 1);
        assert_eq!(impacts[0].action, ImpactAction::Supprime);
    }
}
