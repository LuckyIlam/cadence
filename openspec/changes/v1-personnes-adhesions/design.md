## Context

Application de bureau pour gérer les personnes physiques et les adhésions d'une association. Projet neuf, stack Rust/Tauri avec base SQLite locale, interchangeable avec Postgres via SQLx.

## Goals / Non-Goals

**Goals:**
- Application autonome (binaire unique) fonctionnant sur un poste Windows
- Modèle de données extensible pour accueillir futurs rôles (intervenant, etc.)
- Recherche textuelle performante sur nom/prénom
- Interface responsive adaptable à différentes tailles d'écran
- Migrations de base de données versionnées (SQLx migrate)

**Non-Goals:**
- Pas de mode multi-utilisateur / serveur dans cette V1
- Pas de synchronisation cloud
- Pas de module intervenant / représentation légale (V1 = personnes + adhésions)
- Pas d'authentification utilisateur

## Décisions

### 1. SQLx avec SQLite local, interchangeable avec Postgres
**Choix :** SQLx (async, migrations intégrées)
**Alternatives :** Diesel (trop rigide, courbe raide), rusqlite (pas de migration Postgres facile)
**Raison :** Même API pour SQLite et Postgres, requêtes vérifiées à la compile (`query!`), migrations intégrées.

### 2. Architecture clean mais légère (pas d'hexagonal overhead)
**Choix :** Organisation en couches : `domain/` (types purs), `repositories/` (SQL), `commands/` (orchestration Tauri), `infrastructure/` (DB pool, migrations)
**Raison :** Assez structuré pour rester maintenable en grandissant, sans générique ni trait inutile pour une app de cette taille.

### 3. React 19 + Tailwind CSS + Vite en frontend Tauri
**Choix :** React (que l'utilisateur connaît déjà)
**Raison :** Bundle négligeable avec Tauri (webview système), pas de coût d'apprentissage, écosystème mature.

### 4. Deux états pour l'adhésion : adhesion = ligne avec booléen `reglee`
**Choix :** Une table `adhesions` avec `annee_scolaire`, `reglee`, `note_paiement`, pas de machine à états.
**Raison :** Le cycle de vie adhérent a été simplifié à "inscrit ou pas". Les règles métier (relances, expiration) viendront plus tard.

### 5. Responsable légal : `responsable_id` nullable sur `personnes_physiques`
**Choix :** Clé étrangère simple pointant vers une autre ligne de la même table.
**Raison :** Naturel avec le modèle, pas de table dédiée nécessaire. Le responsable peut être adhérent ou non (parent extérieur).

## Risques / Trade-offs

- **SQLite en concurrence ?** Faible risque : mono-utilisateur, pas de contention.
- **Validation mineur à deux endroits (Rust + BDD)** → complexité supplémentaire mais cohérence garantie.
- **SQLx query! en local avec SQLite** → nécessite une base de données présente à la compile. Alternative : `query_as` non macro ou DB virtuelle pour le check compile.
