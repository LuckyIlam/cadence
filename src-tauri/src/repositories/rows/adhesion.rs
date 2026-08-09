use crate::error::AppError;
use crate::infrastructure::db::{DeserializeRow, RowView};

pub struct IdRow {
    #[allow(dead_code)]
    pub id: i64,
}

impl DeserializeRow for IdRow {
    fn from_row(row: &dyn RowView) -> Result<Self, AppError> {
        Ok(IdRow {
            id: row.get_i64(0)?,
        })
    }
}
