use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Activite {
    pub id: i64,
    pub nom: String,
    pub description: Option<String>,
    pub capacite_max: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateActivite {
    pub nom: String,
    pub description: Option<String>,
    pub capacite_max: Option<i64>,
    pub annee_scolaire: Option<String>,
    pub tarif: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateActivite {
    pub nom: String,
    pub description: Option<String>,
    pub capacite_max: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TarifActivite {
    pub activite_id: i64,
    pub annee_scolaire: String,
    pub tarif: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTarifActivite {
    pub activite_id: i64,
    pub annee_scolaire: String,
    pub tarif: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LiaisonActivitePersonne {
    pub activite_id: i64,
    pub personne_id: i64,
    pub annee_scolaire: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLiaisonActivitePersonne {
    pub activite_id: i64,
    pub personne_id: i64,
    pub annee_scolaire: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PersonneActivite {
    pub id: i64,
    pub nom: String,
    pub prenom: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailActivite {
    pub activite: Activite,
    pub tarif: Option<f64>,
    pub encadrants: Vec<PersonneActivite>,
    pub participants: Vec<PersonneActivite>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivitePersonne {
    pub activite: Activite,
    pub role: String,
}

pub fn valider_role(role: &str) -> Result<(), String> {
    match role {
        "encadrant" | "participant" => Ok(()),
        _ => Err(format!(
            "Rôle invalide : '{}'. Les rôles valides sont 'encadrant' et 'participant'",
            role
        )),
    }
}

pub fn verifier_capacite_max(
    nb_participants: i64,
    capacite_max: Option<i64>,
) -> Result<(), String> {
    if let Some(capacite) = capacite_max {
        if nb_participants >= capacite {
            return Err(format!(
                "Capacité maximale atteinte ({}/{})",
                nb_participants, capacite
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_valide_encadrant() {
        assert!(valider_role("encadrant").is_ok());
    }

    #[test]
    fn test_role_valide_participant() {
        assert!(valider_role("participant").is_ok());
    }

    #[test]
    fn test_role_invalide() {
        assert!(valider_role("admin").is_err());
        assert!(valider_role("").is_err());
        assert!(valider_role("encadrant ").is_err());
    }

    #[test]
    fn test_capacite_max_non_atteinte() {
        assert!(verifier_capacite_max(5, Some(10)).is_ok());
    }

    #[test]
    fn test_capacite_max_atteinte() {
        let err = verifier_capacite_max(10, Some(10)).unwrap_err();
        assert!(err.contains("Capacité maximale atteinte"));
    }

    #[test]
    fn test_capacite_max_depassee() {
        let err = verifier_capacite_max(15, Some(10)).unwrap_err();
        assert!(err.contains("Capacité maximale atteinte"));
    }

    #[test]
    fn test_capacite_max_sans_limite() {
        assert!(verifier_capacite_max(100, None).is_ok());
    }

    #[test]
    fn test_capacite_max_0() {
        let err = verifier_capacite_max(0, Some(0)).unwrap_err();
        assert!(err.contains("Capacité maximale atteinte"));
    }
}
