# Graph Report - cadence  (2026-08-08)

## Corpus Check
- 146 files · ~97,899 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 2145 nodes · 4399 edges · 137 communities (107 shown, 30 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 29 edges (avg confidence: 0.81)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `80647c5e`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- Membres & Adhésions
- OpenSpec Workflow
- Activités & Tarifs
- Dev Setup & Conventions
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
- parametre.rs
- Règles de développement
- SKILL.md
- Décisions
- tasks.md
- Requirement: Créer un créneau horaire pour une activité
- Gestion des personnes
- opsx-explore.md
- SKILL.md
- Design — Plage horaire d'ouverture configurable
- Requirement: Modifier la plage horaire d'ouverture
- Requirement: Modifier la plage horaire d'ouverture
- proposal.md
- Fonctionnalités
- SKILL.md
- tasks.md
- personnes.md
- SKILL.md
- SKILL.md
- SKILL.md
- AppError
- personne.rs
- Gestion des adhésions
- proposal.md
- proposal.md
- proposal.md
- adhesion.rs
- Documentation fonctionnelle — Cadence
- Paramètres
- MockPlanningRepository
- adhesion_repo.rs
- ADDED Requirements
- Decisions
- opencode.json
- dependencies
- graphify.js
- migrations.rs
- proposal.md
- tasks.md
- db.rs
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
- Requirement: Détecter les modifications concurrentes lors d'une mise à jour
- db.rs
- parametre_commands.rs
- activite.rs
- parametre_commands.rs
- personne_commands.rs
- DbRow
- params.rs
- adhesion_commands.rs
- row.rs
- Change `db-driver-abstraction` — proposal
- Change `db-driver-abstraction` — tasks

## God Nodes (most connected - your core abstractions)
1. `AppError` - 288 edges
2. `String` - 87 edges
3. `repo()` - 51 edges
4. `setup_db()` - 50 edges
5. `Db` - 49 edges
6. `DbTransaction` - 49 edges
7. `seed_activite()` - 45 edges
8. `AppState` - 45 edges
9. `setup_app()` - 36 edges
10. `RowView` - 34 edges

## Surprising Connections (you probably didn't know these)
- `Application entry point (index.html)` --conceptually_related_to--> `Cadence`  [INFERRED]
  index.html → README.md
- `CI Workflow` --references--> `Pre-submission Verification Checklist`  [INFERRED]
  .github/workflows/ci.yml → AGENTS.md
- `modifier_plage_horaire()` --calls--> `valider_plage_horaire()`  [INFERRED]
  src-tauri/src/commands/parametre_commands.rs → src-tauri/src/domain/parametre.rs
- `setup_app()` --calls--> `init_app_state()`  [INFERRED]
  src-tauri/src/commands/parametre_commands.rs → src-tauri/src/infrastructure/db/mod.rs
- `ajouter_creneau()` --calls--> `valider_creneau()`  [INFERRED]
  src-tauri/src/commands/planning_commands.rs → src-tauri/src/domain/planning.rs

## Import Cycles
- None detected.

## Communities (137 total, 30 thin omitted)

### Community 0 - "Membres & Adhésions"
Cohesion: 0.20
Nodes (9): Application entry point (index.html), Cadence, Documentation fonctionnelle, Développement, Développement assisté par IA (skills), Fonctionnalités, Prérequis, Stack (+1 more)

### Community 1 - "OpenSpec Workflow"
Cohesion: 0.18
Nodes (10): Check for context, Ending Discovery, Guardrails, Handling Different Entry Points, OpenSpec Awareness, The Stance, What You Don't Have To Do, What You Might Do (+2 more)

### Community 3 - "Dev Setup & Conventions"
Cohesion: 0.33
Nodes (6): Connexion à la base, Description, Impact de la plage, Modifier la plage horaire, Paramètres, Réduire la plage horaire

### Community 4 - "CI/CD Pipelines"
Cohesion: 0.40
Nodes (5): CI Workflow, CI Verification Pipeline, Release Workflow, Release Pipeline, Pre-submission Verification Checklist

### Community 6 - "App Shell"
Cohesion: 0.06
Nodes (68): AdhesionForm(), Props, ConnexionConfigForm(), Props, links, Nav(), PersonneForm(), Props (+60 more)

### Community 7 - "Formulaire Adhésion"
Cohesion: 0.06
Nodes (45): LiaisonActivitePersonne, PersonneActivite, Collision, AppError, Error, From, Self, maintenant_utc() (+37 more)

### Community 8 - "Navigation"
Cohesion: 0.06
Nodes (48): A, age_from_date_naissance(), annee_scolaire_from_date(), CreatePersonne, CriteresRecherchePersonnes, current_annee_scolaire(), est_mineur(), Pagination (+40 more)

### Community 9 - "Pages Activités"
Cohesion: 0.08
Nodes (42): Activite, ActiviteAnneeRow, ActivitePersonneRow, ActiviteRepository, AnneeRow, CompteurRow, create_activite_input(), LiaisonActivitePersonne (+34 more)

### Community 10 - "Détail Personne"
Cohesion: 0.23
Nodes (22): ajouter_personne_activite(), creer_activite(), definir_tarif_activite(), lister_activites(), lister_activites_personne(), lister_annees_activites(), modifier_activite(), obtenir_activite() (+14 more)

### Community 11 - "Liste Personnes"
Cohesion: 0.11
Nodes (57): ajouter_creneau(), ajouter_semaine_banalisee(), lister_creneaux(), lister_semaines_banalisees(), modifier_creneau(), planning_personne(), App, Connection (+49 more)

### Community 12 - "Type Activité"
Cohesion: 0.06
Nodes (70): Row, LibsqlParametreRepository, ParametreRepository, ParametresPlanning, Arc, Result, Self, Send (+62 more)

### Community 13 - "Type ActivitéPersonne"
Cohesion: 0.04
Nodes (46): Purpose, Requirement: Afficher la grille hebdomadaire, Requirement: Ajouter un créneau quand des inscrits existent, Requirement: Ajouter une semaine banalisée, Requirement: Consulter le planning hebdomadaire d'une personne, Requirement: Créer un créneau horaire pour une activité, Requirement: Détecter les collisions horaires à l'inscription, Requirement: Lister les créneaux d'une activité (+38 more)

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
Cohesion: 0.07
Nodes (22): CreateCreneau, CreateSemaineBanalisee, CreneauActivite, CreneauHorsPlage, est_lundi(), format_conflit_plage(), Inscription, PlanningCreneau (+14 more)

### Community 18 - "Type CreateLiaison"
Cohesion: 0.08
Nodes (75): Sized, ActiviteCreneauRow, AutreActiviteRow, CompteurRow, CreneauActivite, CreneauHorsPlage, IdRow, Inscription (+67 more)

### Community 19 - "Type CreatePersonne"
Cohesion: 0.06
Nodes (34): 10. Vérifications finales, 1.1 Créer la migration SQL pour la table `creneaux_activite`, 1.2 Créer la migration SQL pour la table `semaines_banalisees`, 1. Base de données, 2.1 Créer le module `domain/planning.rs`, 2.2 Fonctions de validation dans `domain/planning.rs`, 2.3 Ajouter `pub mod planning` dans `domain/mod.rs`, 2. Backend — Domaine (+26 more)

### Community 20 - "Type CreateTarif"
Cohesion: 0.06
Nodes (33): Architecture, Contexte, `creneaux_activite`, D1 — Créneaux récurrents plutôt que séances individuelles, D2 — Collision à l'inscription, pas à la création du créneau, D3 — Rôles et collisions, D4 — Semaines banalisées par activité, pas globales, D5 — Affichage planning par personne (encadrant ou adhérent) (+25 more)

### Community 21 - "Type Critères Recherche"
Cohesion: 0.06
Nodes (30): dependencies, react, react-dom, react-router-dom, @tauri-apps/api, @tauri-apps/plugin-process, devDependencies, @biomejs/biome (+22 more)

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
Cohesion: 0.12
Nodes (41): AppHandle, Default, ModeConnexion, app_dir(), appliquer_config(), ConfigAffichee, lire_config(), normaliser_url() (+33 more)

### Community 26 - "Type Mineur"
Cohesion: 0.11
Nodes (18): compilerOptions, allowImportingTsExtensions, isolatedModules, jsx, lib, module, moduleDetection, moduleResolution (+10 more)

### Community 27 - "Type FormatDate"
Cohesion: 0.11
Nodes (17): app, security, windows, build, beforeBuildCommand, beforeDevCommand, devUrl, frontendDist (+9 more)

### Community 28 - "Type CurrentAnnée"
Cohesion: 0.08
Nodes (29): F, Rows, execute_avec_retry(), query_avec_retry(), Connection, P, Result, vider_cursor() (+21 more)

### Community 29 - "Type CurrentYear"
Cohesion: 0.12
Nodes (15): compilerOptions, allowImportingTsExtensions, isolatedModules, lib, module, moduleDetection, moduleResolution, noEmit (+7 more)

### Community 30 - "Type Pagination"
Cohesion: 0.13
Nodes (14): Purpose, Requirement: Adhésion unique par an, Requirement: Ajouter une adhésion, Requirement: Lister les adhésions d'une personne, Requirement: Modifier une adhésion, Requirements, Scenario: Ajout désactivé si adhésion existante pour l'année en cours, Scenario: Ajout réussi (+6 more)

### Community 31 - "Type Personne"
Cohesion: 0.05
Nodes (36): 1.1 Créer la migration SQL pour la table `parametres`, 1. Base de données, 2.1 Créer le module `domain/parametre.rs`, 2.2 Enregistrer le module dans `domain/mod.rs`, 2.3 Tests unitaires (obligatoires), 2. Backend — Domaine, 3.1 Créer `repositories/parametre_repo.rs`, 3.2 Enregistrer dans `repositories/mod.rs` (+28 more)

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

### Community 37 - "parametre.rs"
Cohesion: 0.10
Nodes (26): creneau(), heure_en_minutes(), ImpactAction, ImpactCreneau, minutes_en_heure(), ParametresPlanning, CreateCreneau, Option (+18 more)

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

### Community 42 - "Requirement: Créer un créneau horaire pour une activité"
Cohesion: 0.12
Nodes (16): MODIFIED Requirements, Requirement: Afficher la grille hebdomadaire, Requirement: Créer un créneau horaire pour une activité, Requirement: Modifier un créneau horaire, Scenario: Affichage de la grille, Scenario: Ajout d'un deuxième créneau à la même activité, Scenario: Création avec heure_debut > heure_fin, Scenario: Création avec jour_semaine invalide (+8 more)

### Community 43 - "Gestion des personnes"
Cohesion: 0.20
Nodes (10): Consulter le détail d'une personne, Créer une personne, Filtrer par adhésion, Gestion des personnes, Lister les personnes, Modifier une personne, Pagination, Rechercher une personne (+2 more)

### Community 44 - "opsx-explore.md"
Cohesion: 0.20
Nodes (9): Check for context, Ending Discovery, Guardrails, OpenSpec Awareness, The Stance, What You Don't Have To Do, What You Might Do, When a change exists (+1 more)

### Community 45 - "SKILL.md"
Cohesion: 0.20
Nodes (9): Critères d'entrée, Critères de sortie, Création d'un change, Documents consommés, Documents produits, Interactions avec l'équipe, Mise à jour des spécifications, Mission (+1 more)

### Community 46 - "Design — Plage horaire d'ouverture configurable"
Cohesion: 0.17
Nodes (11): Architecture, Backend (Rust / Tauri), Contexte, Design — Plage horaire d'ouverture configurable, Décisions de conception, Frontend (React / TS), Modèle de données, Non-Goals (+3 more)

### Community 47 - "Requirement: Modifier la plage horaire d'ouverture"
Cohesion: 0.12
Nodes (16): Purpose, Requirement: Apercevoir l'impact d'une réduction de plage, Requirement: Consulter la plage horaire d'ouverture des activités, Requirement: Modifier la plage horaire d'ouverture, Requirement: Réduire la plage horaire en gérant les créneaux impactés, Requirements, Scenario: Aperçu avec suppressions et déplacements, Scenario: Aperçu sans impact (+8 more)

### Community 48 - "Requirement: Modifier la plage horaire d'ouverture"
Cohesion: 0.12
Nodes (15): ADDED Requirements, Requirement: Apercevoir l'impact d'une réduction de plage, Requirement: Consulter la plage horaire d'ouverture des activités, Requirement: Modifier la plage horaire d'ouverture, Requirement: Réduire la plage horaire en gérant les créneaux impactés, Scenario: Aperçu avec suppressions et déplacements, Scenario: Aperçu sans impact, Scenario: Déplacement refusé si chevauchement avec une autre activité de l'adhérent (+7 more)

### Community 49 - "proposal.md"
Cohesion: 0.29
Nodes (6): Capabilities, Impact, Modified Capabilities, New Capabilities, What Changes, Why

### Community 50 - "Fonctionnalités"
Cohesion: 0.20
Nodes (10): Concepts, Consulter le planning d'une personne, Description, Détection des collisions, Flux, Fonctionnalités, Gérer les créneaux d'une activité, Gérer les semaines banalisées (+2 more)

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

### Community 57 - "AppError"
Cohesion: 0.09
Nodes (37): hors_plage(), make_db(), make_service(), MockParametreRepository, MockPlanningRepository, ParametreService, ParametreService<'a, R, P>, CreateCreneau (+29 more)

### Community 58 - "personne.rs"
Cohesion: 0.08
Nodes (25): ADDED Requirements, Purpose, Requirement: Afficher un écran de premier lancement si la base n'est pas configurée, Requirement: Appliquer un changement de mode ou de connexion, Requirement: Choisir le mode de fonctionnement, Requirement: Configurer la connexion selon le mode choisi, Requirement: Conserver et relire la configuration localement, Requirement: Tester la connexion en mode multi-utilisateurs (+17 more)

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
Cohesion: 0.09
Nodes (20): to_db_value(), to_libsql_value(), bool, DbValue, f64, i32, i64, Option<bool> (+12 more)

### Community 64 - "Documentation fonctionnelle — Cadence"
Cohesion: 0.33
Nodes (6): Concepts généraux, Documentation fonctionnelle — Cadence, Données et vie privée (RGPD), Flux principaux, Modules fonctionnels, Public visé

### Community 65 - "Paramètres"
Cohesion: 0.08
Nodes (25): Purpose, Requirement: Afficher un écran de premier lancement si la base n'est pas configurée, Requirement: Appliquer un changement de mode ou de connexion, Requirement: Choisir le mode de fonctionnement, Requirement: Configurer la connexion selon le mode choisi, Requirement: Conserver et relire la configuration localement, Requirement: Tester la connexion en mode multi-utilisateurs, Requirements (+17 more)

### Community 66 - "MockPlanningRepository"
Cohesion: 0.18
Nodes (11): Choisir son mode dans les Paramètres, Connexion et mode de fonctionnement, Description, Données indépendantes, Mode mono-utilisateur, Mode multi-utilisateurs, Modifications concurrentes, Nom d'utilisateur et traçabilité (+3 more)

### Community 67 - "adhesion_repo.rs"
Cohesion: 0.16
Nodes (17): DbParams, commit_consomme_la_transaction(), D, DbTransactionExt, Echantillon, FausseTransaction, fetch_all_typed_dans_transaction(), fetch_one_not_found_dans_transaction() (+9 more)

### Community 68 - "ADDED Requirements"
Cohesion: 0.11
Nodes (17): ADDED Requirements, Purpose, Requirement: Détecter les modifications concurrentes lors d'une mise à jour, Requirement: Enregistrer l'auteur et l'horodatage de chaque création ou modification, Requirement: Fournir le nom d'utilisateur aux écritures, Requirement: Ne jamais afficher l'audit, Scenario: Conflit détecté dans les deux modes, Scenario: Création d'une personne (+9 more)

### Community 69 - "Decisions"
Cohesion: 0.12
Nodes (15): 1. Remplacer SQLx/SQLite par `libsql` 0.9.30, qui pilote les deux modes, 2. Runner de migrations maison (`infrastructure/migrations.rs`), 3. Rewrites mécaniques des repositories, 4. Config de connexion stockée localement (`cadence_config.json`), 5. Stack pour le dev : `RUST_MIN_STACK` 512 MiB, 6. Audit des écritures (Phase 3), 7. Changement de mode : redémarrage requis, 8. Détection des modifications concurrentes (optimistic locking, Phase 3) (+7 more)

### Community 73 - "migrations.rs"
Cohesion: 0.06
Nodes (35): Change `db-driver-abstraction` — design, Context, D10 — Compatibilité ascendante pour la CI, D1 — Trait `Db` central, transactions `Box<dyn DbTransaction>`, D2 — `IntoParams` + macro `params!` symétrique à `libsql::params!`, D3 — `RowView` + `DeserializeRow` neutre, D4 — `RetryPolicy` extrait de `hrana_guard`, D5 — `ConnexionConfig` étendu pour préparer Postgres/MySQL (+27 more)

### Community 74 - "proposal.md"
Cohesion: 0.29
Nodes (6): Capabilities, Impact, Modified Capabilities, New Capabilities, What Changes, Why

### Community 75 - "tasks.md"
Cohesion: 0.33
Nodes (5): 1. Socle — dépendance libsql, connexion (mono/multi) et migrations, 2. Basculer les repositories sur libsql, 3. Audit des modifications et conflits, 4. Configuration de la connexion et choix du mode, 5. Vérifications et livraison

### Community 76 - "db.rs"
Cohesion: 0.19
Nodes (20): Adhesion, AdhesionRepository, IdRow, LibsqlAdhesionRepository, repo(), Arc, CreateAdhesion, Result (+12 more)

### Community 120 - "Requirement: Détecter les modifications concurrentes lors d'une mise à jour"
Cohesion: 0.11
Nodes (17): Purpose, Requirement: Détecter les modifications concurrentes lors d'une mise à jour, Requirement: Enregistrer l'auteur et l'horodatage de chaque création ou modification, Requirement: Fournir le nom d'utilisateur aux écritures, Requirement: Ne jamais afficher l'audit, Requirements, Scenario: Conflit détecté dans les deux modes, Scenario: Création d'une personne (+9 more)

### Community 121 - "db.rs"
Cohesion: 0.14
Nodes (16): D, DbExt, Echantillon, execute_batch_forwarded(), fausse_db(), FausseDb, fetch_one_not_found(), fetch_one_typed() (+8 more)

### Community 122 - "parametre_commands.rs"
Cohesion: 0.22
Nodes (19): apercu_creneaux_hors_plage(), modifier_plage_horaire(), obtenir_parametres_planning(), App, Connection, ImpactCreneau, MockRuntime, ParametresPlanning (+11 more)

### Community 123 - "activite.rs"
Cohesion: 0.11
Nodes (26): Display, Formatter, Activite, ActivitePersonne, CreateActivite, CreateLiaisonActivitePersonne, CreateTarifActivite, DetailActivite (+18 more)

### Community 124 - "parametre_commands.rs"
Cohesion: 0.18
Nodes (18): begin_immediate_commit(), execute_et_fetch_optional_roundtrip(), fetch_all_rows_multiple(), fetch_optional_sans_resultat_renvoie_none(), LibsqlDb, LibsqlDbTransaction, row_to_dbrow(), Box (+10 more)

### Community 125 - "personne_commands.rs"
Cohesion: 0.23
Nodes (15): creer_personne(), modifier_personne(), obtenir_detail_personne(), obtenir_personne(), rechercher_personnes(), CreatePersonne, CriteresRecherchePersonnes, Option (+7 more)

### Community 126 - "DbRow"
Cohesion: 0.33
Nodes (3): DbRow, Option, Result

### Community 127 - "params.rs"
Cohesion: 0.20
Nodes (4): (), into_params_tuple(), IntoParams, params_debug()

### Community 128 - "adhesion_commands.rs"
Cohesion: 0.38
Nodes (9): creer_adhesion(), lister_adhesions_personne(), modifier_adhesion(), Adhesion, CreateAdhesion, Result, State, UpdateAdhesion (+1 more)

### Community 129 - "row.rs"
Cohesion: 0.29
Nodes (8): entier_converti_en_bool_et_reel(), index_absent_en_erreur(), lit_les_valeurs_par_index(), null_devient_option_none(), NaiveDate, Self, Vec, type_inattendu_en_erreur()

### Community 130 - "Change `db-driver-abstraction` — proposal"
Cohesion: 0.25
Nodes (7): Capabilities, Change `db-driver-abstraction` — proposal, Impact, Modified Capabilities, New Capabilities, What Changes, Why

### Community 131 - "Change `db-driver-abstraction` — tasks"
Cohesion: 0.33
Nodes (5): 1. PR 1 — Pose des abstractions (zéro changement de comportement), 2. PR 2 — Refactor repositories + services derrière `dyn Db`, 3. PR 3 — Adoption `refinery` (optionnelle, dépend du spike R1), 4. Vérifications globales (avant livraison), Change `db-driver-abstraction` — tasks

## Knowledge Gaps
- **745 isolated node(s):** `$schema`, `plugin`, `@opencode-ai/plugin`, `$schema`, `enabled` (+740 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **30 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `AppError` connect `Formulaire Adhésion` to `adhesion_commands.rs`, `row.rs`, `Navigation`, `Pages Activités`, `Détail Personne`, `Liste Personnes`, `Type Activité`, `Type CreateLiaison`, `Type AnnéeScolaire`, `Type CurrentAnnée`, `AppError`, `adhesion.rs`, `adhesion_repo.rs`, `db.rs`, `db.rs`, `parametre_commands.rs`, `activite.rs`, `parametre_commands.rs`, `personne_commands.rs`, `DbRow`?**
  _High betweenness centrality (0.193) - this node is a cross-community bridge._
- **Why does `String` connect `activite.rs` to `adhesion_commands.rs`, `db.rs`, `adhesion_repo.rs`, `parametre.rs`, `Formulaire Adhésion`, `Navigation`, `Pages Activités`, `Détail Personne`, `Liste Personnes`, `Type Activité`, `AppError`, `Type CreateAdhesion`, `Type CreateLiaison`, `Type AnnéeScolaire`, `parametre_commands.rs`, `params.rs`, `personne_commands.rs`, `adhesion.rs`?**
  _High betweenness centrality (0.057) - this node is a cross-community bridge._
- **Why does `Db` connect `Type Activité` to `Formulaire Adhésion`, `Pages Activités`, `Détail Personne`, `db.rs`, `Type CreateLiaison`, `db.rs`, `parametre_commands.rs`, `AppError`?**
  _High betweenness centrality (0.018) - this node is a cross-community bridge._
- **What connects `$schema`, `plugin`, `@opencode-ai/plugin` to the rest of the system?**
  _758 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `App Shell` be split into smaller, more focused modules?**
  _Cohesion score 0.059499489274770175 - nodes in this community are weakly interconnected._
- **Should `Formulaire Adhésion` be split into smaller, more focused modules?**
  _Cohesion score 0.06167176350662589 - nodes in this community are weakly interconnected._
- **Should `Navigation` be split into smaller, more focused modules?**
  _Cohesion score 0.06459627329192547 - nodes in this community are weakly interconnected._