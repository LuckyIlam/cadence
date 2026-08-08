pub mod activite;
pub mod adhesion;
pub mod personne;
pub mod planning;

use crate::domain::activite::Role;
use crate::error::AppError;
use crate::infrastructure::db::RowView;

pub(crate) fn role_from_row(row: &dyn RowView, idx: usize) -> Result<Role, AppError> {
    crate::domain::activite::role_from_str(row.get_str(idx)?).map_err(AppError::Database)
}
