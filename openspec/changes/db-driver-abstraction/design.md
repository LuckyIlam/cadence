# Change `db-driver-abstraction` — design

## Context

- Cadence a une seule chaîne d'accès aux données (`src-tauri/src/infrastructure/db.rs`),
  bâtie sur `libsql = "=0.9.30"`. Le `Connection` est cloné à chaque repository dans
  `init_app_state` (`infrastructure/db.rs:13-21`, `repositories/mod.rs:4-9`).
- Le passage `sqlx → libsql` a touché ~3 100 lignes de Rust parce que **le type du
  driver a été utilisé comme contrat commun** dans 5 repositories, 11 structs
  domain, 2 services et le runner de migrations. Preuves :
  - `use libsql::Connection` / `use libsql::Transaction` dans 11 fichiers
    (`infrastructure/{db,hrana_guard,migrations}.rs`, 5 `repositories/*.rs`,
    `services/{activite,parametre}_service.rs`,
    `commands/planning_commands.rs`).
  - `&mut libsql::Transaction` fuit dans **3 signatures de traits**
    (`planning_repo.rs:38-46`, `parametre_repo.rs:13,51`,
    `activite_repo.rs:62-85`), mais aussi dans les **services**
    (`activite_service.rs` ~12 occurrences, `parametre_service.rs` ~9)
    et dans **commands/** : `planning_commands.rs:27,86,123` appelle
    directement `state.conn.transaction_with_behavior(Immediate)`, et
    `state.conn` (champ `pub` de `AppState`) est consommé dans
    `activite_commands.rs` (10 sites) et `parametre_commands.rs` (3 sites).
  - `libsql::de::from_row` utilisé dans 11 structs + 2 helpers `fetch_one` /
    `fetch_optional` (`personne_repo.rs:51,67`, `activite_repo.rs` etc.).
  - `libsql::params!` et `libsql::Value` utilisés ~80 fois dans les repositories,
    services, tests et commandes (helpers de paramètres dynamiques dans
    `personne_repo.rs:191-243`).
- L'utilisateur veut **préparer** l'arrivée de drivers alternatifs (Postgres,
  MySQL) sans rejouer ce coût. Décision validée : *abstraction d'abord, sans
  ajout de driver pour l'instant*. Voir la discussion dans
  `multi-utilisateur-review.md` (section « Volumétrie estimée »).
- Décisions validées : migrations en Rust via `refinery`, `RETURNING`
  spécifique par driver (le fallback MySQL sera ajouté par change dédié).

## Goals / Non-Goals

### Goals

1. Faire reposer l'accès aux données sur des traits neutres (`Db`,
   `DbTransaction`, `IntoParams`, `RowView` / `DeserializeRow`, `RetryPolicy`)
   qui ne dépendent plus de `libsql`. Le seul code connaissant `libsql` doit
   résider dans `drivers/libsql/`.
2. Préserver à 100 % le comportement actuel : aucune modif fonctionnelle des
   commands, services, tests E2E mono/multi, specs `connexion-distance` /
   `audit-modifications` / `plage-horaire-activite` / `planning-activites`.
3. Réduire le coût d'ajout d'un driver de ~3 100 lignes à ~1 000 lignes
   (validé par l'analyse préalable).
4. Adopter `refinery` pour les migrations versionnées multi-dialectes
   (sans casser le runner actuel ; la branche libsql reste active
   immédiatement).

### Non-Goals

- **Hors scope** : ajouter Postgres ou MySQL. Aucune dépendance
  `tokio-postgres` / `mysql_async` ajoutée dans cette PR. Les nouveaux
  drivers viendront dans des changes dédiés (un par driver), quand un
  besoin concret émergera (demande d'une association, fonctionnalité
  spécifique d'un driver).
- **Hors scope** : changer la sémantique des transactions
  (deux-requêtes-vs-RETURNING). Les implémentations futures de
  drivers choisiront au cas par cas (cf. D9).
- **Hors scope** : toucher au domaine (`src-tauri/src/domain/*.rs`) au-delà
  de l'ajout des derives nécessaires pour `DeserializeRow` (et seulement
  si on en a besoin).
- **Hors scope** : modifier le front (les types publics TS restent
  inchangés).
- **Hors scope** : remplacer la tokenisation JWT/Turso (décision
  ultérieure, traçable dans `openspec/changes/multi-utilisateurs-turso`).

## Decisions

### D1 — Trait `Db` central, transactions `Box<dyn DbTransaction>`

Définit l'interface du driver. Objectif : rendre la couche `commands/`
agnostique — aujourd'hui elle touche directement le driver
(`planning_commands.rs:27,86,123` appelle
`state.conn.transaction_with_behavior(Immediate)`).

```rust
// infrastructure/db.rs
#[async_trait]
pub trait Db: Send + Sync {
    async fn execute<P>(&self, sql: &str, params: P) -> Result<u64, AppError>
        where P: IntoParams;

    async fn fetch_one<P, T>(&self, sql: &str, params: P) -> Result<Option<T>, AppError>
        where P: IntoParams, T: DeserializeRow;

    async fn fetch_all<P, T>(&self, sql: &str, params: P) -> Result<Vec<T>, AppError>
        where P: IntoParams, T: DeserializeRow;

    async fn fetch_optional<P, T>(&self, sql: &str, params: P) -> Result<Option<T>, AppError>
        where P: IntoParams, T: DeserializeRow;

    async fn begin(&self) -> Result<Box<dyn DbTransaction + '_>, AppError>;

    async fn begin_immediate(&self) -> Result<Box<dyn DbTransaction + '_>, AppError>;

    async fn execute_batch(&self, sql: &str) -> Result<(), AppError>;

    fn driver_name(&self) -> &'static str;
}

#[async_trait]
pub trait DbTransaction: Send + '_ {
    async fn execute<P>(&self, sql: &str, params: P) -> Result<u64, AppError>
        where P: IntoParams;
    async fn fetch_one<P, T>(...) -> Result<Option<T>, AppError> ...;
    async fn fetch_all<P, T>(...) -> Result<Vec<T>, AppError> ...;
    async fn commit(self: Box<Self>) -> Result<(), AppError>;
    async fn rollback(self: Box<Self>) -> Result<(), AppError>;
}
```

Justification : un trait object `dyn Db` est moins rapide qu'un type
statique mais la différence est négligeable face au coût IPC d'un Tauri
command (mesurable). Les repositories deviennent polymorphes sans
template hell.

> **Écart d'implémentation (PR 1) — object-safety.** Le sketch ci-dessus
> (méthodes génériques `P: IntoParams`, `T: DeserializeRow`) n'est **pas**
> object-safe : `Arc<dyn Db>` (PR 2, tâche 2.4) ne peut pas l'utiliser.
> Le code livré en PR 1 scinde donc en deux :
> - **`Db` object-safe** — `execute(&self, sql, params: DbParams)`,
>   `fetch_one_row` / `fetch_optional_row` / `fetch_all_rows` (retournent
>   `Option<DbRow>` / `Vec<DbRow>` neutres), `begin`, `begin_immediate`,
>   `execute_batch`, `driver_name`.
> - **`DbExt`** — trait d'extension générique (`impl<D: Db + ?Sized> DbExt for D`)
>   portant les méthodes typées `fetch_one<T: DeserializeRow>`, `fetch_optional<T>`,
>   `fetch_all<T>`, implémentées via les méthodes object-safe de `Db` +
>   `DeserializeRow::from_row`.
>
> Conséquence : `params!` produit directement `DbParams` (concret), le
> générique `P: IntoParams` n'existe plus sur `Db`/`DbTransaction`.
> Les repositories (PR 2) gardent le même appel : `db.fetch_one::<T>(...)`
> via `DbExt`.

`begin_immediate` préserve la garantie d'atomicité documentée dans
`planning_commands.rs` (« BEGIN IMMEDIATE… pour éviter un conflit non
détecté ») : libsql → `BEGIN IMMEDIATE`, Postgres/MySQL → équivalent du
BEGIN simple (déjà immédiat). `execute_batch` est requis par
`cadence_migrations` (migrations.rs:59,73) si le runner doit rester
derrière `&dyn Db` (mitigation R1).

**Alternatives écartées** :
- `enum DbKind { Libsql(LibsqlDb), Postgres(...), Mysql(...) }` — plus rapide,
  mais force tous les sites à chaque nouveau driver (pattern matching
  obligatoire dans chaque repository). Refusé.
- `generics` sur les repos (`R: Db`) — compile-time perfect, mais propage les
  types dans les signatures services / commands. Pas de gain mesurable.
- ORM (sea-orm, diesel) — risque : on perd les optimisations libsql
  natives (streaming Hrana, retry intégré). Refusé pour cette PR.

### D2 — `IntoParams` + macro `params!` symétrique à `libsql::params!`

Le sucre syntaxique est essentiel pour rester idiomatique.

```rust
// infrastructure/db/params.rs
pub trait IntoParams { fn into_params(self) -> DbParams; }

impl IntoParams for () { ... }
impl<T1: ToDbValue> IntoParams for (T1,) { ... }
impl<T1, T2> IntoParams for (T1, T2) where T1: ToDbValue, T2: ToDbValue { ... }

// Macro `params![x, y, ...]` reproduisant le sucre de libsql::params!.
```

Adapter `Vec<T: ToDbValue>` pour les requêtes dynamiques (`personne_repo.rs`
recherche paginée).

**Alternative écartée** : `query_as!(Db, sql, a, b)` à la sqlx — plus
sûr mais explose la taille des sites d'appel. Acceptable seulement si
l'on utilise pas la DSL macro ; refusé.

### D3 — `RowView` + `DeserializeRow` neutre

Inspiration de l'API row de `sqlx` :

```rust
pub trait RowView<'a>: Send + Sync {
    fn get_i64(&self, idx: usize) -> Result<i64, AppError>;
    fn get_str(&self, idx: usize) -> Result<&str, AppError>;
    fn get_opt_str(&self, idx: usize) -> Result<Option<&str>, AppError>;
    fn get_bool(&self, idx: usize) -> Result<bool, AppError>;
    fn get_opt_bool(&self, idx: usize) -> Result<Option<bool>, AppError>;
    fn get_naive_date(&self, idx: usize) -> Result<NaiveDate, AppError>;
    // — au moins ce que les 11 structs utilisent —
}

pub trait DeserializeRow: Sized {
    fn from_row(row: &dyn RowView<'_>) -> Result<Self, AppError>;
}
```

Chaque struct domain implémente `DeserializeRow` (mécanique, ~15 lignes
par struct). Les helpers `fetch_one` / `fetch_optional` déjà présents
dans `personne_repo.rs:41-78` deviennent la norme.

> **Écart d'implémentation (PR 2, tâches 2.4-2.5)** : les impls
> `DeserializeRow` sont placées dans les fichiers repository (près des
> requêtes qui les lisent), et non dans les fichiers domain ni dans
> `drivers/libsql/row.rs` (cf. layout ci-dessous). Raison : ces impls
> sont mécaniques (grain SQL) et dépendent de `RowView`/`AppError` ; les
> garder près des requêtes évite d'éparpiller la connaissance de la forme
> des colonnes entre domain et infra. Les structs purement techniques
> (`TotalRow`, `IdRow`, `AuditRow`, `ModifieParRow`, `ActiviteCreneauRow`,
> …) sont définies directement dans les repos. Seul le parsing `Role`
> transite par le domaine via `domain::activite::role_from_str` (pur, testé),
> chaque repo le consommant via un helper local `role_from_row(row, idx)`.

**Alternative écartée** : typage générique par colonne (`Personne::FIELDS: &[FieldSpec; 7]`).
Très propre, mais ajoute une indirection runtime ou un proc-macro ; trop
lourd pour le gain.

### D4 — `RetryPolicy` extrait de `hrana_guard`

`hrana_guard.rs` était un cas particulier d'une idée générale : retenter
une opération sur erreur transitoire.

```rust
// infrastructure/retry.rs
#[async_trait]
pub trait RetryPolicy: Send + Sync {
    async fn run<F, Fut, T>(&self, op: F) -> Result<T, AppError>
        where F: FnOnce() -> Fut, Fut: Future<Output = Result<T, AppError>>;
}

pub struct HranaRetryPolicy { max_retries: u32 }    // détecte "stream not found"
pub struct SqlxRetryPolicy { ... }                  // détecte serialization_failure + deadlock (futur)
```

Les repositories appellent `self.policy.run(|| async { db.fetch_one(…) }).await`.
Les tests unitaires `hrana_guard::est_stream_perdu` migrent vers
`HranaRetryPolicy::matches`.

> **Écart d'implémentation (PR 1)** : `run` prend `F: FnMut()` (et non
> `FnOnce()`) — la politique doit pouvoir ré-invoquer l'opération à chaque
> tentative ; `#[async_trait]` est remplacé par un `async fn` déclaré
> directement dans le trait.

**Grain du retry (requête vs transaction)** : `RetryPolicy` opère au niveau
*requête isolée*. C'est sûr pour `HranaRetryPolicy` car l'échec « stream not
found » survient au `prepare`/describe, avant toute exécution côté serveur
(commentaire `hrana_guard.rs:14-15`) — rejouer la requête est sans effet de
bord. Ce n'est **pas** sûr pour un retry aveugle sur `serialization_failure` /
`deadlock` en Postgres : la requête a déjà été (partiellement) exécutée, seule
la **transaction entière** doit être rejouée. → le replay transaction-level est
porté par les appels à `DbTransaction` dans les services, hors `RetryPolicy`.
`RetryPolicy` reste une politique requête-only ; `SqlxRetryPolicy` sera
introduit dans le change dédié Postgres avec cette contrainte.

**Alternative écartée** : embeddings d'un middleware via `tower` —
excessif pour deux politiques. Refusé.

### D5 — `ConnexionConfig` étendu pour préparer Postgres/MySQL

Extension minimale (ne change pas le front) :

```rust
pub enum Driver { Sqlite, Postgres, Mysql }   // variantes futures, désactivées
pub enum Mode { Local, Distant }

pub struct ConnexionConfig {
    pub driver: Driver,                  // = Sqlite aujourd'hui
    pub mode: Mode,                      // Local | Distant
    pub url: Option<String>,
    pub token: Option<String>,
    pub utilisateur: String,
}
```
Aucun appel à `init_connection` n'utilise encore 
`Driver::Postgres` /
`Mysql` ; ces branches sont `unimplemented!()` avec un message clair vers
la PR d'ajout. Le frontend ne le voit pas encore.

> **Écart d'implémentation (PR 1) — `Mode` reporté.** L'`enum Mode { Local,
> Distant }` est **reporté** à une PR ultérieure : renommer
> `ModeConnexion` → `Mode` changerait la signature publique de la commande
> `sauvegarder_config` (critère 1.6 : aucune signature publique ne change).
> PR 1 ne pose donc que l'`enum Driver` + le champ `driver`
> (`#[serde(default)]`, rétro-compatible) ; `mode` conserve son type
> `ModeConnexion` actuel.

**Alternative écartée** : étendre maintenant le front avec choix SQLite /
Postgres / MySQL. Risque UX énorme pour un usage nul. Refusé.

### D6 — Déplacement des `*_repo.rs` par driver, pas par aggregate

Nouveau layout :

```
repositories/
└── mod.rs                  (réexporte les TRAITS uniquement)

drivers/
├── libsql/
│   ├── mod.rs
│   ├── db.rs               (impl Db for LibsqlDb)
│   ├── row.rs              (impl RowView for LibsqlRow + impl DeserializeRow pour les 11 structs)
│   ├── params.rs           (impl IntoParams + ToDbValue)
│   ├── retry.rs            (HranaRetryPolicy)
│   ├── transaction.rs
│   └── repositories/
│       ├── personne.rs     (impl PersonneRepository for LibsqlPersonneRepository)
│       ├── activite.rs
│       ├── adhesion.rs
│       ├── planning.rs
│       └── parametre.rs
└── postgres/   (futur change, dossier vide avec README.md)
```

`repositories/mod.rs` ne réexporte plus que les traits. Les `Libsql*Repository`
proviennent de `drivers::libsql::repositories::*`.

> **Écart d'implémentation (PR 2, tâche 2.3)** : les fichiers sont déplacés
> **en bloc** (trait + `Libsql*Repository` + `DeserializeRow` + tests) vers
> `drivers/libsql/repositories/`, renommés sans suffixe `_repo`
> (`personne.rs`, `activite.rs`, …). Conséquence : les traits sont définis
> physiquement dans `drivers::libsql::repositories` et **réexportés** par
> `repositories/mod.rs` (qui ne réexporte effectivement que les traits). Les
> `Libsql*Repository` sont réexportés par `drivers::libsql::repositories`
> directement. Aucun appelant (services, commands, e2e) n'importe plus les
> modules `*_repo` internes — ils consomment `crate::repositories::{Trait}` ou
> `crate::drivers::libsql::repositories::{Libsql*Repository}`.

**Alternative écartée** : laisser `repositories/*.rs` mais avec `&dyn Db`
au lieu de `&Connection` — minimiserait le déplacement mais entretiendrait
la confusion driver ↔ domaine. Refusé.

### D7 — Migrations en Rust via `refinery`

État de l'art (août 2025) :
- `refinery` 0.8.14, dernière stable, supporte `tokio-postgres`,
  `mysql_async`, `rusqlite`, `tiberius`. **Pas de support natif
  `libsql` dans les features** — à confirmer en spike avant PR 3
  (cf. R1).
- Si `libsql` n'est pas supporté : on garde le runner actuel
  `cadence_migrations` mais on le rebranche dans `refinery::async_migration`
  via `MigrateAsync` (refinery permet un runner custom ; le tracker
  `_cadence_migrations` est conservé).

Structure cible :

```
migrations/
├── Cargo.toml         (crate optionnelle, ou module Rust via include_str!)
├── migrations.toml    (désactivation explicite pour postgres/mysql tant qu'inactifs)
├── sqlite/
│   ├── V001__create_personnes_physiques.sql
│   ├── V002__create_adhesions.sql
│   └── …
├── postgres/          (vide pour l'instant, README "futur change")
└── mysql/             (vide pour l'instant, README "futur change")
```

Le runner `cadence_migrations` (PR 1) reste basé sur `&dyn Db` ; PR 3 le
remplace par refinery.

### D8 — Couper le test mock `e2e_stream` du runner

`e2e_stream.rs` teste le comportement **bas-niveau du driver** (reset du
stream Hrana après un curseur abandonné), pas l'abstraction. *Décision PR 2
(validée) : inchangé*, il reste sur `libsql::Connection` brute et ne passe
ni par `LibsqlDb` ni par `Arc<dyn Db>`. Pas de régression de valeur du test.

### D9 — Stratégie RETURNING : repo-spécifique, jamais imposée par Db

`Db::execute` + `Db::fetch_one` ne savent pas si une requête fait un
INSERT ou un SELECT. Le repository compose la stratégie :

- **libsql / postgres** (RETURNING natif) :
  `db.fetch_one("INSERT … RETURNING …", params)` → 1 round-trip.
- **MySQL** (futur) : `db.execute("INSERT …", params)` puis
  `db.fetch_one("SELECT … WHERE id = ?", params)` → 2 round-trips.

Le trait `Db` ne fournit pas de méthode `insert_returning`. C'est une
contrainte assumée : on garde une API minimale, on reporte la stratégie
au repository, qui peut choisir librement. *Décision compatible avec
le résultat de la discussion précédente (RETURNS spécifique par driver).*

### D10 — Compatibilité ascendante pour la CI

- Les tests existants `e2e_mono.rs`, `e2e_multi.rs`, `e2e_stream.rs`
  doivent continuer à passer **sans modification du contrat externe**.
- Le `set_var("RUST_MIN_STACK", 512 MiB)` reste en place.
- Le thread dédié « cadence-db » reste en place.
- Les ignore `deny.toml` / `.cargo/audit.toml` pour rustls restent en
  place tant que libsql 0.9.x est utilisé.

## Risks / Trade-offs

### R1 — Compat `refinery` ↔ `libsql` (⚠ bloquant pour PR 3)

- État de `refinery` 0.8.14 : features officielles `tokio-postgres`,
  `mysql_async`, `rusqlite`, `tiberius`. Pas de mention explicite de
  `libsql` dans la doc.
- **Mitigation** : PR 1 et PR 2 n'utilisent pas refinery. PR 3 inclut
  un **spike de validation** : tester `refinery::MigrateAsync` sur une
  `libsql::Connection`. Si non supporté, on garde le runner actuel
  factorisé derrière `&dyn Db` (driver-agnostique) avec une variante
  `RefineryDriver::migrations(SqliteMigrations)`. Le coût marginal est
  nul pour cette PR.
- Prérequis de cette mitigation : `cadence_migrations` repose sur
  `execute_batch` (migrations.rs:59,73). Le trait `Db` expose donc
  `execute_batch` dès PR 1 (D1), sinon le runner ne peut pas rester
  derrière `&dyn Db`.
- Action concrète : programmer le spike avant la PR 3.

### R2 — Performance `dyn Db` vs `libsql::Connection` direct

- Indirection `dyn` ajoute 1-2 ns par appel. Sur les commandes Tauri
  (latence IPC ~ms), c'est négligeable.
- À mesurer après la PR 1 sur la suite de tests existante pour
  confirmer.

### R3 — `libsql::Value` dans `personne_repo.rs:191-243`

- La recherche paginée dynamique utilise `Vec<libsql::Value>`. Refactor
  obligatoire : passer par `Vec<DbValue>`, qui sérialise vers
  `libsql::Value` aujourd'hui et `tokio_postgres::types::ToSql` demain.
- Coût estimé : ~80 lignes touchées, mécanique.

### R4 — Erreurs `AppError::Database` masque le type réel

- Aujourd'hui `From<libsql::Error>` met tout dans `Database`. Si on
  ajoute Postgres / MySQL, leurs erreurs sont sémantiquement
  différentes (`serialization_failure`, `deadlock`). On gagnera à
  exposer une variante `AppError::DatabaseKind(DatabaseErrorKind)`
  (refinery propose un enum similaire).
- Action : documenter dans la PR 3 (refinery) mais reporter
  l'enrichissement à une PR dédiée. Hors scope stricte de ce change.

### R5 — Cycle complet de feature flags

- `tokio-postgres` et `mysql_async` ne sont **pas** ajoutés dans cette
  PR. Veiller à ce qu'aucune dépendance indirecte ne les embarque
  (`sqlx-macros` etc.). Vérification : `cargo tree` après PR 1.
- Les dossiers `drivers/postgres/` et `drivers/mysql/` peuvent être
  créés avec un `README.md` placeholder ; aucun `.rs` actif.

### R6 — `Box<dyn DbTransaction>` empêche l'inlining

- Accepter une indirection boîte. Le coût est négligeable sur le
  chemin chaud (transaction = utilisée sur `modifier_plage_horaire`,
  appelé via IPC donc déjà ~ms).

### R7 — `DeserializeRow` par struct : 11 structs × 15 lignes = ~165 L mécaniques

- Scriptable (grep + replace). Aucun risque métier ; testé via la
  régression des tests existants.

### R8 — Risque de régression E2E

- PR 1 : zéro changement de comportement attendu (introductions
  de traits neutres + impl par défaut qui appelle libsql). Tests
  doivent passer sans modif.
- PR 2 : refactor des signatures services + repositories. Tests
  services/commandes doivent passer ; tests repository doivent
  être réécrits (mécanique, libsql::params! → params!).
- PR 3 : adoption refinery ; risque modéré → exiger
  `e2e_mono` ET `e2e_multi` verts avant merge.

### R9 — Effort total & batch size

- Volumétrie cible : ~1 010 lignes ajoutées, ~360 supprimées (cf. plan
  d'engagement précédent). Soit ~1 400 lignes nettes, **en dessous de
  la cible 2 200** posée dans l'analyse initiale (les drivers à venir
  consommeront la différence).
- Ce chiffrage était établi sans le périmètre `commands/*.rs` (angle mort
  découvert en relecture, cf. Context). L'ajout de `commands/*.rs` en PR 2
  (~150 lignes de plus) conserve un total sous la cible.
- 3 PRs : 1 par étape majeure (cf. découpage validé). Risque cumulé
  acceptable.

### R10 — Pas de garantie que la base soit à l'abri d'un lock pendant migration

- Le runner actuel (`cadence_migrations`) est appelé au boot, sous le
  thread BDD 512 MiB. C'est un single-connection check + execute_batch
  séquentiel. Si refinery impose un autre modèle, vérifier la
  compatibilité. *À investiguer dans le spike de R1.*

### R11 — Non-atomicité de `cadence_migrations` (préexistant, non bloquant)

- `cadence_migrations` exécute `execute_batch("BEGIN; …; COMMIT;")` puis,
  dans un **appel séparé**, l'INSERT du tracker (migrations.rs:72-79). Un
  crash entre les deux rejoue la migration au prochain boot : inoffensif
  pour les `CREATE TABLE IF NOT EXISTS`, mais pas forcément pour les
  `ALTER TABLE` (ex. `add_audit.sql`).
- Préexistant, indépendant du driver. PR 3 (refinery) est le bon moment
  pour le traiter : écrire le tracker dans la même transaction que la
  migration.

## Migration Plan

> **Décisions PR 2 (validées)** :
> - **Retry internalisé dans `LibsqlDb`** (et non porté par les repos comme
>   le sketch D4) : `impl Db for LibsqlDb` applique `HranaRetryPolicy` sur
>   `execute` / `fetch_*_row` / `execute_batch` ; les repos appellent
>   simplement `db.fetch_all(…)`. Le grain requête-only est respecté.
> - **`DbTransactionExt` créé en PR 2** : symétrique de `DbExt`
>   (`fetch_one<T>` / `fetch_optional<T>` / `fetch_all<T>`) pour
>   `&mut dyn DbTransaction`, consommé par les 16 méthodes `_tx` des repos.
> - **`e2e_stream.rs` reste sur libsql brut** (D8, cf. ci-dessus).
> - **Déplacement D6 en fin de PR 2** : refactor fonctionnel
>   (`conn → Arc<dyn Db>`, `params!`, `_tx` → `dyn DbTransaction`) d'abord,
>   puis déplacement des impls vers `drivers/libsql/repositories/`.

Trois PRs séquentielles.

### PR 1 — Pose des abstractions (zéro changement de comportement)

- Création de `infrastructure/db/{db, params, row, transaction}.rs`.
  ~400 lignes nouvelles, ~50 supprimées.
- Création de `infrastructure/retry.rs`. ~60 lignes.
- Déplacement de `hrana_guard.rs` → `drivers/libsql/retry.rs`
  (implémente `RetryPolicy`). ~80 lignes déplacées.
- Extension de `ConnexionConfig` (D5). ~20 lignes.
- Tests unitaires pour chaque trait. ~200 lignes.
- Critère : aucune signature publique de command/service/repo ne
  change. `cargo test` reste vert sans modification des tests existants.

### PR 2 — Refactor repositories + services derrière `dyn Db`

- Création de `drivers/libsql/db.rs` : `LibsqlDb` implémente `Db` (retry
  `HranaRetryPolicy` internalisé) + conversions `DbParams`/`DbRow`.
  ~250 lignes nouvelles.
- Création de `DbTransactionExt` (`db/transaction.rs`). ~60 lignes.
- Déplacement des `*_repo.rs` vers `drivers/libsql/repositories/`
  (**en fin de PR 2**, après refactor fonctionnel validé).
- `PersonneService`, `ActiviteService`, `ParametreService` prennent
  `&dyn Db` au lieu de `Connection` ; `begin_immediate()` remplace
  `transaction_with_behavior(Immediate)` (`activite_service.rs:195-198`),
  `begin()` remplace `transaction()` (`parametre_service.rs:300`).
- Macro `params![…]` remplace `libsql::params![…]` sur tous les sites
  (sauf tests e2e) ; helpers dynamiques `Vec<libsql::Value>` →
  `Vec<DbValue>` (R3, `personne_repo.rs` recherche paginée).
- Implémentation de `DeserializeRow` pour les ~20 types lus par requête
  (11 domain + helpers : `CompteurRow`, `TotalRow`, `AnneeRow`, …).
- `AppState` contient `Arc<dyn Db>` (au lieu de `Connection` cloné
  × 5) ; le champ `pub conn` disparaît. **Périmètre `commands/*.rs`**
  (angle mort de la version précédente du plan) :
  - `activite_commands.rs` (10 sites) et `parametre_commands.rs`
    (3 sites) : `state.conn.clone()` → `state.db.clone()` ;
  - `planning_commands.rs:27,86,123` :
    `transaction_with_behavior(Immediate)` → `db.begin_immediate()`
    (garantie d'atomicité conservée).
- `e2e_mono.rs` / `e2e_multi.rs` : `init_connection` renvoie
  `Arc<dyn Db>`. **`e2e_stream.rs` inchangé** (libsql brut, D8).
  ~150 lignes touchées.
- Critère : tous les tests existants passent, `cargo clippy -D warnings`
  reste vert.

### PR 3 — Adoption `refinery` (optionnelle, dépend de R1)

- Si spike refinery positif : migration du runner vers refinery.
- Si spike négatif : on rebranche le runner actuel derrière
  `&dyn Db` (sans dépendance refinery) et on documente la limitation.
- Dossier `migrations/{sqlite,postgres,mysql}/` créé. `sqlite/*` contient
  les 9 SQL actuels. `postgres/*` et `mysql/*` = `README.md` placeholder.
- Critère : `e2e_mono` + `e2e_multi` verts. Pas de régression de perf
  mesurée.

### Rollback

- Chaque PR reste compilable et testée de manière indépendante.
- PR 1 est **réversible** par suppression des traits (impact : nul).
- PR 2 peut être annulée en gardant l'API `Connection` exposée par
  `Arc<LibsqlDb>` (le `dyn Db` est rétrocompatible via blanket impl).
- PR 3 a le plus de risque ; si refinery ne s'intègre pas, on reporte
  l'adoption sans bloquer les PRs précédentes.

## Open Questions

- **Spike refinery/libsql** (PR 3) : à faire avant de figer le plan.
  Bloqueur potentiel identifié : pas de feature libsql explicite
  dans refinery 0.8.14. Si le mécanisme `refinery::Migrate` ne sait pas
  utiliser `libsql::Connection`, on construit un runner driver-spécifique
  qui **se présente** comme un `MigrateAsync` pour respecter l'API
  refinery côté sqlite — rendu transparent pour les autres drivers.
- **MySQL : vendor ou canonique ?** Quand le moment viendra : support
  des particularités (ENUM, JSON, AUTO_INCREMENT vs SERIAL) en propre
  MySQL ou via une couche d'adaptation des types dans `DeserializeRow`.
- **Postgres : types avancés (ENUM, ARRAY, JSONB) utiles pour Cadence ?
** Pas identifié à ce jour ; à garder sous le coude.

## Effets sur le change OpenSpec en cours

- Aucune spec n'est cassée. Les scénarios ADDED Requirements de
  `connexion-distance` et `audit-modifications` restent satisfaits.
- `plage-horaire-activite` et `planning-activites` : aucun impact.
- Le front ne voit rien (aucun type TS modifié dans cette PR).
