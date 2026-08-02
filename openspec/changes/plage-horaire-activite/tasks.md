## Ordre d'implémentation

Les tâches sont dans l'ordre de dépendance. Ne pas commencer une section sans avoir terminé la précédente.

---

## 1. Base de données

### 1.1 Créer la migration SQL pour la table `parametres`

**Fichier :** `src-tauri/migrations/20260802000001_create_parametres.sql`

```sql
CREATE TABLE IF NOT EXISTS parametres (
    id               INTEGER PRIMARY KEY CHECK (id = 1),
    heure_ouverture  TEXT NOT NULL,
    heure_fermeture  TEXT NOT NULL,
    CHECK (heure_ouverture < heure_fermeture)
);

INSERT INTO parametres (id, heure_ouverture, heure_fermeture)
VALUES (1, '08:00', '20:00');
```

---

## 2. Backend — Domaine

### 2.1 Créer le module `domain/parametre.rs`

**Fichier :** `src-tauri/src/domain/parametre.rs`

- Struct `ParametresPlanning` (Serialize, Deserialize, sqlx::FromRow) : `id: i64`, `heure_ouverture: String`, `heure_fermeture: String`
- `valider_plage_horaire(heure_ouverture: &str, heure_fermeture: &str) -> Result<(), String>` : réutilise `valider_heure`, vérifie `ouverture < fermeture`
- `valider_creneau_dans_plage(creneau: &CreateCreneau, heure_ouverture: &str, heure_fermeture: &str) -> Result<(), String>` : `debut >= ouverture` et `fin <= fermeture`

### 2.2 Enregistrer le module dans `domain/mod.rs`

Ajouter `pub mod parametre;`.

### 2.3 Tests unitaires (obligatoires)

- plage valide / ouverture après fermeture / heures égales / format invalide
- créneau dans la plage / début avant ouverture / fin après fermeture

---

## 3. Backend — Repository

### 3.1 Créer `repositories/parametre_repo.rs`

- trait `ParametreRepository` (async_trait, Send + Sync) :
  - `obtenir_parametres_planning(&self) -> Result<ParametresPlanning, AppError>`
  - `mettre_a_jour_plage_horaire(&self, heure_ouverture: &str, heure_fermeture: &str) -> Result<ParametresPlanning, AppError>`
- impl `SqliteParametreRepository` avec `SqlitePool`

### 3.2 Enregistrer dans `repositories/mod.rs`

`pub mod parametre_repo;` + re-export `ParametreRepository, SqliteParametreRepository`.

### 3.3 Tests repository

- lecture des valeurs par défaut (`08:00` / `20:00`)
- mise à jour persistante (relecture depuis la base)

---

## 4. Backend — Commandes + wiring

### 4.1 Créer `commands/parametre_commands.rs`

- `obtenir_parametres_planning(state) -> Result<ParametresPlanning, AppError>`
- `modifier_plage_horaire(state, heure_ouverture: String, heure_fermeture: String) -> Result<ParametresPlanning, AppError>` (valide via `valider_plage_horaire` avant mise à jour)

### 4.2 Enregistrer dans `commands/mod.rs` et `lib.rs`

- `pub mod parametre_commands;`
- commandes ajoutées au `generate_handler!`

### 4.3 Wire le repository dans `infrastructure/db.rs`

- champ `param_repo: SqliteParametreRepository` dans `AppState`
- initialisation dans `init_app_state`

### 4.4 Tests commandes

- obtenir les paramètres par défaut
- modifier la plage avec succès (persistance)
- modifier une plage invalide (erreur `AppError::Validation`)

---

## 5. Backend — Validation des créneaux

### 5.1 Appliquer la plage dans `planning_commands.rs`

- helper privé `valider_creneau_dans_plage_global(state, input)` qui lit la plage via `param_repo` et appelle `valider_creneau_dans_plage`
- appel dans `ajouter_creneau` et `modifier_creneau` juste après `valider_creneau(&input)?`

### 5.2 Tests

- créneau avant l'ouverture refusé
- créneau après la fermeture refusé
- créneau aux bornes exactes de la plage accepté

---

## 6. Frontend — Types et composants

### 6.1 `src/types.ts`

Ajouter l'interface `ParametresPlanning` :

```ts
export interface ParametresPlanning {
  id: number;
  heure_ouverture: string;
  heure_fermeture: string;
}
```

### 6.2 `src/components/PlanningHebdo.tsx`

- nouvelle prop obligatoire `plageHoraire: ParametresPlanning`
- génération dynamique de la grille : heures entre `heure_ouverture` et `heure_fermeture`
- `posY` recalé sur l'heure d'ouverture (remplace le décalage `- 8`)
- hauteur totale de la grille basée sur la durée de la plage

### 6.3 `src/pages/PlanningPage.tsx` et `src/pages/DetailPersonne.tsx`

- state `plageHoraire`, chargement via `invoke("obtenir_parametres_planning")`
- passage de `plageHoraire` à `PlanningHebdo` (rendu conditionnel pendant le chargement)

### 6.4 `src/pages/DetailActivite.tsx`

- state `plageHoraire`, chargement via `invoke("obtenir_parametres_planning")`
- `min`/`max` sur les `<input type="time">` du formulaire créneau
- aide affichant la plage (ex : « Plage horaire d'ouverture de l'activité : 08:00 – 20:00 »)

---

## 7. Frontend — Page Paramètres

### 7.1 `src/pages/ParametresPage.tsx`

- charge les paramètres, deux champs `time` (ouverture / fermeture), bouton Enregistrer
- appel `invoke("modifier_plage_horaire", { heureOuverture, heureFermeture })`
- affichage message succès / erreur

### 7.2 `src/App.tsx` et `src/components/Nav.tsx`

- route `/parametres` → `<ParametresPage />`
- lien « Paramètres » dans la navigation

---

## 8. Vérifications (obligatoires)

Dans `src-tauri/` :

1. `cargo check`
2. `cargo clippy -- -D warnings`
3. `cargo fmt --check`
4. `cargo test`

À la racine :

5. `npm run typecheck`
6. `npm run lint`
7. `npm run build`

Et :

8. `cargo audit`
9. `cargo deny check`
10. `graphify update .`
