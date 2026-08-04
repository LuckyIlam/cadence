## 1. Socle — dépendance libsql, connexion (mono/multi) et migrations

- [x] 1.1 Remplacer `sqlx` par `libsql = { version = "0.9", default-features = false, features = ["remote", "core", "tls"] }` dans `src-tauri/Cargo.toml` (ajouter `local` si nécessaire pour le mode fichier réel)
- [x] 1.2 Créer `infrastructure/migrations.rs` : liste statique des 8 fichiers `src-tauri/migrations/*.sql` via `include_str!` + table `_cadence_migrations`
- [x] 1.3 Implémenter `cadence_migrations(conn: &libsql::Connection)` exécutant chaque fichier via `execute_batch("BEGIN; … COMMIT;")` (vérifier la création, pas de double application)
- [x] 1.4 Définir `ConnexionConfig { mode, url?, token?, utilisateur }` et remplacer `init_pool` par `init_connection(config, app_dir)` qui choisit selon le mode : `new_local(cadence.db)` (mono) ou `new_remote(url, token)` (multi) + migrations communes
- [x] 1.5 Passer `AppState.pool: SqlitePool` → `AppState.conn: libsql::Connection` et adapter `lib.rs` (création via `tauri::async_runtime::block_on`)
- [x] 1.6 Ajouter `std::env::set_var("RUST_MIN_STACK", "536870912")` en tête de `run()` et vérifier en debug distant (mode multi) que Tauri n'override pas la stack (sinon thread dédié grande pile)
- [x] 1.7 Adapter `AppError` : `From<sqlx::Error>` → `From<libsql::Error>` (variante `Database` conservée)
- [x] 1.8 Adoption du bookkeeping SQLx dans `cadence_migrations` : copier `_sqlx_migrations` → `_cadence_migrations` si elle existe ; vérifier l'ouverture d'un `cadence.db` existant avec `new_local` (pas de re-run)
- [x] 1.9 Migrer les helpers de tests (`setup_db`/`setup_app`) vers libsql `:memory:` + `cadence_migrations` ; `cargo check` + `cargo test` + `cargo clippy`

## 2. Basculer les repositories sur libsql

- [x] 2.1 Retirer les derives `sqlx::FromRow` des 11 structs domain (garder `serde` derives), désérialisation via `libsql::de::from_row`
- [x] 2.2 Réécrire `personne_repo.rs` (query, fetch_optional, query_scalar, paramètres dynamiques `Vec<libsql::Value>`)
- [x] 2.3 Réécrire `adhesion_repo.rs`
- [x] 2.4 Réécrire `activite_repo.rs` (y compris la transaction ligne ~96)
- [x] 2.5 Réécrire `planning_repo.rs` (requêtes + transactions `&mut libsql::Transaction`)
- [x] 2.6 Réécrire `parametre_repo.rs` (trait `mettre_a_jour_plage_horaire_tx` → `&mut libsql::Transaction`)
- [x] 2.7 Renommer `SqliteXRepository` → `LibsqlXRepository` (structs, constructeurs, `init_app_state`, mocks des tests)
- [x] 2.8 Adapter `ParametreService` : stocker une `libsql::Connection` clonée au lieu du pool ; `ParametreService::new(..., state.conn.clone())` dans `parametre_commands.rs`
- [x] 2.9 Migrer les tests des repositories, services et commandes vers libsql `:memory:`
- [x] 2.10 `cargo test` + `cargo clippy -- -D warnings` + `cargo fmt --check`

## 3. Audit des modifications et conflits

- [ ] 3.1 Migration : `ALTER TABLE … ADD COLUMN modifie_par TEXT, modifie_le TEXT, version INTEGER NOT NULL DEFAULT 0` sur les 8 tables
- [ ] 3.2 Ajouter le paramètre `utilisateur: String` aux commandes d'écriture (création et modification)
- [ ] 3.3 Écrire `modifie_par` / `modifie_le` (ISO-8601 UTC via chrono) dans les INSERT/UPDATE des repositories
- [ ] 3.4 Refuser une écriture sans nom d'utilisateur (`AppError::Validation`)
- [ ] 3.5 Ne pas exposer les colonnes d'audit dans les lectures renvoyées au front
- [ ] 3.6 Tests unitaires d'audit : création, modification, refus sans utilisateur, non-exposition
- [ ] 3.7 Optimistic locking : `UPDATE … SET …, version = version + 1 WHERE id = ? AND version = ?` via `conn.execute` ; `rows_affected == 0` → `AppError::Conflict` avec message « fiche modifiée entre-temps, rechargez-la »
- [ ] 3.8 Exposer la version dans les commandes de lecture (champ caché, jamais affiché) et la faire renvoyer par le front sur les mises à jour
- [ ] 3.9 Tests unitaires de conflit : mise à jour OK, mise à jour concurrente refusée, version non affichée

## 4. Configuration de la connexion et choix du mode

- [x] 4.1 Module config : charger / sauvegarder `cadence_config.json` dans `app_data_dir` (mode, url/token selon le mode, utilisateur)
- [x] 4.2 Commandes `obtenir_config`, `sauvegarder_config`, `tester_connexion` (connexion éphémère + `SELECT 1`, mode multi uniquement ; le token n'est pas renvoyé en clair au front)
- [ ] 4.3 Écran de premier lancement dans `App.tsx` (garde si non configuré) : choix du mode + champs selon le mode
- [ ] 4.4 Carte « Connexion à la base » dans `ParametresPage.tsx` : sélecteur mono/multi, champs adaptés au mode, bouton tester (multi), modal « redémarrage requis » au changement de mode/URL/token
- [ ] 4.5 Transmettre le nom d'utilisateur aux commandes d'écriture (les deux modes) et l'afficher dans `Nav.tsx`
- [ ] 4.6 Tests config : sauvegarde mono et multi, changement de mode → redémarrage demandé, nom d'utilisateur seul → immédiat
- [ ] 4.7 `npm run typecheck` + `npm run lint` + `npm run build`

## 5. Vérifications et livraison

- [ ] 5.1 `cargo test`, `cargo check`, `cargo clippy -- -D warnings`, `cargo fmt --check`, `cargo audit`, `cargo deny check`
- [ ] 5.2 `npm run typecheck`, `npm run lint`, `npm run build`
- [ ] 5.3 Validation end-to-end : mode multi sur la base de test `cadence-dev` (debug RUST_MIN_STACK + release) et mode mono sur base locale (migrations + CRUD)
- [ ] 5.4 Documentation fonctionnelle : base partagée, écran de configuration, RGPD (Turso sous-traitant UE) — l'import éventuel des données locales existantes dépend de la décision utilisateur (voir design.md, Open Questions)
- [ ] 5.5 `graphify update .`
