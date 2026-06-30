use sqlx::SqlitePool;

use crate::domain::adhesion::{Adhesion, CreateAdhesion, UpdateAdhesion};

pub async fn create(pool: &SqlitePool, input: CreateAdhesion) -> Result<Adhesion, sqlx::Error> {
    let row = sqlx::query_as::<_, Adhesion>(
        "INSERT INTO adhesions (personne_id, annee_scolaire, reglee, note_paiement)
         VALUES (?, ?, ?, ?)
         RETURNING *",
    )
    .bind(input.personne_id)
    .bind(&input.annee_scolaire)
    .bind(input.reglee)
    .bind(&input.note_paiement)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn update(pool: &SqlitePool, id: i64, input: UpdateAdhesion) -> Result<Adhesion, sqlx::Error> {
    let row = sqlx::query_as::<_, Adhesion>(
        "UPDATE adhesions
         SET reglee = ?, note_paiement = ?
         WHERE id = ?
         RETURNING *",
    )
    .bind(input.reglee)
    .bind(&input.note_paiement)
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn find_by_id(pool: &SqlitePool, id: i64) -> Result<Option<Adhesion>, sqlx::Error> {
    let row = sqlx::query_as::<_, Adhesion>(
        "SELECT * FROM adhesions WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn list_by_personne(pool: &SqlitePool, personne_id: i64) -> Result<Vec<Adhesion>, sqlx::Error> {
    let rows = sqlx::query_as::<_, Adhesion>(
        "SELECT * FROM adhesions WHERE personne_id = ? ORDER BY annee_scolaire DESC",
    )
    .bind(personne_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}
