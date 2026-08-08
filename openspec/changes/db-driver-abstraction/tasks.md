# Change `db-driver-abstraction` — tasks

## 1. PR 1 — Pose des abstractions (zéro changement de comportement)

- [x] 1.1 Créer `infrastructure/db/{db, params, row, transaction}.rs` avec les
      traits `Db` (y compris `begin_immediate`, `execute_batch`), `DbTransaction`,
      `IntoParams` + macro `params!`, `RowView` / `DeserializeRow`
- [x] 1.2 Créer `infrastructure/retry.rs` avec le trait `RetryPolicy`
- [x] 1.3 Déplacer `hrana_guard.rs` → `drivers/libsql/retry.rs` (implémente
      `RetryPolicy` ; `est_stream_perdu` → `HranaRetryPolicy::matches`) et
      `hrana_guard.rs` → `drivers/libsql/` (query/execute/vider_cursor)
- [x] 1.4 Étendre `ConnexionConfig` (D5) : enum `Driver`, `Mode`, champ `driver`
      (branches Postgres/Mysql en `unimplemented!()` avec message clair)
- [x] 1.5 Écrire les tests unitaires de chaque trait (grain requête-only du
      retry documenté — cf. D4)
- [x] 1.6 Vérifier qu'aucune signature publique de command/service/repo ne
      change : `cargo test` vert sans modification des tests existants

## 2. PR 2 — Refactor repositories + services derrière `dyn Db`

- [ ] 2.1 Déplacer les `*_repo.rs` vers `drivers/libsql/repositories/`
      (impls `Libsql*Repository`) ; `repositories/mod.rs` ne réexporte que les traits
- [ ] 2.2 Faire prendre aux services `PersonneService`, `ActiviteService`,
      `ParametreService` un `&dyn Db` au lieu de `Connection`
- [ ] 2.3 Remplacer `libsql::params![…]` par `params![…]` sur tous les sites
      (sauf tests e2e) ; adapter les helpers dynamiques `Vec<libsql::Value>` →
      `Vec<DbValue>` (R3, `personne_repo.rs` recherche paginée)
- [ ] 2.4 `AppState` contient `Arc<dyn Db>` ; suppression du champ `pub conn`
- [ ] 2.5 Mettre à jour `commands/*.rs` : `state.conn.clone()` →
      `state.db.clone()` (`activite_commands.rs` ×10, `parametre_commands.rs`
      ×3) ; `transaction_with_behavior(Immediate)` → `db.begin_immediate()`
      (`planning_commands.rs:27,86,123`)
- [ ] 2.6 Adapter `init_connection` (`e2e_mono/multi/stream`) pour renvoyer
      `Arc<dyn Db>` ; refactorer `e2e_stream.rs` pour passer par
      `LibsqlDb::connect(...)` (D8)
- [ ] 2.7 Réécrire les tests repository (mécanique : `libsql::params!` →
      `params!`) ; vérifier tous tests verts + `cargo clippy -D warnings`

## 3. PR 3 — Adoption `refinery` (optionnelle, dépend du spike R1)

- [ ] 3.1 **Spike** : tester `refinery::MigrateAsync` sur une
      `libsql::Connection` (feature libsql absente de refinery 0.8.14)
- [ ] 3.2 Si spike positif : migrer le runner `cadence_migrations` vers
      `refinery` (tracker dans la même transaction que la migration — R11)
- [ ] 3.3 Si spike négatif : rebrancher le runner actuel derrière `&dyn Db`
      (via `execute_batch`, D1) sans dépendance refinery ; documenter la limitation
- [ ] 3.4 Créer `migrations/{sqlite,postgres,mysql}/` ; `sqlite/*` contient les
      9 SQL actuels, `postgres/*` et `mysql/*` un `README.md` placeholder
- [ ] 3.5 Vérifier : `e2e_mono` + `e2e_multi` verts, pas de régression de perf

## 4. Vérifications globales (avant livraison)

- [ ] 4.1 `cargo check`, `cargo clippy -- -D warnings`, `cargo fmt --check`,
      `cargo audit`, `cargo deny check` dans `src-tauri/`
- [ ] 4.2 `npm run typecheck`, `npm run lint`, `npm run build`
- [ ] 4.3 `cargo tree` : aucune dépendance `tokio-postgres` / `mysql_async`
      indirecte (R5)
