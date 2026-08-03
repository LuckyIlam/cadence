## Why

Actuellement, on peut créer un créneau d'activité à n'importe quelle heure de la journée, mais l'affichage d'un planning d'activité se limite aux heures comprises entre 8h et 20h (plage codée en dur dans le composant `PlanningHebdo`). Cette incohérence empêche de créer des créneaux le soir (ex : 20h30–21h30) alors qu'ils ne pourraient jamais s'afficher, et rend impossible d'ajuster la plage affichée aux horaires réels de l'association.

Cette feature introduit une **configuration globale** de la plage horaire d'ouverture des activités (heure d'ouverture / heure de fermeture), utilisée à la fois :
- à la **création / modification** d'un créneau (un créneau doit être entièrement compris dans la plage),
- à la **consultation** d'un planning (la grille hebdomadaire s'affiche entre l'ouverture et la fermeture configurées).

## What Changes

- Nouvelle table `parametres` avec une ligne unique (id = 1) stockant `heure_ouverture` et `heure_fermeture`, initialisée par défaut à `08:00` / `20:00`
- Nouveau module domaine `parametre` avec `valider_plage_horaire`, `valider_creneau_dans_plage`, `trouver_place_deplacement`, `ImpactAction` / `ImpactCreneau` (tests unitaires)
- Nouveau repository `parametre_repo` et nouvelles commandes Tauri `obtenir_parametres_planning` / `modifier_plage_horaire`
- Validation backend dans `ajouter_creneau` et `modifier_creneau` : le créneau doit être entièrement compris dans la plage configurée
- **Gestion de la réduction de plage** : service `ParametreService` (`apercu_impact_plage`, `appliquer_plage`), nouvelle commande `apercu_creneaux_hors_plage`, aperçu des créneaux impactés dans la page Paramètres avec confirmation obligatoire ; créneaux sans inscrit supprimés, créneaux avec inscrits déplacés au plus proche (même jour), réduction bloquée si aucun déplacement possible, le tout en transaction
- Composant `PlanningHebdo` : grille générée dynamiquement entre l'ouverture et la fermeture configurées (suppression du `8h–20h` en dur)
- Pages `PlanningPage` et `DetailPersonne` : chargement de la config et passage à `PlanningHebdo`
- Page `DetailActivite` : bornes `min`/`max` sur les champs heure et aide indiquant la plage
- Nouvelle page **Paramètres** (`/parametres`) pour éditer la plage, avec lien de navigation

## Capabilities

### New Capabilities
- `parametres`: Configuration globale de la plage horaire d'ouverture des activités (ouverture/fermeture), consultée et modifiable, avec gestion des créneaux impactés lors d'une réduction (aperçu, confirmation, suppression/déplacement en transaction)

### Modified Capabilities
- `planning`: la création/modification de créneaux est bornée à la plage configurée, et la grille hebdomadaire s'affiche sur cette plage

## Impact

- Backend : nouveau module domain (`parametre.rs`, `CreneauHorsPlage` dans `planning.rs`), nouveau repository (`parametre_repo.rs`), nouveau service (`parametre_service.rs`), nouvelles commandes (`parametre_commands.rs`), enrichissement du trait `PlanningRepository` (hors plage + tx), migration SQL (1 table), `AppState` enrichi (`param_repo`)
- Frontend : nouveau composant page `ParametresPage`, route `/parametres`, lien nav, modifs `PlanningHebdo`, `PlanningPage`, `DetailPersonne`, `DetailActivite`, types `ParametresPlanning` / `ImpactCreneau` / `ImpactAction`
- Les créneaux existants hors plage sont gérés lors d'une réduction (suppression ou déplacement après confirmation) ; plus de rétrocompatibilité « sans modification »
