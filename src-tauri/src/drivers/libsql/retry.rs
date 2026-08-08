use std::future::Future;

use async_trait::async_trait;

use crate::error::AppError;
use crate::infrastructure::retry::RetryPolicy;

/// Détecte l'erreur serveur « stream not found » (baton Hrana devenu invalide :
/// curseur abandonné ou stream expiré côté serveur Turso).
fn est_stream_perdu(msg: &str) -> bool {
    let m = msg.to_lowercase();
    m.contains("stream not found") || m.contains("stream_not_found")
}

/// Politique de retry spécifique Hrana (libsql distant).
///
/// L'échec « stream not found » survient au `prepare`/describe, avant toute
/// exécution côté serveur : la retry est sûre au grain requête (design D4).
pub struct HranaRetryPolicy {
    max_retries: u32,
}

impl HranaRetryPolicy {
    pub fn new(max_retries: u32) -> Self {
        Self { max_retries }
    }

    /// S'applique-t-il à cette erreur ? (ex-`hrana_guard::est_stream_perdu`).
    pub fn matches(msg: &str) -> bool {
        est_stream_perdu(msg)
    }
}

#[async_trait]
impl RetryPolicy for HranaRetryPolicy {
    async fn run<F, Fut, T>(&self, mut op: F) -> Result<T, AppError>
    where
        F: FnMut() -> Fut + Send,
        Fut: Future<Output = Result<T, AppError>> + Send,
    {
        let mut tentative = 0u32;
        loop {
            match op().await {
                Err(e) if Self::matches(&e.to_string()) && tentative < self.max_retries => {
                    tentative += 1;
                }
                result => return result,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detecte_stream_not_found() {
        assert!(HranaRetryPolicy::matches(
            "Hrana: `api error: `status=404 Not Found, body={\"error\":\"stream not found: 4712e6b3:25d2\"}`"
        ));
        assert!(HranaRetryPolicy::matches("STREAM_NOT_FOUND"));
        assert!(HranaRetryPolicy::matches("stream_not_found"));
    }

    #[test]
    fn ignore_les_autres_erreurs() {
        assert!(!HranaRetryPolicy::matches("no such table: foo"));
        assert!(!HranaRetryPolicy::matches("database is locked"));
        assert!(!HranaRetryPolicy::matches(""));
    }

    #[tokio::test]
    async fn retente_et_reussit() {
        let policy = HranaRetryPolicy::new(2);
        let essais = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let resultat = {
            let compteur = essais.clone();
            policy
                .run(move || {
                    let compteur = compteur.clone();
                    async move {
                        let n = compteur.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                        if n < 3 {
                            Err(AppError::Database("stream_not_found".into()))
                        } else {
                            Ok(42)
                        }
                    }
                })
                .await
        };
        assert_eq!(resultat.unwrap(), 42);
        assert_eq!(essais.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn abandonne_apres_max_retries() {
        let policy = HranaRetryPolicy::new(1);
        let essais = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let resultat = {
            let compteur = essais.clone();
            policy
                .run(move || {
                    let compteur = compteur.clone();
                    async move {
                        compteur.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        Err::<i32, AppError>(AppError::Database("stream_not_found".into()))
                    }
                })
                .await
        };
        assert!(resultat.is_err());
        assert_eq!(essais.load(std::sync::atomic::Ordering::SeqCst), 2); // 1 tentative + 1 retry
    }

    #[tokio::test]
    async fn ne_retente_pas_les_autres_erreurs() {
        let policy = HranaRetryPolicy::new(3);
        let essais = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let resultat = {
            let compteur = essais.clone();
            policy
                .run(move || {
                    let compteur = compteur.clone();
                    async move {
                        compteur.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        Err::<i32, AppError>(AppError::Database("no such table".into()))
                    }
                })
                .await
        };
        assert!(resultat.is_err());
        assert_eq!(essais.load(std::sync::atomic::Ordering::SeqCst), 1);
    }
}
