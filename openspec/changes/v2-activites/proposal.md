## Why

La V1 (personnes + adhésions) est terminée. Le cœur de Cadence — gérer qui sont les membres et leur statut d'adhésion — est en place. La prochaine brique logique est de gérer *ce que fait l'association* : ses activités. Ateliers, cours, entraînements, sorties — chaque association a ses propres activités récurrentes, avec des participants et des encadrants.

## What Changes

- Nouveau module **Activités** : créer, modifier, lister, consulter des activités
- Une activité a un nom, une description optionnelle, une capacité maximale optionnelle
- Tarif variable par année scolaire (table dédiée pour garder l'historique)
- Lien N:N entre personnes et activités avec un rôle (encadrant ou participant)
- Une contrainte empêche une personne d'être à la fois encadrante ET participante pour une même activité
- Navigation : introduction d'un menu de navigation entre "Personnes" et "Activités"
- Page liste des activités avec filtre par année scolaire
- Page détail d'une activité avec ses participants et encadrants
- Section "Activités" dans le détail d'une personne

## Capabilities

### New Capabilities

- `activites`: Création, modification, liste et consultation des activités, avec gestion des tarifs par année scolaire et des inscriptions (participants/encadrants)

### Modified Capabilities

- `personnes`: Ajout d'une section "Activités" dans le détail d'une personne listant ses participations et encadrements
- Navigation: introduction d'un menu principal pour basculer entre les vues "Personnes" et "Activités"

## Impact

- Nouvelles migrations SQL (3 tables : `activites`, `tarifs_activite`, `activite_personnes`)
- Nouveau domaine Rust : `src-tauri/src/domain/activite.rs`
- Nouveau repository : `src-tauri/src/repositories/activite_repo.rs`
- Nouvelles commandes Tauri : `src-tauri/src/commands/activite_commands.rs`
- Nouvelle page frontend `Activites.tsx` et composants associés
- Modification de `App.tsx` pour ajouter le menu de navigation et les routes
- Modification de `DetailPersonne.tsx` pour afficher les activités liées
