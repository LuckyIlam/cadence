use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::domain::adhesion::{Adhesion, CreateAdhesion, UpdateAdhesion};
use crate::error::AppError;

#[async_trait]
pub trait AdhesionRepository: Send + Sync {
    async fn create(&self, input: CreateAdhesion) -> Result<Adhesion, AppError>;
    async fn update(&self, id: i64, input: UpdateAdhesion) -> Result<Adhesion, AppError>;
    async fn list_by_personne(&self, personne_id: i64) -> Result<Vec<Adhesion>, AppError>;
}

pub struct SqliteAdhesionRepository {
    pub(crate) pool: SqlitePool,
}

impl SqliteAdhesionRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AdhesionRepository for SqliteAdhesionRepository {
    async fn create(&self, input: CreateAdhesion) -> Result<Adhesion, AppError> {
        let row = sqlx::query_as::<_, Adhesion>(
            "INSERT INTO adhesions (personne_id, annee_scolaire, reglee, note_paiement)
             VALUES (?, ?, ?, ?)
             RETURNING *",
        )
        .bind(input.personne_id)
        .bind(&input.annee_scolaire)
        .bind(input.reglee)
        .bind(&input.note_paiement)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    async fn update(&self, id: i64, input: UpdateAdhesion) -> Result<Adhesion, AppError> {
        let row = sqlx::query_as::<_, Adhesion>(
            "UPDATE adhesions
             SET reglee = ?, note_paiement = ?
             WHERE id = ?
             RETURNING *",
        )
        .bind(input.reglee)
        .bind(&input.note_paiement)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    async fn list_by_personne(&self, personne_id: i64) -> Result<Vec<Adhesion>, AppError> {
        let rows = sqlx::query_as::<_, Adhesion>(
            "SELECT * FROM adhesions WHERE personne_id = ? ORDER BY annee_scolaire DESC",
        )
        .bind(personne_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }
}
