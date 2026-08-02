## Ordre d'implémentation

Les tâches sont dans l'ordre de dépendance. Ne pas commencer une section sans avoir terminé la précédente.

---

## 1. Base de données

### 1.1 Créer la migration SQL pour la table `creneaux_activite`

**Fichier :** `src-tauri/migrations/20260710000001_create_creneaux_activite.sql`

```sql
CREATE TABLE IF NOT EXISTS creneaux_activite (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    activite_id     INTEGER NOT NULL,
    jour_semaine    INTEGER NOT NULL CHECK (jour_semaine BETWEEN 1 AND 7),
    heure_debut     TEXT NOT NULL,
    heure_fin       TEXT NOT NULL,
    annee_scolaire  TEXT NOT NULL,
    FOREIGN KEY (activite_id) REFERENCES activites(id),
    CHECK (heure_debut < heure_fin)
);

CREATE INDEX IF NOT EXISTS idx_creneaux_activite_activite ON creneaux_activite(activite_id);
CREATE INDEX IF NOT EXISTS idx_creneaux_activite_annee ON creneaux_activite(annee_scolaire);
```

### 1.2 Créer la migration SQL pour la table `semaines_banalisees`

**Fichier :** `src-tauri/migrations/20260710000002_create_semaines_banalisees.sql`

```sql
CREATE TABLE IF NOT EXISTS semaines_banalisees (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    activite_id     INTEGER NOT NULL,
    date_debut      TEXT NOT NULL,
    motif           TEXT,
    annee_scolaire  TEXT NOT NULL,
    FOREIGN KEY (activite_id) REFERENCES activites(id)
);

CREATE INDEX IF NOT EXISTS idx_semaines_banalisees_activite ON semaines_banalisees(activite_id);
```

---

## 2. Backend — Domaine

### 2.1 Créer le module `domain/planning.rs`

**Fichier :** `src-tauri/src/domain/planning.rs`

Types Rust à définir :

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CreneauActivite {
    pub id: i64,
    pub activite_id: i64,
    pub jour_semaine: i64,       // 1=lundi…7=dimanche
    pub heure_debut: String,     // "HH:MM"
    pub heure_fin: String,       // "HH:MM"
    pub annee_scolaire: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCreneau {
    pub activite_id: i64,
    pub jour_semaine: i64,
    pub heure_debut: String,
    pub heure_fin: String,
    pub annee_scolaire: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SemaineBanalisee {
    pub id: i64,
    pub activite_id: i64,
    pub date_debut: String,      // "AAAA-MM-JJ" (lundi)
    pub motif: Option<String>,
    pub annee_scolaire: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSemaineBanalisee {
    pub activite_id: i64,
    pub date_debut: String,
    pub motif: Option<String>,
    pub annee_scolaire: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningCreneau {
    pub creneau: CreneauActivite,
    pub activite: super::activite::Activite,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collision {
    pub activite_conflit: String,    // nom de l'activité en conflit
    pub jour_semaine: i64,
    pub heure_debut: String,
    pub heure_fin: String,
}
```

### 2.2 Fonctions de validation dans `domain/planning.rs`

```rust
pub fn valider_jour_semaine(jour: i64) -> Result<(), String> {
    match jour {
        1..=7 => Ok(()),
        _ => Err(format!("Jour de semaine invalide : {}. Doit être entre 1 (lundi) et 7 (dimanche)", jour)),
    }
}

pub fn valider_heure(heure: &str) -> Result<(), String> {
    let parts: Vec<&str> = heure.split(':').collect();
    if parts.len() != 2 {
        return Err(format!("Format d'heure invalide : '{}'. Attendu HH:MM", heure));
    }
    let h: u32 = parts[0].parse().map_err(|_| "Heure invalide".to_string())?;
    let m: u32 = parts[1].parse().map_err(|_| "Minutes invalides".to_string())?;
    if h > 23 || m > 59 {
        return Err(format!("Heure invalide : '{}'. Les heures vont de 00:00 à 23:59", heure));
    }
    Ok(())
}

pub fn valider_creneau(input: &CreateCreneau) -> Result<(), String> {
    valider_jour_semaine(input.jour_semaine)?;
    valider_heure(&input.heure_debut)?;
    valider_heure(&input.heure_fin)?;
    if input.heure_debut >= input.heure_fin {
        return Err(format!(
            "L'heure de début ({}) doit être avant l'heure de fin ({})",
            input.heure_debut, input.heure_fin
        ));
    }
    Ok(())
}

pub fn est_lundi(date: &str) -> Result<(), String> {
    // Vérifie que la date est un lundi au format AAAA-MM-JJ
    // On parse la date et on vérifie que le jour de la semaine est lundi (1 en chrono)
    let d = date.split('-').collect::<Vec<_>>();
    if d.len() != 3 {
        return Err(format!("Format de date invalide : '{}'. Attendu AAAA-MM-JJ", date));
    }
    // Utilisation de chrono::NaiveDate::parse_from_str
    // Si on préfère une validation simple sans chrono : on vérifie le format regex
    Ok(())
}
```

Note : la fonction `est_lundi` peut utiliser `chrono::NaiveDate` (déjà présent dans les dépendances via Tauri). Sinon, regex simple : `^\d{4}-\d{2}-\d{2}$`.

### 2.3 Ajouter `pub mod planning` dans `domain/mod.rs`

**Fichier :** `src-tauri/src/domain/mod.rs`

Ajouter : `pub mod planning;`

---

## 3. Backend — Repository

### 3.1 Créer `repositories/planning_repo.rs`

**Fichier :** `src-tauri/src/repositories/planning_repo.rs`

Fonctions à implémenter :

```rust
// CRUD Créneaux
pub async fn creer_creneau(pool: &SqlitePool, input: CreateCreneau) -> Result<CreneauActivite, sqlx::Error>
pub async fn supprimer_creneau(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error>
pub async fn modifier_creneau(pool: &SqlitePool, id: i64, input: CreateCreneau) -> Result<CreneauActivite, sqlx::Error>
pub async fn lister_creneaux(pool: &SqlitePool, activite_id: i64, annee_scolaire: &str) -> Result<Vec<CreneauActivite>, sqlx::Error>

// CRUD Semaines banalisées
pub async fn ajouter_semaine_banalisee(pool: &SqlitePool, input: CreateSemaineBanalisee) -> Result<SemaineBanalisee, sqlx::Error>
pub async fn supprimer_semaine_banalisee(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error>
pub async fn lister_semaines_banalisees(pool: &SqlitePool, activite_id: i64) -> Result<Vec<SemaineBanalisee>, sqlx::Error>

// Comptage pour verrouillage
pub async fn compter_inscrits_activite(pool: &SqlitePool, activite_id: i64, annee_scolaire: &str) -> Result<i64, sqlx::Error>

// Collision
pub async fn verifier_collision(
    pool: &SqlitePool,
    personne_id: i64,
    activite_id: i64,
    annee_scolaire: &str,
) -> Result<Option<Collision>, sqlx::Error>

// Planning personne
pub async fn planning_personne_semaine(
    pool: &SqlitePool,
    personne_id: i64,
    date_lundi: &str,           // AAAA-MM-JJ
    annee_scolaire: &str,
) -> Result<Vec<PlanningCreneau>, sqlx::Error>
```

Détail SQL pour `lister_creneaux` : tri par `jour_semaine` puis `heure_debut`.

Détail SQL pour `verifier_collision` :
1. Récupérer les créneaux de l'activité cible
2. Récupérer TOUTES les activités où la personne est inscrite (via `activite_personnes`)
3. Pour chaque activité, récupérer ses créneaux
4. Chercher un overlap : même `jour_semaine` ET `heure_debut < autre.heure_fin` ET `heure_fin > autre.heure_debut`
5. Si overlap, retourner le premier conflit trouvé

Détail SQL pour `planning_personne_semaine` :
```sql
SELECT DISTINCT a.id, a.nom, a.description, a.capacite_max,
       c.id AS creneau_id, c.jour_semaine, c.heure_debut, c.heure_fin, c.annee_scolaire,
       ap.role
FROM activite_personnes ap
JOIN activites a ON a.id = ap.activite_id
JOIN creneaux_activite c ON c.activite_id = a.id
WHERE ap.personne_id = ?
  AND c.annee_scolaire = ?
  AND ap.annee_scolaire = ?
  AND NOT EXISTS (
    SELECT 1 FROM semaines_banalisees sb
    WHERE sb.activite_id = a.id AND sb.date_debut = ?
  )
ORDER BY c.jour_semaine, c.heure_debut
```

Note : `date_lundi` en paramètre est utilisé pour filtrer les semaines banalisées (comparaison directe avec `sb.date_debut`).

### 3.2 Ajouter `pub mod planning_repo` dans `repositories/mod.rs`

**Fichier :** `src-tauri/src/repositories/mod.rs`

Ajouter : `pub mod planning_repo;`

---

## 4. Backend — Commandes

### 4.1 Créer `commands/planning_commands.rs`

**Fichier :** `src-tauri/src/commands/planning_commands.rs`

```rust
#[tauri::command]
pub async fn ajouter_creneau(
    state: State<'_, AppState>,
    input: CreateCreneau,
) -> Result<CreneauActivite, String>
{
    valider_creneau(&input)?;
    // Vérifier que l'activité existe
    // Si l'activité a déjà des inscrits pour cette année, on peut quand même ajouter (règle 2)
    planning_repo::creer_creneau(&state.pool, input).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn supprimer_creneau(
    state: State<'_, AppState>,
    id: i64,
    activite_id: i64,
    annee_scolaire: String,
) -> Result<(), String>
{
    // Vérifier qu'aucun inscrit → sinon refus
    let nb = planning_repo::compter_inscrits_activite(&state.pool, activite_id, &annee_scolaire).await.map_err(|e| e.to_string())?;
    if nb > 0 {
        return Err("Impossible de supprimer un créneau : des personnes sont inscrites à cette activité pour cette année. Retirez d'abord les inscrits.".to_string());
    }
    planning_repo::supprimer_creneau(&state.pool, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn modifier_creneau(
    state: State<'_, AppState>,
    id: i64,
    input: CreateCreneau,
) -> Result<CreneauActivite, String>
{
    valider_creneau(&input)?;
    // Même vérification que pour supprimer
    let nb = planning_repo::compter_inscrits_activite(&state.pool, input.activite_id, &input.annee_scolaire).await.map_err(|e| e.to_string())?;
    if nb > 0 {
        return Err("Impossible de modifier un créneau : des personnes sont inscrites à cette activité pour cette année. Retirez d'abord les inscrits.".to_string());
    }
    planning_repo::modifier_creneau(&state.pool, id, input).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn lister_creneaux(
    state: State<'_, AppState>,
    activite_id: i64,
    annee_scolaire: String,
) -> Result<Vec<CreneauActivite>, String>
{
    planning_repo::lister_creneaux(&state.pool, activite_id, &annee_scolaire).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ajouter_semaine_banalisee(
    state: State<'_, AppState>,
    input: CreateSemaineBanalisee,
) -> Result<SemaineBanalisee, String>
{
    // Valider que date_debut est un lundi
    // Valider le format de date
    planning_repo::ajouter_semaine_banalisee(&state.pool, input).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn supprimer_semaine_banalisee(
    state: State<'_, AppState>,
    id: i64,
) -> Result<(), String>
{
    planning_repo::supprimer_semaine_banalisee(&state.pool, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn lister_semaines_banalisees(
    state: State<'_, AppState>,
    activite_id: i64,
) -> Result<Vec<SemaineBanalisee>, String>
{
    planning_repo::lister_semaines_banalisees(&state.pool, activite_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn planning_personne(
    state: State<'_, AppState>,
    personne_id: i64,
    date_lundi: String,        // AAAA-MM-JJ
    annee_scolaire: String,
) -> Result<Vec<PlanningCreneau>, String>
{
    planning_repo::planning_personne_semaine(&state.pool, personne_id, &date_lundi, &annee_scolaire).await.map_err(|e| e.to_string())
}
```

### 4.2 Intégrer la vérification de collision dans `activite_commands::ajouter_personne_activite`

**Fichier :** `src-tauri/src/commands/activite_commands.rs`

Ajouter dans `ajouter_personne_activite` après la validation du rôle et avant l'insertion :

```rust
// Vérification des collisions de planning
if let Some(collision) = planning_repo::verifier_collision(
    &state.pool,
    input.personne_id,
    input.activite_id,
    &input.annee_scolaire,
)
.await
.map_err(|e| e.to_string())?
{
    return Err(format!(
        "Conflit d'horaire avec l'activité '{}' : jour {} ({}), {}–{}. La personne est déjà inscrite à cette activité avec ce créneau.",
        collision.activite_conflit,
        collision.jour_semaine,
        jour_semaine_texte(collision.jour_semaine),
        collision.heure_debut,
        collision.heure_fin,
    ));
}
```

### 4.3 Enregistrer les nouvelles commandes dans `lib.rs`

**Fichier :** `src-tauri/src/lib.rs`

Ajouter dans `invoke_handler!` :
```rust
commands::planning_commands::ajouter_creneau,
commands::planning_commands::supprimer_creneau,
commands::planning_commands::modifier_creneau,
commands::planning_commands::lister_creneaux,
commands::planning_commands::ajouter_semaine_banalisee,
commands::planning_commands::supprimer_semaine_banalisee,
commands::planning_commands::lister_semaines_banalisees,
commands::planning_commands::planning_personne,
```

Ajouter en haut : `mod planning_commands;` dans `commands/mod.rs`.

---

## 5. Backend — Tests

### 5.1 Tests unitaires dans `domain/planning.rs`

Tester :
- `valider_jour_semaine` : 1..=7 OK, autres erreur
- `valider_heure` : "14:00" OK, "25:00" erreur, "14" erreur, "abc" erreur
- `valider_creneau` : heure_debut < heure_fin OK, inverse erreur
- `est_lundi` : 2025-09-01 (lundi) OK, 2025-09-02 (mardi) erreur (utiliser chrono::NaiveDate::format("%A"))

### 5.2 Tests d'intégration dans `repositories/planning_repo.rs`

Même pattern que `activite_repo.rs` : in-memory SQLite avec migrations.

Tester :
- CRUD créneaux (créer, lister, supprimer)
- CRUD semaines banalisées
- `compter_inscrits_activite` : 0 inscrit → 0, 1 inscrit → 1
- `verifier_collision` : pas de collision, collision détectée
- `planning_personne_semaine` : planning normal, semaine banalisée exclue

---

## 6. Frontend — Types

### 6.1 Ajouter les types TypeScript dans `src/types.ts`

```typescript
export interface CreneauActivite {
  id: number;
  activite_id: number;
  jour_semaine: number;
  heure_debut: string;
  heure_fin: string;
  annee_scolaire: string;
}

export interface CreateCreneau {
  activite_id: number;
  jour_semaine: number;
  heure_debut: string;
  heure_fin: string;
  annee_scolaire: string;
}

export interface SemaineBanalisee {
  id: number;
  activite_id: number;
  date_debut: string;
  motif: string | null;
  annee_scolaire: string;
}

export interface CreateSemaineBanalisee {
  activite_id: number;
  date_debut: string;
  motif: string | null;
  annee_scolaire: string;
}

export interface PlanningCreneau {
  creneau: CreneauActivite;
  activite: Activite;
  role: string;
}
```

### 6.2 Ajouter les helpers dans `src/types.ts`

```typescript
const JOURS_SEMAIRE = [
  "Lundi", "Mardi", "Mercredi", "Jeudi", "Vendredi", "Samedi", "Dimanche",
] as const;

export function jourSemaineTexte(jour: number): string {
  return JOURS_SEMAIRE[jour - 1] ?? `Jour ${jour}`;
}

export function getNumeroSemaineISO(date: Date): number {
  const d = new Date(Date.UTC(date.getFullYear(), date.getMonth(), date.getDate()));
  const dayNum = d.getUTCDay() || 7;
  d.setUTCDate(d.getUTCDate() + 4 - dayNum);
  const yearStart = new Date(Date.UTC(d.getUTCFullYear(), 0, 1));
  return Math.ceil(((d.getTime() - yearStart.getTime()) / 86400000 + 1) / 7);
}

export function getLundiSemaine(date: Date): Date {
  const d = new Date(date);
  const day = d.getDay();
  const diff = day === 0 ? -6 : 1 - day;
  d.setDate(d.getDate() + diff);
  d.setHours(0, 0, 0, 0);
  return d;
}

export function formatDateISO(date: Date): string {
  return date.toISOString().split("T")[0];
}
```

---

## 7. Frontend — Composant PlanningHebdo

### 7.1 Architecture du composant

**Fichier :** `src/components/PlanningHebdo.tsx`

```typescript
interface PlanningHebdoProps {
  creneaux: PlanningCreneau[];
  dateLundi: Date;
  onSemainePrecedente: () => void;
  onSemaineSuivante: () => void;
}
```

Props :
- `creneaux` : liste des créneaux à afficher pour la semaine
- `dateLundi` : date du lundi de la semaine affichée
- `onSemainePrecedente` / `onSemaineSuivante` : callbacks de navigation

### 7.2 Structure

```
┌─────────────────────────────────────────────────────┐
│  ← Semaine 40 (lun 30 sep 2025)      Suivante →    │
├────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┤
│    │ Lun │ Mar │ Mer │ Jeu │ Ven │ Sam │ Dim │
├────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┤
│8h00│     │     │     │     │     │     │     │
│    │     │     │     │     │     │     │     │
│9h00│     │     │     │     │     │     │     │
│ ...│     │     │     │     │     │     │     │
│20h0│     │     │     │     │     │     │     │
└────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┘
```

- Grille CSS : `grid-template-columns: 60px repeat(7, 1fr)` (colonne heures + 7 jours)
- Lignes d'heures : de 8h00 à 20h00, pas de 1h (13 lignes)
- Blocs créneaux : positionnés avec `grid-row` et `grid-column` basés sur l'heure et le jour
- Hauteur d'une ligne = 60px. Calcul de position : `(heure_debut - 8) * 60px`, hauteur = `(heure_fin - heure_debut) * 60px`

### 7.3 Gestion des états

- **Chargement :** le parent gère le chargement, pas le composant
- **Vide :** si `creneaux.length === 0`, afficher "Aucune activité cette semaine"
- **Erreur :** le parent gère les erreurs d'invoke

---

## 8. Frontend — Page Planning

### 8.1 Créer `PlanningPage.tsx`

**Fichier :** `src/pages/PlanningPage.tsx`

```typescript
export default function PlanningPage() {
  // État
  const [personnes, setPersonnes] = useState<Personne[]>([]);
  const [selectedPersonneId, setSelectedPersonneId] = useState<number | null>(null);
  const [creneaux, setCreneaux] = useState<PlanningCreneau[]>([]);
  const [anneesDisponibles, setAnneesDisponibles] = useState<string[]>([]);
  const [anneeScolaire, setAnneeScolaire] = useState("");
  const [dateLundi, setDateLundi] = useState(() => getLundiSemaine(new Date()));
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const navigate = useNavigate();
  const { personneId } = useParams();

  // Au montage : charger liste des personnes + charger planning
  // Si personneId dans l'URL → sélectionner cette personne
  // Navigation semaine : recalculer dateLundi et recharger
  // Si selectedPersonneId change → recharger planning
}
```

Comportement :
- Sélecteur de personne (dropdown avec recherche) pour choisir qui voir
- Par défaut : première personne de la liste (ou aucune)
- Affiche `PlanningHebdo` avec les créneaux chargés
- URL param `?semaine=AAAA-MM-JJ` pour semaine spécifique

### 8.2 Route dans App.tsx

**Fichier :** `src/App.tsx`

Ajouter :
```tsx
import PlanningPage from "./pages/PlanningPage";
// dans <Routes> :
<Route path="/planning" element={<PlanningPage />} />
<Route path="/planning/:personneId" element={<PlanningPage />} />
```

### 8.3 Lien dans Nav.tsx

**Fichier :** `src/components/Nav.tsx`

Remplacer :
```tsx
const links = [
  { to: "/", label: "Personnes" },
  { to: "/planning", label: "Planning" },
  { to: "/activites", label: "Activités" },
];
```

---

## 9. Frontend — Sections dans les pages existantes

### 9.1 Section "Créneaux" dans DetailActivite

**Fichier :** `src/pages/DetailActivite.tsx`

Ajouter un bloc après les infos de l'activité, avant les sections Encadrants/Participants :

```
┌─ Créneaux hebdomadaires ───────────────────────┐
│  [Ajouter un créneau]                           │
│  ┌─────────────────────────────────────────────┐│
│  │ Lundi 14:00 - 16:00   [Modifier] [Suppr.]  ││
│  │ Mercredi 10:00 - 12:00 [Modifier] [Suppr.] ││
│  └─────────────────────────────────────────────┘│
│  (Si inscrits : message "verrouillé")           │
└─────────────────────────────────────────────────┘
```

État du formulaire de créneau :
```typescript
const [showCreneauForm, setShowCreneauForm] = useState(false);
const [newCreneauJour, setNewCreneauJour] = useState(1);
const [newCreneauDebut, setNewCreneauDebut] = useState("08:00");
const [newCreneauFin, setNewCreneauFin] = useState("10:00");
```

Appels Tauri :
- `invoke("lister_creneaux", { activiteId: id, anneeScolaire })` au chargement
- `invoke("ajouter_creneau", { input: {...} })` pour créer
- `invoke("supprimer_creneau", { id, activiteId, anneeScolaire })` pour supprimer

### 9.2 Section "Semaines banalisées" dans DetailActivite

Même page, ajouter un bloc :

```
┌─ Semaines banalisées ──────────────────────────┐
│  [Ajouter une semaine]                           │
│  ┌─────────────────────────────────────────────┐│
│  │ 22/12/2025 - Vacances de Noël [Supprimer]   ││
│  │ 23/02/2026 - Vacances d'hiver   [Supprimer] ││
│  └─────────────────────────────────────────────┘│
└─────────────────────────────────────────────────┘
```

Appels Tauri :
- `invoke("lister_semaines_banalisees", { activiteId: id })`
- `invoke("ajouter_semaine_banalisee", { input: {...} })`
- `invoke("supprimer_semaine_banalisee", { id: ... })`

### 9.3 Section "Planning" dans DetailPersonne

**Fichier :** `src/pages/DetailPersonne.tsx`

Ajouter un onglet ou une section après les adhésions :

```tsx
// Dans DetailPersonne, après la section adhésions
{personne && (
  <div className="mt-6">
    <h3 className="text-lg font-semibold text-gray-900 mb-4">Planning</h3>
    <PlanningHebdo
      creneaux={creneauxPlanning}
      dateLundi={dateLundi}
      onSemainePrecedente={() => {
        const d = new Date(dateLundi);
        d.setDate(d.getDate() - 7);
        setDateLundi(d);
      }}
      onSemaineSuivante={() => {
        const d = new Date(dateLundi);
        d.setDate(d.getDate() + 7);
        setDateLundi(d);
      }}
    />
  </div>
)}
```

Charger les données avec :
```typescript
const r = await invoke<PlanningCreneau[]>("planning_personne", {
  personneId: id,
  dateLundi: formatDateISO(dateLundi),
  anneeScolaire: anneeScolaire,
});
```

---

## 10. Vérifications finales

- [x] 10.1 Exécuter `cargo check` dans `src-tauri/`
- [x] 10.2 Exécuter `cargo clippy -- -D warnings`
- [x] 10.3 Exécuter `cargo fmt --check`
- [x] 10.4 Exécuter `cargo test` pour les tests Rust
- [x] 10.5 Exécuter `npm run typecheck`
- [x] 10.6 Exécuter `npm run lint`
- [x] 10.7 Exécuter `npm run build`
