## Context

- Cadence est aujourd'hui une app Tauri 2 monoposte : la base SQLite locale (`cadence.db` dans app_data_dir) est créée via `init_pool` (`sqlx::migrate!`), et `AppState` expose un `SqlitePool` partagé + les repositories SQLx (voir `infrastructure/db.rs`, `lib.rs:35-41`).
- Les 5 repositories et `ParametreService` reposent sur SQLx (`query_as`, `query_scalar`, `sqlx::Transaction`, `sqlx::FromRow` sur 11 structs domain). Le dialecte SQL est du SQLite pur (placeholders `?`, `RETURNING *`, triggers).
- Objectif validé avec l'utilisateur : **A1 — Turso (libSQL hébergé, région UE)** comme mode **multi-utilisateurs**, en conservant SQL, domaine, services et app bureau. **Le mode mono-utilisateur existant (base locale) est conservé** : l'utilisateur choisit l'un des deux modes, jamais les deux en même temps. Audit des modifications par **nom d'utilisateur** (stocké, non affiché, actif dans les deux modes) ; écran de config dans ParametresPage + écran de premier lancement. Rafraîchissement à chaque commande (pas de temps réel).
- Un spike a validé sur base réelle `cadence-dev` : compilation MSVC de `libsql` 0.9.30 (`remote`/`core`/`tls`), `de::from_row` + `NaiveDate` depuis TEXT, paramètres dynamiques `Vec<libsql::Value>`, les 8 migrations via `execute_batch`, connexion distante + CRUD en release. Voir proposition pour la motivation (proposal.md — Why).

## Goals / Non-Goals

**Goals :**
- Permettre à plusieurs bénévoles (dont à distance) de travailler sur une base partagée hébergée dans l'UE, avec une modification minimale du domaine et des services métier.
- Conserver le mode mono-utilisateur (base locale) : les deux modes coexistent et sont **exclusifs**.
- Conserver le SQL actuel (dialecte SQLite) et l'architecture par couches (commands → services → repositories → SQL).
- Traçabilité des écritures (nom d'utilisateur + horodatage) sans interface d'affichage, dans les deux modes.

**Non-Goals :**
- Pas de synchronisation temps réel ni de conflit de merge (rafraîchissement suffisant).
- Pas de synchronisation entre les bases mono et multi : chaque mode a sa propre base de données.
- Pas de gestion des droits/authentification applicative (Turso fournit l'accès ; l'application n'implémente pas de rôles).
- Pas d'import automatisé des données entre les bases dans ce change (voir Open Questions).
- Pas de remplacement de l'UI métier.

## Decisions

### 1. Remplacer SQLx/SQLite par `libsql` 0.9.30, qui pilote les deux modes

Le SDK `libsql` est le client officiel du protocole Turso, et il sait aussi ouvrir une base SQLite locale (`Builder::new_local`) : **un seul driver pour les deux modes**, donc une seule couche repositories et un seul runner de migrations. Pinned à 0.9.30 (pre-release 0.10 écartée : API en cours de churn, et 0.9 validée par le spike).
- **Alternative écartée** : garder SQLx pour le local + libsql pour le distant — impose une abstraction de connexion dans tous les repositories (double complexité).
- **Alternative écartée** : garder SQLx + relayer vers `libsql-server` — surcouche à maintenir, hors sujet.
- **Alternative écartée** : client Hrana HTTP maison — réinventer ce que `libsql` fait déjà.

`AppState.pool: SqlitePool` devient `AppState.conn: libsql::Connection` (clonage léger). `init_pool(database_url)` devient `init_connection(config: &ConnexionConfig, app_dir: &Path) -> Result<libsql::Connection, AppError>` qui choisit selon le mode :
- `mono` → `Builder::new_local(app_dir.join("cadence.db")).build()` (mode fichier validé sur `:memory:` par le spike ; compatibilité avec un `cadence.db` existant via l'adoption du bookkeeping SQLx, voir décision 2) ;
- `multi` → `Builder::new_remote(url, token).build()` ;
puis application des migrations (communes). Features `remote`/`core`/`tls` (le spike compile `new_local(":memory:")` avec ces features) ; la feature `local` est ajoutée si nécessaire pour le mode fichier réel (à confirmer en Phase 1). `max_connections(1)` n'a plus de sens (la `Connection` gère ses connexions internes).

### 2. Runner de migrations maison (`infrastructure/migrations.rs`)

`sqlx::migrate!` disparaît. Nouveau runner `cadence_migrations(conn: &libsql::Connection)` :
- liste statique des 8 fichiers SQL via `include_str!` (les fichiers `src-tauri/migrations/*.sql` restent la source de vérité) ;
- table `_cadence_migrations` (nom, appliquée_le) pour le suivi ;
- chaque fichier exécuté via `conn.execute_batch("BEGIN; ...; COMMIT;")`, vérifié dans la transaction (validé par le spike : `execute_batch` fonctionne en local et à distance, triggers compris) ;
- **adoption d'une base locale existante** : un `cadence.db` créé par SQLx contient déjà les tables et la table `_sqlx_migrations`. Pour éviter de re-exécuter les 8 migrations (erreur « table already exists »), au premier passage `cadence_migrations` copie les entrées de `_sqlx_migrations` dans `_cadence_migrations` si elle existe. La base Turso (vide) exécute les 8 migrations normalement.

### 3. Rewrites mécaniques des repositories

- `sqlx::query_as::<_, T>(sql).bind(x).fetch_one(&pool)` → `conn.query(sql, params)` + `libsql::de::from_row::<T>(&row)`.
- Paramètres dynamiques : `Vec<libsql::Value>` (les `?` restent inchangés).
- `fetch_optional` → itération sur `Rows` (`row.next()`).
- `query_scalar` → `de::from_row::<(i64,)>` ou `row.get_value(0)`.
- Transactions : `sqlx::Transaction<'_, Sqlite>` → `&mut libsql::Transaction` ; `pool.begin()` → `conn.transaction()`. `ParametreService` stocke `Connection` (clonée) au lieu du pool (`ParametreService::new(..., conn.clone())`, lignes 22-28, appelé dans `parametre_commands.rs`).
- Structs domain : les derives `sqlx::FromRow` sont retirés ; la désérialisation passe par `libsql::de::from_row` **sur les derives serde existants** (`Deserialize`), aucune derive supplémentaire requise (validé par le spike R2, y compris `NaiveDate`/`Option`).
- Renommage `SqliteXRepository` → `LibsqlXRepository` (mécanique, avec les rewrites) pour refléter le driver ; les traits (`PersonneRepository`, …) et les noms de fichiers sont conservés.
- `AppError` : `impl From<sqlx::Error>` → `impl From<libsql::Error>` (variante `Database` conservée). Les messages SQLite de libsql restent exploitables.

### 4. Config de connexion stockée localement (`cadence_config.json`)

Fichier JSON dans `app_data_dir` contenant `{ mode, url?, token?, utilisateur }` :
- `mode`: `"mono"` ou `"multi"` — un seul actif à la fois ;
- `url` / `token`: requis uniquement en mode `multi` (le token est un secret applicatif local) ;
- `utilisateur`: requis dans les deux modes (audit).

Commandes Tauri : `obtenir_config` (booléen `configure` + valeurs, sans renvoyer le token en clair au front), `sauvegarder_config`, `tester_connexion` (connexion éphémère + `SELECT 1`, **mode multi uniquement**). Écran de premier lancement dans `App.tsx` (choix du mode) + carte « Connexion à la base » dans `ParametresPage.tsx` (sélecteur mono/multi, champs adaptés au mode, bouton tester en multi).
- **Alternative écartée** : `keyring` (Credential Manager Windows) — dépendance système supplémentaire, réservée en amélioration future. Le fichier JSON dans app_data_dir est le standard desktop ; les permissions NTFS protègent le dossier utilisateur.

### 5. Stack pour le dev : `RUST_MIN_STACK` 512 MiB

Le spike a établi que le chemin **remote + TLS en build debug** a une récursion fixe dans hyper/hyper-rustls de ~225–256 MiB de pile (release OK à 2 MiB ; `:memory:` OK en debug). Parade : `std::env::set_var("RUST_MIN_STACK", "536870912")` en tête de `run()` (`lib.rs:24`), avant la construction du runtime tokio de Tauri. 512 MiB = réservation virtuelle ; la mémoire réellement utilisée reste faible. **Le mode local (`new_local`) n'est pas concerné** par ce problème : seul le chemin TLS/hyper du mode distant est touché. À vérifier en Phase 1 : que Tauri ne surcharge pas la stack de ses workers tokio (sinon repli : thread dédié à grande pile pour la couche BDD).

### 6. Audit des écritures (Phase 3)

Migration ajoutant `modifie_par TEXT`, `modifie_le TEXT` et `version INTEGER NOT NULL DEFAULT 0` sur les 8 tables — **dans les deux modes** (le schéma est identique). Chaque commande d'écriture reçoit `utilisateur: String` (depuis le front, alimenté par la config) ; les repositories écrivent ces colonnes avec `chrono` UTC ISO-8601. Colonnes jamais lues par l'UI (audit stocké uniquement). Les `RETURNING *` existants restent valides.

### 7. Changement de mode : redémarrage requis

La connexion (mode + URL/token) est créée au démarrage dans `setup()`. Toute modification de `mode`, `url` ou `token` ne peut donc être appliquée qu'après **redémarrage de l'application** : après `sauvegarder_config`, le front affiche un modal « Redémarrer maintenant ? / Plus tard » (via `tauri::process::restart`) ; l'app continue sur l'ancien mode jusqu'au prochain lancement.
- La modification du **nom d'utilisateur seul** ne requiert pas de redémarrage (le nom est transmis par le front aux commandes d'écriture ; il ne fait pas partie de la connexion).
- Règle uniforme : le premier paramétrage (rien → configuré) suit le même chemin (redémarrage), pas de reconnexion à chaud.
- Rappel visuel au basculement : « les données de chaque mode sont indépendantes, pas de synchronisation ».
- **Alternative écartée** : reconstruire `AppState` à chaud (rebuild de la connexion + des repositories) — plus complexe et plus risqué.

### 8. Détection des modifications concurrentes (optimistic locking, Phase 3)

Sans protection, deux bénévoles modifiant la même fiche produisent une perte silencieuse (le dernier `UPDATE` écrase tout, car il réécrit toutes les colonnes). Décision utilisateur : **ajouter un contrôle de version**.
- Chaque `UPDATE` ajoute `version = version + 1` et la condition `WHERE id = ? AND version = ?` (version chargée à l'ouverture du formulaire).
- Via `conn.execute(...)`, si `rows_affected == 0` → `AppError::Conflict` (« cette fiche a été modifiée entre-temps, rechargez-la »).
- La version est exposée au front comme champ caché (renvoyé tel quel par les commandes d'écriture), **jamais affichée**. `modifie_par`/`modifie_le` restent invisibles.
- Aucun verrou bloquant : compatible « rafraîchissement suffisant » ; les créations et suppressions ne sont pas concernées (la suppression n'est pas auditée).
- **Alternative écartée** : accepter le last-write-wins (refusé par l'utilisateur) ; comparer `modifie_le` plutôt qu'un compteur (collision possible sur la précision temporelle).

### 9. Rafraîchissement

Aucune cache front ni backend : chaque commande interroge la base active (distant en mode multi, local en mode mono → fraîcheur triviale) → toute machine voit les écritures des autres au prochain chargement. Pas de polling temps réel (hors scope).

## Risks / Trade-offs

- [Overflow de pile en debug sur base distante (mode multi)] → `RUST_MIN_STACK` 512 MiB (validé en spike, debug multi-thread + realwork OK). En release et en mode local, aucune intervention.
- [Bascule mono ↔ multi : bases indépendantes] → rappel visuel « données non synchronisées » au basculement ; chaque mode conserve sa base, aucune perte de données.
- [`libsql::de::from_row` : écart de type entre le schéma et les structs] → même mapping qu'actuellement (TEXT pour dates/heures) ; les tests unitaires `:memory:` (qui tournent en debug à 2 MiB) couvrent les types.
- [Le token Turso stocké en clair dans le JSON] → fichier dans app_data_dir utilisateur (permissions NTFS) ; jamais transmis au front après la sauvegarde ; rotation possible via `tester_connexion`.
- [Données de mineurs hébergées chez Turso] → sous-traitant RGPD UE (région eu-west-1), données restreintes au besoin ; documentation fonctionnelle + mention dans `docs/fonctionnel/` (Phase 5).
- [Conflit de version à l'enregistrement] → message explicite « fiche modifiée entre-temps, rechargez-la » ; rare dans une petite association, jamais bloquant de façon permanente.
- [Tauri surcharge la stack de ses workers] → repli thread dédié grande pile pour les appels BDD ; vérifié au début de la Phase 1.
- [Grosseur du diff des repositories] → rewrites mécaniques et tests préservés ; les services/domain ne changent pas de logique.

## Migration Plan

1. **Phase 1 — socle** : `infrastructure/migrations.rs` (liste + `cadence_migrations` + adoption `_sqlx_migrations`), `RUST_MIN_STACK`, `init_connection` selon le mode (mono local / multi distant) + `AppState.conn`, suppression de SQLx. `cargo test` (local `:memory:`) + `cargo tauri dev` (distant, debug).
2. **Phase 2 — repositories** : bascule query/transactions/derive, `AppError`, services, commandes. Suite de tests complète en local (les deux modes partagent le même code).
3. **Phase 3 — audit** : migration `modifie_par`/`modifie_le`/`version` + paramètre `utilisateur` sur les commandes d'écriture (les deux modes).
4. **Phase 4 — config** : `cadence_config.json` (mode + champs selon le mode), commandes config/test, écran premier lancement (choix du mode), carte Paramètres (sélecteur de mode, redémarrage requis), `Nav.tsx` (nom utilisateur).
5. **Phase 5 — déploiement** : documentation fonctionnelle, RGPD, vérifs complètes (cargo check/clippy/fmt/audit/deny, npm typecheck/lint/build).
6. **Rollback** : chaque phase reste compilable et testée ; la bascule est un commit par phase. Avant la Phase 3 (audit), la base locale reste lisible par les versions antérieures. La base distante étant nouvelle, un retour arrière après migration du schéma n'affecte pas la base locale.

## Open Questions

- **Import/export des données entre les bases mono et multi** : nécessaire ou non (un bénévole qui passe du local au partagé, et inversement) ? Réponse sans impact sur les Phases 1-4 (l'import, si besoin, serait un utilitaire one-shot en Phase 5). **Décision à demander avant la Phase 5.**
- Nom exact du fichier/écran de premier lancement et libellés de la carte Paramètres (détail d'UI, décidé en Phase 4).
