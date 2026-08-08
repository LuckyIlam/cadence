use async_trait::async_trait;

use crate::error::AppError;

use super::params::DbParams;
use super::row::DbRow;

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
