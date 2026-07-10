## ADDED Requirements

### Requirement: Navigation entre personnes et activités
Le système SHALL offrir un menu de navigation permettant de basculer entre la liste des personnes et la liste des activités.

#### Scenario: Menu visible
- **WHEN** l'utilisateur ouvre l'application
- **THEN** un menu de navigation est affiché avec les entrées "Personnes" et "Activités"
- **THEN** l'entrée active est mise en évidence

## MODIFIED Requirements

### Requirement: Consulter le détail d'une personne
Ajout d'une section "Activités" listant les activités auxquelles la personne participe ou qu'elle encadre.

#### Scenario: Consultation avec activités
- **WHEN** l'utilisateur consulte le détail d'une personne
- **THEN** le système affiche une section "Activités" avec deux sous-listes : "En tant qu'encadrant·e" et "En tant que participant·e"

## REMOVED Requirements

*Aucune suppression.*

## RENAMED Requirements

*Aucun renommage.*
