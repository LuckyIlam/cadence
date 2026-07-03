## Purpose

Gérer les activités d'une association : création, modification, consultation, inscription des personnes (participants et encadrants). Chaque activité peut avoir un tarif différent chaque année scolaire.

## Requirements

### Requirement: Créer une activité
Le système SHALL permettre de créer une activité avec un nom (requis), une description (optionnelle), une capacité maximale (optionnelle), une année scolaire (requise, choix entre l'année courante et la suivante) et un tarif (optionnel). Si l'année est fournie, un tarif par défaut de 0€ est enregistré pour cette année (modifiable ensuite). Si un tarif est saisi, il est utilisé à la place du défaut.

#### Scenario: Création réussie
- **WHEN** l'utilisateur saisit un nom d'activité et sélectionne l'année courante
- **THEN** le système crée l'activité avec un tarif à 0€ pour cette année

#### Scenario: Création avec tarif
- **WHEN** l'utilisateur saisit un nom, une année et un tarif de 200€
- **THEN** le système crée l'activité avec un tarif à 200€ pour cette année

#### Scenario: Création avec description et capacité
- **WHEN** l'utilisateur saisit un nom, une description, et une capacité maximale de 20
- **THEN** le système crée l'activité avec ces informations

#### Scenario: Création pour l'année suivante
- **WHEN** l'utilisateur crée une activité et sélectionne l'année N+1
- **THEN** le système crée l'activité et un tarif pour l'année N+1

#### Scenario: Création refusée si nom vide
- **WHEN** l'utilisateur tente de créer une activité sans nom
- **THEN** le système refuse avec un message d'erreur explicite

### Requirement: Modifier une activité
Le système SHALL permettre de modifier le nom, la description et la capacité maximale d'une activité existante.

#### Scenario: Modification réussie
- **WHEN** l'utilisateur modifie le nom et la description d'une activité
- **THEN** le système met à jour l'activité et retourne ses nouvelles informations

### Requirement: Lister les activités
Le système SHALL retourner la liste des activités, triée par nom.

#### Scenario: Liste complète
- **WHEN** l'utilisateur consulte la liste des activités
- **THEN** le système retourne toutes les activités triées par ordre alphabétique

#### Scenario: Aucune activité
- **WHEN** aucune activité n'existe
- **THEN** le système affiche une liste vide avec un message

### Requirement: Filtrer les activités par année scolaire
Le système SHALL permettre de filtrer la liste des activités par année scolaire. Le filtre n'affiche que les années pour lesquelles au moins une activité a un tarif défini (présentes dans la table `tarifs_activite`). Par défaut, la première année disponible est sélectionnée.

#### Scenario: Filtre année par défaut
- **WHEN** l'utilisateur consulte la liste des activités
- **THEN** le sélecteur d'année affiche les années ayant des tarifs enregistrés
- **THEN** la première année disponible est sélectionnée
- **THEN** seules les activités ayant un tarif pour cette année sont affichées

#### Scenario: Changement d'année
- **WHEN** l'utilisateur sélectionne une autre année scolaire dans le filtre
- **THEN** le système affiche les activités ayant un tarif pour cette année

#### Scenario: Aucune année disponible
- **WHEN** aucune activité n'a de tarif défini
- **THEN** le sélecteur d'année n'est pas affiché
- **THEN** la liste est vide avec un message

### Requirement: Définir le tarif d'une activité pour une année scolaire
Le système SHALL permettre d'ajouter ou modifier le tarif d'une activité pour une année scolaire donnée. Un historique des tarifs est conservé.

#### Scenario: Ajout de tarif
- **WHEN** l'utilisateur définit un tarif de 200€ pour l'activité "Poterie" pour l'année 2025-2026
- **THEN** le système enregistre le tarif

#### Scenario: Modification de tarif pour une autre année
- **WHEN** l'utilisateur définit un tarif de 220€ pour la même activité pour l'année 2026-2027
- **THEN** le système enregistre le nouveau tarif sans affecter l'ancien

### Requirement: Ajouter une personne à une activité
Le système SHALL permettre d'ajouter une personne à une activité avec un rôle (`encadrant` ou `participant`).

#### Scenario: Ajout participant (depuis le bloc Participants)
- **WHEN** l'utilisateur clique sur "Ajouter" dans le bloc Participants
- **THEN** le panneau de recherche s'ouvre dans le bloc Participants
- **WHEN** l'utilisateur sélectionne une personne dans les résultats
- **THEN** le système crée la liaison avec le rôle "participant"

#### Scenario: Ajout encadrant (depuis le bloc Encadrants)
- **WHEN** l'utilisateur clique sur "Ajouter" dans le bloc Encadrants
- **THEN** le panneau de recherche s'ouvre dans le bloc Encadrants
- **WHEN** l'utilisateur sélectionne une personne dans les résultats
- **THEN** le système crée la liaison avec le rôle "encadrant"

#### Scenario: Ajout refusé si capacité atteinte
- **WHEN** l'activité a une capacité maximale de 10 et qu'il y a déjà 10 participants
- **THEN** le système refuse l'ajout d'un nouveau participant avec un message explicite

#### Scenario: Ajout refusé si doublon
- **WHEN** la personne est déjà liée à cette activité (même rôle ou autre)
- **THEN** le système refuse avec un message explicite (contrainte UNIQUE)

#### Scenario: Ajout refusé si une personne est déjà encadrante d'une activité et qu'on tente de l'ajouter comme participante (et inversement)
- **WHEN** une personne est déjà encadrante de l'activité "Poterie" et qu'on tente de l'ajouter comme participante à la même activité
- **THEN** le système refuse avec un message explicite

### Requirement: Supprimer une personne d'une activité
Le système SHALL permettre de retirer une personne d'une activité (suppression de la liaison).

#### Scenario: Retrait réussi
- **WHEN** l'utilisateur retire une personne d'une activité
- **THEN** le système supprime la liaison

### Requirement: Consulter le détail d'une activité
Le système SHALL afficher toutes les informations d'une activité, y compris la liste des participants et des encadrants, triés par nom.

#### Scenario: Consultation
- **WHEN** l'utilisateur clique sur une activité
- **THEN** le système affiche le nom, la description, la capacité, le tarif pour l'année sélectionnée
- **THEN** le système affiche deux sections : "Encadrants" et "Participants"
- **THEN** chaque section liste les personnes triées par nom, prénom

#### Scenario: Tarif non défini
- **WHEN** l'activité n'a pas de tarif pour l'année sélectionnée
- **THEN** le système affiche "Tarif non défini" dans la zone tarif

### Requirement: Afficher les activités d'une personne
Le système SHALL afficher, dans le détail d'une personne, la liste des activités auxquelles elle participe ou qu'elle encadre (toutes années confondues).

#### Scenario: Consultation personne
- **WHEN** l'utilisateur consulte le détail d'une personne
- **THEN** le système affiche une section "Activités" avec deux sous-listes : "En tant qu'encadrant·e" et "En tant que participant·e"

#### Scenario: Activités sur plusieurs années
- **WHEN** une personne est inscrite à des activités sur différentes années scolaires
- **THEN** la section affiche toutes les activités sans filtre d'année

### Requirement: Navigation
Le système SHALL offrir un menu de navigation permettant de basculer entre la liste des personnes et la liste des activités.

#### Scenario: Menu visible
- **WHEN** l'utilisateur ouvre l'application
- **THEN** un menu de navigation est affiché avec les entrées "Personnes" et "Activités"
- **THEN** l'entrée active est mise en évidence
