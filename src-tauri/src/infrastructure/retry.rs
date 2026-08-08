use std::future::Future;

use async_trait::async_trait;

use crate::error::AppError;

/// Politique de nouvelle tentative sur erreur transitoire, indépendante du driver.
///
/// Le grain est la **requête isolée** (cf. design D4) : une politique comme
/// `HranaRetryPolicy` peut rejouer une requête car son échec survient au
/// `prepare`/describe, avant toute exécution côté serveur. Rejouer une
/// transaction entière est hors de ce trait : ce replay est porté par les
/// appels à `DbTransaction` dans les services.
///
/// `F` est `FnMut` (et non `FnOnce`) car une politique retente plusieurs fois :
/// elle doit pouvoir ré-invoquer l'opération.
#[async_trait]
pub trait RetryPolicy: Send + Sync {
    async fn run<F, Fut, T>(&self, op: F) -> Result<T, AppError>
    where
        F: FnMut() -> Fut + Send,
        Fut: Future<Output = Result<T, AppError>> + Send;
}
