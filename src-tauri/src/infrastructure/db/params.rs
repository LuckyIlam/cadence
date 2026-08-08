/// Valeur neutre portée par les requêtes, indépendante du driver.
///
/// L'adaptation vers le type natif du driver (ex. `libsql::Value`,
/// `tokio_postgres::types::ToSql`) se fait dans le driver.
#[derive(Debug, Clone, PartialEq)]
pub enum DbValue {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Bool(bool),
}

impl DbValue {
    pub fn is_null(&self) -> bool {
        matches!(self, DbValue::Null)
    }
}

impl From<i64> for DbValue {
    fn from(v: i64) -> Self {
        DbValue::Integer(v)
    }
}

impl From<i32> for DbValue {
    fn from(v: i32) -> Self {
        DbValue::Integer(v as i64)
    }
}

impl From<u32> for DbValue {
    fn from(v: u32) -> Self {
        DbValue::Integer(v as i64)
    }
}

impl From<f64> for DbValue {
    fn from(v: f64) -> Self {
        DbValue::Real(v)
    }
}

impl From<bool> for DbValue {
    fn from(v: bool) -> Self {
        DbValue::Bool(v)
    }
}

impl From<String> for DbValue {
    fn from(v: String) -> Self {
        DbValue::Text(v)
    }
}

impl From<&str> for DbValue {
    fn from(v: &str) -> Self {
        DbValue::Text(v.to_string())
    }
}

impl From<Option<i64>> for DbValue {
    fn from(v: Option<i64>) -> Self {
        v.map(DbValue::Integer).unwrap_or(DbValue::Null)
    }
}

impl From<Option<i32>> for DbValue {
    fn from(v: Option<i32>) -> Self {
        v.map(|x| DbValue::Integer(x as i64))
            .unwrap_or(DbValue::Null)
    }
}

impl From<Option<f64>> for DbValue {
    fn from(v: Option<f64>) -> Self {
        v.map(DbValue::Real).unwrap_or(DbValue::Null)
    }
}

impl From<Option<bool>> for DbValue {
    fn from(v: Option<bool>) -> Self {
        v.map(DbValue::Bool).unwrap_or(DbValue::Null)
    }
}

impl From<Option<String>> for DbValue {
    fn from(v: Option<String>) -> Self {
        v.map(DbValue::Text).unwrap_or(DbValue::Null)
    }
}

impl From<Option<&str>> for DbValue {
    fn from(v: Option<&str>) -> Self {
        v.map(|s| DbValue::Text(s.to_string()))
            .unwrap_or(DbValue::Null)
    }
}

/// Type des paramètres d'une requête : une séquence de valeurs positionnelles.
pub type DbParams = Vec<DbValue>;

/// Convertit un type Rust en `DbValue` (sans perdre l'ordre dans `params!`).
pub trait ToDbValue {
    fn to_db_value(self) -> DbValue;
}

impl ToDbValue for i64 {
    fn to_db_value(self) -> DbValue {
        DbValue::Integer(self)
    }
}

impl ToDbValue for i32 {
    fn to_db_value(self) -> DbValue {
        DbValue::Integer(self as i64)
    }
}

impl ToDbValue for u32 {
    fn to_db_value(self) -> DbValue {
        DbValue::Integer(self as i64)
    }
}

impl ToDbValue for f64 {
    fn to_db_value(self) -> DbValue {
        DbValue::Real(self)
    }
}

impl ToDbValue for bool {
    fn to_db_value(self) -> DbValue {
        DbValue::Bool(self)
    }
}

impl ToDbValue for &str {
    fn to_db_value(self) -> DbValue {
        DbValue::Text(self.to_string())
    }
}

impl ToDbValue for String {
    fn to_db_value(self) -> DbValue {
        DbValue::Text(self)
    }
}

impl ToDbValue for &String {
    fn to_db_value(self) -> DbValue {
        DbValue::Text(self.clone())
    }
}

impl ToDbValue for Option<i64> {
    fn to_db_value(self) -> DbValue {
        DbValue::from(self)
    }
}

impl ToDbValue for Option<i32> {
    fn to_db_value(self) -> DbValue {
        DbValue::from(self)
    }
}

impl ToDbValue for Option<f64> {
    fn to_db_value(self) -> DbValue {
        DbValue::from(self)
    }
}

impl ToDbValue for Option<bool> {
    fn to_db_value(self) -> DbValue {
        DbValue::from(self)
    }
}

impl ToDbValue for Option<&str> {
    fn to_db_value(self) -> DbValue {
        DbValue::from(self)
    }
}

impl ToDbValue for Option<String> {
    fn to_db_value(self) -> DbValue {
        DbValue::from(self)
    }
}

/// Toute chose convertible en `DbParams`.
pub trait IntoParams {
    fn into_params(self) -> DbParams;
}

impl IntoParams for DbParams {
    fn into_params(self) -> DbParams {
        self
    }
}

impl IntoParams for () {
    fn into_params(self) -> DbParams {
        Vec::new()
    }
}

macro_rules! impl_into_params_tuple {
    ($($T:ident),+) => {
        impl<$($T: ToDbValue),+> IntoParams for ($($T,)+) {
            #[allow(non_snake_case)]
            fn into_params(self) -> DbParams {
                let ($($T,)+) = self;
                vec![$(ToDbValue::to_db_value($T)),+]
            }
        }
    };
}

impl_into_params_tuple!(T1);
impl_into_params_tuple!(T1, T2);
impl_into_params_tuple!(T1, T2, T3);
impl_into_params_tuple!(T1, T2, T3, T4);
impl_into_params_tuple!(T1, T2, T3, T4, T5);
impl_into_params_tuple!(T1, T2, T3, T4, T5, T6);
impl_into_params_tuple!(T1, T2, T3, T4, T5, T6, T7);
impl_into_params_tuple!(T1, T2, T3, T4, T5, T6, T7, T8);

/// Sucré syntaxique symétrique à `libsql::params!`, produit une `DbParams`.
///
/// Exemple (doctest ignoré : le module `infrastructure` est privé, la macro
/// est consommée en interne par les repositories en PR 2) :
///
/// ```ignore
/// let p = cadence_lib::params![1i64, "deux", true];   // positionnel
/// let _vide = cadence_lib::params![];                  // aucun paramètre
/// let _terminal = cadence_lib::params![1i64, 2i64,];  // virgule terminale tolérée
/// assert_eq!(p.len(), 3);
/// ```
#[macro_export]
macro_rules! params {
    () => {
        $crate::infrastructure::db::params::DbParams::new()
    };
    ($($arg:expr),+ $(,)?) => {{
        let mut __db_params = $crate::infrastructure::db::params::DbParams::new();
        $(
            __db_params.push($crate::infrastructure::db::params::ToDbValue::to_db_value($arg));
        )+
        __db_params
    }};
}

/// Convertit une `DbParams` en message d'erreur lisible (debug).
pub fn params_debug(params: &DbParams) -> String {
    format!("{params:?}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn params_vide() {
        assert_eq!(params![], DbParams::new());
    }

    #[test]
    fn params_scalaires() {
        let p = params![1i64, "deux", true, 3.5];
        assert_eq!(
            p,
            vec![
                DbValue::Integer(1),
                DbValue::Text("deux".into()),
                DbValue::Bool(true),
                DbValue::Real(3.5),
            ]
        );
    }

    #[test]
    fn params_option() {
        let p = params![Some(5i64), None::<i64>, Some("x".to_string())];
        assert_eq!(
            p,
            vec![
                DbValue::Integer(5),
                DbValue::Null,
                DbValue::Text("x".into()),
            ]
        );
    }

    #[test]
    fn params_virgule_terminale() {
        let p = params![1i64, 2i64,];
        assert_eq!(p, vec![DbValue::Integer(1), DbValue::Integer(2)]);
    }

    #[test]
    fn into_params_tuple() {
        let p = (1i64, "a".to_string()).into_params();
        assert_eq!(p, vec![DbValue::Integer(1), DbValue::Text("a".into())]);
    }

    #[test]
    fn from_variantes() {
        assert_eq!(DbValue::from(Some("ok")), DbValue::Text("ok".into()));
        assert_eq!(DbValue::from(None::<i64>), DbValue::Null);
        assert!(DbValue::Null.is_null());
    }
}
