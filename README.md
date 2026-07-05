# Cadence

Application de bureau pour gérer les adhérents et les activités d'une association.

## Stack

- **Frontend** : React 19 + TypeScript + Tailwind CSS + Vite
- **Backend** : Rust + Tauri 2
- **Base de données** : SQLite (via SQLx, interchangeable avec Postgres)

## Fonctionnalités (V1)

- Gestion des personnes physiques (nom, prénom, date naissance, email, téléphone)
- Lien responsable légal pour les mineurs avec sélection via recherche
- Adhésions annuelles par année scolaire (réglé/en attente, note de paiement)
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
├── src/                    # Frontend React
│   ├── main.tsx
│   ├── App.tsx
│   ├── types.ts
│   ├── pages/
│   │   ├── ListePersonnes.tsx
│   │   ├── DetailPersonne.tsx
│   │   ├── Activites.tsx
│   │   └── DetailActivite.tsx
│   └── components/
│       ├── Nav.tsx
│       ├── PersonneForm.tsx
│       └── AdhesionForm.tsx
├── src-tauri/              # Backend Rust
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── domain/        # Types métier
│   │   ├── repositories/  # Accès BDD (SQLx)
│   │   ├── commands/      # IPC Tauri
│   │   └── infrastructure/ # Pool, migrations
│   └── migrations/        # SQL versionné
├── openspec/               # Spécifications et changements
├── docs/                   # Documentation
├── scripts/                # Scripts utilitaires
├── public/                 # Assets statiques
├── package.json
└── vite.config.ts
```

## Documentation fonctionnelle

→ [docs/fonctionnel/](docs/fonctionnel/README.md) — Documentation destinée aux utilisateurs de l'application (bénévoles, secrétaires, trésoriers).

## Modèle de données

- **personnes_physiques** : identité de base (nom, prénom, date naissance, email, téléphone, responsable_id)
- **adhesions** : adhésion annuelle (personne_id, année scolaire, réglé, note paiement)
