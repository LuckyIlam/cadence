use crate::error::AppError;
use crate::infrastructure::db::{DeserializeRow, RowView};

pub struct TotalRow {
    pub count: i64,
}

impl DeserializeRow for TotalRow {
    fn from_row(row: &dyn RowView) -> Result<Self, AppError> {
        Ok(TotalRow {
            count: row.get_i64(0)?,
        })
    }
}
