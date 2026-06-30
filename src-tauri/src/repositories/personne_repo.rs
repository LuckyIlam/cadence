use sqlx::SqlitePool;

use crate::domain::personne::{CreatePersonne, Personne, UpdatePersonne};

pub async fn create(pool: &SqlitePool, input: CreatePersonne) -> Result<Personne, sqlx::Error> {
    let row = sqlx::query_as::<_, Personne>(
        "INSERT INTO personnes_physiques (nom, prenom, date_naissance, email, telephone, responsable_id)
         VALUES (?, ?, ?, ?, ?, ?)
         RETURNING *",
    )
    .bind(&input.nom)
    .bind(&input.prenom)
    .bind(input.date_naissance)
    .bind(&input.email)
    .bind(&input.telephone)
    .bind(input.responsable_id)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn update(
    pool: &SqlitePool,
    id: i64,
    input: UpdatePersonne,
) -> Result<Personne, sqlx::Error> {
    let row = sqlx::query_as::<_, Personne>(
        "UPDATE personnes_physiques
         SET nom = ?, prenom = ?, date_naissance = ?, email = ?, telephone = ?, responsable_id = ?
         WHERE id = ?
         RETURNING *",
    )
    .bind(&input.nom)
    .bind(&input.prenom)
    .bind(input.date_naissance)
    .bind(&input.email)
    .bind(&input.telephone)
    .bind(input.responsable_id)
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn find_by_id(pool: &SqlitePool, id: i64) -> Result<Option<Personne>, sqlx::Error> {
    let row = sqlx::query_as::<_, Personne>("SELECT * FROM personnes_physiques WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    Ok(row)
}

pub async fn list_all(pool: &SqlitePool) -> Result<Vec<Personne>, sqlx::Error> {
    let rows =
        sqlx::query_as::<_, Personne>("SELECT * FROM personnes_physiques ORDER BY nom, prenom")
            .fetch_all(pool)
            .await?;

    Ok(rows)
}

pub async fn search(pool: &SqlitePool, query: &str) -> Result<Vec<Personne>, sqlx::Error> {
    let pattern = format!("%{}%", query);
    let rows = sqlx::query_as::<_, Personne>(
        "SELECT * FROM personnes_physiques
         WHERE LOWER(nom) LIKE LOWER(?) OR LOWER(prenom) LIKE LOWER(?)
         ORDER BY nom, prenom",
    )
    .bind(&pattern)
    .bind(&pattern)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}
