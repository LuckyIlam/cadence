## ADDED Requirements

### Requirement: Créer un créneau horaire pour une activité
Le système SHALL permettre d'ajouter un créneau horaire récurrent à une activité pour une année scolaire donnée. Un créneau est défini par un jour de la semaine (1=lundi…7=dimanche), une heure de début et une heure de fin (format HH:MM). Une activité peut avoir plusieurs créneaux pour une même année.

#### Scenario: Création réussie
- **WHEN** l'utilisateur ajoute un créneau lundi 14:00-16:00 à l'activité "Poterie" pour l'année 2025-2026
- **THEN** le système crée le créneau et l'associe à l'activité

#### Scenario: Création avec heure_debut > heure_fin
- **WHEN** l'utilisateur tente de créer un créneau avec heure_debut > heure_fin
- **THEN** le système refuse avec un message explicite

#### Scenario: Création avec jour_semaine invalide
- **WHEN** l'utilisateur tente de créer un créneau avec jour_semaine = 0
- **THEN** le système refuse avec un message explicite

#### Scenario: Ajout d'un deuxième créneau à la même activité
- **WHEN** l'utilisateur ajoute un créneau mercredi 10:00-12:00 à l'activité "Poterie" (qui a déjà un créneau lundi 14:00-16:00)
- **THEN** le système crée le deuxième créneau sans erreur

### Requirement: Supprimer un créneau horaire
Le système SHALL permettre de supprimer un créneau horaire d'une activité, à condition qu'aucune personne ne soit inscrite à l'activité pour l'année scolaire concernée.

#### Scenario: Suppression réussie
- **WHEN** l'activité "Poterie" n'a aucun inscrit pour 2025-2026 et l'utilisateur supprime un de ses créneaux
- **THEN** le système supprime le créneau

#### Scenario: Suppression refusée si inscrits
- **WHEN** l'activité "Poterie" a au moins un inscrit pour 2025-2026 et l'utilisateur tente de supprimer un créneau
- **THEN** le système refuse avec un message indiquant qu'il faut d'abord retirer les inscrits

### Requirement: Modifier un créneau horaire
Le système SHALL permettre de modifier le jour, l'heure de début ou l'heure de fin d'un créneau, à condition qu'aucune personne ne soit inscrite à l'activité pour l'année scolaire concernée.

#### Scenario: Modification réussie
- **WHEN** l'activité "Poterie" n'a aucun inscrit pour 2025-2026 et l'utilisateur modifie l'heure de fin d'un créneau
- **THEN** le système met à jour le créneau

#### Scenario: Modification refusée si inscrits
- **WHEN** l'activité "Poterie" a au moins un inscrit pour 2025-2026 et l'utilisateur tente de modifier un créneau
- **THEN** le système refuse avec un message indiquant qu'il faut d'abord retirer les inscrits

### Requirement: Ajouter un créneau quand des inscrits existent
Le système SHALL permettre d'ajouter un nouveau créneau à une activité même si des personnes y sont déjà inscrites.

#### Scenario: Ajout d'un créneau supplémentaire
- **WHEN** l'activité "Poterie" a 5 participants pour 2025-2026 et l'utilisateur ajoute un nouveau créneau mercredi 10:00-12:00
- **THEN** le système crée le créneau sans erreur

### Requirement: Lister les créneaux d'une activité
Le système SHALL retourner la liste des créneaux d'une activité pour une année scolaire donnée, triés par jour_semaine puis heure_debut.

#### Scenario: Consultation des créneaux
- **WHEN** l'utilisateur consulte les créneaux de l'activité "Poterie" pour 2025-2026
- **THEN** le système affiche tous les créneaux triés par jour puis heure

#### Scenario: Aucun créneau
- **WHEN** l'activité "Poterie" n'a pas de créneau défini pour 2025-2026
- **THEN** le système affiche une liste vide

### Requirement: Ajouter une semaine banalisée
Le système SHALL permettre de marquer une semaine comme banalisée (sans activité) pour une activité donnée. Une semaine banalisée est définie par sa date de début (lundi) et peut avoir un motif optionnel.

#### Scenario: Ajout réussi
- **WHEN** l'utilisateur ajoute une semaine banalisée au 2025-12-22 (lundi) pour l'activité "Poterie" avec motif "Vacances de Noël"
- **THEN** le système enregistre la semaine banalisée

#### Scenario: Ajout sans motif
- **WHEN** l'utilisateur ajoute une semaine banalisée sans motif
- **THEN** le système enregistre la semaine banalisée avec motif null

#### Scenario: Ajout avec une date qui n'est pas un lundi
- **WHEN** l'utilisateur tente d'ajouter une semaine banalisée avec une date qui n'est pas un lundi
- **THEN** le système refuse avec un message explicite

### Requirement: Supprimer une semaine banalisée
Le système SHALL permettre de supprimer une semaine banalisée d'une activité.

#### Scenario: Suppression réussie
- **WHEN** l'utilisateur supprime une semaine banalisée de l'activité "Poterie"
- **THEN** le système supprime l'enregistrement

### Requirement: Lister les semaines banalisées d'une activité
Le système SHALL retourner la liste des semaines banalisées d'une activité pour l'année scolaire en cours, triées par date_debut.

#### Scenario: Consultation
- **WHEN** l'utilisateur consulte les semaines banalisées de l'activité "Poterie"
- **THEN** le système affiche la liste triée par date

### Requirement: Détecter les collisions horaires à l'inscription
Le système SHALL vérifier, lors de l'ajout d'une personne à une activité, que les créneaux de cette activité ne chevauchent pas ceux d'une autre activité où la personne est déjà inscrite (quel que soit le rôle). En cas de collision, le système refuse l'inscription.

#### Scenario: Pas de collision
- **WHEN** une personne n'a aucune activité le lundi 14:00-16:00 et qu'on l'ajoute comme participante à "Poterie" (créneau lundi 14:00-16:00)
- **THEN** le système accepte l'inscription

#### Scenario: Collision avec un encadrement existant
- **WHEN** une personne est déjà encadrante de "Théâtre" (lundi 14:00-16:00) et qu'on tente de l'ajouter comme participante à "Poterie" (lundi 14:00-16:00)
- **THEN** le système refuse avec un message indiquant le conflit avec "Théâtre"

#### Scenario: Collision avec une participation existante
- **WHEN** une personne participe déjà à "Couture" (mercredi 10:00-12:00) et qu'on tente de l'ajouter comme encadrante à "Danse" (mercredi 10:30-12:30)
- **THEN** le système refuse avec un message indiquant le conflit avec "Couture"

#### Scenario: Pas de collision si même activité
- **WHEN** l'utilisateur change le rôle d'une personne déjà inscrite à une activité
- **THEN** pas de vérification de collision (l'utilisateur est déjà dans l'activité)

#### Scenario: Pas de collision si créneaux différents
- **WHEN** une personne est inscrite à "Poterie" (lundi 14:00-16:00) et qu'on l'ajoute à "Théâtre" (mercredi 10:00-12:00)
- **THEN** le système accepte l'inscription

### Requirement: Consulter le planning hebdomadaire d'une personne
Le système SHALL afficher le planning hebdomadaire d'une personne pour une semaine donnée, montrant tous les créneaux (activité + rôle) où elle est inscrite, à l'exclusion des semaines banalisées.

#### Scenario: Planning avec activités
- **WHEN** l'utilisateur consulte le planning de la personne X pour la semaine du 2025-09-01
- **THEN** le système affiche les créneaux de toutes les activités où X est inscrite pour cette semaine non banalisée

#### Scenario: Semaine banalisée
- **WHEN** l'utilisateur consulte le planning pour une semaine marquée comme banalisée pour une activité
- **THEN** les créneaux de cette activité n'apparaissent pas dans le planning

#### Scenario: Aucune activité
- **WHEN** la personne n'est inscrite à aucune activité
- **THEN** la grille hebdomadaire est vide

### Requirement: Naviguer entre les semaines dans le planning
Le système SHALL permettre de naviguer entre les semaines dans la vue planning, avec affichage du numéro de semaine ISO et de la date du lundi.

#### Scenario: Navigation semaine suivante
- **WHEN** l'utilisateur clique sur "Semaine suivante"
- **THEN** le système affiche la semaine suivante avec le numéro de semaine et la date du lundi mis à jour

#### Scenario: Navigation semaine précédente
- **WHEN** l'utilisateur clique sur "Semaine précédente"
- **THEN** le système affiche la semaine précédente

### Requirement: Afficher la grille hebdomadaire
Le système SHALL afficher le planning sous forme d'une grille hebdomadaire allant du lundi au dimanche, de 8h à 20h, avec les créneaux positionnés aux horaires correspondants.

#### Scenario: Affichage de la grille
- **WHEN** l'utilisateur consulte un planning avec des créneaux
- **THEN** les créneaux sont affichés comme des blocs positionnés sur la grille à l'intersection du jour et de l'heure
- **THEN** chaque bloc affiche le nom de l'activité et le rôle de la personne
