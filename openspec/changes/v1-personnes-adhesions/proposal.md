## Why

Les associations ont besoin d'un outil simple et léger pour gérer leurs adhérents — leurs inscriptions, leurs cotisations, et le suivi des personnes physiques (mineurs, parents, intervenants). Les solutions existantes sont soit trop lourdes (ERP associatif), soit trop chères, soit en ligne alors que beaucoup d'associations locales fonctionnent sur un seul poste.

## What Changes

- Application de bureau autonome (Rust/Tauri) avec base locale SQLite
- Gestion des personnes physiques : nom, prénom, date naissance, email, téléphone
- Gestion du lien mineur → responsable légal (parent/tuteur extérieur)
- Gestion des adhésions annuelles : année scolaire, réglé/pas réglé, note de paiement
- Liste des personnes avec recherche textuelle (nom/prénom, insensible à la casse)
- Consultation détail d'une personne avec ses adhésions
- Migration possible vers Postgres (via SQLx)

## Capabilities

### New Capabilities
- `personnes`: Création, modification, liste et recherche de personnes physiques, avec lien responsable légal pour les mineurs
- `adhesions`: Gestion des adhésions annuelles liées à une personne (année scolaire, statut de paiement, note)

### Modified Capabilities

*None — first version.*

## Impact

Nouveau projet. Stack : Rust, Tauri, SQLx (SQLite en local, interchangeable avec Postgres), React 19 + Tailwind CSS + Vite en frontend. Application de bureau, binaire autonome ~15 Mo.
