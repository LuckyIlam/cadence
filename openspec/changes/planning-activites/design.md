# Design — Planning des activités

## Contexte

Cadence gère déjà les personnes, les adhésions annuelles et les activités (cours, ateliers, sorties). Chaque activité peut avoir des participants et des encadrants, un tarif par année scolaire, et une capacité maximale.

Ce qui manque aujourd'hui : **la dimension temporelle**. Les activités existent comme des entités statiques — on ne sait pas *quand* elles ont lieu dans la semaine. Il n'existe pas de vue d'ensemble permettant à un encadrant de consulter son emploi du temps, ni à un adhérent de visualiser ses activités semaine par semaine.

## Problème

- Un encadrant ne peut pas savoir s'il est disponible pour encadrer une nouvelle activité sans consulter manuellement ses créneaux
- Un adhérent inscrit à plusieurs activités ne peut pas vérifier qu'elles ne se chevauchent pas
- L'association ne peut pas publier un planning hebdomadaire des activités
- Les semaines banalisées (vacances, ponts) ne peuvent pas être signalées

## Goals

1. **Créneaux hebdomadaires récurrents** — chaque activité peut avoir un ou plusieurs créneaux fixes dans la semaine (jour + heure début + heure fin), liés à une année scolaire
2. **Semaines banalisées** — possibilité d'exclure certaines semaines du planning pour une activité (vacances, fermeture exceptionnelle)
3. **Planning encadrant** — vue hebdomadaire (lundi→dimanche, 8h→20h) des activités qu'il encadre, avec navigation par semaine (numéro de semaine + date du lundi)
4. **Planning adhérent** — vue hebdomadaire des activités auxquelles il participe (intégré dans le détail de la personne ou page dédiée)
5. **Planning par activité** — dans le détail de l'activité, visualisation et gestion des créneaux et semaines banalisées
6. **Détection de collisions** — à l'inscription d'une personne à une activité, vérifier qu'aucun de ses créneaux ne chevauche ceux d'une autre activité où elle a déjà un rôle (encadrant ou participant)

## Non-Goals

- Pas de gestion de salles / ressources (réservation de locaux)
- Pas de génération de planning PDF / export
- Pas de rappels automatiques
- Pas de gestion des présences aux séances individuelles (absence, retard)
- Pas de modification des créneaux par les encadrants eux-mêmes (saisie réservée au secrétaire depuis la fiche activité)
- Pas de synchronisation calendrier externe (.ics, Google Calendar) dans cette première version

## Modèle de données

### `creneaux_activite`

Créneaux hebdomadaires récurrents d'une activité pour une année scolaire.

```sql
CREATE TABLE creneaux_activite (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    activite_id     INTEGER NOT NULL,
    jour_semaine    INTEGER NOT NULL CHECK (jour_semaine BETWEEN 1 AND 7),
    heure_debut     TEXT NOT NULL,  -- format HH:MM
    heure_fin       TEXT NOT NULL,  -- format HH:MM
    annee_scolaire  TEXT NOT NULL,
    FOREIGN KEY (activite_id) REFERENCES activites(id),
    CHECK (heure_debut < heure_fin)
);

CREATE INDEX idx_creneaux_activite ON creneaux_activite(activite_id);
```

- `jour_semaine` : 1=lundi, 2=mardi, …, 7=dimanche (norme ISO 8601)
- Une activité peut avoir plusieurs créneaux (ex: Poterie le lundi 14h-16h ET le mercredi 10h-12h)

### `semaines_banalisees`

Semaines où une activité n'a pas lieu (vacances, pont, fermeture).

```sql
CREATE TABLE semaines_banalisees (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    activite_id     INTEGER NOT NULL,
    date_debut      TEXT NOT NULL,  -- date du lundi de la semaine (ISO: AAAA-MM-JJ)
    motif           TEXT,
    annee_scolaire  TEXT NOT NULL,
    FOREIGN KEY (activite_id) REFERENCES activites(id)
);

CREATE INDEX idx_semaines_banalisees_activite ON semaines_banalisees(activite_id);
```

- `date_debut` est toujours un lundi (la semaine ISO 8601)
- Validation : `date_debut` doit être un lundi (ou on stocke la date du lundi)

## Décisions architecturales

### D1 — Créneaux récurrents plutôt que séances individuelles

**Choix :** On stocke des créneaux hebdomadaires récurrents, pas des occurrences individuelles.

**Raison :** Une activité type "cours de poterie" a lieu chaque semaine à la même heure. Générer N occurrences pour chaque semaine de l'année serait volumineux (plusieurs milliers de lignes par activité) et complexifierait la gestion des semaines banalisées. Le modèle récurrent + exceptions est plus léger et plus naturel.

**Alternative :** Générer toutes les séances individuelles dans une table `seances`. Rejeté car plus lourd et redondant.

**Conséquence :** L'affichage planning pour une semaine donnée est calculé : on prend les créneaux de l'activité, on retire ceux des semaines banalisées. Pas de stockage des occurrences.

### D2 — Collision à l'inscription, pas à la création du créneau

**Choix :** La détection de collision se fait au moment d'ajouter une personne à une activité (ou de modifier le créneau si c'est le seul moyen).

**Raison :** Un créneau en soi n'est pas problématique — c'est le croisement personne + créneaux qui crée le conflit. On vérifie donc à l'inscription : la personne a-t-elle déjà un créneau qui chevauche celui de la nouvelle activité ? Cela permet à deux activités différentes d'avoir des créneaux identiques sans erreur, tant qu'aucune personne n'est inscrite aux deux.

**Règle complémentaire :** Si une personne est déjà inscrite à une activité, les créneaux de cette activité deviennent verrouillés (non modifiables, non supprimables) pour l'année concernée.

### D3 — Rôles et collisions

**Choix :** La collision s'applique quel que soit le rôle. Si une personne est encadrante à 14h sur une activité, elle ne peut pas être participant·e à 14h sur une autre (ni encadrante sur une autre, ni participante).

**Raison :** La règle métier est simple — une personne ne peut pas être à deux endroits à la fois. Traiter les rôles séparément (encadrant vs participant) ajouterait de la complexité sans valeur ajoutée claire.

### D4 — Semaines banalisées par activité, pas globales

**Choix :** Chaque activité a sa propre liste de semaines banalisées.

**Raison :** Les activités ne suivent pas forcément le même calendrier. Certaines ont lieu pendant les vacances scolaires, d'autres non. Certaines s'arrêtent pour un pont spécifique. Un modèle global (ex: "vacances de Noël") ne couvrirait pas tous les cas.

### D5 — Affichage planning par personne (encadrant ou adhérent)

**Choix :** Deux points d'entrée :
1. Depuis le détail d'une personne, un onglet / section "Planning" affiche sa semaine
2. Une nouvelle page `/planning` avec un sélecteur de personne permet de consulter n'importe quel planning

**Raison :** Le planning encadrant est un besoin de navigation transverse ("voir mon planning") tandis que le planning adhérent est consulté dans le cadre du suivi d'une personne. Les deux partagent le même composant d'affichage (grille hebdomadaire).

## Architecture

```
┌─ Frontend ──────────────────────────────────────┐
│                                                  │
│  /planning → PlanningPage (sélecteur personne)   │
│  /personnes/:id → DetailPersonne + section       │
│    PlanningPersonne (grille hebdo)                │
│  /activites/:id → DetailActivite + section       │
│    CreneauxActivite + SemainesBanalisees          │
│                                                  │
└──────────────────────┬───────────────────────────┘
                       │ invoke()
┌─ Backend (Tauri) ────┴───────────────────────────┐
│                                                  │
│  Commands Rust :                                  │
│  - ajouter_creneau / supprimer_creneau            │
│  - lister_creneaux(activite_id, annee_scolaire)   │
│  - ajouter_semaine_banalisee / supprimer_...      │
│  - lister_semaines_banalisees(activite_id)        │
│  - verifier_collision(personne_id, activite_id)   │
│  - planning_personne(personne_id, date_debut)     │
│                                                   │
│  Domain : CreneauActivite, SemaineBanalisee        │
│  Repo   : planning_repo.rs                        │
│                                                   │
└───────────────────────────────────────────────────┘
```

## Règles de gestion détaillées

### Règle 1 — Création de créneaux

- Au moins un créneau doit exister pour qu'une activité soit visible dans le planning
- Un créneau doit avoir `heure_debut < heure_fin`
- Les heures sont au format HH:MM (pas de secondes)
- `jour_semaine` est entre 1 (lundi) et 7 (dimanche)
- L'`annee_scolaire` du créneau doit correspondre à une année valide

### Règle 2 — Immutabilité des créneaux avec inscrits

- Si au moins une personne est inscrite (participant ou encadrant) à l'activité pour l'année, les créneaux de cette année ne peuvent plus être **modifiés ni supprimés**
- On peut toujours **ajouter** un nouveau créneau (cela n'affecte pas les inscriptions existantes)
- Pour modifier/supprimer un créneau existant : il faut d'abord retirer toutes les personnes inscrites à l'activité pour cette année

### Règle 3 — Détection de collision à l'inscription

Quand on appelle `ajouter_personne_activite` :
1. Récupérer les créneaux de l'activité cible
2. Récupérer toutes les activités où la personne a un rôle (quel qu'il soit)
3. Pour chaque activité, récupérer ses créneaux
4. Vérifier pour chaque créneau de l'activité cible s'il chevauche un créneau d'une autre activité où la personne est inscrite
   - Même `jour_semaine` ET `heure_debut < autre_heure_fin` ET `heure_fin > autre_heure_debut`
5. Si collision → refuser avec message explicite listant l'activité et le créneau en conflit

### Règle 4 — Créneaux verrouillés si personnes inscrites

- Si une personne est inscrite à une activité ayant des créneaux, les créneaux sont verrouillés pour l'année en cours
- Pas besoin de vérouillage par créneau individuel — le vérouillage est au niveau de l'activité pour l'année

### Règle 5 — Semaines banalisées

- Une semaine banalisée est définie par sa date de début (lundi)
- La validation vérifie que `date_debut` est bien un lundi
- Le `motif` est optionnel (ex: "Vacances de Noël", "Pont de l'Ascension")
- Les semaines banalisées peuvent être ajoutées/supprimées librement (pas de verrouillage lié aux inscriptions)

### Règle 6 — Calcul du planning pour une personne

Pour une personne donnée et une semaine donnée (date_debut, un lundi) :
1. Récupérer toutes les activités où la personne est inscrite (tous rôles)
2. Pour chaque activité, récupérer ses créneaux
3. Pour chaque activité, vérifier si la semaine est banalisée (via `semaines_banalisees`)
4. Ne garder que les créneaux dont la semaine n'est pas banalisée
5. Retourner la liste des créneaux avec les infos d'activité

## Risques et trade-offs

### Risque 1 — Performance du planning calculé

Le planning n'est pas stocké mais calculé à chaque requête. Pour une association typique (< 50 activités, < 500 personnes), c'est négligeable. Si le volume explose, on pourra matérialiser les occurrences.

### Risque 2 — Verrouillage large des créneaux

Le verrouillage est par activité/année, pas par créneau individuel ni par inscription. Si une activité a 3 créneaux et qu'une personne est inscrite sur un seul des 3, les 3 sont verrouillés. C'est volontaire : rouvrir un créneau "vide" alors que d'autres créneaux de la même activité ont des inscrits serait source d'erreur. Si le besoin apparaît, on pourra affiner.

### Risque 3 — Navigation hebdomadaire sans bornes

L'interface de navigation par semaine doit limiter la plage de navigation à l'année scolaire sélectionnée pour éviter la confusion. On pourrait aussi proposer un "Aujourd'hui" pour revenir rapidement à la semaine courante.

### Risque 4 — Cohérence avec les années scolaires multiples

Un adhérent inscrit à une activité sur plusieurs années voit ses activités cumulées. Mais le planning est par année scolaire. Le filtre par année scolaire doit être clair dans l'interface pour ne pas mélanger les années.

## Pages / Routes

### Nouvelles routes

| Route | Page | Description |
|-------|------|-------------|
| `/planning` | `PlanningPage` | Sélecteur de personne + grille hebdomadaire. Par défaut : personne connectée (ou première personne avec rôle encadrant) |
| `/planning/:personneId?semaine=2026-07-06` | même composant | Lien direct vers le planning d'une personne pour une semaine donnée |

### Modifications dans les pages existantes

**DetailActivite** — Ajouter deux sections :
- **Créneaux** : liste des créneaux de l'activité pour l'année sélectionnée, avec bouton "Ajouter un créneau". Affichage verrouillé si des inscrits existent.
- **Semaines banalisées** : liste des semaines banalisées, avec bouton "Ajouter une semaine". Pas de verrouillage.

**DetailPersonne** — Ajouter une section ou un onglet "Planning" :
- Vue hebdomadaire avec les créneaux de l'activité où la personne a un rôle
- Navigation par semaine (précédent/suivant, avec numéro de semaine et date)

### Navigation

Ajouter un lien "Planning" dans la barre de navigation (Nav.tsx) :
```tsx
const links = [
  { to: "/", label: "Personnes" },
  { to: "/planning", label: "Planning" },
  { to: "/activites", label: "Activités" },
];
```

## Types à ajouter (frontend, `types.ts`)

```typescript
export interface CreneauActivite {
  id: number;
  activite_id: number;
  jour_semaine: number;      // 1=lundi…7=dimanche
  heure_debut: string;       // HH:MM
  heure_fin: string;         // HH:MM
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
  date_debut: string;        // AAAA-MM-JJ (lundi)
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

## Tâches d'implémentation (ordre suggéré)

1. Migration SQL : créer les tables `creneaux_activite` et `semaines_banalisees`
2. Domain Rust : types `CreneauActivite`, `SemaineBanalisee`, `CreateCreneau`, etc.
3. Repository Rust : `planning_repo.rs` — CRUD créneaux + semaines banalisées + requêtes planning
4. Commands Rust : `planning_commands.rs` — ajouter/supprimer créneaux, ajouter/supprimer semaines banalisées, planning_personne, vérifier_collision
5. Intégrer la vérification de collision dans `ajouter_personne_activite` (modification de `activite_commands.rs`)
6. Types frontend : ajouter les types TypeScript
7. Composant `PlanningHebdo` : grille hebdomadaire réutilisable
8. Page `PlanningPage` : sélecteur de personne + grille
9. Section planning dans `DetailPersonne`
10. Section créneaux + semaines banalisées dans `DetailActivite`
11. Ajouter "Planning" dans la navigation
12. Tests unitaires (Rust) pour la logique de collision et le calcul de planning
