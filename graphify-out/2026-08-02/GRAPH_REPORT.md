# Graph Report - cadence  (2026-08-02)

## Corpus Check
- 98 files · ~60,346 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 1302 nodes · 2329 edges · 102 communities (72 shown, 30 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 14 edges (avg confidence: 0.82)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `a92d3b6f`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- Membres & Adhésions
- OpenSpec Workflow
- Activités & Tarifs
- CI/CD Pipelines
- Dépendances
- App Shell
- Formulaire Adhésion
- Navigation
- Pages Activités
- Détail Personne
- Liste Personnes
- Type Activité
- Type ActivitéPersonne
- Type Adhésion
- Type Âge
- Type CreateActivite
- Type CreateAdhesion
- Type CreateLiaison
- Type CreatePersonne
- Type CreateTarif
- Type Critères Recherche
- Type DateValidation
- Type DateNaissance
- Type DetailActivite
- Type AnnéeScolaire
- Type Mineur
- Type FormatDate
- Type CurrentAnnée
- Type CurrentYear
- Type Pagination
- Type Personne
- Type PersonneActivite
- Type PersonneDetail
- Type RésultatRecherche
- Type UpdateActivite
- Type UpdateAdhesion
- Règles de développement
- SKILL.md
- Décisions
- tasks.md
- Gestion des personnes
- opsx-explore.md
- SKILL.md
- Fonctionnalités
- SKILL.md
- tasks.md
- personnes.md
- SKILL.md
- SKILL.md
- SKILL.md
- Gestion des adhésions
- proposal.md
- proposal.md
- proposal.md
- adhesion.rs
- Documentation fonctionnelle — Cadence
- opencode.json
- dependencies
- graphify.js
- Cadence Desktop Application
- Encadrant
- Module Activités
- Participant
- Règle pas de double rôle encadrant/participant
- Tarif
- Module Adhésions
- Règle une seule adhésion par année scolaire
- Module Personnes
- Règle validité date de naissance (post-1920, pas future)
- Règle mineur et responsable légal
- Adhésion
- Année scolaire
- Personne physique
- Responsable légal
- Error Handling Rules — No expect, crash log, double write
- Explore Mode Thinking Stance
- Intelligent Merging for Delta Specs
- Adhésion au statut binaire réglée/non
- Architecture en couches domain/repositories/commands
- SQLx avec SQLite interchangeable Postgres
- V1 Personnes et Adhésions
- OpenSpec Experimental Change Workflow
- Mandatory Unit Tests for New Business Functions
- Thin Frontend Pattern — Business Logic in Backend

## God Nodes (most connected - your core abstractions)
1. `AppError` - 127 edges
2. `repo()` - 42 edges
3. `setup_db()` - 41 edges
4. `AppState` - 38 edges
5. `seed_activite()` - 37 edges
6. `setup_app()` - 33 edges
7. `MockActiviteRepository` - 25 edges
8. `seed_activite()` - 23 edges
9. `SqliteActiviteRepository` - 21 edges
10. `MockPlanningRepository` - 19 edges

## Surprising Connections (you probably didn't know these)
- `Application entry point (index.html)` --conceptually_related_to--> `Cadence`  [INFERRED]
  index.html → README.md
- `CI Workflow` --references--> `Pre-submission Verification Checklist`  [INFERRED]
  .github/workflows/ci.yml → AGENTS.md
- `ajouter_creneau()` --calls--> `valider_creneau()`  [INFERRED]
  src-tauri/src/commands/planning_commands.rs → src-tauri/src/domain/planning.rs
- `modifier_creneau()` --calls--> `valider_creneau()`  [INFERRED]
  src-tauri/src/commands/planning_commands.rs → src-tauri/src/domain/planning.rs
- `ajouter_semaine_banalisee()` --calls--> `est_lundi()`  [INFERRED]
  src-tauri/src/commands/planning_commands.rs → src-tauri/src/domain/planning.rs

## Import Cycles
- None detected.

## Communities (102 total, 30 thin omitted)

### Community 0 - "Membres & Adhésions"
Cohesion: 0.22
Nodes (9): Application entry point (index.html), Cadence, Documentation fonctionnelle, Développement, Développement assisté par IA (skills), Fonctionnalités, Prérequis, Stack (+1 more)

### Community 1 - "OpenSpec Workflow"
Cohesion: 0.18
Nodes (10): Check for context, Ending Discovery, Guardrails, Handling Different Entry Points, OpenSpec Awareness, The Stance, What You Don't Have To Do, What You Might Do (+2 more)

### Community 4 - "CI/CD Pipelines"
Cohesion: 0.40
Nodes (5): CI Workflow, CI Verification Pipeline, Release Workflow, Release Pipeline, Pre-submission Verification Checklist

### Community 6 - "App Shell"
Cohesion: 0.06
Nodes (55): App(), AdhesionForm(), Props, links, Nav(), PersonneForm(), Props, hauteurBloc() (+47 more)

### Community 7 - "Formulaire Adhésion"
Cohesion: 0.07
Nodes (39): Display, From, P, LiaisonActivitePersonne, Role, Collision, AppError, Error (+31 more)

### Community 8 - "Navigation"
Cohesion: 0.11
Nodes (30): A, date(), make_service(), MockAdhesionRepository, MockPersonneRepository, next_id(), PersonneService, PersonneService<'a, R, A> (+22 more)

### Community 9 - "Pages Activités"
Cohesion: 0.12
Nodes (56): PlanningRepository, repo(), CreateCreneau, CreateSemaineBanalisee, CreneauActivite, Option, PlanningCreneau, Result (+48 more)

### Community 10 - "Détail Personne"
Cohesion: 0.08
Nodes (54): ajouter_personne_activite(), creer_activite(), definir_tarif_activite(), lister_activites(), lister_activites_personne(), lister_annees_activites(), modifier_activite(), obtenir_activite() (+46 more)

### Community 11 - "Liste Personnes"
Cohesion: 0.12
Nodes (54): App, MockRuntime, ajouter_creneau(), ajouter_semaine_banalisee(), lister_creneaux(), lister_semaines_banalisees(), modifier_creneau(), planning_personne() (+46 more)

### Community 12 - "Type Activité"
Cohesion: 0.12
Nodes (32): TarifActivite, ActiviteRepository, create_activite_input(), repo(), Activite, ActivitePersonne, CreateActivite, CreateLiaisonActivitePersonne (+24 more)

### Community 13 - "Type ActivitéPersonne"
Cohesion: 0.05
Nodes (41): Purpose, Requirement: Afficher la grille hebdomadaire, Requirement: Ajouter un créneau quand des inscrits existent, Requirement: Ajouter une semaine banalisée, Requirement: Consulter le planning hebdomadaire d'une personne, Requirement: Créer un créneau horaire pour une activité, Requirement: Détecter les collisions horaires à l'inscription, Requirement: Lister les créneaux d'une activité (+33 more)

### Community 14 - "Type Adhésion"
Cohesion: 0.05
Nodes (40): ADDED Requirements, Requirement: Afficher la grille hebdomadaire, Requirement: Ajouter un créneau quand des inscrits existent, Requirement: Ajouter une semaine banalisée, Requirement: Consulter le planning hebdomadaire d'une personne, Requirement: Créer un créneau horaire pour une activité, Requirement: Détecter les collisions horaires à l'inscription, Requirement: Lister les créneaux d'une activité (+32 more)

### Community 15 - "Type Âge"
Cohesion: 0.05
Nodes (39): Purpose, Requirement: Affichage des dates en français, Requirement: Consulter le détail d'une personne, Requirement: Créer une personne physique, Requirement: Filtrer par statut d'adhésion, Requirement: Lister les personnes, Requirement: Modifier une personne physique, Requirement: Navigation entre personnes et activités (+31 more)

### Community 16 - "Type CreateActivite"
Cohesion: 0.05
Nodes (36): Purpose, Requirement: Afficher les activités d'une personne, Requirement: Ajouter une personne à une activité, Requirement: Consulter le détail d'une activité, Requirement: Créer une activité, Requirement: Définir le tarif d'une activité pour une année scolaire, Requirement: Filtrer les activités par année scolaire, Requirement: Lister les activités (+28 more)

### Community 17 - "Type CreateAdhesion"
Cohesion: 0.09
Nodes (20): CreateCreneau, CreateSemaineBanalisee, CreneauActivite, est_lundi(), PlanningCreneau, Activite, Option, Result (+12 more)

### Community 18 - "Type CreateLiaison"
Cohesion: 0.17
Nodes (30): PersonneRepository, repo(), CreatePersonne, CriteresRecherchePersonnes, Option, Pagination, Personne, Result (+22 more)

### Community 19 - "Type CreatePersonne"
Cohesion: 0.06
Nodes (34): 10. Vérifications finales, 1.1 Créer la migration SQL pour la table `creneaux_activite`, 1.2 Créer la migration SQL pour la table `semaines_banalisees`, 1. Base de données, 2.1 Créer le module `domain/planning.rs`, 2.2 Fonctions de validation dans `domain/planning.rs`, 2.3 Ajouter `pub mod planning` dans `domain/mod.rs`, 2. Backend — Domaine (+26 more)

### Community 20 - "Type CreateTarif"
Cohesion: 0.06
Nodes (33): Architecture, Contexte, `creneaux_activite`, D1 — Créneaux récurrents plutôt que séances individuelles, D2 — Collision à l'inscription, pas à la création du créneau, D3 — Rôles et collisions, D4 — Semaines banalisées par activité, pas globales, D5 — Affichage planning par personne (encadrant ou adhérent) (+25 more)

### Community 21 - "Type Critères Recherche"
Cohesion: 0.07
Nodes (29): dependencies, react, react-dom, react-router-dom, @tauri-apps/api, devDependencies, @biomejs/biome, lefthook (+21 more)

### Community 22 - "Type DateValidation"
Cohesion: 0.07
Nodes (28): source, assist, actions, files, ignoreUnknown, includes, formatter, enabled (+20 more)

### Community 23 - "Type DateNaissance"
Cohesion: 0.07
Nodes (26): ADDED Requirements, Requirement: Affichage des dates, Requirement: Consulter le détail d'une personne, Requirement: Créer une personne physique, Requirement: Lister les personnes, Requirement: Modifier une personne physique, Requirement: Rechercher une personne, Requirement: Validation âge et responsable (+18 more)

### Community 24 - "Type DetailActivite"
Cohesion: 0.08
Nodes (25): ADDED Requirements, MODIFIED Requirements, REMOVED Requirements, RENAMED Requirements, Requirement: Ajouter une personne à une activité, Requirement: Consulter le détail d'une activité, Requirement: Créer une activité, Requirement: Définir le tarif d'une activité pour une année scolaire (+17 more)

### Community 25 - "Type AnnéeScolaire"
Cohesion: 0.16
Nodes (17): Formatter, Activite, ActivitePersonne, CreateActivite, CreateLiaisonActivitePersonne, CreateTarifActivite, DetailActivite, PersonneActivite (+9 more)

### Community 26 - "Type Mineur"
Cohesion: 0.11
Nodes (18): compilerOptions, allowImportingTsExtensions, isolatedModules, jsx, lib, module, moduleDetection, moduleResolution (+10 more)

### Community 27 - "Type FormatDate"
Cohesion: 0.11
Nodes (17): app, security, windows, build, beforeBuildCommand, beforeDevCommand, devUrl, frontendDist (+9 more)

### Community 28 - "Type CurrentAnnée"
Cohesion: 0.18
Nodes (11): AdhesionRepository, Adhesion, CreateAdhesion, Result, Self, Send, SqlitePool, Sync (+3 more)

### Community 29 - "Type CurrentYear"
Cohesion: 0.12
Nodes (15): compilerOptions, allowImportingTsExtensions, isolatedModules, lib, module, moduleDetection, moduleResolution, noEmit (+7 more)

### Community 30 - "Type Pagination"
Cohesion: 0.13
Nodes (14): Purpose, Requirement: Adhésion unique par an, Requirement: Ajouter une adhésion, Requirement: Lister les adhésions d'une personne, Requirement: Modifier une adhésion, Requirements, Scenario: Ajout désactivé si adhésion existante pour l'année en cours, Scenario: Ajout réussi (+6 more)

### Community 31 - "Type Personne"
Cohesion: 0.16
Nodes (19): age_from_date_naissance(), annee_scolaire_from_date(), CreatePersonne, CriteresRecherchePersonnes, current_annee_scolaire(), est_mineur(), Pagination, Personne (+11 more)

### Community 32 - "Type PersonneActivite"
Cohesion: 0.14
Nodes (13): ADDED Requirements, Requirement: Adhésion unique par an, Requirement: Ajouter une adhésion, Requirement: Lister les adhésions d'une personne, Requirement: Modifier une adhésion, Scenario: Ajout désactivé si adhésion existante pour l'année en cours, Scenario: Ajout réussi, Scenario: Année scolaire invalide (+5 more)

### Community 33 - "Type PersonneDetail"
Cohesion: 0.40
Nodes (4): Commandes, Fichiers, Mission, Règles

### Community 34 - "Type RésultatRecherche"
Cohesion: 0.15
Nodes (13): Activités, Ajouter une personne à une activité, Concepts, Consulter les activités d'une personne, Créer une activité, Description, Fonctionnalités, Gérer les créneaux horaires (+5 more)

### Community 35 - "Type UpdateActivite"
Cohesion: 0.15
Nodes (12): Critères d'entrée, Critères de sortie, Cybersécurité, Documentation, Documents consommés, Frontend (TypeScript / React), Interactions avec l'équipe, Mission (+4 more)

### Community 36 - "Type UpdateAdhesion"
Cohesion: 0.15
Nodes (12): 1. SQLx avec SQLite local, interchangeable avec Postgres, 2. Architecture clean mais légère (pas d'hexagonal overhead), 3. React 19 + Tailwind CSS + Vite en frontend Tauri, 4. Deux états pour l'adhésion : adhesion = ligne avec booléen `reglee`, 5. Responsable légal : `responsable_id` nullable sur `personnes_physiques`, 6. Validation date de naissance : <= 1920 et pas dans le futur, 7. Format d'affichage des dates : JJ/MM/AAAA, 8. Désactivation du bouton d'ajout d'adhésion si l'année en cours existe déjà (+4 more)

### Community 38 - "Règles de développement"
Cohesion: 0.15
Nodes (13): Architecture, Couverture de code, Documentations, Gestion des erreurs, Graphe de connaissances (graphify), Organisation de l'équipe, Principe, Règles de développement (+5 more)

### Community 39 - "SKILL.md"
Cohesion: 0.17
Nodes (11): Critères d'entrée, Critères de sortie, Documents consommés, Documents produits / maintenus, Interactions avec l'équipe, Mise à jour de la documentation fonctionnelle, Mise à jour du graphe de connaissance, Mission (+3 more)

### Community 40 - "Décisions"
Cohesion: 0.17
Nodes (11): 1. Table unique `activite_personnes` avec rôle et année scolaire, 2. L'utilisateur ne peut pas être à la fois encadrant et participant pour une même activité, 3. Tarifs dans une table séparée, 4. Capacité maximale optionnelle sur `activites`, 5. Navigation via un menu dans `App.tsx`, 6. Liste des activités filtrée par année scolaire, 7. Architecture en couches inchangée, Context (+3 more)

### Community 41 - "tasks.md"
Cohesion: 0.18
Nodes (10): 10. Vérifications finales, 1. Base de données, 2. Couche domaine, 3. Repository, 4. Tauri commands, 5. Frontend — Navigation, 6. Frontend — Liste des activités, 7. Frontend — Détail d'une activité (+2 more)

### Community 43 - "Gestion des personnes"
Cohesion: 0.20
Nodes (10): Consulter le détail d'une personne, Créer une personne, Filtrer par adhésion, Gestion des personnes, Lister les personnes, Modifier une personne, Pagination, Rechercher une personne (+2 more)

### Community 44 - "opsx-explore.md"
Cohesion: 0.20
Nodes (9): Check for context, Ending Discovery, Guardrails, OpenSpec Awareness, The Stance, What You Don't Have To Do, What You Might Do, When a change exists (+1 more)

### Community 45 - "SKILL.md"
Cohesion: 0.20
Nodes (9): Critères d'entrée, Critères de sortie, Création d'un change, Documents consommés, Documents produits, Interactions avec l'équipe, Mise à jour des spécifications, Mission (+1 more)

### Community 50 - "Fonctionnalités"
Cohesion: 0.22
Nodes (9): Concepts, Consulter le planning d'une personne, Description, Détection des collisions, Flux, Fonctionnalités, Gérer les créneaux d'une activité, Gérer les semaines banalisées (+1 more)

### Community 51 - "SKILL.md"
Cohesion: 0.22
Nodes (8): Critères d'entrée, Critères de sortie, Documents consommés, Documents produits, Interactions avec l'équipe, Mission, Règles, Workflow

### Community 52 - "tasks.md"
Cohesion: 0.22
Nodes (8): 1. Initialisation du projet, 2. Base de données, 3. Couche domaine, 4. Repositories, 5. Tauri commands, 6. Frontend — Liste des personnes, 7. Frontend — Détail d'une personne, 8. Validation et finitions

### Community 53 - "personnes.md"
Cohesion: 0.22
Nodes (8): ADDED Requirements, MODIFIED Requirements, REMOVED Requirements, RENAMED Requirements, Requirement: Consulter le détail d'une personne, Requirement: Navigation entre personnes et activités, Scenario: Consultation avec activités, Scenario: Menu visible

### Community 54 - "SKILL.md"
Cohesion: 0.25
Nodes (7): Critères d'entrée, Critères de sortie, Document produit, Documents consommés, Interactions avec l'équipe, Mission, Workflow

### Community 55 - "SKILL.md"
Cohesion: 0.25
Nodes (7): Critères d'entrée, Critères de sortie, Documents consommés, Documents produits, Interactions avec l'équipe, Mission, Workflow

### Community 56 - "SKILL.md"
Cohesion: 0.25
Nodes (7): Critères d'entrée, Critères de sortie, Documents consommés, Documents produits, Interactions avec l'équipe, Mission, Workflow

### Community 59 - "Gestion des adhésions"
Cohesion: 0.29
Nodes (7): Ajouter une adhésion, Format de l'année scolaire, Gestion des adhésions, Lister les adhésions d'une personne, Modifier une adhésion, Règle : une seule adhésion par année, Suivi des règlements

### Community 60 - "proposal.md"
Cohesion: 0.29
Nodes (6): Capabilities, Impact, Modified Capabilities, New Capabilities, What Changes, Why

### Community 61 - "proposal.md"
Cohesion: 0.29
Nodes (6): Capabilities, Impact, Modified Capabilities, New Capabilities, What Changes, Why

### Community 62 - "proposal.md"
Cohesion: 0.29
Nodes (6): Capabilities, Impact, Modified Capabilities, New Capabilities, What Changes, Why

### Community 63 - "adhesion.rs"
Cohesion: 0.60
Nodes (5): Adhesion, CreateAdhesion, Option, String, UpdateAdhesion

### Community 64 - "Documentation fonctionnelle — Cadence"
Cohesion: 0.40
Nodes (5): Concepts généraux, Documentation fonctionnelle — Cadence, Flux principaux, Modules fonctionnels, Public visé

## Knowledge Gaps
- **522 isolated node(s):** `$schema`, `plugin`, `@opencode-ai/plugin`, `$schema`, `enabled` (+517 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **30 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `AppError` connect `Formulaire Adhésion` to `Navigation`, `Pages Activités`, `Détail Personne`, `Liste Personnes`, `Type Activité`, `Type CreateLiaison`, `Type CurrentAnnée`?**
  _High betweenness centrality (0.133) - this node is a cross-community bridge._
- **Why does `Role` connect `Formulaire Adhésion` to `Type AnnéeScolaire`, `Type CreateAdhesion`?**
  _High betweenness centrality (0.010) - this node is a cross-community bridge._
- **Why does `AppState` connect `Détail Personne` to `Pages Activités`, `Liste Personnes`, `Type Activité`, `Type CreateLiaison`, `Type CurrentAnnée`?**
  _High betweenness centrality (0.009) - this node is a cross-community bridge._
- **What connects `$schema`, `plugin`, `@opencode-ai/plugin` to the rest of the system?**
  _535 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `App Shell` be split into smaller, more focused modules?**
  _Cohesion score 0.06376811594202898 - nodes in this community are weakly interconnected._
- **Should `Formulaire Adhésion` be split into smaller, more focused modules?**
  _Cohesion score 0.07277701778385773 - nodes in this community are weakly interconnected._
- **Should `Navigation` be split into smaller, more focused modules?**
  _Cohesion score 0.10821256038647344 - nodes in this community are weakly interconnected._