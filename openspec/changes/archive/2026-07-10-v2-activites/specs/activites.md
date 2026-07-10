## ADDED Requirements

### Requirement: Créer une activité
Le système SHALL permettre de créer une activité avec un nom (requis), une description (optionnelle), une capacité maximale (optionnelle) et un tarif par année scolaire.

#### Scenario: Création réussie
- **WHEN** l'utilisateur saisit un nom d'activité et une année scolaire
- **THEN** le système crée l'activité avec un tarif à 0€ pour cette année

#### Scenario: Création avec tarif personnalisé
- **WHEN** l'utilisateur saisit un nom, une année et un tarif
- **THEN** le système crée l'activité avec le tarif saisi

#### Scenario: Création refusée si nom vide
- **WHEN** l'utilisateur tente de créer une activité sans nom
- **THEN** le système refuse avec un message d'erreur

### Requirement: Modifier une activité
Le système SHALL permettre de modifier le nom, la description et la capacité maximale d'une activité existante.

#### Scenario: Modification réussie
- **WHEN** l'utilisateur modifie le nom et la description d'une activité
- **THEN** le système met à jour l'activité

### Requirement: Lister les activités avec filtre par année scolaire
Le système SHALL retourner la liste des activités filtrée par année scolaire, triée par nom.

#### Scenario: Filtre par année
- **WHEN** l'utilisateur consulte la liste des activités
- **THEN** le système affiche un sélecteur d'année avec les années disponibles
- **THEN** seules les activités ayant un tarif pour l'année sélectionnée sont affichées

#### Scenario: Aucune activité
- **WHEN** aucune activité n'existe pour l'année sélectionnée
- **THEN** le système affiche une liste vide

### Requirement: Définir le tarif d'une activité pour une année scolaire
Le système SHALL permettre d'ajouter ou modifier le tarif d'une activité pour une année scolaire donnée. Un historique des tarifs est conservé.

#### Scenario: Ajout de tarif
- **WHEN** l'utilisateur définit un tarif pour une activité et une année
- **THEN** le système enregistre le tarif sans affecter les autres années

### Requirement: Ajouter une personne à une activité
Le système SHALL permettre d'ajouter une personne à une activité avec un rôle (`encadrant` ou `participant`).

#### Scenario: Ajout participant
- **WHEN** l'utilisateur ajoute une personne comme participante
- **THEN** le système crée la liaison avec le rôle "participant"

#### Scenario: Ajout encadrant
- **WHEN** l'utilisateur ajoute une personne comme encadrante
- **THEN** le système crée la liaison avec le rôle "encadrant"

#### Scenario: Ajout refusé si capacité atteinte
- **WHEN** l'activité a une capacité maximale déjà atteinte
- **THEN** le système refuse l'ajout d'un nouveau participant

#### Scenario: Ajout refusé si doublon
- **WHEN** la personne est déjà liée à cette activité
- **THEN** le système refuse avec un message explicite

#### Scenario: Ajout refusé si double rôle
- **WHEN** une personne est déjà encadrante et qu'on tente de l'ajouter comme participante (ou inversement)
- **THEN** le système refuse avec un message explicite

### Requirement: Supprimer une personne d'une activité
Le système SHALL permettre de retirer une personne d'une activité.

#### Scenario: Retrait réussi
- **WHEN** l'utilisateur retire une personne d'une activité
- **THEN** le système supprime la liaison

### Requirement: Consulter le détail d'une activité
Le système SHALL afficher toutes les informations d'une activité, y compris la liste des participants et des encadrants.

#### Scenario: Consultation
- **WHEN** l'utilisateur clique sur une activité
- **THEN** le système affiche le nom, la description, la capacité, le tarif
- **THEN** le système affiche deux sections : "Encadrants" et "Participants"

## MODIFIED Requirements

*Aucune modification sur une capability existante.*

## REMOVED Requirements

*Aucune suppression.*

## RENAMED Requirements

*Aucun renommage.*
