use libsql::Connection;

use crate::error::AppError;

const MIGRATIONS: &[(&str, &str)] = &[
    (
        "20260630000001_create_personnes_physiques.sql",
        include_str!("../../migrations/20260630000001_create_personnes_physiques.sql"),
    ),
    (
        "20260630000002_create_adhesions.sql",
        include_str!("../../migrations/20260630000002_create_adhesions.sql"),
    ),
    (
        "20260703000001_create_activites.sql",
        include_str!("../../migrations/20260703000001_create_activites.sql"),
    ),
    (
        "20260703000002_create_tarifs_activite.sql",
        include_str!("../../migrations/20260703000002_create_tarifs_activite.sql"),
    ),
    (
        "20260703000003_create_activite_personnes.sql",
        include_str!("../../migrations/20260703000003_create_activite_personnes.sql"),
    ),
    (
        "20260710000001_create_creneaux_activite.sql",
        include_str!("../../migrations/20260710000001_create_creneaux_activite.sql"),
    ),
    (
        "20260710000002_create_semaines_banalisees.sql",
        include_str!("../../migrations/20260710000002_create_semaines_banalisees.sql"),
    ),
    (
        "20260802000001_create_parametres.sql",
        include_str!("../../migrations/20260802000001_create_parametres.sql"),
    ),
];

fn maintenant_utc() -> String {
    chrono::Utc::now().to_rfc3339()
}

async fn table_existe(conn: &Connection, nom: &str) -> Result<bool, AppError> {
    let mut rows = conn
        .query(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name = ?",
            libsql::params![nom],
        )
        .await?;
    Ok(rows.next().await?.is_some())
}

async fn migration_appliquee(conn: &Connection, nom: &str) -> Result<bool, AppError> {
    let mut rows = conn
        .query(
            "SELECT 1 FROM _cadence_migrations WHERE nom = ?",
            libsql::params![nom],
        )
        .await?;
    Ok(rows.next().await?.is_some())
}

async fn copier_bookkeeping_sqlx(conn: &Connection) -> Result<(), AppError> {
    #[derive(Debug, Clone, serde::Deserialize)]
    struct SqlxMigrationRow {
        version: i64,
        description: String,
    }

    if !table_existe(conn, "_sqlx_migrations").await? {
        return Ok(());
    }

    let mut rows = conn
        .query(
            "SELECT version, description FROM _sqlx_migrations WHERE success = 1",
            libsql::params![],
        )
        .await?;

    while let Some(row) = rows.next().await? {
        let r = libsql::de::from_row::<SqlxMigrationRow>(&row)?;
        let nom = format!("{}_{}.sql", r.version, r.description);
        conn.execute(
            "INSERT OR IGNORE INTO _cadence_migrations (nom, appliquee_le) VALUES (?, ?)",
            libsql::params![nom, maintenant_utc()],
        )
        .await?;
    }

    Ok(())
}

pub async fn cadence_migrations(conn: &Connection) -> Result<(), AppError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _cadence_migrations (
            nom          TEXT PRIMARY KEY,
            appliquee_le TEXT NOT NULL
        );",
    )
    .await?;

    copier_bookkeeping_sqlx(conn).await?;

    for (nom, sql) in MIGRATIONS {
        if migration_appliquee(conn, nom).await? {
            continue;
        }

        let batch = format!("BEGIN;\n{}\nCOMMIT;", sql);
        conn.execute_batch(&batch).await?;

        conn.execute(
            "INSERT INTO _cadence_migrations (nom, appliquee_le) VALUES (?, ?)",
            libsql::params![nom, maintenant_utc()],
        )
        .await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn mem_conn() -> Connection {
        libsql::Builder::new_local(":memory:")
            .build()
            .await
            .expect("failed to create test db")
            .connect()
            .expect("failed to connect test db")
    }

    #[derive(Debug, Clone, serde::Deserialize)]
    struct CompteurRow {
        count: i64,
    }

    async fn nb_migrations(conn: &Connection) -> i64 {
        let mut rows = conn
            .query(
                "SELECT COUNT(*) AS count FROM _cadence_migrations",
                libsql::params![],
            )
            .await
            .unwrap();
        let row = rows.next().await.unwrap().unwrap();
        libsql::de::from_row::<CompteurRow>(&row).unwrap().count
    }

    #[tokio::test]
    async fn applique_toutes_les_migrations() {
        let conn = mem_conn().await;
        cadence_migrations(&conn).await.expect("migrations failed");

        assert_eq!(nb_migrations(&conn).await, MIGRATIONS.len() as i64);

        let mut rows = conn
            .query(
                "SELECT COUNT(*) AS count FROM sqlite_master WHERE type = 'table' AND name = 'personnes_physiques'",
                libsql::params![],
            )
            .await
            .unwrap();
        let row = rows.next().await.unwrap().unwrap();
        let count = libsql::de::from_row::<CompteurRow>(&row).unwrap().count;
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn ne_reapplique_pas() {
        let conn = mem_conn().await;
        cadence_migrations(&conn).await.expect("migrations failed");
        cadence_migrations(&conn).await.expect("second run failed");

        assert_eq!(nb_migrations(&conn).await, MIGRATIONS.len() as i64);
    }
}
