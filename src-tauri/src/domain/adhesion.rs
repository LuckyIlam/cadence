use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Adhesion {
    pub id: i64,
    pub personne_id: i64,
    pub annee_scolaire: String,
    pub reglee: bool,
    pub note_paiement: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAdhesion {
    pub personne_id: i64,
    pub annee_scolaire: String,
    pub reglee: bool,
    pub note_paiement: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAdhesion {
    pub reglee: bool,
    pub note_paiement: Option<String>,
}
