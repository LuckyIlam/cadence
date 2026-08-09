use libsql::Connection;
use serde::Serialize;

use crate::error::AppError;
use crate::infrastructure::hrana_guard;
use crate::infrastructure::migrations::noms_migrations;

pub fn version_app() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[derive(Debug, Clone, Serialize)]
pub struct Compatibilite {
    pub compatible: bool,
    pub version_installee: String,
    pub migrations_inconnues: Vec<String>,
}

fn compatible() -> Compatibilite {
    Compatibilite {
        compatible: true,
        version_installee: version_app(),
        migrations_inconnues: Vec::new(),
    }
}

/// Compare les migrations appliquées à la base (`_cadence_migrations`) avec la
/// liste connue du binaire. Une migration inconnue signifie que la base a été
/// mise à jour par une version plus récente de l'application : blocage.
pub async fn verifier_compatibilite(conn: &Connection) -> Result<Compatibilite, AppError> {
    let resultat = hrana_guard::query_avec_retry(
        conn,
        "SELECT nom FROM _cadence_migrations",
        libsql::params![],
    )
    .await;

    let mut rows = match resultat {
        Ok(rows) => rows,
        // Base vierge (table absente) : aucune migration appliquée → compatible.
        Err(e) if e.to_string().to_lowercase().contains("no such table") => {
            return Ok(compatible());
        }
        Err(e) => return Err(e),
    };

    let connus: std::collections::HashSet<&str> = noms_migrations().collect();
    let mut migrations_inconnues: Vec<String> = Vec::new();

    while let Some(row) = rows.next().await? {
        let nom: String = row.get(0)?;
        if !connus.contains(nom.as_str()) {
            migrations_inconnues.push(nom);
        }
    }
    hrana_guard::vider_cursor(&mut rows).await?;

    if migrations_inconnues.is_empty() {
        Ok(compatible())
    } else {
        Ok(Compatibilite {
            compatible: false,
            version_installee: version_app(),
            migrations_inconnues,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::migrations::cadence_migrations;

    async fn mem_conn() -> Connection {
        libsql::Builder::new_local(":memory:")
            .build()
            .await
            .expect("failed to create test db")
            .connect()
            .expect("failed to connect test db")
    }

    #[tokio::test]
    async fn base_vierge_compatible() {
        let conn = mem_conn().await;
        let compat = verifier_compatibilite(&conn).await.expect("vérification");
        assert!(compat.compatible);
        assert!(compat.migrations_inconnues.is_empty());
        assert!(!compat.version_installee.is_empty());
    }

    #[tokio::test]
    async fn migrations_connues_compatibles() {
        let conn = mem_conn().await;
        cadence_migrations(&conn).await.expect("migrations");
        let compat = verifier_compatibilite(&conn).await.expect("vérification");
        assert!(compat.compatible);
        assert!(compat.migrations_inconnues.is_empty());
        assert_eq!(compat.version_installee, version_app());
    }

    #[tokio::test]
    async fn migration_inconnue_incompatible() {
        let conn = mem_conn().await;
        conn.execute_batch(
            "CREATE TABLE _cadence_migrations (
                nom          TEXT PRIMARY KEY,
                appliquee_le TEXT NOT NULL
            );",
        )
        .await
        .expect("création table");
        conn.execute(
            "INSERT INTO _cadence_migrations (nom, appliquee_le) VALUES (?, ?)",
            libsql::params!["99999999999999_futur.sql", "2026-08-09T00:00:00Z"],
        )
        .await
        .expect("insertion");
        let compat = verifier_compatibilite(&conn).await.expect("vérification");
        assert!(!compat.compatible);
        assert_eq!(
            compat.migrations_inconnues,
            vec!["99999999999999_futur.sql".to_string()]
        );
        assert_eq!(compat.version_installee, version_app());
    }

    #[test]
    fn version_app_non_vide() {
        assert!(!version_app().is_empty());
    }
}
