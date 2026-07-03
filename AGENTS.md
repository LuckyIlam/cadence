# Règles de développement

## Architecture

- Logique métier côté backend (Rust, commandes Tauri)
- Le frontend ne fait qu'afficher et appeler les commandes
- Créer des types spécifiques selon l'usage (ex: `PersonneDetail`) plutôt que surcharger les types existants

## Tests

- Tests unitaires obligatoires pour toute nouvelle fonction métier (Rust)
- Exécuter `cargo test` avant toute soumission

## Stack

- Rust + Tauri 2 + React 19 + TypeScript + Tailwind CSS
- SQLite via SQLx
- Vite pour le build frontend

## Vérifications

Toujours exécuter avant de proposer le travail :

1. `cargo check` (dans `src-tauri/`)
2. `cargo clippy -- -D warnings`
3. `cargo fmt --check`
4. `cargo audit`
5. `cargo deny check`
6. `npm run typecheck`
7. `npm run lint`
8. `npm run build`

## Couverture de code

```powershell
# Installer (une fois) : cargo install cargo-llvm-cov --locked
cargo llvm-cov --html  # dans src-tauri/
```
