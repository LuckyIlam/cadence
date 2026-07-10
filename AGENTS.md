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

## Gestion des erreurs

- **Jamais de `.expect()` ni `.unwrap()` dans le code de production** — uniquement dans les tests
- Utiliser l'opérateur `?` pour propager les erreurs
- Pour les erreurs fatales au démarrage (dans `setup()`), écrire un fichier `cadence_crash.log` via `write_crash_log()` défini dans `lib.rs` avant de panic
- Tenir compte de `windows_subsystem = "windows"` (pas de console visible en release) : toujours enregistrer les erreurs fatales dans un fichier
- `write_crash_log()` écrit dans deux emplacements : le répertoire courant et `%TEMP%`
- Ne jamais laisser une erreur utilisateur sans message ou log exploitable

## Couverture de code

```powershell
# Installer (une fois) : cargo install cargo-llvm-cov --locked
cargo llvm-cov --html  # dans src-tauri/
```

## Documentations

→ [docs/fonctionnel/](docs/fonctionnel/README.md) — Documentation fonctionnelle destinée aux utilisateurs de l'application (bénévoles, secrétaires, trésoriers).
→ [openspec/specs/](openspec/specs/) - Specification technique rédiger par l'agent AI.

## Organisation de l'équipe

Les skills sont organisés en deux catégories : les **outils** (skills openspec*) qui produisent des documents, et les **rôles** qui orchestrent ces outils.

### Workflow

```
Idée ──▶ Architecte ──▶ PM ──▶ Concepteur Tech ──▶ Développeur ──▶ Réviseur ──▶ Livraison
                                │                     │              │
                                └─────────┬───────────┘              │
                                          ▼                         │
                                    Documentaliste ◀────────────────┘
```

### Rôles

| Rôle | Skill utilisé | Produit |
|------|---------------|---------|
| `architecte` | `openspec-explore` | `design.md` (goals, décisions, trade-offs) |
| `product-manager` | `openspec-propose` + `openspec-sync-specs` | `proposal.md`, `specs/*/spec.md`, `tasks.md` |
| `concepteur-technique` | — | Plan technique détaillé (migrations, interfaces, découpage) |
| `developpeur` | `openspec-apply-change` | Code (Rust backend + React frontend), tests |
| `reviseur` | — | Revue qualité + cybersécurité |
| `documentaliste` | `openspec-sync-specs` | `docs/fonctionnel/`, specs synchronisées |

### Principe

- Les rôles en amont (Architecte, PM) définissent *quoi* construire
- Le Concepteur Technique définit *comment* le construire
- Le Développeur exécute le plan
- Le Réviseur valide la qualité + sécurité
- Le Documentaliste maintient la connaissance à jour
- Chaque rôle livre un document ou un artefact au rôle suivant