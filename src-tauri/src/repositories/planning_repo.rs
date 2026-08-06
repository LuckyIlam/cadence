use async_trait::async_trait;
use libsql::Connection;

use crate::domain::activite::Role;
use crate::domain::planning::{
    Collision, CreateCreneau, CreateSemaineBanalisee, CreneauActivite, CreneauHorsPlage,
    Inscription, PlanningCreneau, SemaineBanalisee,
};
use crate::error::AppError;
use crate::infrastructure::hrana_guard;

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
        tx: &mut libsql::Transaction,
        id: i64,
    ) -> Result<(), AppError>;
    async fn deplacer_creneau_tx(
        &self,
        tx: &mut libsql::Transaction,
        id: i64,
        heure_debut: &str,
        heure_fin: &str,
        utilisateur: &str,
    ) -> Result<CreneauActivite, AppError>;
    async fn creer_creneau_tx(
        &self,
        tx: &mut libsql::Transaction,
        input: CreateCreneau,
        utilisateur: &str,
    ) -> Result<CreneauActivite, AppError>;
    async fn modifier_creneau_tx(
        &self,
        tx: &mut libsql::Transaction,
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
        tx: &mut libsql::Transaction,
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
        tx: &mut libsql::Transaction,
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
        tx: &mut libsql::Transaction,
        personne_id: i64,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<Option<Collision>, AppError>;
    async fn lister_creneaux_tx(
        &self,
        tx: &mut libsql::Transaction,
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

pub struct LibsqlPlanningRepository {
    pub(crate) conn: Connection,
}

impl LibsqlPlanningRepository {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
struct CompteurRow {
    count: i64,
}

#[async_trait]
impl PlanningRepository for LibsqlPlanningRepository {
    async fn creer_creneau(
        &self,
        input: CreateCreneau,
        utilisateur: &str,
    ) -> Result<CreneauActivite, AppError> {
        let maintenant = crate::infrastructure::audit::maintenant_utc();
        let mut rows = hrana_guard::query_avec_retry(
            &self.conn,
                "INSERT INTO creneaux_activite (activite_id, jour_semaine, heure_debut, heure_fin, annee_scolaire, modifie_par, modifie_le)
                 VALUES (?, ?, ?, ?, ?, ?, ?)
                 RETURNING id, activite_id, jour_semaine, heure_debut, heure_fin, annee_scolaire, version",
                libsql::params![
                    input.activite_id,
                    input.jour_semaine,
                    input.heure_debut,
                    input.heure_fin,
                    input.annee_scolaire,
                    utilisateur,
                    maintenant
                ],
            )
            .await?;

        let row = rows
            .next()
            .await?
            .ok_or(AppError::NotFound("Créneau introuvable".into()))?;
        let valeur = libsql::de::from_row::<CreneauActivite>(&row)?;
        hrana_guard::vider_cursor(&mut rows).await?;
        Ok(valeur)
    }

    async fn creer_creneau_tx(
        &self,
        tx: &mut libsql::Transaction,
        input: CreateCreneau,
        utilisateur: &str,
    ) -> Result<CreneauActivite, AppError> {
        let maintenant = crate::infrastructure::audit::maintenant_utc();
        let mut rows = tx
            .query(
                "INSERT INTO creneaux_activite (activite_id, jour_semaine, heure_debut, heure_fin, annee_scolaire, modifie_par, modifie_le)
                 VALUES (?, ?, ?, ?, ?, ?, ?)
                 RETURNING id, activite_id, jour_semaine, heure_debut, heure_fin, annee_scolaire, version",
                libsql::params![
                    input.activite_id,
                    input.jour_semaine,
                    input.heure_debut,
                    input.heure_fin,
                    input.annee_scolaire,
                    utilisateur,
                    maintenant
                ],
            )
            .await?;

        let row = rows
            .next()
            .await?
            .ok_or(AppError::NotFound("Créneau introuvable".into()))?;
        let valeur = libsql::de::from_row::<CreneauActivite>(&row)?;
        hrana_guard::vider_cursor(&mut rows).await?;
        Ok(valeur)
    }

    async fn supprimer_creneau(&self, id: i64) -> Result<(), AppError> {
        hrana_guard::execute_avec_retry(
            &self.conn,
            "DELETE FROM creneaux_activite WHERE id = ?",
            libsql::params![id],
        )
        .await?;

        Ok(())
    }

    async fn modifier_creneau(
        &self,
        id: i64,
        input: CreateCreneau,
        version: i64,
        utilisateur: &str,
    ) -> Result<CreneauActivite, AppError> {
        let maintenant = crate::infrastructure::audit::maintenant_utc();
        let affected = hrana_guard::execute_avec_retry(
            &self.conn,
            "UPDATE creneaux_activite
                 SET jour_semaine = ?, heure_debut = ?, heure_fin = ?,
                     modifie_par = ?, modifie_le = ?, version = version + 1
                 WHERE id = ? AND version = ?",
            libsql::params![
                input.jour_semaine,
                input.heure_debut,
                input.heure_fin,
                utilisateur,
                maintenant,
                id,
                version
            ],
        )
        .await?;
        if affected == 0 {
            let mut existe_rows = hrana_guard::query_avec_retry(
                &self.conn,
                "SELECT id FROM creneaux_activite WHERE id = ?",
                libsql::params![id],
            )
            .await?;
            let existe = existe_rows.next().await?.is_some();
            hrana_guard::vider_cursor(&mut existe_rows).await?;
            if existe {
                return Err(AppError::Conflict(
                    crate::infrastructure::audit::MESSAGE_CONFLIT.to_string(),
                ));
            }
            return Err(AppError::NotFound("Créneau introuvable".into()));
        }
        let mut rows = hrana_guard::query_avec_retry(
            &self.conn,
            "SELECT id, activite_id, jour_semaine, heure_debut, heure_fin, annee_scolaire, version
                 FROM creneaux_activite WHERE id = ?",
            libsql::params![id],
        )
        .await?;
        let row = rows
            .next()
            .await?
            .ok_or(AppError::NotFound("Créneau introuvable".into()))?;
        let valeur = libsql::de::from_row::<CreneauActivite>(&row)?;
        hrana_guard::vider_cursor(&mut rows).await?;
        Ok(valeur)
    }

    async fn modifier_creneau_tx(
        &self,
        tx: &mut libsql::Transaction,
        id: i64,
        input: CreateCreneau,
        version: i64,
        utilisateur: &str,
    ) -> Result<CreneauActivite, AppError> {
        let maintenant = crate::infrastructure::audit::maintenant_utc();
        let affected = tx
            .execute(
                "UPDATE creneaux_activite
                 SET jour_semaine = ?, heure_debut = ?, heure_fin = ?,
                     modifie_par = ?, modifie_le = ?, version = version + 1
                 WHERE id = ? AND version = ?",
                libsql::params![
                    input.jour_semaine,
                    input.heure_debut,
                    input.heure_fin,
                    utilisateur,
                    maintenant,
                    id,
                    version
                ],
            )
            .await?;
        if affected == 0 {
            let mut existe_rows = tx
                .query(
                    "SELECT id FROM creneaux_activite WHERE id = ?",
                    libsql::params![id],
                )
                .await?;
            let existe = existe_rows.next().await?.is_some();
            hrana_guard::vider_cursor(&mut existe_rows).await?;
            if existe {
                return Err(AppError::Conflict(
                    crate::infrastructure::audit::MESSAGE_CONFLIT.to_string(),
                ));
            }
            return Err(AppError::NotFound("Créneau introuvable".into()));
        }
        let mut rows = tx
            .query(
                "SELECT id, activite_id, jour_semaine, heure_debut, heure_fin, annee_scolaire, version
                 FROM creneaux_activite WHERE id = ?",
                libsql::params![id],
            )
            .await?;
        let row = rows
            .next()
            .await?
            .ok_or(AppError::NotFound("Créneau introuvable".into()))?;
        let valeur = libsql::de::from_row::<CreneauActivite>(&row)?;
        hrana_guard::vider_cursor(&mut rows).await?;
        Ok(valeur)
    }

    async fn lister_creneaux(
        &self,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<Vec<CreneauActivite>, AppError> {
        let mut rows = hrana_guard::query_avec_retry(
            &self.conn,
            "SELECT id, activite_id, jour_semaine, heure_debut, heure_fin, annee_scolaire, version
                 FROM creneaux_activite
                 WHERE activite_id = ? AND annee_scolaire = ?
                 ORDER BY jour_semaine, heure_debut",
            libsql::params![activite_id, annee_scolaire],
        )
        .await?;

        let mut donnees = Vec::new();
        while let Some(row) = rows.next().await? {
            donnees.push(libsql::de::from_row::<CreneauActivite>(&row)?);
        }

        Ok(donnees)
    }

    async fn lister_creneaux_tx(
        &self,
        tx: &mut libsql::Transaction,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<Vec<CreneauActivite>, AppError> {
        let mut rows = tx
            .query(
                "SELECT id, activite_id, jour_semaine, heure_debut, heure_fin, annee_scolaire, version
                 FROM creneaux_activite
                 WHERE activite_id = ? AND annee_scolaire = ?
                 ORDER BY jour_semaine, heure_debut",
                libsql::params![activite_id, annee_scolaire],
            )
            .await?;

        let mut donnees = Vec::new();
        while let Some(row) = rows.next().await? {
            donnees.push(libsql::de::from_row::<CreneauActivite>(&row)?);
        }

        Ok(donnees)
    }

    async fn lister_tous_creneaux(&self) -> Result<Vec<CreneauActivite>, AppError> {
        let mut rows = hrana_guard::query_avec_retry(
            &self.conn,
            "SELECT id, activite_id, jour_semaine, heure_debut, heure_fin, annee_scolaire, version
                 FROM creneaux_activite ORDER BY id",
            libsql::params![],
        )
        .await?;

        let mut donnees = Vec::new();
        while let Some(row) = rows.next().await? {
            donnees.push(libsql::de::from_row::<CreneauActivite>(&row)?);
        }

        Ok(donnees)
    }

    async fn lister_creneaux_hors_plage(
        &self,
        heure_ouverture: &str,
        heure_fermeture: &str,
    ) -> Result<Vec<CreneauHorsPlage>, AppError> {
        let mut rows = hrana_guard::query_avec_retry(
            &self.conn,
            "SELECT c.id AS creneau_id,
                        c.activite_id AS activite_id,
                        a.nom AS activite_nom,
                        c.jour_semaine AS jour_semaine,
                        c.heure_debut AS heure_debut,
                        c.heure_fin AS heure_fin,
                        c.annee_scolaire AS annee_scolaire,
                        (SELECT COUNT(*) FROM activite_personnes ap
                         WHERE ap.activite_id = c.activite_id
                           AND ap.annee_scolaire = c.annee_scolaire) AS nb_inscrits
                 FROM creneaux_activite c
                 JOIN activites a ON a.id = c.activite_id
                 WHERE c.heure_debut < ? OR c.heure_fin > ?
                 ORDER BY c.id",
            libsql::params![heure_ouverture, heure_fermeture],
        )
        .await?;

        let mut donnees = Vec::new();
        while let Some(row) = rows.next().await? {
            donnees.push(libsql::de::from_row::<CreneauHorsPlage>(&row)?);
        }

        Ok(donnees)
    }

    async fn lister_inscriptions(&self) -> Result<Vec<Inscription>, AppError> {
        let mut rows = hrana_guard::query_avec_retry(
            &self.conn,
            "SELECT ap.activite_id AS activite_id,
                        ap.personne_id AS personne_id,
                        ap.annee_scolaire AS annee_scolaire,
                        a.nom AS activite_nom
                 FROM activite_personnes ap
                 JOIN activites a ON a.id = ap.activite_id
                 ORDER BY ap.activite_id, ap.personne_id",
            libsql::params![],
        )
        .await?;

        let mut donnees = Vec::new();
        while let Some(row) = rows.next().await? {
            donnees.push(libsql::de::from_row::<Inscription>(&row)?);
        }

        Ok(donnees)
    }

    async fn supprimer_creneau_tx(
        &self,
        tx: &mut libsql::Transaction,
        id: i64,
    ) -> Result<(), AppError> {
        tx.execute(
            "DELETE FROM creneaux_activite WHERE id = ?",
            libsql::params![id],
        )
        .await?;

        Ok(())
    }

    async fn deplacer_creneau_tx(
        &self,
        tx: &mut libsql::Transaction,
        id: i64,
        heure_debut: &str,
        heure_fin: &str,
        utilisateur: &str,
    ) -> Result<CreneauActivite, AppError> {
        let maintenant = crate::infrastructure::audit::maintenant_utc();
        let mut rows = tx
            .query(
                "UPDATE creneaux_activite
                 SET heure_debut = ?, heure_fin = ?, modifie_par = ?, modifie_le = ?
                 WHERE id = ?
                 RETURNING id, activite_id, jour_semaine, heure_debut, heure_fin, annee_scolaire, version",
                libsql::params![heure_debut, heure_fin, utilisateur, maintenant, id],
            )
            .await?;

        let row = rows
            .next()
            .await?
            .ok_or(AppError::NotFound("Créneau introuvable".into()))?;
        let valeur = libsql::de::from_row::<CreneauActivite>(&row)?;
        hrana_guard::vider_cursor(&mut rows).await?;
        Ok(valeur)
    }

    async fn ajouter_semaine_banalisee(
        &self,
        input: CreateSemaineBanalisee,
        utilisateur: &str,
    ) -> Result<SemaineBanalisee, AppError> {
        let maintenant = crate::infrastructure::audit::maintenant_utc();
        let mut rows = hrana_guard::query_avec_retry(
            &self.conn,
                "INSERT INTO semaines_banalisees (activite_id, date_debut, motif, annee_scolaire, modifie_par, modifie_le)
                 VALUES (?, ?, ?, ?, ?, ?)
                 RETURNING id, activite_id, date_debut, motif, annee_scolaire",
                libsql::params![
                    input.activite_id,
                    input.date_debut,
                    input.motif,
                    input.annee_scolaire,
                    utilisateur,
                    maintenant
                ],
            )
            .await?;

        let row = rows
            .next()
            .await?
            .ok_or(AppError::NotFound("Semaine banalisée introuvable".into()))?;
        let valeur = libsql::de::from_row::<SemaineBanalisee>(&row)?;
        hrana_guard::vider_cursor(&mut rows).await?;
        Ok(valeur)
    }

    async fn supprimer_semaine_banalisee(&self, id: i64) -> Result<(), AppError> {
        hrana_guard::execute_avec_retry(
            &self.conn,
            "DELETE FROM semaines_banalisees WHERE id = ?",
            libsql::params![id],
        )
        .await?;

        Ok(())
    }

    async fn lister_semaines_banalisees(
        &self,
        activite_id: i64,
    ) -> Result<Vec<SemaineBanalisee>, AppError> {
        let mut rows = hrana_guard::query_avec_retry(
            &self.conn,
            "SELECT id, activite_id, date_debut, motif, annee_scolaire
                 FROM semaines_banalisees
                 WHERE activite_id = ?
                 ORDER BY date_debut",
            libsql::params![activite_id],
        )
        .await?;

        let mut donnees = Vec::new();
        while let Some(row) = rows.next().await? {
            donnees.push(libsql::de::from_row::<SemaineBanalisee>(&row)?);
        }

        Ok(donnees)
    }

    async fn verifier_conflit_creneaux(
        &self,
        activite_id: i64,
        annee_scolaire: &str,
        jour_semaine: i64,
        heure_debut: &str,
        heure_fin: &str,
        exclure_id: Option<i64>,
    ) -> Result<Vec<CreneauActivite>, AppError> {
        let mut rows = hrana_guard::query_avec_retry(
            &self.conn,
            "SELECT id, activite_id, jour_semaine, heure_debut, heure_fin, annee_scolaire, version
                 FROM creneaux_activite
                 WHERE activite_id = ?
                   AND annee_scolaire = ?
                   AND jour_semaine = ?
                   AND heure_debut < ?
                   AND heure_fin > ?
                   AND (? IS NULL OR id != ?)",
            libsql::params![
                activite_id,
                annee_scolaire,
                jour_semaine,
                heure_fin,
                heure_debut,
                exclure_id,
                exclure_id
            ],
        )
        .await?;

        let mut donnees = Vec::new();
        while let Some(row) = rows.next().await? {
            donnees.push(libsql::de::from_row::<CreneauActivite>(&row)?);
        }

        Ok(donnees)
    }

    #[allow(clippy::too_many_arguments)]
    async fn verifier_conflit_creneaux_tx(
        &self,
        tx: &mut libsql::Transaction,
        activite_id: i64,
        annee_scolaire: &str,
        jour_semaine: i64,
        heure_debut: &str,
        heure_fin: &str,
        exclure_id: Option<i64>,
    ) -> Result<Vec<CreneauActivite>, AppError> {
        let mut rows = tx
            .query(
                "SELECT id, activite_id, jour_semaine, heure_debut, heure_fin, annee_scolaire, version
                 FROM creneaux_activite
                 WHERE activite_id = ?
                   AND annee_scolaire = ?
                   AND jour_semaine = ?
                   AND heure_debut < ?
                   AND heure_fin > ?
                   AND (? IS NULL OR id != ?)",
                libsql::params![
                    activite_id,
                    annee_scolaire,
                    jour_semaine,
                    heure_fin,
                    heure_debut,
                    exclure_id,
                    exclure_id
                ],
            )
            .await?;

        let mut donnees = Vec::new();
        while let Some(row) = rows.next().await? {
            donnees.push(libsql::de::from_row::<CreneauActivite>(&row)?);
        }

        Ok(donnees)
    }

    async fn compter_inscrits_activite(
        &self,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<i64, AppError> {
        let mut rows = hrana_guard::query_avec_retry(
            &self.conn,
            "SELECT COUNT(*) AS count FROM activite_personnes
                 WHERE activite_id = ? AND annee_scolaire = ?",
            libsql::params![activite_id, annee_scolaire],
        )
        .await?;

        let row = rows
            .next()
            .await?
            .ok_or(AppError::Database("Aucune ligne de comptage".into()))?;
        let count = libsql::de::from_row::<CompteurRow>(&row)?.count;
        hrana_guard::vider_cursor(&mut rows).await?;
        Ok(count)
    }

    async fn compter_inscrits_activite_tx(
        &self,
        tx: &mut libsql::Transaction,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<i64, AppError> {
        let mut rows = tx
            .query(
                "SELECT COUNT(*) AS count FROM activite_personnes
                 WHERE activite_id = ? AND annee_scolaire = ?",
                libsql::params![activite_id, annee_scolaire],
            )
            .await?;

        let row = rows
            .next()
            .await?
            .ok_or(AppError::Database("Aucune ligne de comptage".into()))?;
        let count = libsql::de::from_row::<CompteurRow>(&row)?.count;
        hrana_guard::vider_cursor(&mut rows).await?;
        Ok(count)
    }

    async fn verifier_collision(
        &self,
        personne_id: i64,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<Option<Collision>, AppError> {
        #[derive(Debug, Clone, serde::Deserialize)]
        struct AutreActiviteRow {
            activite_id: i64,
        }

        #[derive(Debug, Clone, serde::Deserialize)]
        struct NomActiviteRow {
            nom: String,
        }

        let creneaux_cibles = self.lister_creneaux(activite_id, annee_scolaire).await?;
        if creneaux_cibles.is_empty() {
            return Ok(None);
        }

        let mut rows = hrana_guard::query_avec_retry(
            &self.conn,
            "SELECT activite_id FROM activite_personnes
                 WHERE personne_id = ? AND annee_scolaire = ? AND activite_id != ?",
            libsql::params![personne_id, annee_scolaire, activite_id],
        )
        .await?;

        let mut autres_activites = Vec::new();
        while let Some(row) = rows.next().await? {
            autres_activites.push(libsql::de::from_row::<AutreActiviteRow>(&row)?.activite_id);
        }

        for autre_id in autres_activites {
            let creneaux_autre = self.lister_creneaux(autre_id, annee_scolaire).await?;
            for cible in &creneaux_cibles {
                for autre in &creneaux_autre {
                    if cible.jour_semaine == autre.jour_semaine
                        && cible.heure_debut < autre.heure_fin
                        && cible.heure_fin > autre.heure_debut
                    {
                        let mut nom_rows = self
                            .conn
                            .query(
                                "SELECT nom FROM activites WHERE id = ?",
                                libsql::params![autre_id],
                            )
                            .await?;
                        let nom_row = nom_rows
                            .next()
                            .await?
                            .ok_or(AppError::NotFound("Activité introuvable".into()))?;
                        let nom = libsql::de::from_row::<NomActiviteRow>(&nom_row)?.nom;
                        hrana_guard::vider_cursor(&mut nom_rows).await?;

                        return Ok(Some(Collision {
                            activite_conflit: nom,
                            jour_semaine: cible.jour_semaine,
                            heure_debut: cible.heure_debut.clone(),
                            heure_fin: cible.heure_fin.clone(),
                        }));
                    }
                }
            }
        }

        Ok(None)
    }

    async fn verifier_collision_tx(
        &self,
        tx: &mut libsql::Transaction,
        personne_id: i64,
        activite_id: i64,
        annee_scolaire: &str,
    ) -> Result<Option<Collision>, AppError> {
        #[derive(Debug, Clone, serde::Deserialize)]
        struct AutreActiviteRow {
            activite_id: i64,
        }

        #[derive(Debug, Clone, serde::Deserialize)]
        struct NomActiviteRow {
            nom: String,
        }

        let creneaux_cibles = self
            .lister_creneaux_tx(tx, activite_id, annee_scolaire)
            .await?;
        if creneaux_cibles.is_empty() {
            return Ok(None);
        }

        let mut rows = tx
            .query(
                "SELECT activite_id FROM activite_personnes
                 WHERE personne_id = ? AND annee_scolaire = ? AND activite_id != ?",
                libsql::params![personne_id, annee_scolaire, activite_id],
            )
            .await?;

        let mut autres_activites = Vec::new();
        while let Some(row) = rows.next().await? {
            autres_activites.push(libsql::de::from_row::<AutreActiviteRow>(&row)?.activite_id);
        }

        for autre_id in autres_activites {
            let creneaux_autre = self
                .lister_creneaux_tx(tx, autre_id, annee_scolaire)
                .await?;
            for cible in &creneaux_cibles {
                for autre in &creneaux_autre {
                    if cible.jour_semaine == autre.jour_semaine
                        && cible.heure_debut < autre.heure_fin
                        && cible.heure_fin > autre.heure_debut
                    {
                        let mut nom_rows = tx
                            .query(
                                "SELECT nom FROM activites WHERE id = ?",
                                libsql::params![autre_id],
                            )
                            .await?;
                        let nom_row = nom_rows
                            .next()
                            .await?
                            .ok_or(AppError::NotFound("Activité introuvable".into()))?;
                        let nom = libsql::de::from_row::<NomActiviteRow>(&nom_row)?.nom;
                        hrana_guard::vider_cursor(&mut nom_rows).await?;

                        return Ok(Some(Collision {
                            activite_conflit: nom,
                            jour_semaine: cible.jour_semaine,
                            heure_debut: cible.heure_debut.clone(),
                            heure_fin: cible.heure_fin.clone(),
                        }));
                    }
                }
            }
        }

        Ok(None)
    }

    async fn planning_personne_semaine(
        &self,
        personne_id: i64,
        date_lundi: &str,
        annee_scolaire: &str,
    ) -> Result<Vec<PlanningCreneau>, AppError> {
        #[derive(Debug, Clone, serde::Deserialize)]
        struct ActiviteCreneauRow {
            activite_id: i64,
            nom: String,
            description: Option<String>,
            capacite_max: Option<i64>,
            activite_version: i64,
            creneau_id: i64,
            jour_semaine: i64,
            heure_debut: String,
            heure_fin: String,
            annee_scolaire: String,
            creneau_version: i64,
            role: Role,
        }

        let mut rows = hrana_guard::query_avec_retry(
            &self.conn,
                "SELECT a.id AS activite_id, a.nom, a.description, a.capacite_max, a.version AS activite_version,
                        c.id AS creneau_id, c.jour_semaine, c.heure_debut, c.heure_fin, c.annee_scolaire,
                        c.version AS creneau_version,
                        ap.role
                 FROM activite_personnes ap
                 JOIN activites a ON a.id = ap.activite_id
                 JOIN creneaux_activite c ON c.activite_id = a.id
                 WHERE ap.personne_id = ?
                   AND c.annee_scolaire = ?
                   AND ap.annee_scolaire = ?
                   AND NOT EXISTS (
                       SELECT 1 FROM semaines_banalisees sb
                       WHERE sb.activite_id = a.id AND sb.date_debut = ?
                   )
                 ORDER BY c.jour_semaine, c.heure_debut",
                libsql::params![personne_id, annee_scolaire, annee_scolaire, date_lundi],
            )
            .await?;

        let mut donnees = Vec::new();
        while let Some(row) = rows.next().await? {
            let r = libsql::de::from_row::<ActiviteCreneauRow>(&row)?;
            donnees.push(PlanningCreneau {
                creneau: CreneauActivite {
                    id: r.creneau_id,
                    activite_id: r.activite_id,
                    jour_semaine: r.jour_semaine,
                    heure_debut: r.heure_debut,
                    heure_fin: r.heure_fin,
                    annee_scolaire: r.annee_scolaire,
                    version: r.creneau_version,
                },
                activite: crate::domain::activite::Activite {
                    id: r.activite_id,
                    nom: r.nom,
                    description: r.description,
                    capacite_max: r.capacite_max,
                    version: r.activite_version,
                },
                role: r.role,
            });
        }

        Ok(donnees)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::activite::Role;
    use crate::domain::planning::{CreateCreneau, CreateSemaineBanalisee};

    #[derive(Debug, Clone, serde::Deserialize)]
    struct IdRow {
        id: i64,
    }

    async fn setup_db() -> Connection {
        let conn = libsql::Builder::new_local(":memory:")
            .build()
            .await
            .expect("failed to create test db")
            .connect()
            .expect("failed to connect test db");
        crate::infrastructure::migrations::cadence_migrations(&conn)
            .await
            .expect("failed to run migrations");
        conn
    }

    fn repo(conn: Connection) -> LibsqlPlanningRepository {
        LibsqlPlanningRepository::new(conn)
    }

    async fn seed_activite(conn: &Connection, nom: &str) -> i64 {
        let mut rows = conn
            .query(
                "INSERT INTO activites (nom, description, capacite_max)
                 VALUES (?, ?, ?) RETURNING id",
                libsql::params![nom, None::<String>, None::<i64>],
            )
            .await
            .expect("failed to seed activite");
        let row = rows.next().await.expect("no row").expect("no row");
        libsql::de::from_row::<IdRow>(&row)
            .expect("failed to read id")
            .id
    }

    async fn seed_personne(conn: &Connection) -> i64 {
        let mut rows = conn
            .query(
                "INSERT INTO personnes_physiques (nom, prenom, date_naissance)
                 VALUES (?, ?, ?) RETURNING id",
                libsql::params!["Test", "User", "2000-01-15"],
            )
            .await
            .expect("failed to seed personne");
        let row = rows.next().await.expect("no row").expect("no row");
        libsql::de::from_row::<IdRow>(&row)
            .expect("failed to read id")
            .id
    }

    async fn seed_inscrit(conn: &Connection, activite_id: i64, personne_id: i64, annee: &str) {
        conn.execute(
            "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
             VALUES (?, ?, ?, ?)",
            libsql::params![activite_id, personne_id, annee, "participant"],
        )
        .await
        .expect("failed to seed inscrit");
    }

    #[tokio::test]
    async fn test_creer_creneau() {
        let conn = setup_db().await;
        let activite_id = seed_activite(&conn, "Poterie").await;
        let r = repo(conn.clone());

        let c = r
            .creer_creneau(
                CreateCreneau {
                    activite_id,
                    jour_semaine: 1,
                    heure_debut: "14:00".to_string(),
                    heure_fin: "16:00".to_string(),
                    annee_scolaire: "2025-2026".to_string(),
                },
                "test",
            )
            .await
            .expect("failed to create creneau");

        assert_eq!(c.activite_id, activite_id);
        assert_eq!(c.jour_semaine, 1);
        assert_eq!(c.heure_debut, "14:00");
        assert_eq!(c.heure_fin, "16:00");
    }

    #[tokio::test]
    async fn test_lister_creneaux() {
        let conn = setup_db().await;
        let activite_id = seed_activite(&conn, "Poterie").await;
        let r = repo(conn.clone());

        r.creer_creneau(
            CreateCreneau {
                activite_id,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
            "test",
        )
        .await
        .unwrap();

        r.creer_creneau(
            CreateCreneau {
                activite_id,
                jour_semaine: 3,
                heure_debut: "10:00".to_string(),
                heure_fin: "12:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
            "test",
        )
        .await
        .unwrap();

        let list = r.lister_creneaux(activite_id, "2025-2026").await.unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].jour_semaine, 1);
        assert_eq!(list[1].jour_semaine, 3);
    }

    #[tokio::test]
    async fn test_lister_creneaux_autre_annee() {
        let conn = setup_db().await;
        let activite_id = seed_activite(&conn, "Poterie").await;
        let r = repo(conn.clone());

        r.creer_creneau(
            CreateCreneau {
                activite_id,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2024-2025".to_string(),
            },
            "test",
        )
        .await
        .unwrap();

        let list = r.lister_creneaux(activite_id, "2025-2026").await.unwrap();
        assert_eq!(list.len(), 0);
    }

    #[tokio::test]
    async fn test_supprimer_creneau() {
        let conn = setup_db().await;
        let activite_id = seed_activite(&conn, "Poterie").await;
        let r = repo(conn.clone());

        let c = r
            .creer_creneau(
                CreateCreneau {
                    activite_id,
                    jour_semaine: 1,
                    heure_debut: "14:00".to_string(),
                    heure_fin: "16:00".to_string(),
                    annee_scolaire: "2025-2026".to_string(),
                },
                "test",
            )
            .await
            .unwrap();

        r.supprimer_creneau(c.id).await.unwrap();

        let list = r.lister_creneaux(activite_id, "2025-2026").await.unwrap();
        assert_eq!(list.len(), 0);
    }

    #[tokio::test]
    async fn test_modifier_creneau() {
        let conn = setup_db().await;
        let activite_id = seed_activite(&conn, "Poterie").await;
        let r = repo(conn.clone());

        let c = r
            .creer_creneau(
                CreateCreneau {
                    activite_id,
                    jour_semaine: 1,
                    heure_debut: "14:00".to_string(),
                    heure_fin: "16:00".to_string(),
                    annee_scolaire: "2025-2026".to_string(),
                },
                "test",
            )
            .await
            .unwrap();

        let updated = r
            .modifier_creneau(
                c.id,
                CreateCreneau {
                    activite_id,
                    jour_semaine: 2,
                    heure_debut: "09:00".to_string(),
                    heure_fin: "11:00".to_string(),
                    annee_scolaire: "2025-2026".to_string(),
                },
                c.version,
                "test",
            )
            .await
            .unwrap();

        assert_eq!(updated.jour_semaine, 2);
        assert_eq!(updated.heure_debut, "09:00");
        assert_eq!(updated.heure_fin, "11:00");
    }

    #[tokio::test]
    async fn test_ajouter_semaine_banalisee() {
        let conn = setup_db().await;
        let activite_id = seed_activite(&conn, "Poterie").await;
        let r = repo(conn.clone());

        let sb = r
            .ajouter_semaine_banalisee(
                CreateSemaineBanalisee {
                    activite_id,
                    date_debut: "2025-12-22".to_string(),
                    motif: Some("Vacances de Noël".to_string()),
                    annee_scolaire: "2025-2026".to_string(),
                },
                "test",
            )
            .await
            .unwrap();

        assert_eq!(sb.date_debut, "2025-12-22");
        assert_eq!(sb.motif, Some("Vacances de Noël".to_string()));
    }

    #[tokio::test]
    async fn test_ajouter_semaine_banalisee_sans_motif() {
        let conn = setup_db().await;
        let activite_id = seed_activite(&conn, "Poterie").await;
        let r = repo(conn.clone());

        let sb = r
            .ajouter_semaine_banalisee(
                CreateSemaineBanalisee {
                    activite_id,
                    date_debut: "2025-12-22".to_string(),
                    motif: None,
                    annee_scolaire: "2025-2026".to_string(),
                },
                "test",
            )
            .await
            .unwrap();

        assert_eq!(sb.motif, None);
    }

    #[tokio::test]
    async fn test_lister_semaines_banalisees() {
        let conn = setup_db().await;
        let activite_id = seed_activite(&conn, "Poterie").await;
        let r = repo(conn.clone());

        r.ajouter_semaine_banalisee(
            CreateSemaineBanalisee {
                activite_id,
                date_debut: "2025-12-22".to_string(),
                motif: Some("Noël".to_string()),
                annee_scolaire: "2025-2026".to_string(),
            },
            "test",
        )
        .await
        .unwrap();

        r.ajouter_semaine_banalisee(
            CreateSemaineBanalisee {
                activite_id,
                date_debut: "2026-02-23".to_string(),
                motif: Some("Hiver".to_string()),
                annee_scolaire: "2025-2026".to_string(),
            },
            "test",
        )
        .await
        .unwrap();

        let list = r.lister_semaines_banalisees(activite_id).await.unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].date_debut, "2025-12-22");
        assert_eq!(list[1].date_debut, "2026-02-23");
    }

    #[tokio::test]
    async fn test_supprimer_semaine_banalisee() {
        let conn = setup_db().await;
        let activite_id = seed_activite(&conn, "Poterie").await;
        let r = repo(conn.clone());

        let sb = r
            .ajouter_semaine_banalisee(
                CreateSemaineBanalisee {
                    activite_id,
                    date_debut: "2025-12-22".to_string(),
                    motif: None,
                    annee_scolaire: "2025-2026".to_string(),
                },
                "test",
            )
            .await
            .unwrap();

        r.supprimer_semaine_banalisee(sb.id).await.unwrap();

        let list = r.lister_semaines_banalisees(activite_id).await.unwrap();
        assert_eq!(list.len(), 0);
    }

    #[tokio::test]
    async fn test_compter_inscrits_activite() {
        let conn = setup_db().await;
        let activite_id = seed_activite(&conn, "Poterie").await;
        let pid = seed_personne(&conn).await;
        let r = repo(conn.clone());

        let count = r
            .compter_inscrits_activite(activite_id, "2025-2026")
            .await
            .unwrap();
        assert_eq!(count, 0);

        seed_inscrit(&r.conn, activite_id, pid, "2025-2026").await;

        let count = r
            .compter_inscrits_activite(activite_id, "2025-2026")
            .await
            .unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_verifier_conflit_creneaux_doublon() {
        let conn = setup_db().await;
        let a = seed_activite(&conn, "Poterie").await;
        let r = repo(conn.clone());

        r.creer_creneau(
            CreateCreneau {
                activite_id: a,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
            "test",
        )
        .await
        .unwrap();

        let conflits = r
            .verifier_conflit_creneaux(a, "2025-2026", 1, "14:00", "16:00", None)
            .await
            .unwrap();
        assert_eq!(conflits.len(), 1);
    }

    #[tokio::test]
    async fn test_verifier_conflit_creneaux_exclure_id() {
        let conn = setup_db().await;
        let a = seed_activite(&conn, "Poterie").await;
        let r = repo(conn.clone());

        let c = r
            .creer_creneau(
                CreateCreneau {
                    activite_id: a,
                    jour_semaine: 1,
                    heure_debut: "14:00".to_string(),
                    heure_fin: "16:00".to_string(),
                    annee_scolaire: "2025-2026".to_string(),
                },
                "test",
            )
            .await
            .unwrap();

        let conflits = r
            .verifier_conflit_creneaux(a, "2025-2026", 1, "14:00", "16:00", Some(c.id))
            .await
            .unwrap();
        assert!(conflits.is_empty());
    }

    #[tokio::test]
    async fn test_verifier_conflit_creneaux_partiel() {
        let conn = setup_db().await;
        let a = seed_activite(&conn, "Poterie").await;
        let r = repo(conn.clone());

        r.creer_creneau(
            CreateCreneau {
                activite_id: a,
                jour_semaine: 1,
                heure_debut: "10:00".to_string(),
                heure_fin: "12:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
            "test",
        )
        .await
        .unwrap();

        let conflits = r
            .verifier_conflit_creneaux(a, "2025-2026", 1, "11:00", "13:00", None)
            .await
            .unwrap();
        assert_eq!(conflits.len(), 1);
    }

    #[tokio::test]
    async fn test_verifier_conflit_creneaux_adjacent() {
        let conn = setup_db().await;
        let a = seed_activite(&conn, "Poterie").await;
        let r = repo(conn.clone());

        r.creer_creneau(
            CreateCreneau {
                activite_id: a,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
            "test",
        )
        .await
        .unwrap();

        let conflits = r
            .verifier_conflit_creneaux(a, "2025-2026", 1, "16:00", "18:00", None)
            .await
            .unwrap();
        assert!(conflits.is_empty());
    }

    #[tokio::test]
    async fn test_verifier_conflit_creneaux_autre_activite() {
        let conn = setup_db().await;
        let a1 = seed_activite(&conn, "Poterie").await;
        let a2 = seed_activite(&conn, "Théâtre").await;
        let r = repo(conn.clone());

        r.creer_creneau(
            CreateCreneau {
                activite_id: a1,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
            "test",
        )
        .await
        .unwrap();

        let conflits = r
            .verifier_conflit_creneaux(a2, "2025-2026", 1, "14:00", "16:00", None)
            .await
            .unwrap();
        assert!(conflits.is_empty());
    }

    #[tokio::test]
    async fn test_verifier_conflit_creneaux_autre_annee() {
        let conn = setup_db().await;
        let a = seed_activite(&conn, "Poterie").await;
        let r = repo(conn.clone());

        r.creer_creneau(
            CreateCreneau {
                activite_id: a,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2024-2025".to_string(),
            },
            "test",
        )
        .await
        .unwrap();

        let conflits = r
            .verifier_conflit_creneaux(a, "2025-2026", 1, "14:00", "16:00", None)
            .await
            .unwrap();
        assert!(conflits.is_empty());
    }

    #[tokio::test]
    async fn test_verifier_confrit_creneaux_autre_jour() {
        let conn = setup_db().await;
        let a = seed_activite(&conn, "Poterie").await;
        let r = repo(conn.clone());

        r.creer_creneau(
            CreateCreneau {
                activite_id: a,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
            "test",
        )
        .await
        .unwrap();

        let conflits = r
            .verifier_conflit_creneaux(a, "2025-2026", 2, "14:00", "16:00", None)
            .await
            .unwrap();
        assert!(conflits.is_empty());
    }

    #[tokio::test]
    async fn test_verifier_collision_pas_de_conflit() {
        let conn = setup_db().await;
        let a1 = seed_activite(&conn, "Poterie").await;
        let a2 = seed_activite(&conn, "Théâtre").await;
        let pid = seed_personne(&conn).await;
        let r = repo(conn.clone());

        r.creer_creneau(
            CreateCreneau {
                activite_id: a1,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
            "test",
        )
        .await
        .unwrap();

        r.creer_creneau(
            CreateCreneau {
                activite_id: a2,
                jour_semaine: 3,
                heure_debut: "10:00".to_string(),
                heure_fin: "12:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
            "test",
        )
        .await
        .unwrap();

        seed_inscrit(&r.conn, a1, pid, "2025-2026").await;

        let collision = r.verifier_collision(pid, a2, "2025-2026").await.unwrap();
        assert!(collision.is_none());
    }

    #[tokio::test]
    async fn test_verifier_collision_conflit() {
        let conn = setup_db().await;
        let a1 = seed_activite(&conn, "Poterie").await;
        let a2 = seed_activite(&conn, "Théâtre").await;
        let pid = seed_personne(&conn).await;
        let r = repo(conn.clone());

        r.creer_creneau(
            CreateCreneau {
                activite_id: a1,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
            "test",
        )
        .await
        .unwrap();

        r.creer_creneau(
            CreateCreneau {
                activite_id: a2,
                jour_semaine: 1,
                heure_debut: "15:00".to_string(),
                heure_fin: "17:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
            "test",
        )
        .await
        .unwrap();

        r.conn
            .execute(
                "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
                 VALUES (?, ?, ?, ?)",
                libsql::params![a1, pid, "2025-2026", "encadrant"],
            )
            .await
            .unwrap();

        let collision = r.verifier_collision(pid, a2, "2025-2026").await.unwrap();
        assert!(collision.is_some());
        let c = collision.unwrap();
        assert!(c.activite_conflit.contains("Poterie"));
    }

    #[tokio::test]
    async fn test_verifier_collision_meme_activite_ignoree() {
        let conn = setup_db().await;
        let a1 = seed_activite(&conn, "Poterie").await;
        let pid = seed_personne(&conn).await;
        let r = repo(conn.clone());

        r.creer_creneau(
            CreateCreneau {
                activite_id: a1,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
            "test",
        )
        .await
        .unwrap();

        seed_inscrit(&r.conn, a1, pid, "2025-2026").await;

        let collision = r.verifier_collision(pid, a1, "2025-2026").await.unwrap();
        assert!(collision.is_none());
    }

    #[tokio::test]
    async fn test_planning_personne_semaine() {
        let conn = setup_db().await;
        let a1 = seed_activite(&conn, "Poterie").await;
        let a2 = seed_activite(&conn, "Théâtre").await;
        let pid = seed_personne(&conn).await;
        let r = repo(conn.clone());

        r.creer_creneau(
            CreateCreneau {
                activite_id: a1,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
            "test",
        )
        .await
        .unwrap();

        r.creer_creneau(
            CreateCreneau {
                activite_id: a2,
                jour_semaine: 3,
                heure_debut: "10:00".to_string(),
                heure_fin: "12:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
            "test",
        )
        .await
        .unwrap();

        seed_inscrit(&r.conn, a1, pid, "2025-2026").await;

        r.conn
            .execute(
                "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
                 VALUES (?, ?, ?, ?)",
                libsql::params![a2, pid, "2025-2026", "encadrant"],
            )
            .await
            .unwrap();

        let planning = r
            .planning_personne_semaine(pid, "2025-09-01", "2025-2026")
            .await
            .unwrap();
        assert_eq!(planning.len(), 2);
        assert_eq!(planning[0].role, Role::Participant);
        assert_eq!(planning[1].role, Role::Encadrant);
    }

    #[tokio::test]
    async fn test_planning_personne_semaine_banalisee_exclue() {
        let conn = setup_db().await;
        let a1 = seed_activite(&conn, "Poterie").await;
        let pid = seed_personne(&conn).await;
        let r = repo(conn.clone());

        r.creer_creneau(
            CreateCreneau {
                activite_id: a1,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
            "test",
        )
        .await
        .unwrap();

        r.ajouter_semaine_banalisee(
            CreateSemaineBanalisee {
                activite_id: a1,
                date_debut: "2025-12-22".to_string(),
                motif: Some("Noël".to_string()),
                annee_scolaire: "2025-2026".to_string(),
            },
            "test",
        )
        .await
        .unwrap();

        seed_inscrit(&r.conn, a1, pid, "2025-2026").await;

        let planning = r
            .planning_personne_semaine(pid, "2025-12-22", "2025-2026")
            .await
            .unwrap();
        assert_eq!(planning.len(), 0);

        let planning = r
            .planning_personne_semaine(pid, "2025-09-01", "2025-2026")
            .await
            .unwrap();
        assert_eq!(planning.len(), 1);
    }

    #[tokio::test]
    async fn test_planning_personne_semaine_aucune_activite() {
        let conn = setup_db().await;
        let pid = seed_personne(&conn).await;
        let r = repo(conn.clone());

        let planning = r
            .planning_personne_semaine(pid, "2025-09-01", "2025-2026")
            .await
            .unwrap();
        assert_eq!(planning.len(), 0);
    }

    #[tokio::test]
    async fn test_verifier_collision_exact_overlap() {
        let conn = setup_db().await;
        let a1 = seed_activite(&conn, "Poterie").await;
        let a2 = seed_activite(&conn, "Théâtre").await;
        let pid = seed_personne(&conn).await;
        let r = repo(conn.clone());

        r.creer_creneau(
            CreateCreneau {
                activite_id: a1,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
            "test",
        )
        .await
        .unwrap();

        r.creer_creneau(
            CreateCreneau {
                activite_id: a2,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
            "test",
        )
        .await
        .unwrap();

        seed_inscrit(&r.conn, a1, pid, "2025-2026").await;

        let collision = r.verifier_collision(pid, a2, "2025-2026").await.unwrap();
        assert!(collision.is_some());
    }

    #[tokio::test]
    async fn test_verifier_collision_contenant_contenu() {
        let conn = setup_db().await;
        let a1 = seed_activite(&conn, "Poterie").await;
        let a2 = seed_activite(&conn, "Théâtre").await;
        let pid = seed_personne(&conn).await;
        let r = repo(conn.clone());

        r.creer_creneau(
            CreateCreneau {
                activite_id: a1,
                jour_semaine: 1,
                heure_debut: "10:00".to_string(),
                heure_fin: "18:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
            "test",
        )
        .await
        .unwrap();

        r.creer_creneau(
            CreateCreneau {
                activite_id: a2,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
            "test",
        )
        .await
        .unwrap();

        r.conn
            .execute(
                "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
                 VALUES (?, ?, ?, ?)",
                libsql::params![a1, pid, "2025-2026", "encadrant"],
            )
            .await
            .unwrap();

        let collision = r.verifier_collision(pid, a2, "2025-2026").await.unwrap();
        assert!(collision.is_some());
    }

    #[tokio::test]
    async fn test_verifier_collision_adjacent_no_overlap() {
        let conn = setup_db().await;
        let a1 = seed_activite(&conn, "Poterie").await;
        let a2 = seed_activite(&conn, "Théâtre").await;
        let pid = seed_personne(&conn).await;
        let r = repo(conn.clone());

        r.creer_creneau(
            CreateCreneau {
                activite_id: a1,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
            "test",
        )
        .await
        .unwrap();

        r.creer_creneau(
            CreateCreneau {
                activite_id: a2,
                jour_semaine: 1,
                heure_debut: "16:00".to_string(),
                heure_fin: "18:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
            "test",
        )
        .await
        .unwrap();

        seed_inscrit(&r.conn, a1, pid, "2025-2026").await;

        let collision = r.verifier_collision(pid, a2, "2025-2026").await.unwrap();
        assert!(collision.is_none());
    }

    #[tokio::test]
    async fn test_verifier_collision_activite_sans_creneaux() {
        let conn = setup_db().await;
        let a1 = seed_activite(&conn, "Poterie").await;
        let a2 = seed_activite(&conn, "Théâtre").await;
        let pid = seed_personne(&conn).await;
        let r = repo(conn.clone());

        r.creer_creneau(
            CreateCreneau {
                activite_id: a2,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
            "test",
        )
        .await
        .unwrap();

        seed_inscrit(&r.conn, a1, pid, "2025-2026").await;

        let collision = r.verifier_collision(pid, a2, "2025-2026").await.unwrap();
        assert!(collision.is_none());
    }

    #[tokio::test]
    async fn test_verifier_collision_personne_sans_activite() {
        let conn = setup_db().await;
        let a1 = seed_activite(&conn, "Poterie").await;
        let pid = seed_personne(&conn).await;
        let r = repo(conn.clone());

        r.creer_creneau(
            CreateCreneau {
                activite_id: a1,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
            "test",
        )
        .await
        .unwrap();

        let collision = r.verifier_collision(pid, a1, "2025-2026").await.unwrap();
        assert!(collision.is_none());
    }

    #[tokio::test]
    async fn test_compter_inscrits_encadrant_et_participant() {
        let conn = setup_db().await;
        let activite_id = seed_activite(&conn, "Poterie").await;
        let pid1 = seed_personne(&conn).await;
        let pid2 = seed_personne(&conn).await;
        let r = repo(conn.clone());

        r.conn
            .execute(
                "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
                 VALUES (?, ?, ?, ?)",
                libsql::params![activite_id, pid1, "2025-2026", "encadrant"],
            )
            .await
            .unwrap();

        seed_inscrit(&r.conn, activite_id, pid2, "2025-2026").await;

        let count = r
            .compter_inscrits_activite(activite_id, "2025-2026")
            .await
            .unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_compter_inscrits_autre_annee() {
        let conn = setup_db().await;
        let activite_id = seed_activite(&conn, "Poterie").await;
        let pid = seed_personne(&conn).await;
        let r = repo(conn.clone());

        seed_inscrit(&r.conn, activite_id, pid, "2024-2025").await;

        let count = r
            .compter_inscrits_activite(activite_id, "2025-2026")
            .await
            .unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_planning_personne_meme_jour_trie_par_heure() {
        let conn = setup_db().await;
        let a1 = seed_activite(&conn, "Poterie").await;
        let a2 = seed_activite(&conn, "Théâtre").await;
        let pid = seed_personne(&conn).await;
        let r = repo(conn.clone());

        r.creer_creneau(
            CreateCreneau {
                activite_id: a1,
                jour_semaine: 1,
                heure_debut: "16:00".to_string(),
                heure_fin: "18:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
            "test",
        )
        .await
        .unwrap();

        r.creer_creneau(
            CreateCreneau {
                activite_id: a2,
                jour_semaine: 1,
                heure_debut: "10:00".to_string(),
                heure_fin: "12:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
            "test",
        )
        .await
        .unwrap();

        seed_inscrit(&r.conn, a1, pid, "2025-2026").await;
        seed_inscrit(&r.conn, a2, pid, "2025-2026").await;

        let planning = r
            .planning_personne_semaine(pid, "2025-09-01", "2025-2026")
            .await
            .unwrap();
        assert_eq!(planning.len(), 2);
        assert_eq!(planning[0].creneau.heure_debut, "10:00");
        assert_eq!(planning[1].creneau.heure_debut, "16:00");
    }

    #[tokio::test]
    async fn test_creer_creneau_plusieurs_activites() {
        let conn = setup_db().await;
        let a1 = seed_activite(&conn, "Poterie").await;
        let a2 = seed_activite(&conn, "Théâtre").await;
        let r = repo(conn.clone());

        let c1 = r
            .creer_creneau(
                CreateCreneau {
                    activite_id: a1,
                    jour_semaine: 1,
                    heure_debut: "14:00".to_string(),
                    heure_fin: "16:00".to_string(),
                    annee_scolaire: "2025-2026".to_string(),
                },
                "test",
            )
            .await
            .unwrap();

        let c2 = r
            .creer_creneau(
                CreateCreneau {
                    activite_id: a2,
                    jour_semaine: 3,
                    heure_debut: "10:00".to_string(),
                    heure_fin: "12:00".to_string(),
                    annee_scolaire: "2025-2026".to_string(),
                },
                "test",
            )
            .await
            .unwrap();

        assert_eq!(c1.activite_id, a1);
        assert_eq!(c2.activite_id, a2);
    }

    #[tokio::test]
    async fn test_semaine_banalisee_meme_date_deux_activites() {
        let conn = setup_db().await;
        let a1 = seed_activite(&conn, "Poterie").await;
        let a2 = seed_activite(&conn, "Théâtre").await;
        let r = repo(conn.clone());

        let sb1 = r
            .ajouter_semaine_banalisee(
                CreateSemaineBanalisee {
                    activite_id: a1,
                    date_debut: "2025-12-22".to_string(),
                    motif: Some("Noël".to_string()),
                    annee_scolaire: "2025-2026".to_string(),
                },
                "test",
            )
            .await
            .unwrap();

        let sb2 = r
            .ajouter_semaine_banalisee(
                CreateSemaineBanalisee {
                    activite_id: a2,
                    date_debut: "2025-12-22".to_string(),
                    motif: Some("Noël".to_string()),
                    annee_scolaire: "2025-2026".to_string(),
                },
                "test",
            )
            .await
            .unwrap();

        assert_eq!(sb1.date_debut, sb2.date_debut);
        assert_ne!(sb1.id, sb2.id);
    }

    #[tokio::test]
    async fn test_lister_creneaux_tri_par_jour_puis_heure() {
        let conn = setup_db().await;
        let activite_id = seed_activite(&conn, "Poterie").await;
        let r = repo(conn.clone());

        r.creer_creneau(
            CreateCreneau {
                activite_id,
                jour_semaine: 3,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
            "test",
        )
        .await
        .unwrap();

        r.creer_creneau(
            CreateCreneau {
                activite_id,
                jour_semaine: 1,
                heure_debut: "14:00".to_string(),
                heure_fin: "16:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
            "test",
        )
        .await
        .unwrap();

        let list = r.lister_creneaux(activite_id, "2025-2026").await.unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].jour_semaine, 1);
        assert_eq!(list[1].jour_semaine, 3);
    }

    #[tokio::test]
    async fn test_modifier_creneau_inexistant() {
        let conn = setup_db().await;
        let r = repo(conn.clone());

        let result = r
            .modifier_creneau(
                99999,
                CreateCreneau {
                    activite_id: 1,
                    jour_semaine: 1,
                    heure_debut: "14:00".to_string(),
                    heure_fin: "16:00".to_string(),
                    annee_scolaire: "2025-2026".to_string(),
                },
                1,
                "test",
            )
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_supprimer_creneau_inexistant() {
        let conn = setup_db().await;
        let r = repo(conn.clone());

        let result = r.supprimer_creneau(99999).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_planning_personne_activite_sans_creneaux() {
        let conn = setup_db().await;
        let a1 = seed_activite(&conn, "Poterie").await;
        let pid = seed_personne(&conn).await;
        let r = repo(conn.clone());

        seed_inscrit(&r.conn, a1, pid, "2025-2026").await;

        let planning = r
            .planning_personne_semaine(pid, "2025-09-01", "2025-2026")
            .await
            .unwrap();
        assert_eq!(planning.len(), 0);
    }

    #[tokio::test]
    async fn test_semaine_banalisee_meme_activite_deux_dates() {
        let conn = setup_db().await;
        let activite_id = seed_activite(&conn, "Poterie").await;
        let r = repo(conn.clone());

        r.ajouter_semaine_banalisee(
            CreateSemaineBanalisee {
                activite_id,
                date_debut: "2025-12-22".to_string(),
                motif: None,
                annee_scolaire: "2025-2026".to_string(),
            },
            "test",
        )
        .await
        .unwrap();

        r.ajouter_semaine_banalisee(
            CreateSemaineBanalisee {
                activite_id,
                date_debut: "2025-12-29".to_string(),
                motif: None,
                annee_scolaire: "2025-2026".to_string(),
            },
            "test",
        )
        .await
        .unwrap();

        let list = r.lister_semaines_banalisees(activite_id).await.unwrap();
        assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn test_lister_creneaux_hors_plage_av_apres_partiel() {
        let conn = setup_db().await;
        let activite_id = seed_activite(&conn, "Poterie").await;
        let pid = seed_personne(&conn).await;
        let r = repo(conn.clone());

        let c1 = r
            .creer_creneau(
                CreateCreneau {
                    activite_id,
                    jour_semaine: 1,
                    heure_debut: "07:00".to_string(),
                    heure_fin: "08:00".to_string(),
                    annee_scolaire: "2025-2026".to_string(),
                },
                "test",
            )
            .await
            .unwrap();

        let c2 = r
            .creer_creneau(
                CreateCreneau {
                    activite_id,
                    jour_semaine: 1,
                    heure_debut: "09:00".to_string(),
                    heure_fin: "11:00".to_string(),
                    annee_scolaire: "2025-2026".to_string(),
                },
                "test",
            )
            .await
            .unwrap();

        let c3 = r
            .creer_creneau(
                CreateCreneau {
                    activite_id,
                    jour_semaine: 2,
                    heure_debut: "19:00".to_string(),
                    heure_fin: "21:00".to_string(),
                    annee_scolaire: "2025-2026".to_string(),
                },
                "test",
            )
            .await
            .unwrap();

        let c4 = r
            .creer_creneau(
                CreateCreneau {
                    activite_id,
                    jour_semaine: 3,
                    heure_debut: "07:30".to_string(),
                    heure_fin: "09:00".to_string(),
                    annee_scolaire: "2025-2026".to_string(),
                },
                "test",
            )
            .await
            .unwrap();

        seed_inscrit(&r.conn, activite_id, pid, "2025-2026").await;

        let hors = r
            .lister_creneaux_hors_plage("08:00", "20:00")
            .await
            .unwrap();

        let ids: Vec<i64> = hors.iter().map(|h| h.creneau_id).collect();
        assert_eq!(ids, vec![c1.id, c3.id, c4.id]);
        assert!(!ids.contains(&c2.id));
        for h in &hors {
            assert_eq!(h.annee_scolaire, "2025-2026");
            assert_eq!(h.nb_inscrits, 1);
        }
    }

    #[tokio::test]
    async fn test_lister_creneaux_hors_plage_aucun() {
        let conn = setup_db().await;
        let activite_id = seed_activite(&conn, "Poterie").await;
        let r = repo(conn.clone());

        r.creer_creneau(
            CreateCreneau {
                activite_id,
                jour_semaine: 1,
                heure_debut: "09:00".to_string(),
                heure_fin: "11:00".to_string(),
                annee_scolaire: "2025-2026".to_string(),
            },
            "test",
        )
        .await
        .unwrap();

        let hors = r
            .lister_creneaux_hors_plage("08:00", "20:00")
            .await
            .unwrap();
        assert!(hors.is_empty());
    }

    #[tokio::test]
    async fn test_lister_tous_creneaux_sans_filtre() {
        let conn = setup_db().await;
        let a1 = seed_activite(&conn, "Poterie").await;
        let a2 = seed_activite(&conn, "Théâtre").await;
        let r = repo(conn.clone());

        let c1 = r
            .creer_creneau(
                CreateCreneau {
                    activite_id: a1,
                    jour_semaine: 1,
                    heure_debut: "14:00".to_string(),
                    heure_fin: "16:00".to_string(),
                    annee_scolaire: "2025-2026".to_string(),
                },
                "test",
            )
            .await
            .unwrap();

        let c2 = r
            .creer_creneau(
                CreateCreneau {
                    activite_id: a2,
                    jour_semaine: 3,
                    heure_debut: "10:00".to_string(),
                    heure_fin: "12:00".to_string(),
                    annee_scolaire: "2024-2025".to_string(),
                },
                "test",
            )
            .await
            .unwrap();

        let tous = r.lister_tous_creneaux().await.unwrap();
        assert_eq!(tous.len(), 2);
        assert_eq!(tous[0].id, c1.id);
        assert_eq!(tous[1].id, c2.id);
    }

    #[tokio::test]
    async fn test_supprimer_creneau_tx_commit_visible() {
        let conn = setup_db().await;
        let activite_id = seed_activite(&conn, "Poterie").await;
        let r = repo(conn.clone());

        let c = r
            .creer_creneau(
                CreateCreneau {
                    activite_id,
                    jour_semaine: 1,
                    heure_debut: "14:00".to_string(),
                    heure_fin: "16:00".to_string(),
                    annee_scolaire: "2025-2026".to_string(),
                },
                "test",
            )
            .await
            .unwrap();

        let mut tx = conn.transaction().await.unwrap();
        r.supprimer_creneau_tx(&mut tx, c.id).await.unwrap();
        tx.commit().await.unwrap();

        let list = r.lister_creneaux(activite_id, "2025-2026").await.unwrap();
        assert!(list.is_empty());
    }

    #[tokio::test]
    async fn test_supprimer_creneau_tx_rollback_sans_effet() {
        let conn = setup_db().await;
        let activite_id = seed_activite(&conn, "Poterie").await;
        let r = repo(conn.clone());

        let c = r
            .creer_creneau(
                CreateCreneau {
                    activite_id,
                    jour_semaine: 1,
                    heure_debut: "14:00".to_string(),
                    heure_fin: "16:00".to_string(),
                    annee_scolaire: "2025-2026".to_string(),
                },
                "test",
            )
            .await
            .unwrap();

        let mut tx = conn.transaction().await.unwrap();
        r.supprimer_creneau_tx(&mut tx, c.id).await.unwrap();
        tx.rollback().await.unwrap();

        let list = r.lister_creneaux(activite_id, "2025-2026").await.unwrap();
        assert_eq!(list.len(), 1);
    }

    #[tokio::test]
    async fn test_deplacer_creneau_tx_commit_visible() {
        let conn = setup_db().await;
        let activite_id = seed_activite(&conn, "Poterie").await;
        let r = repo(conn.clone());

        let c = r
            .creer_creneau(
                CreateCreneau {
                    activite_id,
                    jour_semaine: 1,
                    heure_debut: "07:00".to_string(),
                    heure_fin: "09:00".to_string(),
                    annee_scolaire: "2025-2026".to_string(),
                },
                "test",
            )
            .await
            .unwrap();

        let mut tx = conn.transaction().await.unwrap();
        r.deplacer_creneau_tx(&mut tx, c.id, "09:00", "11:00", "test")
            .await
            .unwrap();
        tx.commit().await.unwrap();

        let list = r.lister_creneaux(activite_id, "2025-2026").await.unwrap();
        assert_eq!(list[0].heure_debut, "09:00");
        assert_eq!(list[0].heure_fin, "11:00");
    }

    #[tokio::test]
    async fn test_deplacer_creneau_tx_rollback_sans_effet() {
        let conn = setup_db().await;
        let activite_id = seed_activite(&conn, "Poterie").await;
        let r = repo(conn.clone());

        let c = r
            .creer_creneau(
                CreateCreneau {
                    activite_id,
                    jour_semaine: 1,
                    heure_debut: "07:00".to_string(),
                    heure_fin: "09:00".to_string(),
                    annee_scolaire: "2025-2026".to_string(),
                },
                "test",
            )
            .await
            .unwrap();

        let mut tx = conn.transaction().await.unwrap();
        r.deplacer_creneau_tx(&mut tx, c.id, "09:00", "11:00", "test")
            .await
            .unwrap();
        tx.rollback().await.unwrap();

        let list = r.lister_creneaux(activite_id, "2025-2026").await.unwrap();
        assert_eq!(list[0].heure_debut, "07:00");
        assert_eq!(list[0].heure_fin, "09:00");
    }

    #[tokio::test]
    async fn test_lister_inscriptions_nom_joint() {
        let conn = setup_db().await;
        let a1 = seed_activite(&conn, "Poterie").await;
        let a2 = seed_activite(&conn, "Théâtre").await;
        let pid1 = seed_personne(&conn).await;
        let pid2 = seed_personne(&conn).await;
        let r = repo(conn.clone());

        seed_inscrit(&r.conn, a1, pid1, "2025-2026").await;
        r.conn
            .execute(
                "INSERT INTO activite_personnes (activite_id, personne_id, annee_scolaire, role)
                 VALUES (?, ?, ?, ?)",
                libsql::params![a1, pid2, "2025-2026", "encadrant"],
            )
            .await
            .unwrap();
        seed_inscrit(&r.conn, a2, pid1, "2024-2025").await;

        let inscrits = r.lister_inscriptions().await.unwrap();
        assert_eq!(inscrits.len(), 3);

        let poterie = inscrits
            .iter()
            .find(|i| i.activite_id == a1 && i.personne_id == pid1)
            .unwrap();
        assert_eq!(poterie.activite_nom, "Poterie");
        assert_eq!(poterie.annee_scolaire, "2025-2026");

        let theatre = inscrits.iter().find(|i| i.activite_id == a2).unwrap();
        assert_eq!(theatre.activite_nom, "Théâtre");
        assert_eq!(theatre.annee_scolaire, "2024-2025");
    }
}
