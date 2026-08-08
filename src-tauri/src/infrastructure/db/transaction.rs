use async_trait::async_trait;

use crate::error::AppError;

use super::params::DbParams;
use super::row::{DbRow, DeserializeRow};

/// Transaction neutre, indépendante du driver.
///
/// **Object-safe** (types concrets `DbParams` / `DbRow`) pour pouvoir être
/// consommée via `Box<dyn DbTransaction>`. `commit` / `rollback` consomment
/// la transaction (`self: Box<Self>`) : l'état final est déterministe, un
/// double `commit` est impossible.
#[async_trait]
pub trait DbTransaction: Send + Sync {
    async fn execute(&self, sql: &str, params: DbParams) -> Result<u64, AppError>;

    async fn fetch_optional_row(
        &self,
        sql: &str,
        params: DbParams,
    ) -> Result<Option<DbRow>, AppError>;

    async fn fetch_all_rows(&self, sql: &str, params: DbParams) -> Result<Vec<DbRow>, AppError>;

    async fn commit(self: Box<Self>) -> Result<(), AppError>;
    async fn rollback(self: Box<Self>) -> Result<(), AppError>;
}

/// API générique typée par-dessus un `DbTransaction` object-safe.
///
/// Symétrique de `DbExt` : les méthodes `*_tx` des repositories doivent lire
/// des lignes typées dans la transaction sans connaitre le driver. Consommée
/// via `&mut *tx` (déréférencement d'un `Box<dyn DbTransaction>`).
#[async_trait]
pub trait DbTransactionExt: DbTransaction {
    /// Récupère exactement une ligne ; `NotFound` si la requête ne renvoie rien.
    async fn fetch_one<T>(&self, sql: &str, params: DbParams) -> Result<T, AppError>
    where
        T: DeserializeRow + Send;

    /// Récupère au plus une ligne (retourne `None` si rien).
    async fn fetch_optional<T>(&self, sql: &str, params: DbParams) -> Result<Option<T>, AppError>
    where
        T: DeserializeRow + Send;

    /// Récupère toutes les lignes décodées.
    async fn fetch_all<T>(&self, sql: &str, params: DbParams) -> Result<Vec<T>, AppError>
    where
        T: DeserializeRow + Send;
}

#[async_trait]
impl<D: DbTransaction + ?Sized> DbTransactionExt for D {
    async fn fetch_one<T>(&self, sql: &str, params: DbParams) -> Result<T, AppError>
    where
        T: DeserializeRow + Send,
    {
        match self.fetch_optional_row(sql, params).await? {
            Some(row) => T::from_row(&row),
            None => Err(AppError::NotFound("aucune ligne renvoyée".into())),
        }
    }

    async fn fetch_optional<T>(&self, sql: &str, params: DbParams) -> Result<Option<T>, AppError>
    where
        T: DeserializeRow + Send,
    {
        match self.fetch_optional_row(sql, params).await? {
            Some(row) => T::from_row(&row).map(Some),
            None => Ok(None),
        }
    }

    async fn fetch_all<T>(&self, sql: &str, params: DbParams) -> Result<Vec<T>, AppError>
    where
        T: DeserializeRow + Send,
    {
        let mut resultat = Vec::new();
        for row in self.fetch_all_rows(sql, params).await? {
            resultat.push(T::from_row(&row)?);
        }
        Ok(resultat)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    use crate::infrastructure::db::params::DbValue;
    use crate::infrastructure::db::row::RowView;

    #[derive(Debug, PartialEq)]
    struct Echantillon {
        id: i64,
        nom: String,
    }

    impl DeserializeRow for Echantillon {
        fn from_row(row: &dyn RowView) -> Result<Self, AppError> {
            Ok(Echantillon {
                id: row.get_i64(0)?,
                nom: row.get_str(1)?.to_string(),
            })
        }
    }

    struct FausseTransaction {
        lignes: Mutex<Vec<Vec<DbValue>>>,
        commitee: Mutex<bool>,
    }

    impl FausseTransaction {
        fn nouvelle() -> Self {
            Self {
                lignes: Mutex::new(vec![vec![
                    DbValue::Integer(7),
                    DbValue::Text("Dupont".into()),
                ]]),
                commitee: Mutex::new(false),
            }
        }
    }

    #[async_trait]
    impl DbTransaction for FausseTransaction {
        async fn execute(&self, _sql: &str, _params: DbParams) -> Result<u64, AppError> {
            Ok(1)
        }

        async fn fetch_optional_row(
            &self,
            _sql: &str,
            _params: DbParams,
        ) -> Result<Option<DbRow>, AppError> {
            let lignes = self.lignes.lock().unwrap();
            Ok(lignes.first().cloned().map(DbRow::new))
        }

        async fn fetch_all_rows(
            &self,
            _sql: &str,
            _params: DbParams,
        ) -> Result<Vec<DbRow>, AppError> {
            let lignes = self.lignes.lock().unwrap();
            Ok(lignes.iter().cloned().map(DbRow::new).collect())
        }

        async fn commit(self: Box<Self>) -> Result<(), AppError> {
            *self.commitee.lock().unwrap() = true;
            Ok(())
        }

        async fn rollback(self: Box<Self>) -> Result<(), AppError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn fetch_one_typed_dans_transaction() {
        let tx: Box<dyn DbTransaction> = Box::new(FausseTransaction::nouvelle());
        let e: Echantillon = tx.fetch_one("SELECT", DbParams::new()).await.unwrap();
        assert_eq!(
            e,
            Echantillon {
                id: 7,
                nom: "Dupont".into()
            }
        );
    }

    #[tokio::test]
    async fn fetch_one_not_found_dans_transaction() {
        let tx: Box<dyn DbTransaction> = Box::new(FausseTransaction {
            lignes: Mutex::new(Vec::new()),
            commitee: Mutex::new(false),
        });
        let e: Result<Echantillon, _> = tx.fetch_one("SELECT", DbParams::new()).await;
        assert!(matches!(e, Err(AppError::NotFound(_))));
    }

    #[tokio::test]
    async fn fetch_optional_typed_dans_transaction() {
        let tx: Box<dyn DbTransaction> = Box::new(FausseTransaction::nouvelle());
        let e: Option<Echantillon> = tx.fetch_optional("SELECT", DbParams::new()).await.unwrap();
        assert!(e.is_some());
    }

    #[tokio::test]
    async fn fetch_all_typed_dans_transaction() {
        let tx: Box<dyn DbTransaction> = Box::new(FausseTransaction {
            lignes: Mutex::new(vec![
                vec![DbValue::Integer(1), DbValue::Text("A".into())],
                vec![DbValue::Integer(2), DbValue::Text("B".into())],
            ]),
            commitee: Mutex::new(false),
        });
        let lignes: Vec<Echantillon> = tx.fetch_all("SELECT", DbParams::new()).await.unwrap();
        assert_eq!(lignes.len(), 2);
        assert_eq!(lignes[0].nom, "A");
        assert_eq!(lignes[1].nom, "B");
    }

    #[tokio::test]
    async fn commit_consomme_la_transaction() {
        let tx: Box<dyn DbTransaction> = Box::new(FausseTransaction::nouvelle());
        tx.commit().await.unwrap();
    }
}
