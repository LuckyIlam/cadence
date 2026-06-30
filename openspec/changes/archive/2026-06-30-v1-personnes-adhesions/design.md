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

### 6. Validation date de naissance : <= 1920 et pas dans le futur
**Choix :** Validation côté Rust (dans les commands) + côté frontend (avant soumission)
**Raison :** Double validation pour UX immédiate (frontend) et sécurité (backend). Une personne née avant 1920 aurait plus de 106 ans aujourd'hui — incohérent avec le contexte associatif.

### 7. Format d'affichage des dates : JJ/MM/AAAA
**Choix :** Format français explicite via une fonction utilitaire `formatDate()` dans le frontend. Stockage en ISO (AAAA-MM-JJ) dans SQLite.
**Raison :** ISO pour le stockage et les échanges (standard), JJ/MM/AAAA pour l'affichage utilisateur. Pas de dépendance à la locale du système.

### 8. Désactivation du bouton d'ajout d'adhésion si l'année en cours existe déjà
**Choix :** Vérification côté frontend : comparaison de l'année scolaire courante avec la liste des adhésions. Infobulle au survol.
**Raison :** UX plus propre que de laisser l'utilisateur cliquer et recevoir une erreur. La contrainte UNIQUE en BDD reste la file de sécurité.

## Risques / Trade-offs

- **SQLite en concurrence ?** Faible risque : mono-utilisateur, pas de contention.
- **Validation mineur à deux endroits (Rust + BDD)** → complexité supplémentaire mais cohérence garantie.
- **SQLx query! en local avec SQLite** → nécessite une base de données présente à la compile. Alternative : `query_as` non macro ou DB virtuelle pour le check compile.
- **Validation date frontend + backend** → redondance volontaire pour UX fluide sans aller-retour.
