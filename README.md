# Cadence

Application de bureau pour gérer les adhérents et les activités d'une association.

## Stack

- **Frontend** : React 19 + TypeScript + Tailwind CSS + Vite
- **Backend** : Rust + Tauri 2
- **Base de données** : SQLite (via SQLx, interchangeable avec Postgres)
- **IA** : Développement assisté par [OpenCode](https://opencode.ai), [OpenSpec](https://openspec.dev) et [Graphify](https://graphify.net)

## Fonctionnalités

- Gestion des personnes physiques (nom, prénom, date naissance, email, téléphone)
- Lien responsable légal pour les mineurs avec sélection via recherche
- Adhésions annuelles par année scolaire (réglé/en attente, note de paiement)
- Gestion des activités (création, modification, tarifs par année scolaire)
- Inscription des personnes aux activités avec rôles (encadrant / participant)
- Contrôle de capacité maximale par activité
- Recherche textuelle insensible à la casse (nom/prénom)
- Interface responsive

## Prérequis

- Rust 1.96+
- Node.js 22+
- Tauri CLI : `cargo install tauri-cli`

## Développement

```bash
# Lancer l'application en mode développement
cd src-tauri
cargo tauri dev

# Ou depuis la racine
npm run tauri dev
```

## Structure du projet

```
cadence/
├── src/                          # Frontend React
│   ├── main.tsx
│   ├── App.tsx
│   ├── types.ts
│   ├── index.css
│   ├── pages/
│   │   ├── ListePersonnes.tsx
│   │   ├── DetailPersonne.tsx
│   │   ├── Activites.tsx
│   │   └── DetailActivite.tsx
│   └── components/
│       ├── Nav.tsx
│       ├── PersonneForm.tsx
│       └── AdhesionForm.tsx
├── src-tauri/                    # Backend Rust
│   ├── src/
│   │   ├── main.rs / lib.rs
│   │   ├── domain/              # Types métier (personne, adhesion, activite)
│   │   ├── repositories/        # Accès BDD (SQLx)
│   │   ├── commands/            # IPC Tauri
│   │   └── infrastructure/      # Pool, migrations
│   ├── migrations/              # SQL versionné
│   ├── tauri.conf.json
│   └── capabilities/
├── openspec/                     # Spécifications et changements
│   ├── specs/                   # Spécifications par domaine
│   └── changes/archive/         # Historique des changements
├── docs/fonctionnel/             # Documentation utilisateur
├── graphify-out/                 # Graphe de connaissance du code
├── .opencode/                    # Configuration OpenCode
├── .github/                      # CI/CD
├── package.json
└── AGENTS.md
```

## Documentation fonctionnelle

→ [docs/fonctionnel/](docs/fonctionnel/README.md) — Documentation fonctionnelle destinée aux utilisateurs de l'application (bénévoles, secrétaires, trésoriers).
→ [openspec/specs/](openspec/specs/) — Spécifications techniques générées par l'agent IA.
