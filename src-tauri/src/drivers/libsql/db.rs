//! Implémentation libsql du contrat `Db`.
//!
//! `LibsqlDb` porte le retry « requête-only » (`HranaRetryPolicy`) en interne
//! (design D4) : les repositories appellent `db.fetch_all(…)` sans connaître
//! le driver. `LibsqlDbTransaction` expose une `libsql::Transaction` derrière
//! le trait object-safe `DbTransaction`.
//!
//! Adopté par les repositories en tâche 2.4 : `dead_code` est neutralisé
//! jusqu'à cette adoption.

#![allow(dead_code)]

use async_trait::async_trait;
use libsql::Connection;

use crate::error::AppError;
use crate::infrastructure::db::params::DbValue;
use crate::infrastructure::db::{Db, DbParams, DbRow, DbTransaction};
use crate::infrastructure::retry::RetryPolicy;

use super::hrana;
use super::retry::HranaRetryPolicy;

/// Driver libsql (SQLite local + Turso distant via Hrana).
pub struct LibsqlDb {
    conn: Connection,
}

impl LibsqlDb {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
}

/// Conversion `DbParams → Vec<libsql::Value>` (params positionnels).
///
/// `Vec<libsql::Value>` implémente `IntoParams` chez libsql ; le booléen du
/// contrat (`DbValue::Bool`) n'existant pas côté libsql, il est encodé en
/// INTEGER 0/1 (SQLite ne connaît pas le type booléen).
fn to_libsql_params(params: DbParams) -> Vec<libsql::Value> {
    params.into_iter().map(to_libsql_value).collect()
}

fn to_libsql_value(v: DbValue) -> libsql::Value {
    match v {
        DbValue::Null => libsql::Value::Null,
        DbValue::Integer(i) => libsql::Value::Integer(i),
        DbValue::Real(f) => libsql::Value::Real(f),
        DbValue::Text(s) => libsql::Value::Text(s),
        DbValue::Bool(b) => libsql::Value::Integer(if b { 1 } else { 0 }),
    }
}

/// Conversion `libsql::Row → DbRow` : chaque colonne est lue par index.
fn row_to_dbrow(row: &libsql::Row) -> Result<DbRow, AppError> {
    let mut colonnes = Vec::with_capacity(row.column_count() as usize);
    for idx in 0..row.column_count() {
        let v = row.get_value(idx).map_err(AppError::from)?;
        colonnes.push(to_db_value(v)?);
    }
    Ok(DbRow::new(colonnes))
}

fn to_db_value(v: libsql::Value) -> Result<DbValue, AppError> {
    Ok(match v {
        libsql::Value::Null => DbValue::Null,
        libsql::Value::Integer(i) => DbValue::Integer(i),
        libsql::Value::Real(f) => DbValue::Real(f),
        libsql::Value::Text(s) => DbValue::Text(s),
        libsql::Value::Blob(_) => {
            return Err(AppError::Database(
                "type BLOB non supporté par le contrat Db".into(),
            ))
        }
    })
}

#[async_trait]
impl Db for LibsqlDb {
    async fn execute(&self, sql: &str, params: DbParams) -> Result<u64, AppError> {
        hrana::execute_avec_retry(&self.conn, sql, to_libsql_params(params)).await
    }

    async fn fetch_optional_row(
        &self,
        sql: &str,
        params: DbParams,
    ) -> Result<Option<DbRow>, AppError> {
        let mut rows = hrana::query_avec_retry(&self.conn, sql, to_libsql_params(params)).await?;
        match rows.next().await? {
            Some(row) => {
                let dbrow = row_to_dbrow(&row)?;
                hrana::vider_cursor(&mut rows).await?;
                Ok(Some(dbrow))
            }
            None => Ok(None),
        }
    }

    async fn fetch_all_rows(&self, sql: &str, params: DbParams) -> Result<Vec<DbRow>, AppError> {
        let mut rows = hrana::query_avec_retry(&self.conn, sql, to_libsql_params(params)).await?;
        let mut resultat = Vec::new();
        while let Some(row) = rows.next().await? {
            resultat.push(row_to_dbrow(&row)?);
        }
        Ok(resultat)
    }

    async fn begin<'a>(&'a self) -> Result<Box<dyn DbTransaction + 'a>, AppError> {
        let tx = self.conn.transaction().await?;
        Ok(Box::new(LibsqlDbTransaction::new(tx)))
    }

    async fn begin_immediate<'a>(&'a self) -> Result<Box<dyn DbTransaction + 'a>, AppError> {
        let tx = self
            .conn
            .transaction_with_behavior(libsql::TransactionBehavior::Immediate)
            .await?;
        Ok(Box::new(LibsqlDbTransaction::new(tx)))
    }

    async fn execute_batch(&self, sql: &str) -> Result<(), AppError> {
        let policy = HranaRetryPolicy::new(1);
        policy
            .run(|| async {
                match self.conn.execute_batch(sql).await {
                    Ok(_) => Ok(()),
                    Err(e) if HranaRetryPolicy::matches(&e.to_string()) => {
                        self.conn.reset().await;
                        Err(AppError::from(e))
                    }
                    Err(e) => Err(AppError::from(e)),
                }
            })
            .await
    }

    fn driver_name(&self) -> &'static str {
        "libsql"
    }
}

/// Transaction libsql derrière `Box<dyn DbTransaction>`.
struct LibsqlDbTransaction {
    tx: libsql::Transaction,
}

impl LibsqlDbTransaction {
    fn new(tx: libsql::Transaction) -> Self {
        Self { tx }
    }
}

#[async_trait]
impl DbTransaction for LibsqlDbTransaction {
    async fn execute(&self, sql: &str, params: DbParams) -> Result<u64, AppError> {
        self.tx
            .execute(sql, to_libsql_params(params))
            .await
            .map_err(AppError::from)
    }

    async fn fetch_optional_row(
        &self,
        sql: &str,
        params: DbParams,
    ) -> Result<Option<DbRow>, AppError> {
        let mut rows = self.tx.query(sql, to_libsql_params(params)).await?;
        match rows.next().await? {
            Some(row) => {
                let dbrow = row_to_dbrow(&row)?;
                hrana::vider_cursor(&mut rows).await?;
                Ok(Some(dbrow))
            }
            None => Ok(None),
        }
    }

    async fn fetch_all_rows(&self, sql: &str, params: DbParams) -> Result<Vec<DbRow>, AppError> {
        let mut rows = self.tx.query(sql, to_libsql_params(params)).await?;
        let mut resultat = Vec::new();
        while let Some(row) = rows.next().await? {
            resultat.push(row_to_dbrow(&row)?);
        }
        Ok(resultat)
    }

    async fn commit(self: Box<Self>) -> Result<(), AppError> {
        self.tx.commit().await.map_err(AppError::from)
    }

    async fn rollback(self: Box<Self>) -> Result<(), AppError> {
        self.tx.rollback().await.map_err(AppError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::db::row::RowView;
    use crate::infrastructure::db::DbParams;

    async fn test_db() -> LibsqlDb {
        let database = libsql::Builder::new_local(":memory:")
            .build()
            .await
            .expect("base en mémoire");
        let conn = database.connect().expect("connexion");
        LibsqlDb::new(conn)
    }

    #[tokio::test]
    async fn execute_et_fetch_optional_roundtrip() {
        let db = test_db().await;
        db.execute_batch(
            "CREATE TABLE t (id INTEGER PRIMARY KEY, nom TEXT, note REAL, actif INTEGER)",
        )
        .await
        .unwrap();
        let n = db
            .execute(
                "INSERT INTO t (nom, note, actif) VALUES (?, ?, ?)",
                crate::params!["Dupont", 4.5, true],
            )
            .await
            .unwrap();
        assert_eq!(n, 1);

        let row = db
            .fetch_optional_row(
                "SELECT id, nom, note, actif FROM t WHERE id = 1",
                crate::params![1i64],
            )
            .await
            .unwrap()
            .expect("une ligne");
        assert_eq!(row.get_i64(0).unwrap(), 1);
        assert_eq!(row.get_str(1).unwrap(), "Dupont");
        assert_eq!(row.get_f64(2).unwrap(), 4.5);
        assert!(row.get_bool(3).unwrap());
    }

    #[tokio::test]
    async fn fetch_optional_sans_resultat_renvoie_none() {
        let db = test_db().await;
        db.execute_batch("CREATE TABLE t (id INTEGER PRIMARY KEY, nom TEXT)")
            .await
            .unwrap();
        let row = db
            .fetch_optional_row("SELECT id FROM t", DbParams::new())
            .await
            .unwrap();
        assert!(row.is_none());
    }

    #[tokio::test]
    async fn fetch_all_rows_multiple() {
        let db = test_db().await;
        db.execute_batch("CREATE TABLE t (nom TEXT)").await.unwrap();
        db.execute("INSERT INTO t (nom) VALUES (?)", crate::params!["a"])
            .await
            .unwrap();
        db.execute("INSERT INTO t (nom) VALUES (?)", crate::params!["b"])
            .await
            .unwrap();

        let lignes = db
            .fetch_all_rows("SELECT nom FROM t ORDER BY nom", DbParams::new())
            .await
            .unwrap();
        assert_eq!(lignes.len(), 2);
        assert_eq!(lignes[0].get_str(0).unwrap(), "a");
        assert_eq!(lignes[1].get_str(0).unwrap(), "b");
    }

    #[tokio::test]
    async fn transaction_commit_persiste() {
        let db = test_db().await;
        db.execute_batch("CREATE TABLE t (id INTEGER PRIMARY KEY, nom TEXT)")
            .await
            .unwrap();

        let tx = db.begin().await.unwrap();
        tx.execute("INSERT INTO t (nom) VALUES (?)", crate::params!["x"])
            .await
            .unwrap();
        tx.commit().await.unwrap();

        let lignes = db
            .fetch_all_rows("SELECT nom FROM t", DbParams::new())
            .await
            .unwrap();
        assert_eq!(lignes.len(), 1);
    }

    #[tokio::test]
    async fn transaction_rollback_annule() {
        let db = test_db().await;
        db.execute_batch("CREATE TABLE t (id INTEGER PRIMARY KEY, nom TEXT)")
            .await
            .unwrap();

        let tx = db.begin().await.unwrap();
        tx.execute("INSERT INTO t (nom) VALUES (?)", crate::params!["x"])
            .await
            .unwrap();
        tx.rollback().await.unwrap();

        let lignes = db
            .fetch_all_rows("SELECT nom FROM t", DbParams::new())
            .await
            .unwrap();
        assert!(lignes.is_empty());
    }

    #[tokio::test]
    async fn begin_immediate_commit() {
        let db = test_db().await;
        db.execute_batch("CREATE TABLE t (id INTEGER PRIMARY KEY, nom TEXT)")
            .await
            .unwrap();

        let tx = db.begin_immediate().await.unwrap();
        tx.execute("INSERT INTO t (nom) VALUES (?)", crate::params!["imm"])
            .await
            .unwrap();
        tx.commit().await.unwrap();

        let lignes = db
            .fetch_all_rows("SELECT nom FROM t", DbParams::new())
            .await
            .unwrap();
        assert_eq!(lignes.len(), 1);
    }

    #[test]
    fn bool_encode_en_integer_0_1() {
        assert_eq!(
            to_libsql_value(DbValue::Bool(true)),
            libsql::Value::Integer(1)
        );
        assert_eq!(
            to_libsql_value(DbValue::Bool(false)),
            libsql::Value::Integer(0)
        );
    }

    #[test]
    fn blob_non_supporte() {
        assert!(to_db_value(libsql::Value::Blob(vec![1, 2, 3])).is_err());
    }
}
