use libsql::params::Params;
use libsql::Connection;

use crate::error::AppError;

/// Détecte l'erreur serveur « stream not found » (baton Hrana devenu invalide :
/// curseur abandonné ou stream expiré côté serveur Turso).
fn est_stream_perdu(msg: &str) -> bool {
    let m = msg.to_lowercase();
    m.contains("stream not found") || m.contains("stream_not_found")
}

/// Exécute une requête en réinitialisant le stream Hrana et en réessayant une
/// fois en cas d'erreur « stream not found ». L'échec survient au moment du
/// `prepare`/describe, avant toute exécution côté serveur : la retry est sûre.
pub async fn query_avec_retry<P: libsql::params::IntoParams>(
    conn: &Connection,
    sql: &str,
    params: P,
) -> Result<libsql::Rows, AppError> {
    let params: Params = params.into_params().map_err(AppError::from)?;
    match conn.query(sql, &params).await {
        Ok(rows) => Ok(rows),
        Err(e) if est_stream_perdu(&e.to_string()) => {
            conn.reset().await;
            conn.query(sql, &params).await.map_err(AppError::from)
        }
        Err(e) => Err(AppError::from(e)),
    }
}

/// Exécute une commande d'écriture en réinitialisant le stream Hrana et en
/// réessayant une fois en cas d'erreur « stream not found ».
pub async fn execute_avec_retry<P: libsql::params::IntoParams>(
    conn: &Connection,
    sql: &str,
    params: P,
) -> Result<u64, AppError> {
    let params: Params = params.into_params().map_err(AppError::from)?;
    match conn.execute(sql, &params).await {
        Ok(n) => Ok(n),
        Err(e) if est_stream_perdu(&e.to_string()) => {
            conn.reset().await;
            conn.execute(sql, &params).await.map_err(AppError::from)
        }
        Err(e) => Err(AppError::from(e)),
    }
}

/// Consomme les lignes restantes d'un curseur pour ne jamais l'abandonner côté
/// serveur (un curseur abandonné ferme le stream Hrana distant).
pub async fn vider_cursor(rows: &mut libsql::Rows) -> Result<(), AppError> {
    while let Some(_row) = rows.next().await? {}
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detecte_stream_not_found() {
        assert!(est_stream_perdu(
            "Hrana: `api error: `status=404 Not Found, body={\"error\":\"stream not found: 4712e6b3:25d2\"}`"
        ));
        assert!(est_stream_perdu("STREAM_NOT_FOUND"));
        assert!(est_stream_perdu("stream_not_found"));
    }

    #[test]
    fn ignore_les_autres_erreurs() {
        assert!(!est_stream_perdu("no such table: foo"));
        assert!(!est_stream_perdu("database is locked"));
        assert!(!est_stream_perdu(""));
    }
}
