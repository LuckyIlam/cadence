use async_trait::async_trait;

use crate::error::AppError;

use super::params::DbParams;
use super::row::{DbRow, DeserializeRow};
use super::transaction::DbTransaction;

/// Interface neutre d'accès aux données, indépendante du driver.
///
/// **Object-safe** : toutes les méthodes utilisent des types concrets
/// (`DbParams`, `DbRow`) pour pouvoir être consommées via `dyn Db`
/// (ex. `Arc<dyn Db>` dans `AppState`). L'API générique typée
/// (`fetch_one::<T>`, …) est portée par `DbExt`.
#[async_trait]
pub trait Db: Send + Sync {
    /// Exécute une requête d'écriture et renvoie le nombre de lignes affectées.
    async fn execute(&self, sql: &str, params: DbParams) -> Result<u64, AppError>;

    /// Récupère la première ligne (`None` si aucune), en `DbRow` neutre.
    async fn fetch_optional_row(
        &self,
        sql: &str,
        params: DbParams,
    ) -> Result<Option<DbRow>, AppError>;

    /// Récupère toutes les lignes, en `DbRow` neutre.
    async fn fetch_all_rows(&self, sql: &str, params: DbParams) -> Result<Vec<DbRow>, AppError>;

    /// Ouvre une transaction (`BEGIN` simple).
    async fn begin<'a>(&'a self) -> Result<Box<dyn DbTransaction + 'a>, AppError>;

    /// Ouvre une transaction `BEGIN IMMEDIATE` : prend le verrou d'écriture
    /// immédiatement. Préserve la garantie d'atomicité documentée dans
    /// `planning_commands.rs` ; équivalent du `BEGIN` simple pour Postgres/MySQL.
    async fn begin_immediate<'a>(&'a self) -> Result<Box<dyn DbTransaction + 'a>, AppError>;

    /// Exécute plusieurs instructions SQL dans un même appel (migrations).
    async fn execute_batch(&self, sql: &str) -> Result<(), AppError>;

    fn driver_name(&self) -> &'static str;
}

/// API générique typée par-dessus un `Db` object-safe.
///
/// `params!` produit directement une `DbParams` : les méthodes n'ont donc
/// besoin que du type de retour `T: DeserializeRow`. Implémenté par défaut
/// pour tout `D: Db` (y compris `dyn Db` via `?Sized`).
#[async_trait]
pub trait DbExt: Db {
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
impl<D: Db + ?Sized> DbExt for D {
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

    use chrono::NaiveDate;

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

    struct FausseDb {
        lignes: Mutex<Vec<Vec<DbValue>>>,
        execute_batch_appele: Mutex<bool>,
    }

    #[async_trait]
    impl Db for FausseDb {
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

        async fn begin<'a>(&'a self) -> Result<Box<dyn DbTransaction + 'a>, AppError> {
            unimplemented!("hors test")
        }

        async fn begin_immediate<'a>(&'a self) -> Result<Box<dyn DbTransaction + 'a>, AppError> {
            unimplemented!("hors test")
        }

        async fn execute_batch(&self, _sql: &str) -> Result<(), AppError> {
            *self.execute_batch_appele.lock().unwrap() = true;
            Ok(())
        }

        fn driver_name(&self) -> &'static str {
            "faux"
        }
    }

    fn fausse_db() -> FausseDb {
        FausseDb {
            lignes: Mutex::new(vec![vec![
                DbValue::Integer(7),
                DbValue::Text("Dupont".into()),
            ]]),
            execute_batch_appele: Mutex::new(false),
        }
    }

    #[tokio::test]
    async fn fetch_one_typed() {
        let db = fausse_db();
        let e: Echantillon = db.fetch_one("SELECT", DbParams::new()).await.unwrap();
        assert_eq!(
            e,
            Echantillon {
                id: 7,
                nom: "Dupont".into()
            }
        );
    }

    #[tokio::test]
    async fn fetch_one_not_found() {
        let db = FausseDb {
            lignes: Mutex::new(Vec::new()),
            execute_batch_appele: Mutex::new(false),
        };
        let e: Result<Echantillon, _> = db.fetch_one("SELECT", DbParams::new()).await;
        assert!(matches!(e, Err(AppError::NotFound(_))));
    }

    #[tokio::test]
    async fn fetch_optional_typed() {
        let db = fausse_db();
        let e: Option<Echantillon> = db.fetch_optional("SELECT", DbParams::new()).await.unwrap();
        assert!(e.is_some());
    }

    #[tokio::test]
    async fn execute_batch_forwarded() {
        let db = fausse_db();
        db.execute_batch("BEGIN; COMMIT;").await.unwrap();
        assert!(*db.execute_batch_appele.lock().unwrap());
    }

    #[test]
    fn naive_date_utilisee_pour_le_contrat() {
        let _ = NaiveDate::from_ymd_opt(2000, 1, 15);
    }
}
