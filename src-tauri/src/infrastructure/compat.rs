use serde::Serialize;

use crate::error::AppError;
use crate::infrastructure::db::row::RowView;
use crate::infrastructure::db::Db;
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
pub async fn verifier_compatibilite(db: &dyn Db) -> Result<Compatibilite, AppError> {
    let resultat = db
        .fetch_all_rows("SELECT nom FROM _cadence_migrations", crate::params![])
        .await;

    let rows = match resultat {
        Ok(rows) => rows,
        // Base vierge (table absente) : aucune migration appliquée → compatible.
        Err(e) if e.to_string().to_lowercase().contains("no such table") => {
            return Ok(compatible());
        }
        Err(e) => return Err(e),
    };

    let connus: std::collections::HashSet<&str> = noms_migrations().collect();
    let mut migrations_inconnues: Vec<String> = Vec::new();

    for row in &rows {
        let nom = row.get_str(0)?.to_string();
        if !connus.contains(nom.as_str()) {
            migrations_inconnues.push(nom);
        }
    }

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
    use crate::drivers::libsql::db::LibsqlDb;
    use crate::infrastructure::migrations::cadence_migrations;

    async fn mem_db() -> LibsqlDb {
        let conn = libsql::Builder::new_local(":memory:")
            .build()
            .await
            .expect("failed to create test db")
            .connect()
            .expect("failed to connect test db");
        LibsqlDb::new(conn)
    }

    #[tokio::test]
    async fn base_vierge_compatible() {
        let db = mem_db().await;
        let compat = verifier_compatibilite(&db).await.expect("vérification");
        assert!(compat.compatible);
        assert!(compat.migrations_inconnues.is_empty());
        assert!(!compat.version_installee.is_empty());
    }

    #[tokio::test]
    async fn migrations_connues_compatibles() {
        let db = mem_db().await;
        db.execute_batch(
            "CREATE TABLE IF NOT EXISTS _cadence_migrations (
                nom          TEXT PRIMARY KEY,
                appliquee_le TEXT NOT NULL
            );",
        )
        .await
        .expect("création table");
        for nom in noms_migrations() {
            db.execute(
                "INSERT INTO _cadence_migrations (nom, appliquee_le) VALUES (?, ?)",
                crate::params![nom, "2026-08-09T00:00:00Z"],
            )
            .await
            .expect("insertion");
        }
        // Vérifie aussi que le chemin migrations réel reste compatible.
        let conn = libsql::Builder::new_local(":memory:")
            .build()
            .await
            .expect("failed to create test db")
            .connect()
            .expect("failed to connect test db");
        cadence_migrations(&conn).await.expect("migrations");
        let compat = verifier_compatibilite(&db).await.expect("vérification");
        assert!(compat.compatible);
        assert!(compat.migrations_inconnues.is_empty());
        assert_eq!(compat.version_installee, version_app());
    }

    #[tokio::test]
    async fn migration_inconnue_incompatible() {
        let db = mem_db().await;
        db.execute_batch(
            "CREATE TABLE _cadence_migrations (
                nom          TEXT PRIMARY KEY,
                appliquee_le TEXT NOT NULL
            );",
        )
        .await
        .expect("création table");
        db.execute(
            "INSERT INTO _cadence_migrations (nom, appliquee_le) VALUES (?, ?)",
            crate::params!["99999999999999_futur.sql", "2026-08-09T00:00:00Z"],
        )
        .await
        .expect("insertion");
        let compat = verifier_compatibilite(&db).await.expect("vérification");
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
