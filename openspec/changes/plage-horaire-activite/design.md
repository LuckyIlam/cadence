# Design — Plage horaire d'ouverture configurable

## Contexte

Le module Planning (change `planning-activites`) permet de créer des créneaux hebdomadaires récurrents par activité et d'afficher un planning hebdomadaire. Deux problèmes :

1. **Création sans borne** : `valider_creneau()` (dans `domain/planning.rs`) ne vérifie que le format `HH:MM`, que début < fin, et que les heures sont dans `00:00–23:59`. On peut créer un créneau à n'importe quelle heure (ex : 21h00–22h00).
2. **Affichage codé en dur** : `PlanningHebdo.tsx` définit `HEURES = Array.from({ length: 13 }, (_, i) => i + 8)` (8h→20h) et `posY()` utilise un décalage `- 8`.

L'utilisateur veut une **configuration globale** de la plage d'ouverture, utilisée à la création et à la consultation.

## Décisions de conception

- **Plage globale unique** : une seule plage (heure ouverture / heure fermeture) appliquée à toutes les activités, stockée en base. Pas de plage par activité (mot-clé « globale »).
- **Valeur par défaut** : `08:00` – `20:00` (comportement actuel conservé).
- **Validation stricte** : un créneau doit être *entièrement compris* dans la plage (début ≥ ouverture et fin ≤ fermeture), validé côté backend (logique métier backend). Le frontend pose des bornes `min`/`max` en aide, mais la source de vérité reste backend.
- **Réduction avec impact contrôlé** : lors d'une réduction de plage, un créneau qui sort de la plage (même partiellement) est **supprimé** s'il n'a pas d'inscrit, ou **déplacé** vers la place libre la plus proche (même jour, durée conservée) s'il a des inscrits. L'utilisateur voit un **aperçu** des impacts avant de confirmer ; toute réduction impactante est **refusée sans confirmation explicite**, et **refusée** si un créneau avec inscrits ne trouve aucune place libre. Suppressions, déplacements et mise à jour de la plage sont appliqués **en transaction**.
- **Page Paramètres** : édition de la plage via une nouvelle page `/parametres`, accessible depuis la navigation.

## Modèle de données

### `parametres`

Table à une seule ligne (id fixé à 1) pour les paramètres globaux de l'application.

```sql
CREATE TABLE parametres (
    id               INTEGER PRIMARY KEY CHECK (id = 1),
    heure_ouverture  TEXT NOT NULL,
    heure_fermeture  TEXT NOT NULL,
    CHECK (heure_ouverture < heure_fermeture)
);

INSERT INTO parametres (id, heure_ouverture, heure_fermeture)
VALUES (1, '08:00', '20:00');
```

- Le `CHECK (heure_ouverture < heure_fermeture)` garantit une plage cohérente au niveau base.
- La contrainte `CHECK (id = 1)` force l'existence d'une seule ligne.

## Architecture

### Backend (Rust / Tauri)

| Couche | Fichier | Contenu |
|---|---|---|
| Domaine | `domain/parametre.rs` | `ParametresPlanning` (struct), `valider_plage_horaire`, `valider_creneau_dans_plage`, `trouver_place_deplacement`, `ImpactAction`, `ImpactCreneau` + tests |
| Domaine | `domain/planning.rs` | `CreneauHorsPlage` (créneau sortant de plage avec nom d'activité et nombre d'inscrits) |
| Repository | `repositories/parametre_repo.rs` | trait `ParametreRepository` + `SqliteParametreRepository` (`obtenir_parametres_planning`, `mettre_a_jour_plage_horaire_tx`) |
| Repository | `repositories/planning_repo.rs` | ajout `lister_creneaux_hors_plage`, `lister_tous_creneaux`, `supprimer_creneau_tx`, `deplacer_creneau_tx` au trait `PlanningRepository` |
| Service | `services/parametre_service.rs` | `ParametreService` : `apercu_impact_plage` (lecture seule), `appliquer_plage` (transaction + confirmation) |
| Commande | `commands/parametre_commands.rs` | `obtenir_parametres_planning`, `apercu_creneaux_hors_plage`, `modifier_plage_horaire` (avec `confirmer_suppression`) |
| Wiring | `infrastructure/db.rs`, `lib.rs`, `domain/mod.rs`, `repositories/mod.rs`, `commands/mod.rs` | enregistrement du repo (`param_repo` dans `AppState`) et des commandes |

**Validation dans `planning_commands.rs`** : dans `ajouter_creneau` et `modifier_creneau`, après `valider_creneau(&input)?`, on appelle un helper privé `valider_creneau_dans_plage_global(&state, &input)` qui lit la plage via `param_repo` puis appelle `valider_creneau_dans_plage`. En cas de hors plage, erreur `AppError::Validation` avec message explicite.

**Réduction de plage (logique croisée `parametre_repo` + `planning_repo`)** : placée dans `ParametreService` (service), pas dans les repositories — cf. graphe de connaissances : les deux repositories n'ont pas de couplage direct. Le service calcule les impacts à partir de `lister_creneaux_hors_plage` (jointure `activites` + `activite_personnes`) et de `lister_tous_creneaux` (créneaux qui restent en place + destinations déjà retenues comme occupées), puis applique le tout dans une transaction via les méthodes `*_tx` du trait. L'occupation pour le déplacement considère les créneaux de la **même activité** pour la même (année scolaire, jour), en excluant le créneau déplacé : cohérent avec la création/modification, deux activités distinctes peuvent légitimement partager le même créneau (Option C).

**Contrôle adhérent à l'état final** : un adhérent d'un créneau déplacé peut être inscrit à une autre activité qui occupe le nouvel horaire. Le service vérifie donc, pour chaque déplacement, l'emploi du temps final de chaque adhérent du créneau (positions finales : créneaux restants + créneaux déplacés, créneaux supprimés retirés) : si un adhérent a une autre activité (même année) dont un créneau (même jour) chevauche le nouvel horaire, le déplacement est marqué `DeplaceImpossible` avec une `raison` nommant l'activité en conflit, et la réduction est bloquée. Ce contrôle couvre aussi deux créneaux déplacés d'activités différentes suivis par le même adhérent.

### Frontend (React / TS)

| Fichier | Rôle |
|---|---|
| `types.ts` | interface `ParametresPlanning { id, heure_ouverture, heure_fermeture }` |
| `components/PlanningHebdo.tsx` | nouvelle prop `plageHoraire` ; génération dynamique des heures entre ouverture et fermeture ; `posY` recalé sur l'ouverture |
| `pages/PlanningPage.tsx`, `pages/DetailPersonne.tsx` | chargent la config (`obtenir_parametres_planning`) et la passent à `PlanningHebdo` |
| `pages/DetailActivite.tsx` | borne `min`/`max` des `<input type="time">` + aide indiquant la plage |
| `pages/ParametresPage.tsx` | édition de la plage via `modifier_plage_horaire` |
| `App.tsx`, `components/Nav.tsx` | route `/parametres` + lien « Paramètres » |

## Règles de validation

- `valider_plage_horaire(ouverture, fermeture)` : formats `HH:MM` valides et `ouverture < fermeture`.
- `valider_creneau_dans_plage(creneau, ouverture, fermeture)` : `creneau.heure_debut >= ouverture` et `creneau.heure_fin <= fermeture`.

## Non-Goals

- Pas de plage horaire par activité (config globale uniquement)
- Pas de contrainte de chevauchement de plages (il n'y en a qu'une)
- Pas de déplacement d'un créneau vers un autre jour (même jour uniquement)
- Pas d'export/ICS de la plage

## Risques

- **Migration sur base existante** : la migration est idempotente (`CREATE TABLE IF NOT EXISTS` + `INSERT`). Sur une base existante, la table n'existe pas encore → créée avec la valeur par défaut. Si la table existait déjà (cas improbable), l'`INSERT` avec `id = 1` échouerait → utilisé `INSERT` simple (pas `OR REPLACE`) pour préserver une éventuelle configuration existante.
- **Créneaux existants hors plage** : traités lors d'une réduction via l'aperçu (suppression/déplacement), en transaction. Si un créneau avec inscrits ne peut pas être déplacé, la réduction est bloquée.
- **Grille trop grande** : si l'utilisateur configure une plage très large (ex : 00:00–23:00), la grille devient haute. Comportement attendu ; les blocs restent positionnés correctement.
