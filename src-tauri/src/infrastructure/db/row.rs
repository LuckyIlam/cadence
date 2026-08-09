use chrono::NaiveDate;

use crate::error::AppError;

use super::params::DbValue;

/// Vue neutre d'une ligne de résultat, indépendante du driver.
///
/// L'implémentation canonique est `DbRow` (valeurs typées en mémoire) ; un
/// driver peut aussi l'implémenter sur son type natif.
pub trait RowView: Send + Sync {
    fn get_i64(&self, idx: usize) -> Result<i64, AppError>;
    fn get_opt_i64(&self, idx: usize) -> Result<Option<i64>, AppError>;
    fn get_str(&self, idx: usize) -> Result<&str, AppError>;
    fn get_opt_str(&self, idx: usize) -> Result<Option<&str>, AppError>;
    fn get_bool(&self, idx: usize) -> Result<bool, AppError>;
    fn get_opt_bool(&self, idx: usize) -> Result<Option<bool>, AppError>;
    fn get_f64(&self, idx: usize) -> Result<f64, AppError>;
    fn get_opt_f64(&self, idx: usize) -> Result<Option<f64>, AppError>;
    /// Date au format ISO `YYYY-MM-DD` (stockée en TEXT par les migrations).
    fn get_naive_date(&self, idx: usize) -> Result<NaiveDate, AppError>;
}

/// Ligne neutre en mémoire : une `Vec<DbValue>` typée.
///
/// Produite par l'implémentation `Db` de chaque driver (conversion du type
/// natif), lue par `DeserializeRow`. `Send + Sync` et 'static : la ligne peut
/// traverser les `await` et être traitée hors du borrow du curseur.
#[derive(Debug, Clone, PartialEq)]
pub struct DbRow {
    colonnes: Vec<DbValue>,
}

impl DbRow {
    pub fn new(colonnes: Vec<DbValue>) -> Self {
        Self { colonnes }
    }

    pub fn len(&self) -> usize {
        self.colonnes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.colonnes.is_empty()
    }

    fn valeur(&self, idx: usize) -> Result<&DbValue, AppError> {
        self.colonnes
            .get(idx)
            .ok_or_else(|| AppError::Database(format!("colonne {idx} absente de la ligne")))
    }
}

impl RowView for DbRow {
    fn get_i64(&self, idx: usize) -> Result<i64, AppError> {
        match self.valeur(idx)? {
            DbValue::Integer(v) => Ok(*v),
            DbValue::Text(t) => t.parse().map_err(|_| {
                AppError::Database(format!("colonne {idx} : {t} n'est pas un entier"))
            }),
            autre => Err(AppError::Database(format!(
                "colonne {idx} : i64 attendu, trouvé {autre:?}"
            ))),
        }
    }

    fn get_opt_i64(&self, idx: usize) -> Result<Option<i64>, AppError> {
        match self.valeur(idx)? {
            DbValue::Null => Ok(None),
            DbValue::Integer(v) => Ok(Some(*v)),
            autre => Err(AppError::Database(format!(
                "colonne {idx} : Option<i64> attendu, trouvé {autre:?}"
            ))),
        }
    }

    fn get_str(&self, idx: usize) -> Result<&str, AppError> {
        match self.valeur(idx)? {
            DbValue::Text(s) => Ok(s),
            autre => Err(AppError::Database(format!(
                "colonne {idx} : texte attendu, trouvé {autre:?}"
            ))),
        }
    }

    fn get_opt_str(&self, idx: usize) -> Result<Option<&str>, AppError> {
        match self.valeur(idx)? {
            DbValue::Null => Ok(None),
            DbValue::Text(s) => Ok(Some(s)),
            autre => Err(AppError::Database(format!(
                "colonne {idx} : Option<texte> attendu, trouvé {autre:?}"
            ))),
        }
    }

    fn get_bool(&self, idx: usize) -> Result<bool, AppError> {
        match self.valeur(idx)? {
            DbValue::Bool(b) => Ok(*b),
            DbValue::Integer(v) => Ok(*v != 0),
            autre => Err(AppError::Database(format!(
                "colonne {idx} : booléen attendu, trouvé {autre:?}"
            ))),
        }
    }

    fn get_opt_bool(&self, idx: usize) -> Result<Option<bool>, AppError> {
        match self.valeur(idx)? {
            DbValue::Null => Ok(None),
            DbValue::Bool(b) => Ok(Some(*b)),
            autre => Err(AppError::Database(format!(
                "colonne {idx} : Option<bool> attendu, trouvé {autre:?}"
            ))),
        }
    }

    fn get_f64(&self, idx: usize) -> Result<f64, AppError> {
        match self.valeur(idx)? {
            DbValue::Real(v) => Ok(*v),
            DbValue::Integer(v) => Ok(*v as f64),
            autre => Err(AppError::Database(format!(
                "colonne {idx} : réel attendu, trouvé {autre:?}"
            ))),
        }
    }

    fn get_opt_f64(&self, idx: usize) -> Result<Option<f64>, AppError> {
        match self.valeur(idx)? {
            DbValue::Null => Ok(None),
            DbValue::Real(v) => Ok(Some(*v)),
            autre => Err(AppError::Database(format!(
                "colonne {idx} : Option<réel> attendu, trouvé {autre:?}"
            ))),
        }
    }

    fn get_naive_date(&self, idx: usize) -> Result<NaiveDate, AppError> {
        let s = self.get_str(idx)?;
        NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .map_err(|e| AppError::Database(format!("colonne {idx} : date invalide « {s} » : {e}")))
    }
}

/// Conversion d'une `RowView` en un type applicatif.
///
/// Chaque struct (domain ou helper de requête) implémente ce trait de
/// manière mécanique (~15 lignes) en lisant les colonnes par index, dans
/// l'ordre du `SELECT`.
pub trait DeserializeRow: Sized {
    fn from_row(row: &dyn RowView) -> Result<Self, AppError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lit_les_valeurs_par_index() {
        let row = DbRow::new(vec![
            DbValue::Integer(42),
            DbValue::Text("Dupont".into()),
            DbValue::Bool(true),
            DbValue::Real(3.5),
            DbValue::Text("2000-01-15".into()),
        ]);
        assert_eq!(row.get_i64(0).unwrap(), 42);
        assert_eq!(row.get_str(1).unwrap(), "Dupont");
        assert!(row.get_bool(2).unwrap());
        assert_eq!(row.get_f64(3).unwrap(), 3.5);
        assert_eq!(
            row.get_naive_date(4).unwrap(),
            NaiveDate::from_ymd_opt(2000, 1, 15).unwrap()
        );
    }

    #[test]
    fn null_devient_option_none() {
        let row = DbRow::new(vec![DbValue::Null]);
        assert_eq!(row.get_opt_i64(0).unwrap(), None);
        assert_eq!(row.get_opt_str(0).unwrap(), None);
        assert_eq!(row.get_opt_bool(0).unwrap(), None);
        assert_eq!(row.get_opt_f64(0).unwrap(), None);
    }

    #[test]
    fn entier_converti_en_bool_et_reel() {
        let row = DbRow::new(vec![DbValue::Integer(1)]);
        assert!(row.get_bool(0).unwrap());
        assert_eq!(row.get_f64(0).unwrap(), 1.0);
    }

    #[test]
    fn index_absent_en_erreur() {
        let row = DbRow::new(vec![DbValue::Integer(1)]);
        assert!(row.get_i64(5).is_err());
    }

    #[test]
    fn type_inattendu_en_erreur() {
        let row = DbRow::new(vec![DbValue::Text("x".into())]);
        assert!(row.get_i64(0).is_err());
    }
}
