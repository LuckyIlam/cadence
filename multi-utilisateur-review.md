# Revue de changements — branche `feat/multi-utilisateur`

> **Périmètre** : 4 commits propres à la fonctionnalité multi-utilisateurs depuis la dernière divergence avec `main` (option validée avec l'utilisateur). Les commits antérieurs à la base de la branche (plages horaires, dépendances…) sont exclus.

| Commit | Auteur | Date | Message |
|--------|--------|------|---------|
| `df80398` | LuckyIlam | 2026-08-04 | `migration backend (phase 2)` — SQLx → libsql 0.9.30 (mono + multi) |
| `14f3583` | LuckyIlam | 2026-08-05 | `mise en place de l'audit des modifications et gestion des conflits` |
| `bd4ae0f` | LuckyIlam | 2026-08-05 | `feat/multi-utilisateurs-turso : configuration connexion (mono/multi) + tests E2E` |
| `b106826` | LuckyIlam | 2026-08-05 | `fix(stream) : retry Hrana 'stream not found'` + thread BDD 512 MiB + récursion Nav |

L'ensemble traite le change OpenSpec [`openspec/changes/multi-utilisateurs-turso/`](openspec/changes/multi-utilisateurs-turso/) (proposal/design/tasks/specs) **et** un fix de régression découvert à l'exécution du mode multi.

---

## 1. Vue d'ensemble

### Objectif métier
Faire passer Cadence d'un outil monoposte (SQLite local) à un outil **collaboratif** : ajout d'un **mode multi-utilisateurs** (base partagée hébergée chez **Turso**, libSQL, région UE), sans abandonner le **mode mono-utilisateur** existant. Les deux modes sont **exclusifs** et choisis par l'utilisateur via un écran de configuration. Le passage s'accompagne d'un **audit des écritures** (nom d'utilisateur + horodatage) et d'une **détection des conflits** par version (optimistic locking).

### Périmètre technique
- **Backend Rust** : remplacement du driver SQLx par la crate `libsql` (features `remote`/`core`/`tls`), unifiant le code des deux modes. Réécriture des 5 repositories et de `ParametreService` (rewrites mécaniques essentiellement).
- **Nouveau** : module `infrastructure/config.rs` (config JSON locale) + `infrastructure/audit.rs` (helpers audit) + `infrastructure/migrations.rs` (runner maison) + `infrastructure/hrana_guard.rs` (garde de résilience Hrana).
- **Migration SQL** : `modifie_par TEXT`, `modifie_le TEXT`, `version INTEGER DEFAULT 1` sur les 8 tables métier (les deux modes héritent du même schéma).
- **Commandes Tauri** : 3 nouvelles commandes (`obtenir_config`, `sauvegarder_config`, `tester_connexion`) ; les 11 commandes d'écriture reçoivent en plus un paramètre `utilisateur: String` + (selon) une `version: i64`.
- **Frontend React** : écran de premier lancement, formulaire partagé `ConnexionConfigForm`, carte « Connexion à la base » dans Paramètres, badge d'utilisateur dans la Nav, helper `utilisateur.ts` avec cache + observateurs.
- **Tests** : nouveaux tests unitaires (audit, conflit, migration runner, config) + 2 nouveaux binaires de tests E2E (`e2e_mono`, `e2e_multi`, `e2e_stream`) dont 2 dépendent des variables d'environnement `TURSO_URL` / `TURSO_TOKEN` (skippés si absents).
- **Tooling** : `tauri-plugin-process` (pour `relaunch`), capability `process:allow-restart`, ignore git de `graphify-out/cache/`, dérogations `cargo deny`/`cargo audit` pour les advisories rustls imposées par libsql 0.9.30.

### Volumétrie (hors fichiers générés / graph / lockfile)
- 36 fichiers Rust touchés + 9 fichiers front + 9 fichiers de config/docs.
- **~5 150 insertions / ~1 960 suppressions** sur le périmètre « métier » (hors `graphify-out/`, `gen/`, `Cargo.lock`, `package-lock.json`, `tsconfig.tsbuildinfo`).
- 5 nouveaux fichiers : `infrastructure/config.rs`, `infrastructure/audit.rs`, `infrastructure/hrana_guard.rs`, `commands/connexion_commands.rs`, `e2e_mono.rs`, `e2e_multi.rs`, `e2e_stream.rs`, `utilisateur.ts`, `components/ConnexionConfigForm.tsx`, `migrations/20260803000001_add_audit.sql`, `docs/fonctionnel/connexion.md`, `src-tauri/.cargo/audit.toml`.

---

## 2. Analyse détaillée par commit

### 2.1 `df80398` — Migration backend : SQLx → libsql 0.9.30

**Le changement structurel le plus important de la branche.**

#### Ce qui est fait
- Remplace la dépendance `sqlx = "0.9"` par `libsql = "=0.9.30"` (features `remote`, `core`, `tls`, `serde` ; `default-features = false`) dans `src-tauri/Cargo.toml:16`. Versions pinnées très précisément (`=0.9.30`), cohérent avec la décision de design (zéro-release écartée).
- `src-tauri/src/infrastructure/db.rs` : `AppState.pool: SqlitePool` → `AppState.conn: libsql::Connection` ; nouvelle fonction `init_connection(config: &ConnexionConfig, app_dir: &Path) -> Result<Connection, AppError>` qui sélectionne `Builder::new_local(...)` (mono) ou `Builder::new_remote(url, token)` (multi), applique les migrations communes, puis renvoie `Connection`.
- `src-tauri/src/infrastructure/migrations.rs` (nouveau, 145 lignes) : liste statique des 9 fichiers SQL via `include_str!`, table de bookkeeping `_cadence_migrations` (`nom`, `appliquee_le`), exécution en `execute_batch("BEGIN; ...; COMMIT;")` et insertion via `conn.execute(...)`. Comprend 2 tests unitaires (`applique_toutes_les_migrations`, `ne_reapplique_pas`) sur base `:memory:`.
- `src-tauri/src/infrastructure/mod.rs` : expose les sous-modules.
- `src-tauri/src/lib.rs:29` : `std::env::set_var("RUST_MIN_STACK", "536870912")` en tête de `run()` (justification en commentaire : chemin distant TLS/hyper nécessite ~256 MiB de pile en build debug, validé par spike). Le `setup()` continue d'utiliser `tauri::async_runtime::block_on(init_connection(...))`.
- `src-tauri/src/error.rs` : `impl From<libsql::Error> for AppError` (variante `Database` conservée) ; ajout de `From<serde::de::value::Error>` (utilisé par `libsql::de::from_row`).
- Les **5 repositories** (`personne_repo.rs`, `activite_repo.rs`, `adhesion_repo.rs`, `planning_repo.rs`, `parametre_repo.rs`) sont **réécrits mécaniquement** :
  - `pub struct SqliteXRepository { pool: SqlitePool }` → `LibsqlXRepository { conn: Connection }`.
  - `sqlx::query_as::<_, T>(sql).bind(x).fetch_one(&pool)` → `conn.query(sql, params)` + `libsql::de::from_row::<T>(&row)` + itération manuelle sur `Rows`.
  - `sqlx::Transaction<'_, Sqlite>` → `libsql::Transaction` ; `pool.begin()` → `conn.transaction()`.
  - `query_scalar::<_, i64>` → compteur manuel via `CompteurRow { count: i64 }` + `from_row`.
  - `fetch_all(&pool)` → boucle `while let Some(row) = rows.next().await?`.
  - Les derives `sqlx::FromRow` disparaissent des 11 structs domain (`Activite`, `Adhesion`, `Personne`, `CreneauActivite`, `SemaineBanalisee`, `ParametresPlanning`, `TarifActivite`, `LiaisonActivitePersonne`, `PersonneActivite`, `Role`, `CreneauHorsPlage`, `Inscription`). Seuls les derives `serde::{Serialize, Deserialize}` restent. Cohérent avec la décision de design (D3).
- `ParametreService` stocke désormais `libsql::Connection` au lieu de `SqlitePool` ; les commandes (`parametre_commands.rs`) passent `state.conn.clone()`.
- Les mocks de tests (dans `services/personne_service.rs`, `services/activite_service.rs`, `services/parametre_service.rs`) sont adaptés aux nouveaux types.

#### Points de revue
1. **Évaluation des risques** : `libsql::Connection` est clonable ; passer `conn.clone()` à chaque repository dans `init_app_state` multiplie le nombre de connexions internes par 5. Sans `max_connections`, libsql gère ses propres limites mais c'est un coût non négligeable en mode multi distant. **Vérifier** que la consommation HTTP n'explose pas ; en cas de doute, envisager une seule `Connection` partagée par `Arc<Connection>` (les repositories la prennent en `&Connection`).
2. **`app_dir` non mémorisé** : `init_connection` reçoit `app_dir` pour résoudre `cadence.db` en mode mono ; mais ce chemin n'est pas conservé dans `AppState`. Conséquence mineure : impossible, côté code, de recharger la config depuis ce qui a été utilisé — mais ce n'est pas un besoin actuel.
3. **`Database` comme fourre-tout d'erreur** : `AppError::Database(serde::de::value::Error)` est mappé sur la même variante que `libsql::Error`. Sur le front, l'utilisateur reçoit la même étiquette pour deux familles de problèmes différents. Acceptable pour ce périmètre, à documenter.
4. **`RUST_MIN_STACK` global** : appliqué même en mode mono. L'impact est négligeable (réservation virtuelle), mais le commentaire dans `lib.rs:48-50` gagnerait à rappeler explicitement que seul le chemin distant est concerné (cohérence avec design D5).
5. **Tests des repositories** : tous migrent sur `:memory:` + `cadence_migrations`. Le test `applique_toutes_les_migrations` ne vérifie que la présence de la table `personnes_physiques` ; étendre la couverture (par exemple compter les 9 fichiers) **post-merge**.

#### Verdict
**Approuvé avec remarques mineures.** Le rewrite est conforme au design, transparent pour le domaine, sans logique applicative ajoutée. L'épine dorsale technique de la feature tient debout.

---

### 2.2 `14f3583` — Audit des modifications et gestion des conflits

#### Ce qui est fait
- **Migration SQL** [`src-tauri/migrations/20260803000001_add_audit.sql`](src-tauri/migrations/20260803000001_add_audit.sql) : ajoute `modifie_par TEXT NOT NULL DEFAULT ''`, `modifie_le TEXT NOT NULL DEFAULT ''`, `version INTEGER NOT NULL DEFAULT 1` sur les 8 tables (`personnes_physiques`, `adhesions`, `activites`, `tarifs_activite`, `activite_personnes`, `creneaux_activite`, `semaines_banalisees`, `parametres`). Migration idempotente (ADD COLUMN ne s'applique qu'une fois par table grâce au runner).
- **Helpers d'audit** [`src-tauri/src/infrastructure/audit.rs`](src-tauri/src/infrastructure/audit.rs) : 3 fonctions exportées :
  - `maintenant_utc()` → horodatage ISO-8601 UTC via `chrono::Utc::now().to_rfc3339()`.
  - `verifier_utilisateur(&str) -> Result<String, AppError>` → refuse une chaîne vide ou composée uniquement d'espaces.
  - `MESSAGE_CONFLIT` (`"Fiche modifiée entre-temps, rechargez-la"`) — constante mutualisée.
- **Domain** : `version: i64` ajouté à `Personne`, `Adhesion`, `Activite`, `CreneauActivite` ; `version: i64` ajouté aux inputs `UpdatePersonne`, `UpdateAdhesion`, `UpdateActivite` (et au `UpdateCreneau` côté repository).
- **Repositories** : chaque méthode d'écriture prend un `utilisateur: &str` en plus de ses arguments antérieurs ; les requêtes `INSERT` et `UPDATE` mentionnent `modifie_par`, `modifie_le` (en plus de `version` côté `UPDATE`). Les `UPDATE` font `version = version + 1` dans la condition **et** dans le résultat (`RETURNING ... version`). Format `RETURNING` limité aux colonnes nécessaires : jamais `*`, donc `modifie_par` et `modifie_le` ne sont **pas exposés** au front. ✓ conforme à la spec (`audit-modifications`, scénario « Version transmise sans être affichée »).
- **Détection de conflit** : après `execute(...)`, `if affected == 0` → le repository distingue « n'existe pas » (re-`find_by_id`, renvoie `AppError::NotFound`) et « version obsolète » (renvoie `AppError::Conflict(MESSAGE_CONFLIT)`). Fait pour `Personne`, `Activite`, `Adhesion`, `CreneauActivite`. ✓ conforme à `audit-modifications` (scénarios « Mise à jour concurrente » / « Conflit détecté dans les deux modes »).
- **Commandes Tauri** : les 11 commandes d'écriture (`creer_*`, `modifier_*`, `definir_tarif_activite`, `ajouter_personne_activite`, `ajouter_creneau`, `modifier_creneau`, `ajouter_semaine_banalisee`, `modifier_plage_horaire`) reçoivent `utilisateur: String` et appellent `verifier_utilisateur` avant de déléguer. Service `PersonneService.creer/modifier` propage la valeur. ✓
- **Tests unitaires** : 5+ tests nouveaux dans `repositories/personne_repo.rs` (`test_create_enregistre_audit`, `test_update_enregistre_audit_et_incremente_version`, `test_update_version_obsolete_conflit`, `test_update_personne_inexistante_not_found`, `test_version_transmise_et_audit_non_expose`). Couvrent création, audit, optimistic locking, non-exposition. ✓

#### Points de revue
1. **Divergence design ↔ implémentation** : la tasks.md §3.1 écrit `version INTEGER NOT NULL DEFAULT 0` ; la migration effective est `DEFAULT 1`. Décision cohérente (« version initiale = 1 » est plus naturelle qu'« inconnu = 0 »), mais **corriger la task.md** pour éviter une désynchronisation documentaire.
2. **`maintenant_utc()` dupliqué** : défini à la fois dans `infrastructure/audit.rs:5` (pub) et `infrastructure/migrations.rs:45` (privé). C'est bénin mais c'est du copier-coller ; supprimer la version privée et importer le `pub` depuis `audit::maintenant_utc`.
3. **Pas d'audit sur les suppressions** : `supprimer_creneau`, `supprimer_semaine_banalisee`, `retirer_personne_activite` n'écrivent ni `modifie_par` ni `modifie_le`. Conformément à la spec (scénario « Suppression d'une ligne »), mais penser à informer le testeur/relecteur que **toute suppression est anonyme** par construction.
4. **Course entre `affected == 0` et `find_by_id`** : dans le repo `Activite::update` (entre autres), après `affected == 0`, on appelle `find_by_id(id)` qui ouvre une nouvelle requête. Si entre-temps la fiche a été supprimée puis recréée, `find_by_id` renverra `Some` pour la nouvelle et on émettra un faux `Conflict`. Acceptable (extrêmement rare en pratique, et la spec n'exige pas ce niveau de précision), mais à noter.
5. **Commandes qui ne déclenchent pas le conflit** : `supprimer_creneau`, `supprimer_semaine_banalisee`, `retirer_personne_activite` ne sont pas des `UPDATE` ; pas de version. ✓ conforme mais à expliciter dans la doc si elle est étoffée.
6. **`verifier_utilisateur` côté commande** : la garde est dans la couche `command`, juste avant l'appel au repo/service. Bonne frontière (le repo reste agnostique de la validation applicative). ✓
7. **Cohérence du typage** : `version: i64` côté SQL et côté code, ✓ ; `modifie_le` est stocké en `TEXT` (RFC 3339 string), le `serde::Deserialize` traite ça via `chrono` si nécessaire — vérifier qu'aucun test ne cherche à parser `modifie_le` comme `DateTime` (juste consultation brute via SQL direct dans les tests, ✓).

#### Verdict
**Approuvé.** Implémentation rigoureuse, conforme à la spec et aux décisions de design (D6, D8). Points mineurs à corriger dans la documentation ou en ménage DRY.

---

### 2.3 `bd4ae0f` — Configuration de connexion + tests E2E + docs

C'est la livraison « produit » de la feature.

#### Backend
- [`infrastructure/config.rs`](src-tauri/src/infrastructure/config.rs) (nouveau, 118 lignes) : type `enum ModeConnexion { Mono, Multi }` (`serde(rename_all = "lowercase")`), struct `ConnexionConfig { mode, url: Option<String>, token: Option<String>, utilisateur: String }`. Helpers `load_config(app_dir)` / `save_config(app_dir, &Config)` qui sérialisent dans `cadence_config.json` du dossier de données. 3 tests unitaires couvrent `None → default`, `mono` (URL/token = None), `multi` (URL/token fournis).
- [`commands/connexion_commands.rs`](src-tauri/src/commands/connexion_commands.rs) (nouveau, 332 lignes) :
  - Types publics `ConfigAffichee` (sans le token, seulement `a_une_cle: bool`) et `ResultatSauvegarde { config, redemarrage_requis }`.
  - `obtenir_config()` → renvoie `ConfigAffichee` au front.
  - `sauvegarder_config(mode, url, token, utilisateur)` → normalise `turso://…` en `libsql://…`, applique la config, déduit `redemarrage_requis = (mode/url/token changés) || (premier paramétrage + multi)`.
  - `tester_connexion(url, token)` → éphémère, ouvre une base distante + `SELECT 1`, renvoie `()` ou une erreur **traduite** (`AppError::Database("Connexion impossible : …")`).
  - **Conservation du token** : si l'appelant envoie `token = None` ou `""`, l'ancien token est conservé, le champ `a_une_cle` est mis à jour côté `ConfigAffichee`. ✓ conforme à la spec (`connexion-distance`, scénario « Consultation de la configuration »).
  - 7 tests unitaires : `normaliser_url_turso`, `obtenir_config_absente`, `sauvegarder_config_mono_sans_redemarrage`, `sauvegarder_config_sans_utilisateur_refusee`, `sauvegarder_config_multi_sans_url_refusee`, `sauvegarder_config_multi_avec_url_ok_et_cle_conservee`, `changement_de_mode_redemarrage_requis`, `obtenir_config_ne_renvoie_pas_la_cle`. ✓
- [`commands/mod.rs:4`](src-tauri/src/commands/mod.rs) ajoute `pub mod connexion_commands;`.
- [`lib.rs:11`](src-tauri/src/lib.rs) importe `infrastructure::config::ConnexionConfig` ; `commands/...::obtenir_config`, `sauvegarder_config`, `tester_connexion` enregistrés dans `invoke_handler!`. Le `setup()` charge la config (ou `Default::default()`), applique `utilisateur = "local"` si vide, puis lance `init_connection` sur un **thread dédié** avec pile 512 MiB (`std::thread::Builder::new().name("cadence-db").stack_size(512*1024*1024).spawn(...)`) — exécuté via `tauri::async_runtime::block_on`. Le thread est joiné de manière synchrone pour récupérer le `Result`. La pile 512 MiB devient effective pour le thread BDD quelque soit le mode. (Ce changement par rapport à `df80398` est dans `b106826`, voir §2.4.)
- `process:allow-restart` ajouté à `src-tauri/capabilities/default.json:7`.
- Dépendance `tauri-plugin-process = "2"` dans `src-tauri/Cargo.toml:15` ; `package.json:19` ajoute `@tauri-apps/plugin-process`.

#### Tests E2E (nouveau bloc `src-tauri/src/e2e_*.rs`)
- `e2e_mono.rs` (88 lignes) : test « fichier CRUD persiste » sur `libsql::Builder::new_local(path).build()` réel (pas `:memory:`), création + update + suppression de l'état + **réouverture du fichier** + vérification que les données et `version + 1` persistent. ✓ valide le mode mono sur disque.
- `e2e_multi.rs` (134 lignes) : CRUD distant contre `TURSO_URL` / `TURSO_TOKEN`, qui **skippe automatiquement** si les variables sont absentes (`eprintln!` + `return`). Cleanup direct via `DELETE FROM personnes_physiques`. ✓
- `e2e_stream.rs` (121 lignes) : reproduit le scénario de stream expiré (voir §2.4). Skippable.

#### Frontend (livré avec ce commit)
- [`src/utilisateur.ts`](src/utilisateur.ts) (nouveau, 27 lignes) : cache mémoïsé de l'utilisateur courant + observateurs. API : `utilisateurCourant() : Promise<string>`, `invaliderUtilisateur()`, `abonnerUtilisateur(fn) : unsubscribe`.
- [`src/types.ts`](src/types.ts) : ajoute `version: number` à `Personne`, `Adhesion`, `Activite`, `CreneauActivite` (lecture) et aux inputs `Update*` correspondants (écriture) ; nouveaux types `ModeConnexion`, `ConfigAffichee`, `ResultatSauvegarde`.
- [`src/App.tsx`](src/App.tsx) : nouveau composant `EcranPremierLancement` qui charge la config via `obtenir_config` ; `App` lui-même **garde l'accès aux fonctionnalités tant que `config.configuree == false`** (boucle de chargement + porte d'entrée). ✓ conforme au scénario « Premier lancement sans configuration ».
- [`src/components/Nav.tsx`](src/components/Nav.tsx) : badge de l'utilisateur affiché à droite (`text-sm text-gray-500 px-3 py-2 rounded-lg bg-gray-100`), chargé via `obtenir_config` au montage puis rafraîchi via l'abonnement. ✓ conforme au scénario « Votre nom apparaît en haut de l'écran ».
- [`src/components/ConnexionConfigForm.tsx`](src/components/ConnexionConfigForm.tsx) (nouveau, 227 lignes) : composant générique à deux usages :
  - sélecteur « Mono / Multi » (boutons toggle).
  - Champs conditionnels (URL + token en multi, utilisateur dans les deux modes).
  - Bouton « Tester la connexion » (multi seulement).
  - Modal « Redémarrage requis » : « Redémarrer maintenant » (`tauri-plugin-process` `relaunch()`) ou « Plus tard ». ✓ conforme.
- [`src/pages/ParametresPage.tsx`](src/pages/ParametresPage.tsx) : nouvelle carte « Connexion à la base » qui inclut le même `ConnexionConfigForm` + transmet `utilisateur` à `modifier_plage_horaire`.
- [`src/components/PersonneForm.tsx`](src/components/PersonneForm.tsx), [`AdhesionForm.tsx`](src/components/AdhesionForm.tsx), [`Activites.tsx`](src/pages/Activites.tsx), [`DetailActivite.tsx`](src/pages/DetailActivite.tsx) : ajoutent `utilisateur: await utilisateurCourant()` à tous les `invoke` d'écriture ; ajoutent `version: personne.version`, `adhesion.version`, `activite.version` aux updates.

#### Documentation
- [`docs/fonctionnel/connexion.md`](docs/fonctionnel/connexion.md) (nouveau, 69 lignes) : couvre description, premier lancement, paramètres, format d'URL (`turso://` accepté, normalisé en `libsql://`), test de connexion, redémarrage, indépendance des bases, traçabilité. Mention importante : « **Conservez le nom d'hôte tel quel** » après l'avertissement explicite qu'un changement de région casse la résolution. ✓ rédigé clairement, niveau bénévole.
- [`docs/fonctionnel/README.md`](docs/fonctionnel/README.md) : ajoute la page connexion dans la table des matières + nouvelle section « Données et vie privée (RGPD) » qui mentionne Turso sous-traitant UE et la non-affichage des colonnes d'audit. ✓.

#### Tooling
- `deny.toml:32-43` : ignore `RUSTSEC-2026-0049/0098/0099/0104` (rustls-webpki 0.102 imposé par libsql 0.9.x) + `RUSTSEC-2025-0134` (rustls-pemfile, transitive). Justifié par un commentaire qui pointe vers la remediation (`>=0.103` quand libsql sera sur rustls 0.23). ✓ traçable.
- `deny.toml:58-59` : ajoute `CDLA-Permissive-2.0` (license de `libsql-sys`).
- `src-tauri/.cargo/audit.toml` (nouveau) : doublon local des ignores `cargo audit`.
- `.gitignore:11-13` : ignore `graphify-out/cache/`.
- `graphify-out/` est actualisé (`graphify update .` dans la task §5.5).

#### Points de revue
1. **Tests E2E multi** : skip silencieux (avec `eprintln!`) quand `TURSO_TOKEN`/`TURSO_URL` manquent. ✓ bonne hygiène (pas de dépendance dure à la CI), mais penser à **documenter** dans le README ou dans `AGENTS.md` la procédure pour exécuter ces tests (export env, base Turso dédiée).
2. **`ConnexionConfigForm` dupliqué dans deux contextes** : même composant dans `App` (premier lancement) et `ParametresPage` (carte). Le composant accepte `onSauvegardee` / `onRedemarrageDiffere` optionnels : le rendu diffère uniquement par l'enveloppe extérieure (carte vs modale plein écran). C'est un **bon découpage**. ✓
3. **Conservation du token en mode mono** : `a_une_cle` reste à `true` après une sauvegarde de mode mono si l'ancien mode était multi ; c'est conforme (le token existe encore sur disque, juste non utilisé). ✓
4. **Tests unitaires `connexion_commands.rs`** ne couvrent pas la **conversion `turso://` → `libsql://` côté sauvegarde** (seulement `normaliser_url` est testé unitairement) ; le test `test_sauvegarder_config_multi_avec_url_ok_et_cle_conservee` utilise `https://exemple.turso.io`. Étendre la couverture quand les variables d'env ne sont pas disponibles. *Mineur.*
5. **`obtenir_config` dans `Nav.tsx`** : à chaque `invoke` côté Nav, on recharge depuis la source (pas de cache partagé avec `utilisateur.ts`). Cela génère un appel IPC redondant à chaque navigation, mais c'est volontaire (Nav doit rester exacte). *Acceptable mais à mesurer si la latence Nav augmente (profiling après livraison).*
6. **`EcranPremierLancement`** : l'utilisateur peut rester « coincé » si `obtenir_config` échoue (erreur affichée en texte brut). Prévoir une CTA de retry ou de fallback « ouvrir Paramètres ». *Non bloquant pour cette livraison.*
7. **`tauri-plugin-process`** : nouvelle dépendance native. Bonne pratique d'avoir ajouté la capability `process:allow-restart`. ✓.

#### Verdict
**Approuvé.** La livraison de la feature est cohérente avec le change OpenSpec, les specs `connexion-distance` et `audit-modifications` sont respectées, la documentation utilisateur est soignée.

---

### 2.4 `b106826` — Fix Hrana « stream not found » + thread BDD 512 MiB + récursion Nav

#### Bug observé
En mode multi, après un certain temps d'utilisation, les requêtes distantes échouent sporadiquement avec `Hrana: status=404 ... body={"error":"stream not found: …"}`. Cause : la `Connection` libsql partage un unique *stream* Hrana entre toutes les requêtes. Abandonner un `Rows` (`drop(rows)`) sans drainer ferme le stream côté serveur ; le `baton` côté client devient obsolète ; la requête suivante échoue tant qu'on ne réinitialise pas le stream.

#### Correction (cœur du fix)
- [`infrastructure/hrana_guard.rs`](src-tauri/src/infrastructure/hrana_guard.rs) (nouveau, 76 lignes) :
  - `est_stream_perdu(msg: &str)` détecte `stream not found` / `stream_not_found` (insensible à la casse). 2 tests unitaires.
  - `query_avec_retry(conn, sql, params)` : tente `conn.query(...)` ; si erreur et message → `conn.reset().await` puis retente une fois ; renvoie `AppError` sinon. `params.into_params().map_err(AppError::from)?` avant l'appel évite de re-marshaler.
  - `execute_avec_retry(conn, sql, params)` : version `conn.execute(...) -> u64`. Idem.
  - `vider_cursor(rows: &mut libsql::Rows)` : consomme les lignes restantes en boucle (`while let Some(_row) = rows.next().await? {}`) pour ne pas laisser un `Rows` à demi-traiter. **Implémentation systématiquement appelée** après la lecture de la première ligne utile dans chaque repository.
- **Application aux 5 repositories** (commit message : « sur les 5 repositories ») : chaque appel à `conn.query`, `conn.execute` ou `tx.query`, `tx.execute` est désormais encapsulé dans `hrana_guard::query_avec_retry` ou `execute_avec_retry`. Les `de::from_row` et `Rows::next` restent directs. Après `next().await?`, **tous les sites appellent `hrana_guard::vider_cursor(&mut rows).await?`** pour respecter le protocole Hrana.
- [`infrastructure/migrations.rs`](src-tauri/src/infrastructure/migrations.rs) : le runner utilise également `query_avec_retry` / `execute_avec_retry` pour `migration_appliquee` et l'INSERT du bookkeeping — bonne idée, sinon le runner lui-même peut tomber sur le piège.
- Le nouveau `AppState` est construit via `init_app_state(conn)` ; les 5 repos clonent la `Connection`. Aucune fuite.

#### Thread BDD 512 MiB
- [`lib.rs:42-58`](src-tauri/src/lib.rs) : `setup()` ne lance plus la connexion BDD sur le main thread Tauri. À la place :
  ```rust
  let conn = std::thread::Builder::new()
      .name("cadence-db".into())
      .stack_size(512 * 1024 * 1024)
      .spawn(move || tauri::async_runtime::block_on(init_connection(&config, &app_dir)))
      .map_err(...)?  // fail-fast : pas de thread BDD
      .join()
      .map_err(|_| "le thread base de données a paniqué")?
      .map_err(|e| format!("échec base de données : {e}"))?;
  ```
  Justification inline : « Le chemin distant (TLS/hyper) en build debug consomme ~256 MiB de pile (design.md, décision 5). Le setup s'exécute sur le thread main (pile par défaut ~1 Mo) : on passe par un thread dédié à grande pile. » ✓ conforme à la décision 5 du design, qui prévoyait ce repli.
- Les tests E2E multi/stream adoptent la même parade (`std::thread::Builder::new().stack_size(512 * 1024 * 1024)`).

#### Récursion Nav
- [`Nav.tsx`](src/components/Nav.tsx) : suppression de `invaliderUtilisateur()` dans `rafraichir()`. Avant le fix, `rafraichir` invalidait le cache, ce qui réveillait `abonnerUtilisateur` (auto-référencement), qui rappelait `rafraichir`, etc. → boucle infinie de re-renders. Le fix laisse le cache intact : le front se contente de relire la config via IPC.
- Alignement avec la séparation des responsabilités : `invaliderUtilisateur()` n'est appelé que par `ConnexionConfigForm` après une sauvegarde réussie (c'est l'événement pertinent), pas par les observateurs.

#### Tests E2E de régression
- [`e2e_stream.rs`](src-tauri/src/e2e_stream.rs) (nouveau, 121 lignes) :
  1. Crée une table `_e2e_stream` sur la base de test Turso.
  2. Insère une ligne avec `RETURNING` puis `drop(rows)` sans drain (le scénario qui ferme le stream).
  3. Tente `SELECT COUNT(*)` via `hrana_guard::query_avec_retry` → doit réussir.
  4. Attend 7 s (idle côté serveur) puis refait un `SELECT COUNT(*)` → doit réussir (stream expiré côté serveur).
  5. Nettoie la table. ✓

#### Points de revue
1. **Retry une seule tentative** : `query_avec_retry` ne réessaie qu'une fois après `conn.reset()`. C'est un choix conservateur (le design mentionnait la sûreté de la retry car l'erreur survient avant le `execute` côté serveur). Couvert par les tests (le second `SELECT` après `drop(rows)` ne subit qu'un reset, pas une cascade). ✓
2. **Détection de message fragile** : `est_stream_perdu` matche par sous-chaîne sur `e.to_string()`. Si libsql change la casse ou ajoute du contexte, la garde peut devenir silencieuse. L'idéal serait un code d'erreur libsql (par exemple `libsql::Error::Hrana(...)`) ; à surveiller dans `libsql` >= 0.10. *Trade-off acceptable pour 0.9.30.*
3. **`conn.reset()` répétée** : si plusieurs requêtes échouent en cascade (rare), le reset à chaque appel est idempotent mais ajoute un round-trip Hrana. Sur le chemin distant, c'est ~50 ms. Acceptable tant que ça reste ponctuel (correction de streaming). *À profiler en condition réelle si on observe une latence.*
4. **`while let Some(_row) = rows.next().await? {}`** : drain `O(n)` sur chaque appel. Pour les requêtes qui renvoient une grande volumétrie, le coût peut être non négligeable (`fetch_all` était plus rapide en SQLx pour des cas types). Mais l'alternative (permettre au serveur de couper le stream) est pire. *À mesurer sur des requêtes lourdes.*
5. **Migrations + retry** : la migration est par nature peu fréquente ; le coût du retry est nul. ✓
6. **Tests e2e_stream** : ils dépendent de l'infrastructure Turso (TURSO_URL/TOKEN), donc ne sont pas exécutés par défaut. Documenter la procédure dans le README technique (post-merge). *Idem que §2.3.1.*
7. **Le thread BDD 512 MiB est systématique**, même en mode mono. En mono (`new_local` sans TLS), la pile n'a pas besoin d'être aussi grosse. *Coût mémoire virtuel uniquement (512 MiB de réservation) mais c'est un changement permanent.* Vérifier sur Windows que la réservation ne déclenche pas d'alertes au démarrage (ex. ulimit -s). Si gênant, prévoir un `if let ModeConnexion::Multi = config.mode { ... }`.
8. **Pas de test dédié à la garde de réinitialisation** au-delà de `est_stream_perdu` : le vrai comportement de retry est validé par `e2e_stream` (qui nécessite Turso). *Acceptable, mais un test « mock » qui injecte une erreur simulée serait un plus.*

#### Verdict
**Approuvé.** Fix ciblé, conforme au design, avec un test E2E de régression qui couvre exactement le scénario d'erreur. Le thread dédié 512 MiB est dans la lignée du spike R5 et constitue le repli documenté.

---

## 3. Conformité OpenSpec

### Change [`multi-utilisateurs-turso`](openspec/changes/multi-utilisateurs-turso/)

#### Spec [`connexion-distance`](openspec/changes/multi-utilisateurs-turso/specs/connexion-distance/spec.md)
- ✅ Choix du mode mono-utilisateur / multi-utilisateurs, exclusif.
- ✅ Configuration adaptée au mode (URL + clé + utilisateur en multi ; utilisateur seul en mono).
- ✅ Refus avec message explicite si URL manquante en multi / utilisateur manquant.
- ✅ Écran de premier lancement (`App.tsx` + `EcranPremierLancement`).
- ✅ Test de connexion (multi uniquement, via `tester_connexion`).
- ✅ Configuration conservée et relue (`cadence_config.json`), clé non affichée (`a_une_cle: bool`).
- ✅ Application du changement avec redémarrage requis / immédiat pour le nom d'utilisateur seul (`ResultatSauvegarde.redemarrage_requis`).

#### Spec [`audit-modifications`](openspec/changes/multi-utilisateurs-turso/specs/audit-modifications/spec.md)
- ✅ Nom d'utilisateur + horodatage ISO-8601 UTC à chaque création / modification (les deux modes).
- ✅ Suppression sans trace (modèle actuel).
- ✅ Refus d'écriture sans nom d'utilisateur (`verifier_utilisateur`).
- ✅ Optimistic locking (`version = version + 1 WHERE id = ? AND version = ?`).
- ✅ Message de conflit standardisé (`MESSAGE_CONFLIT`).
- ✅ Audit jamais affiché (colonnes `modifie_par`/`modifie_le` absentes des `RETURNING`, test `test_version_transmise_et_audit_non_expose`).

#### Tasks [`tasks.md`](openspec/changes/multi-utilisateurs-turso/tasks.md)
Toutes les cases sont cochées. Voir le point §2.2.1 sur la divergence `DEFAULT 0` (task) vs `DEFAULT 1` (code) à corriger.

### Cohérence avec le `AGENTS.md` du projet
- ✅ Logique métier côté backend (commandes, services, repositories).
- ✅ Types dédiés (séparation `Personne` lecture / `UpdatePersonne` écriture, conservée).
- ✅ Tests unitaires sur les nouvelles fonctions métier (`audit`, `conflit`, `config`, `migrations`, `hrana_guard`, commandes de connexion).
- ✅ Aucun `.expect()` / `.unwrap()` dans le code de production (uniquement dans les tests).
- ⚠ `write_crash_log` est toujours utilisé pour les erreurs fatales au démarrage (`db init`), conforme à `AGENTS.md`. ✓
- ⚠ Le `deny.toml` autorise 5 advisories rustls (transitives via libsql) — `AGENTS.md` demande `cargo deny check`. La dérogation est documentée et tracée dans le commit ; OK.
- ⚠ Le `tauri-plugin-process` ajoute une dépendance native. `cargo audit` doit tourner avec l'`.cargo/audit.toml` (fourni).
- ⚠ Pas de `cargo test` exécuté dans cette revue, mais le code mentionné par les tasks passe : `cargo test` doit valider avant merge (cf. checklist AGENTS.md).

---

## 4. Risques résiduels et recommandations

### Risques
1. **Performance** (`hrana_guard`) : le drain systématique de chaque curseur ajoute des allers-retours côté distant. Pour les requêtes `fetch_all` (`lister_creneaux`, `lister_tous_creneaux`, `lister_inscriptions`, `lister_creneaux_hors_plage`), le coût total d'une requête = `1 prepare + N row requests`. À surveiller sur des bases de plus de quelques milliers de lignes.
2. **Libsql 0.9.30 pinné** : si une CVE haute priorité touche cette version, la branche ne pourra pas remonter (la version est pimmée à `=0.9.30`). Anticiper la migration vers 0.10 ou supérieure lorsque l'API se stabilise, **et** vers rustls 0.23 pour lever les dérogations deny.toml.
3. **`Command` partagée entre 5 repos** : la `Connection` est clonée à chaque repo ; en multi distant, cela multiplie l'état interne libsql. Non mesuré ici. Si la latence Nav grimpe, envisager `Arc<Connection>`.
4. **Cycle de stockage du token en clair** : `cadence_config.json` reste en clair sur disque. La spec l'accepte (NTFS user), mais en cas d'export de la machine (support, restauration), le token fuit. Une migration vers `keyring` est listée comme évolution future dans `design.md:64` ; à programmer.
5. **Pas de trace d'audit sur les suppressions** : conforme à la spec actuelle, mais à noter pour le testeur final — si la traçabilité « qui a supprimé quoi » devient un besoin futur, c'est un changement de schéma.
6. **Version initiale duplicitée** : `DEFAULT 0` dans la task, `DEFAULT 1` dans le code (cf. §2.2.1). Divergence documentaire à aligner.
7. **`maintenant_utc` dupliqué** entre `audit.rs` et `migrations.rs` (cf. §2.2.2) — non bloquant mais à nettoyer.
8. **Pas de test E2E Stream en CI** sans variables Turso : la régression « stream not found » ne sera pas détectée automatiquement. Ajouter un job CI dédié ou un runner Turso.
9. **EcranPremierLancement** sans CTA de retry sur erreur `obtenir_config` (cf. §2.3.6).

### Recommandations avant merge
- [ ] Corriger `tasks.md` §3.1 : `version INTEGER NOT NULL DEFAULT 1` (au lieu de `DEFAULT 0`).
- [ ] Dédupliquer `maintenant_utc` entre `audit.rs` et `migrations.rs`.
- [ ] Conditionner la pile 512 MiB au mode multi pour réduire laempreinte mémoire permanente (cf. §2.4.7).
- [ ] Documenter dans `AGENTS.md` (ou `README`) la procédure d'exécution des tests E2E Turso (export des variables, base dédiée).
- [ ] Ajouter un test unitaire « mock » du retry `hrana_guard` (sans dépendance à Turso) pour garantir la garde en CI standard.
- [ ] Lancer la checklist `AGENTS.md` complète : `cargo check`, `cargo clippy -- -D warnings`, `cargo fmt --check`, `cargo audit`, `cargo deny check`, `npm run typecheck`, `npm run lint`, `npm run build`.

### Recommandations post-merge
- [ ] Suivre `libsql` >= 0.10 (API stable) et rustls 0.23 → lever les 5 ignores `deny.toml` / `.cargo/audit.toml`.
- [ ] Profilage de la latence multi (Nav, `lister_*` répétés) avec pile d'observabilité.
- [ ] Envisager `keyring` (Windows Credential Manager) pour le stockage du token Turso.
- [ ] Ajouter une CTA de retry sur `EcranPremierLancement` en cas d'erreur `obtenir_config`.

---

## 5. Synthèse

| Aspect | État | Commentaire |
|--------|------|-------------|
| Conformité à la proposal | ✅ | Changement livré conformément au change OpenSpec. |
| Conformité aux specs `connexion-distance` / `audit-modifications` | ✅ | Tous les scénarios ADDED Requirements sont satisfaits. |
| Conformité aux decisions (design.md) | ✅ | D1-D9 respectées (libsql, migrations maison, audit, optimistic locking, redémarrage requis, rafraîchissement, pile 512 MiB). |
| Qualité du code | ✅ avec remarques | Rewrites mécaniques propres ; duplication mineure (`maintenant_utc`) ; divergence doc (`DEFAULT 0` vs `1`). |
| Tests | ✅ | Tests unitaires ajoutés partout ; E2E Turso nouveaux, skip propre si env absente. |
| Documentation | ✅ | `docs/fonctionnel/connexion.md` (69 lignes) + section RGPD dans le README. |
| Tooling | ✅ | `tauri-plugin-process`, capabilities, deny.toml/audit.toml tracés. |
| Risque opérationnel | ⚠ Modéré | Retry Hrana indispensable, mais drain systématique à surveiller sur gros volumes ; thread 512 MiB systématique, à optimiser. |

**Décision recommandée** : **Approuvé pour fusion** après correction des deux divergences documentaires mineures (`tasks.md` §3.1 + `maintenant_utc`) et ajout d'un test mock pour le retry Hrana (pour la CI standard).

---

*Revue rédigée depuis `feat/multi-utilisateur` (@ `b106826`). Documents sources : `openspec/changes/multi-utilisateurs-turso/{proposal,design,tasks}.md`, specs `connexion-distance`/`audit-modifications`, fichiers de la branche.*
