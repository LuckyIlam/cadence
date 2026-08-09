use libsql::params::Params;
use libsql::Connection;

use crate::error::AppError;
use crate::infrastructure::retry::RetryPolicy;

use super::retry::HranaRetryPolicy;

/// Exécute une requête en réinitialisant le stream Hrana et en réessayant une
/// fois en cas d'erreur « stream not found ». L'échec survient au moment du
/// `prepare`/describe, avant toute exécution côté serveur : la retry est sûre.
///
/// La logique du retry est portée par `HranaRetryPolicy` (1 retry) ; la
/// fermeture se contente de réinitialiser le stream avant de propager l'erreur.
pub async fn query_avec_retry<P: libsql::params::IntoParams>(
    conn: &Connection,
    sql: &str,
    params: P,
) -> Result<libsql::Rows, AppError> {
    let params: Params = params.into_params().map_err(AppError::from)?;
    let policy = HranaRetryPolicy::new(1);
    policy
        .run(|| async {
            match conn.query(sql, &params).await {
                Ok(rows) => Ok(rows),
                Err(e) if HranaRetryPolicy::matches(&e.to_string()) => {
                    conn.reset().await;
                    Err(AppError::from(e))
                }
                Err(e) => Err(AppError::from(e)),
            }
        })
        .await
}

/// Exécute une commande d'écriture en réinitialisant le stream Hrana et en
/// réessayant une fois en cas d'erreur « stream not found ».
pub async fn execute_avec_retry<P: libsql::params::IntoParams>(
    conn: &Connection,
    sql: &str,
    params: P,
) -> Result<u64, AppError> {
    let params: Params = params.into_params().map_err(AppError::from)?;
    let policy = HranaRetryPolicy::new(1);
    policy
        .run(|| async {
            match conn.execute(sql, &params).await {
                Ok(n) => Ok(n),
                Err(e) if HranaRetryPolicy::matches(&e.to_string()) => {
                    conn.reset().await;
                    Err(AppError::from(e))
                }
                Err(e) => Err(AppError::from(e)),
            }
        })
        .await
}

/// Consomme les lignes restantes d'un curseur pour ne jamais l'abandonner côté
/// serveur (un curseur abandonné ferme le stream Hrana distant).
pub async fn vider_cursor(rows: &mut libsql::Rows) -> Result<(), AppError> {
    while let Some(_row) = rows.next().await? {}
    Ok(())
}
