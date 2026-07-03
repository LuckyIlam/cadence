## Context

Extension de Cadence V1 (personnes + adhésions) avec un module activités pour les associations.

## Goals / Non-Goals

**Goals:**
- Module activités complet : CRUD + inscriptions (participants/encadrants) + tarifs par année
- Navigation entre les vues "Personnes" et "Activités"
- Consultation croisée : activités d'une personne, personnes d'une activité
- Filtre par année scolaire sur la liste des activités

**Non-Goals:**
- Pas de gestion des séances / créneaux / planning
- Pas de notion de fréquence (aucune récurrence modélisée)
- Pas de gestion des présences / feuille d'émargement
- Pas de modules intervenant ou représentation légale (toujours V2+, voir roadmap)
- Pas de vérification du statut d'adhésion pour participer (confiance utilisateur)

## Décisions

### 1. Table unique `activite_personnes` avec rôle et année scolaire
**Choix :** Une table unique avec `activite_id`, `personne_id`, `annee_scolaire`, `role` (TEXT CHECK encadrant/participant), UNIQUE(activite_id, personne_id, annee_scolaire).
**Alternatives :** Deux tables distinctes (`activite_encadrants`, `activite_participants`), ou une table sans année scolaire.
**Raison :** L'unicité (activité, personne, année) empêche les doublons naturellement. L'utilisateur a demandé de pouvoir consulter année par année. Une seule table simplifie les requêtes "toutes les activités d'une personne". La contrainte qu'une personne ne peut être à la fois encadrante et participante pour la même activité est vérifiée côté Rust.

### 2. L'utilisateur ne peut pas être à la fois encadrant et participant pour une même activité
**Choix :** Validation côté Rust uniquement (pas de trigger SQL).
**Raison :** Contrainte métier, pas d'intégrité référentielle. KISS — une vérification avant insert/update suffit.

### 3. Tarifs dans une table séparée
**Choix :** Table `tarifs_activite(activite_id, annee_scolaire, tarif)` avec UNIQUE(activite_id, annee_scolaire).
**Alternatives :** Colonne `tarif` directement sur `activites` (perte d'historique), colonne `tarif` nullable avec année (dénormalisé).
**Raison :** L'utilisateur a explicitement demandé l'historique des prix par année scolaire. Table séparée = clean, pas de NULL, requêtes faciles pour "tarif de l'année".

### 4. Capacité maximale optionnelle sur `activites`
**Choix :** Colonne `capacite_max` INTEGER NULLABLE sur `activites`. Vérification côté Rust avant ajout d'un participant.
**Raison :** Simple, optionnel, pas de table ni de complexité supplémentaire.

### 5. Navigation via un menu dans `App.tsx`
**Choix :** Barre de navigation (horizontale ou latérale) avec les entrées "Personnes" et "Activités" utilisant React Router.
**Raison :** L'utilisateur a explicitement demandé un menu. React Router est déjà en place dans `App.tsx` pour gérer les routes.

### 6. Liste des activités filtrée par année scolaire
**Choix :** Par défaut, l'année scolaire courante (calculée comme pour les adhésions). L'utilisateur peut changer l'année via un sélecteur. Le filtre s'applique côté backend via une jointure avec `tarifs_activite`.
**Alternatives :** Filtre côté frontend (toutes les activités chargées puis filtrées).
**Raison :** Cohérent avec le pattern existant (recherche personnes). Une association peut avoir des dizaines d'activités sur plusieurs années, pas de raison de tout charger.

### 7. Architecture en couches inchangée
**Choix :** Même pattern que V1 : `domain/activite.rs` (types purs), `repositories/activite_repo.rs` (SQL), `commands/activite_commands.rs` (orchestration Tauri).
**Raison :** Cohérence avec l'existant.

## Risques / Trade-offs

- **Capacité max et concurrence :** Pas de concurrence (mono-utilisateur), pas de race condition.
- **Validation rôle unique (Rust uniquement) :** Sans trigger SQL, une requête directe pourrait contourner la validation. Acceptable car mono-utilisateur.
- **Pas de lien adhésion → activité :** Si plus tard on veut "activité réservée aux adhérents", il faudra ajouter la vérification. Pour l'instant, KISS.
- **Requêtes année scolaire :** Les jointures avec `tarifs_activite` ajoutent une légère complexité mais restent simples avec SQLx.
