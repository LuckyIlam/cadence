# Graph Report - .  (2026-07-09)

## Corpus Check
- 131 files · ~61,691 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 86 nodes · 75 edges · 38 communities (5 shown, 33 thin omitted)
- Extraction: 52% EXTRACTED · 48% INFERRED · 0% AMBIGUOUS · INFERRED: 36 edges (avg confidence: 0.91)
- Token cost: 0 input · 0 output

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
- Type UpdatePersonne

## God Nodes (most connected - your core abstractions)
1. `Module Activités` - 9 edges
2. `OpenSpec Experimental Change Workflow` - 7 edges
3. `Cadence` - 7 edges
4. `Personne physique` - 7 edges
5. `V2 Module Activités` - 6 edges
6. `AGENTS.md Development Rules` - 5 edges
7. `Cadence Desktop Application` - 5 edges
8. `Pre-submission Verification Checklist` - 5 edges
9. `Handle Issue Skill` - 4 edges
10. `Implement Feature Skill` - 4 edges

## Surprising Connections (you probably didn't know these)
- `CI Workflow` --references--> `Pre-submission Verification Checklist`  [INFERRED]
  .github/workflows/ci.yml → AGENTS.md
- `Cadence Prepare Release Skill` --references--> `Cadence Desktop Application`  [INFERRED]
  .opencode/skills/cadence-prepare-release/SKILL.md → README.md
- `Cadence Prepare Release Skill` --references--> `Pre-submission Verification Checklist`  [INFERRED]
  .opencode/skills/cadence-prepare-release/SKILL.md → AGENTS.md
- `Application entry point (index.html)` --conceptually_related_to--> `Cadence`  [INFERRED]
  index.html → docs/fonctionnel/README.md
- `Architecture en couches domain/repositories/commands` --rationale_for--> `Cadence`  [INFERRED]
  openspec/changes/archive/2026-06-30-v1-personnes-adhesions/design.md → docs/fonctionnel/README.md

## Import Cycles
- None detected.

## Hyperedges (group relationships)
- **Pre-release Verification Pipeline** — verification_checklist, github_workflows_ci, opencode_skills_cadence_prepare_release_skill, opencode_skills_handle_issue_skill, opencode_skills_implement_feature_skill [INFERRED 0.85]
- **OpsX-OpenSpec Command-to-Skill Mapping** — opencode_commands_opsx_apply, opencode_skills_openspec_apply_change_skill, opencode_commands_opsx_archive, opencode_skills_openspec_archive_change_skill, opencode_commands_opsx_explore, opencode_skills_openspec_explore_skill, opencode_commands_opsx_propose, opencode_skills_openspec_propose_skill, opencode_commands_opsx_sync, opencode_skills_openspec_sync_specs_skill, openspec_workflow [INFERRED 0.85]
- **Inscription à une activité** — docs_fonctionnel_activites_activite, docs_fonctionnel_readme_personne_physique, docs_fonctionnel_activites_participant, docs_fonctionnel_activites_encadrant, docs_fonctionnel_readme_annee_scolaire, docs_fonctionnel_activites_tarif, docs_fonctionnel_activites_regle_double_role [INFERRED 0.85]
- **Cycle adhésion annuelle** — docs_fonctionnel_readme_personne_physique, docs_fonctionnel_readme_adhesion, docs_fonctionnel_readme_annee_scolaire, docs_fonctionnel_adhesions_regle_adhesion_unique [INFERRED 0.85]

## Communities (38 total, 33 thin omitted)

### Community 0 - "Membres & Adhésions"
Cohesion: 0.21
Nodes (14): Module Adhésions, Règle une seule adhésion par année scolaire, Module Personnes, Règle validité date de naissance (post-1920, pas future), Règle mineur et responsable légal, Adhésion, Cadence, Personne physique (+6 more)

### Community 1 - "OpenSpec Workflow"
Cohesion: 0.18
Nodes (13): Explore Mode Thinking Stance, Intelligent Merging for Delta Specs, OpsX Apply Command, OpsX Archive Command, OpsX Explore Command, OpsX Propose Command, OpsX Sync Command, OpenSpec Apply Change Skill (+5 more)

### Community 2 - "Activités & Tarifs"
Cohesion: 0.26
Nodes (12): Activité, Encadrant, Module Activités, Participant, Règle pas de double rôle encadrant/participant, Tarif, Année scolaire, Navigation par menu React Router (Personnes/Activités) (+4 more)

### Community 3 - "Dev Setup & Conventions"
Cohesion: 0.33
Nodes (10): AGENTS.md Development Rules, Cadence Desktop Application, Error Handling Rules — No expect, crash log, double write, Cadence Prepare Release Skill, Handle Issue Skill, Implement Feature Skill, README.md Project Overview, Mandatory Unit Tests for New Business Functions (+2 more)

### Community 4 - "CI/CD Pipelines"
Cohesion: 0.50
Nodes (4): CI Workflow, CI Verification Pipeline, Release Workflow, Release Pipeline

## Knowledge Gaps
- **43 isolated node(s):** `App`, `AdhesionForm`, `Nav`, `Activites`, `DetailPersonne` (+38 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **33 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Module Activités` connect `Activités & Tarifs` to `Membres & Adhésions`?**
  _High betweenness centrality (0.031) - this node is a cross-community bridge._
- **Why does `Cadence` connect `Membres & Adhésions` to `Activités & Tarifs`?**
  _High betweenness centrality (0.029) - this node is a cross-community bridge._
- **Why does `Personne physique` connect `Membres & Adhésions` to `Activités & Tarifs`?**
  _High betweenness centrality (0.019) - this node is a cross-community bridge._
- **Are the 2 inferred relationships involving `Module Activités` (e.g. with `Navigation par menu React Router (Personnes/Activités)` and `Table unique activite_personnes avec rôle et année`) actually correct?**
  _`Module Activités` has 2 INFERRED edges - model-reasoned connections that need verification._
- **Are the 2 inferred relationships involving `OpenSpec Experimental Change Workflow` (e.g. with `Explore Mode Thinking Stance` and `Intelligent Merging for Delta Specs`) actually correct?**
  _`OpenSpec Experimental Change Workflow` has 2 INFERRED edges - model-reasoned connections that need verification._
- **Are the 3 inferred relationships involving `Cadence` (e.g. with `Application entry point (index.html)` and `Architecture en couches domain/repositories/commands`) actually correct?**
  _`Cadence` has 3 INFERRED edges - model-reasoned connections that need verification._
- **Are the 3 inferred relationships involving `Personne physique` (e.g. with `Encadrant` and `Participant`) actually correct?**
  _`Personne physique` has 3 INFERRED edges - model-reasoned connections that need verification._