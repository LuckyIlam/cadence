# Change `db-driver-abstraction` — proposal

## Why

Le passage `sqlx → libsql` a coûté ~3 100 lignes de Rust parce que le type
du driver (`libsql::Connection`, `libsql::Transaction`) a été utilisé comme
contrat commun dans les repositories, services et commandes. L'utilisateur
veut pouvoir ajouter des drivers alternatifs (Postgres, MySQL) à terme sans
rejouer ce coût. L'objectif est de **préparer** cette évolution par une
abstraction neutre, sans ajouter de driver maintenant.

## What Changes

- **Abstraction de l'accès aux données** : introduction de traits neutres
  (`Db`, `DbTransaction`, `IntoParams`, `RowView` / `DeserializeRow`,
  `RetryPolicy`) qui ne dépendent plus de `libsql`. Le seul code connaissant
  `libsql` est déplacé sous `drivers/libsql/`.
- **`commands/` rendu agnostique** : `AppState.pub conn: Connection` devient
  `Arc<dyn Db>` ; les appels directs
  `transaction_with_behavior(Immediate)` remontent dans `Db::begin_immediate()`.
- **Migrations versionnées multi-dialectes** : adoption de `refinery` (avec
  spike de validation préalable), runner actuel conservé si incompatible.
- **`ConnexionConfig` étendu** : enum `Driver` (`Sqlite` actif, `Postgres` /
  `Mysql` désactivés) et `Mode` — sans changement du front.
- **Aucun changement de comportement utilisateur** : pure refactor
  d'infrastructure. Aucun driver supplémentaire n'est ajouté dans ce change.

## Capabilities

### New Capabilities

_Aucune._ Le comportement exposé aux utilisateurs (commands Tauri, types TS,
tests E2E) est préservé à 100 %. Ce change est un refactor d'infrastructure
sans changement de spec — `skip_specs: true` est posé dans `.openspec.yaml`.

### Modified Capabilities

_Aucune._ Les specs existantes (`activites`, `adhesions`, `personnes`,
`planning`, `parametres`, `connexion-distance`, `audit-modifications`) ne
voient aucune exigence changer.

## Impact

- **Code** : `src-tauri/src/` — `infrastructure/{db,hrana_guard,migrations}.rs`,
  5 `repositories/*.rs`, `services/{activite,parametre}_service.rs`,
  `commands/*.rs`, nouveaux traits sous `infrastructure/db/` et
  `infrastructure/retry.rs`.
- **Dépendances** : `refinery` (PR 3, sous réserve du spike) ; aucune
  dépendance `tokio-postgres` / `mysql_async` ajoutée.
- **Tests** : `e2e_mono.rs`, `e2e_multi.rs`, `e2e_stream.rs` doivent passer
  sans modification du contrat externe.
- **Front** : aucun type TS modifié.
