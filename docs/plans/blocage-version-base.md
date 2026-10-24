# Plan : blocage des versions obsolètes (multi-utilisateurs)

## Problème

Deux utilisateurs partagent la base Turso : l'un avec Cadence vN, l'autre vN-1. Le binaire vN-1 ne connaît pas les migrations que vN a pu appliquer à la base → risque de comportement erroné. Il faut bloquer le client obsolète à l'écran d'accueil et l'inviter à mettre à jour.

## Décisions retenues (confirmées avec l'utilisateur)

- **Mécanisme** : comparaison du changelog — la base `_cadence_migrations` contient déjà les migrations appliquées ; le binaire embarque la liste `MIGRATIONS`. Si la base contient une migration inconnue du binaire → base plus récente → blocage.
- **Périmètre** : mono + multi (uniforme ; sans effet en mono car la base locale est créée par la même version).
- **Niveau** : gate au démarrage dans `App.tsx`.

## Backend (Rust)

1. **Nouveau module `src-tauri/src/infrastructure/compat.rs`** + déclaration dans `mod.rs` :
   - `pub fn version_app() -> String` → `env!("CARGO_PKG_VERSION")` (source unique `Cargo.toml`, verrouillée par `release.yml`).
   - `pub struct Compatibilite { compatible: bool, version_installee: String, migrations_inconnues: Vec<String> }` (`Serialize, Clone, Debug`).
   - `pub async fn verifier_compatibilite(conn: &Connection) -> Result<Compatibilite, AppError>` :
     - `SELECT nom FROM _cadence_migrations` via `hrana_guard::query_avec_retry`.
     - Si une migration appliquée n'est pas dans la liste connue du binaire → `compatible = false` et la liste des migrations inconnues.
     - Table absente (base vierge) → compatible.

2. **`migrations.rs`** : exposer les noms connus via `pub fn noms_migrations() -> impl Iterator<Item = &'static str>` (premier élément de chaque tuple de `MIGRATIONS`), pour comparaison dans `compat.rs`.

3. **`lib.rs` setup** : le bloc `setup` est une closure synchrone et le chemin Turso distant (TLS/hyper) consomme ~256 Mo de pile en debug. La vérification doit donc être exécutée **dans le même thread à grande pile que `init_connection`** (et non après le `.join()`, qui revient sur le thread principal sans contexte async ni grande pile). Le thread retourne `(conn, compat)` :

   ```rust
   let (conn, compat) = std::thread::Builder::new()
       .name("cadence-db".into())
       .stack_size(512 * 1024 * 1024)
       .spawn(move || {
           let conn = tauri::async_runtime::block_on(init_connection(&config, &app_dir))?;
           let compat = tauri::async_runtime::block_on(verifier_compatibilite(&conn))?;
           Ok::<_, AppError>((conn, compat))
       })
       .map_err(|e| format!("échec création du thread base de données : {e}"))?
       .join()
       .map_err(|_| "le thread base de données a paniqué".to_string())?
       .map_err(|e| format!("échec base de données : {e}"))?;

   app.manage(compat);
   app.manage(init_app_state(conn));
   ```

   Aucun changement de signature de `init_connection`/`init_app_state` → aucun impact sur les tests e2e existants.

   **Justification de l'ordre migrations → compat** : `cadence_migrations` s'exécute dans `init_connection`, avant la vérification. C'est sûr car `migration_appliquee` compare par nom : un binaire vN-1 qui ne connaît pas les migrations de vN les ignore silencieusement, sans les ré-appliquer ni planter.

4. **`connexion_commands.rs`** : nouvelle commande conforme à la convention du projet (`pub async fn ... -> Result<T, AppError>`) :

   ```rust
   #[tauri::command]
   pub async fn obtenir_compatibilite(
       state: State<'_, Compatibilite>,
   ) -> Result<Compatibilite, AppError> {
       Ok(state.inner().clone())
   }
   ```

   Enregistrement dans `lib.rs` (`invoke_handler`).

## Frontend (React/TS)

5. **`types.ts`** : interface `Compatibilite { compatible: boolean; version_installee: string; migrations_inconnues: string[] }`.

6. **`App.tsx`** : le gate compat doit être **prioritaire sur tous les autres écrans** (chargement, premier lancement, app normale). Restructurer l'état de chargement pour attendre les deux promesses (`obtenir_config` + `obtenir_compatibilite`) via `Promise.all` avant de décider quoi afficher :

   ```tsx
   const [etat, setEtat] = useState<{ config: ConfigAffichee; compat: Compatibilite } | null>(null);
   // au montage : const [config, compat] = await Promise.all([invoke("obtenir_config"), invoke("obtenir_compatibilite")]);
   ```

   Rendus dans l'ordre de priorité : (a) tant qu'aucun résultat → « Chargement... » ; (b) `!compat.compatible` → écran bloquant plein écran (même avant l'écran de premier lancement) ; (c) `!config.configuree` → premier lancement ; (d) sinon → app. Message du blocage :

   > « Votre version de Cadence (X.Y.Z) est obsolète. Cette base de données a été mise à jour par une version plus récente de l'application. Mettez à jour Cadence pour continuer à l'utiliser. »

## Tests (obligatoires, AGENTS.md)

7. Tests unitaires dans `compat.rs` :
   - base avec uniquement des migrations connues → compatible ;
   - base avec une migration inconnue injectée → incompatible + migration listée ;
   - base vierge (sans table `_cadence_migrations`) → compatible ;
   - `version_app()` renvoie une version non vide.

## Workflow OpenSpec (convention équipe)

8. Créer le change OpenSpec `blocage-version-base` (proposal → specs delta → tasks), suivre les rôles Architecte/PM/Concepteur/Développeur, puis implémenter selon ce plan. Le delta spec `compatibilite-version` couvrira : détection via changelog, blocage au démarrage, message d'invite à mettre à jour.

## Vérifications finales (AGENTS.md)

`cargo test`, `cargo check`, `cargo clippy -- -D warnings`, `cargo fmt --check`, `cargo audit`, `cargo deny check`, `npm run typecheck`, `npm run lint`, `npm run build` (dans `src-tauri/` pour Rust, racine pour npm).
