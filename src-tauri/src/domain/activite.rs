use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Encadrant,
    Participant,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Encadrant => write!(f, "encadrant"),
            Role::Participant => write!(f, "participant"),
        }
    }
}

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
    pub role: Role,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLiaisonActivitePersonne {
    pub activite_id: i64,
    pub personne_id: i64,
    pub annee_scolaire: String,
    pub role: Role,
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
    pub role: Role,
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
    fn test_role_deserialisation() {
        let r: Role = serde_json::from_str("\"encadrant\"").unwrap();
        assert_eq!(r, Role::Encadrant);
        let r: Role = serde_json::from_str("\"participant\"").unwrap();
        assert_eq!(r, Role::Participant);
        assert!(serde_json::from_str::<Role>("\"admin\"").is_err());
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
