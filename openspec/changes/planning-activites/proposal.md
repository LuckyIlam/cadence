## Why

Les activités existent dans Cadence comme des entités statiques (nom, tarif, participants) sans dimension temporelle. Les encadrants ne peuvent pas consulter leur emploi du semaine, les adhérents ne savent pas quand ont lieu leurs activités, et l'association ne peut pas publier un planning hebdomadaire ni signaler les semaines banalisées (vacances, ponts). Cette feature ajoute la gestion des créneaux horaires récurrents, des semaines banalisées, et des vues planning par personne et par activité, avec détection de collisions.

## What Changes

- Nouveau module **Planning** avec créneaux hebdomadaires récurrents par activité
- Gestion des semaines banalisées (exclusions) par activité
- Vue planning hebdomadaire (lundi→dimanche) pour un encadrant ou un adhérent
- Détection de collisions horaires à l'inscription d'une personne à une activité
- Sections créneaux et semaines banalisées dans le détail d'une activité
- Section planning dans le détail d'une personne
- Nouvelle page `/planning` avec navigation par semaine

## Capabilities

### New Capabilities
- `planning`: Gestion des créneaux hebdomadaires, semaines banalisées, vues planning personne et activité, détection de collisions

### Modified Capabilities
<!-- Aucun changement de requirement au niveau spec existant — les modifications sont d'implémentation (ajout de sections dans les pages existantes) -->

## Impact

- Backend : nouveau module domain (`creneau.rs`, `semaine_banalisee.rs`), nouveau repository (`planning_repo.rs`), nouvelles commandes Tauri (`planning_commands.rs`), migration SQL (2 tables)
- Frontend : nouveau composant `PlanningHebdo`, nouvelle page `PlanningPage`, sections dans `DetailActivite` et `DetailPersonne`, nouveau lien de navigation
- Modification de `activite_commands::ajouter_personne_activite` pour intégrer la vérification de collision
