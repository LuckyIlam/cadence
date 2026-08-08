//! Point d'entrée historique — la logique vit désormais dans
//! `drivers::libsql::hrana`. Ce module n'est conservé que pour ne pas casser
//! les imports existants (repositories, tests e2e) pendant le refactor PR 2.

pub use crate::drivers::libsql::hrana::{execute_avec_retry, query_avec_retry, vider_cursor};
