use serde::{Deserialize, Serialize};

use super::planning::{valider_heure, CreateCreneau};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ParametresPlanning {
    pub id: i64,
    pub heure_ouverture: String,
    pub heure_fermeture: String,
}

/// Valide une plage horaire d'ouverture : formats HH:MM valides et ouverture avant fermeture.
pub fn valider_plage_horaire(heure_ouverture: &str, heure_fermeture: &str) -> Result<(), String> {
    valider_heure(heure_ouverture)?;
    valider_heure(heure_fermeture)?;
    if heure_ouverture >= heure_fermeture {
        return Err(format!(
            "Plage horaire invalide : l'heure d'ouverture ({}) doit être avant l'heure de fermeture ({}).",
            heure_ouverture, heure_fermeture
        ));
    }
    Ok(())
}

/// Vérifie qu'un créneau est entièrement compris dans la plage d'ouverture de l'activité.
pub fn valider_creneau_dans_plage(
    creneau: &CreateCreneau,
    heure_ouverture: &str,
    heure_fermeture: &str,
) -> Result<(), String> {
    if (creneau.heure_debut.as_str()) < heure_ouverture {
        return Err(format!(
            "L'heure de début ({}) est avant l'ouverture de l'activité ({}). Les créneaux doivent être compris entre {} et {}.",
            creneau.heure_debut, heure_ouverture, heure_ouverture, heure_fermeture
        ));
    }
    if (creneau.heure_fin.as_str()) > heure_fermeture {
        return Err(format!(
            "L'heure de fin ({}) est après la fermeture de l'activité ({}). Les créneaux doivent être compris entre {} et {}.",
            creneau.heure_fin, heure_fermeture, heure_ouverture, heure_fermeture
        ));
    }
    Ok(())
}

/// Convertit une heure "HH:MM" en minutes depuis minuit.
pub(crate) fn heure_en_minutes(heure: &str) -> Option<u32> {
    let parts: Vec<&str> = heure.split(':').collect();
    if parts.len() != 2 {
        return None;
    }
    let h: u32 = parts[0].parse().ok()?;
    let m: u32 = parts[1].parse().ok()?;
    if h > 23 || m > 59 {
        return None;
    }
    Some(h * 60 + m)
}

/// Convertit des minutes depuis minuit en "HH:MM".
pub(crate) fn minutes_en_heure(minutes: u32) -> String {
    format!("{:02}:{:02}", minutes / 60, minutes % 60)
}

/// Cherche la place libre la plus proche pour placer un créneau entièrement dans la plage
/// [heure_ouverture, heure_fermeture], sur le même jour, sans chevaucher `occupes`.
///
/// Retourne `(heure_debut, heure_fin)` de la position retenue, ou `None` si aucun emplacement
/// ne peut accueillir le créneau (durée trop grande pour la plage ou journée complète).
pub fn trouver_place_deplacement(
    heure_debut: &str,
    heure_fin: &str,
    heure_ouverture: &str,
    heure_fermeture: &str,
    occupes: &[(String, String)],
) -> Option<(String, String)> {
    let debut = heure_en_minutes(heure_debut)?;
    let fin = heure_en_minutes(heure_fin)?;
    let ouverture = heure_en_minutes(heure_ouverture)?;
    let fermeture = heure_en_minutes(heure_fermeture)?;

    let duree = fin.checked_sub(debut)?;
    if duree == 0 || duree > fermeture - ouverture {
        return None;
    }

    let mut intervalles: Vec<(u32, u32)> = occupes
        .iter()
        .filter_map(|(d, f)| {
            let a = heure_en_minutes(d)?;
            let b = heure_en_minutes(f)?;
            Some((a.max(ouverture), b.min(fermeture)))
        })
        .filter(|(a, b)| a < b)
        .collect();
    intervalles.sort_unstable();
    intervalles.dedup();

    // Fusionne les intervalles se chevauchant, bornés à la plage.
    let mut fusion: Vec<(u32, u32)> = Vec::new();
    for (a, b) in intervalles {
        if let Some(last) = fusion.last_mut() {
            if a <= last.1 {
                last.1 = last.1.max(b);
                continue;
            }
        }
        fusion.push((a, b));
    }

    // Fenêtres libres dans [ouverture, fermeture].
    let mut fenetres: Vec<(u32, u32)> = Vec::new();
    let mut curseur = ouverture;
    for (a, b) in &fusion {
        if *a > curseur {
            fenetres.push((curseur, *a));
        }
        curseur = curseur.max(*b);
    }
    if curseur < fermeture {
        fenetres.push((curseur, fermeture));
    }

    let mut meilleure: Option<(u32, u32)> = None; // (distance, nouveau_debut)
    for (ws, we) in fenetres {
        if we - ws < duree {
            continue;
        }
        let dernier_debut = we - duree;
        let candidat = if debut >= ws && debut <= dernier_debut {
            debut
        } else if debut < ws {
            ws
        } else {
            dernier_debut
        };
        let distance = candidat.abs_diff(debut);
        let meilleure_que = match meilleure {
            None => true,
            Some((d, de)) => distance < d || (distance == d && candidat < de),
        };
        if meilleure_que {
            meilleure = Some((distance, candidat));
        }
    }

    meilleure.map(|(_, debut2)| {
        let fin2 = debut2 + duree;
        (minutes_en_heure(debut2), minutes_en_heure(fin2))
    })
}

/// Action appliquée à un créneau lors d'une réduction de plage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImpactAction {
    Supprime,
    Deplace,
    DeplaceImpossible,
}

/// Impact d'une réduction de plage sur un créneau.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactCreneau {
    pub creneau_id: i64,
    pub activite_id: i64,
    pub activite_nom: String,
    pub jour_semaine: i64,
    pub heure_debut: String,
    pub heure_fin: String,
    pub annee_scolaire: String,
    pub action: ImpactAction,
    pub nouveau_debut: Option<String>,
    pub nouveau_fin: Option<String>,
    pub raison: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn creneau(debut: &str, fin: &str) -> CreateCreneau {
        CreateCreneau {
            activite_id: 1,
            jour_semaine: 1,
            heure_debut: debut.to_string(),
            heure_fin: fin.to_string(),
            annee_scolaire: "2025-2026".to_string(),
        }
    }

    #[test]
    fn test_valider_plage_horaire_ok() {
        assert!(valider_plage_horaire("08:00", "20:00").is_ok());
        assert!(valider_plage_horaire("09:30", "12:00").is_ok());
    }

    #[test]
    fn test_valider_plage_horaire_ouverture_apres_fermeture() {
        let err = valider_plage_horaire("20:00", "08:00").unwrap_err();
        assert!(err.contains("avant l'heure de fermeture"));
    }

    #[test]
    fn test_valider_plage_horaire_egales() {
        assert!(valider_plage_horaire("08:00", "08:00").is_err());
    }

    #[test]
    fn test_valider_plage_horaire_format_invalide() {
        assert!(valider_plage_horaire("8am", "20:00").is_err());
        assert!(valider_plage_horaire("08:00", "25:00").is_err());
    }

    #[test]
    fn test_valider_creneau_dans_plage_ok() {
        assert!(valider_creneau_dans_plage(&creneau("08:00", "20:00"), "08:00", "20:00").is_ok());
        assert!(valider_creneau_dans_plage(&creneau("09:00", "18:00"), "08:00", "20:00").is_ok());
    }

    #[test]
    fn test_valider_creneau_debut_avant_ouverture() {
        let err =
            valider_creneau_dans_plage(&creneau("07:00", "09:00"), "08:00", "20:00").unwrap_err();
        assert!(err.contains("avant l'ouverture"));
    }

    #[test]
    fn test_valider_creneau_fin_apres_fermeture() {
        let err =
            valider_creneau_dans_plage(&creneau("18:00", "21:00"), "08:00", "20:00").unwrap_err();
        assert!(err.contains("après la fermeture"));
    }

    #[test]
    fn test_trouver_place_deplacement_place_libre() {
        let place = trouver_place_deplacement("19:00", "20:30", "08:00", "20:00", &[]);
        assert_eq!(place, Some(("18:30".to_string(), "20:00".to_string())));
    }

    #[test]
    fn test_trouver_place_deplacement_place_identique() {
        let place = trouver_place_deplacement("09:00", "10:00", "08:00", "20:00", &[]);
        assert_eq!(place, Some(("09:00".to_string(), "10:00".to_string())));
    }

    #[test]
    fn test_trouver_place_deplacement_evite_occupes() {
        let occupes = vec![
            ("09:00".to_string(), "10:00".to_string()),
            ("14:00".to_string(), "16:00".to_string()),
        ];
        let place = trouver_place_deplacement("09:30", "10:30", "08:00", "20:00", &occupes);
        assert_eq!(place, Some(("10:00".to_string(), "11:00".to_string())));
    }

    #[test]
    fn test_trouver_place_deplacement_plus_proche_egalite_tot() {
        let occupes = vec![("12:00".to_string(), "13:00".to_string())];
        // 11:00-12:00 et 13:00-14:00 sont à distance égale de 12:00-13:00 -> on garde le plus tôt.
        let place = trouver_place_deplacement("12:00", "13:00", "08:00", "20:00", &occupes);
        assert_eq!(place, Some(("11:00".to_string(), "12:00".to_string())));
    }

    #[test]
    fn test_trouver_place_deplacement_plus_proche_avant() {
        let occupes = vec![("13:30".to_string(), "16:00".to_string())];
        // Créneau 13:00-14:00 chevauche partiellement : place avant (12:30) plus proche que après (16:00).
        let place = trouver_place_deplacement("13:00", "14:00", "08:00", "20:00", &occupes);
        assert_eq!(place, Some(("12:30".to_string(), "13:30".to_string())));
    }

    #[test]
    fn test_trouver_place_deplacement_plus_proche_apres() {
        let occupes = vec![("10:00".to_string(), "10:45".to_string())];
        // Créneau 10:30-11:30 chevauche partiellement : place après (10:45) plus proche que avant (09:00).
        let place = trouver_place_deplacement("10:30", "11:30", "08:00", "20:00", &occupes);
        assert_eq!(place, Some(("10:45".to_string(), "11:45".to_string())));
    }

    #[test]
    fn test_trouver_place_deplacement_debut_apres_fermeture() {
        let place = trouver_place_deplacement("20:00", "21:00", "08:00", "20:00", &[]);
        assert_eq!(place, Some(("19:00".to_string(), "20:00".to_string())));
    }

    #[test]
    fn test_trouver_place_deplacement_a_cheval_avant_ouverture() {
        // 07:30-09:00 partiellement hors plage : début ramené au plus proche (09:00-10:30 impossible
        // car occupé, sinon conservé). Ici place vide -> on garde la position la plus proche.
        let place = trouver_place_deplacement("07:30", "09:00", "08:00", "20:00", &[]);
        assert_eq!(place, Some(("08:00".to_string(), "09:30".to_string())));
    }

    #[test]
    fn test_trouver_place_deplacement_journee_complete() {
        let place = trouver_place_deplacement("10:00", "18:00", "08:00", "20:00", &[]);
        assert_eq!(place, Some(("10:00".to_string(), "18:00".to_string())));
    }

    #[test]
    fn test_trouver_place_deplacement_trop_grand() {
        let place = trouver_place_deplacement("09:00", "19:00", "10:00", "18:00", &[]);
        assert_eq!(place, None);
    }

    #[test]
    fn test_trouver_place_deplacement_aucun_emplacement() {
        let occupes = vec![
            ("08:00".to_string(), "12:00".to_string()),
            ("13:00".to_string(), "20:00".to_string()),
        ];
        // Fenêtre libre de 60 min (12:00-13:00) trop petite pour un créneau de 120 min.
        let place = trouver_place_deplacement("11:00", "13:00", "08:00", "20:00", &occupes);
        assert_eq!(place, None);
    }

    #[test]
    fn test_heure_en_minutes() {
        assert_eq!(heure_en_minutes("08:30"), Some(510));
        assert_eq!(heure_en_minutes("00:00"), Some(0));
        assert_eq!(heure_en_minutes("23:59"), Some(1439));
        assert_eq!(heure_en_minutes("24:00"), None);
        assert_eq!(heure_en_minutes("08:60"), None);
        assert_eq!(heure_en_minutes("8h30"), None);
        assert_eq!(heure_en_minutes("abc"), None);
    }

    #[test]
    fn test_minutes_en_heure() {
        assert_eq!(minutes_en_heure(0), "00:00");
        assert_eq!(minutes_en_heure(510), "08:30");
        assert_eq!(minutes_en_heure(1439), "23:59");
    }
}
