## MODIFIED Requirements

### Requirement: Créer un créneau horaire pour une activité
Le système SHALL permettre d'ajouter un créneau horaire récurrent à une activité pour une année scolaire donnée. Un créneau est défini par un jour de la semaine (1=lundi…7=dimanche), une heure de début et une heure de fin (format HH:MM). Une activité peut avoir plusieurs créneaux pour une même année. Le créneau doit être entièrement compris dans la plage horaire d'ouverture configurée des activités.

#### Scenario: Création réussie
- **WHEN** l'utilisateur ajoute un créneau lundi 14:00-16:00 à l'activité "Poterie" pour l'année 2025-2026 et que ce créneau est compris dans la plage d'ouverture configurée
- **THEN** le système crée le créneau et l'associe à l'activité

#### Scenario: Création avec heure_debut > heure_fin
- **WHEN** l'utilisateur tente de créer un créneau avec heure_debut > heure_fin
- **THEN** le système refuse avec un message explicite

#### Scenario: Création avec jour_semaine invalide
- **WHEN** l'utilisateur tente de créer un créneau avec jour_semaine = 0
- **THEN** le système refuse avec un message explicite

#### Scenario: Créneau avant l'ouverture
- **WHEN** l'utilisateur tente de créer un créneau dont l'heure de début est avant l'heure d'ouverture configurée (ex : 07:00 alors que l'ouverture est 08:00)
- **THEN** le système refuse avec un message indiquant que le créneau doit être compris entre l'ouverture et la fermeture

#### Scenario: Créneau après la fermeture
- **WHEN** l'utilisateur tente de créer un créneau dont l'heure de fin est après l'heure de fermeture configurée (ex : 21:00 alors que la fermeture est 20:00)
- **THEN** le système refuse avec un message indiquant que le créneau doit être compris entre l'ouverture et la fermeture

#### Scenario: Créneau aux bornes exactes
- **WHEN** l'utilisateur crée un créneau démarrant exactement à l'ouverture et se terminant exactement à la fermeture (ex : 08:00-20:00)
- **THEN** le système accepte la création

#### Scenario: Ajout d'un deuxième créneau à la même activité
- **WHEN** l'utilisateur ajoute un créneau mercredi 10:00-12:00 à l'activité "Poterie" (qui a déjà un créneau lundi 14:00-16:00)
- **THEN** le système crée le deuxième créneau sans erreur

### Requirement: Modifier un créneau horaire
Le système SHALL permettre de modifier le jour, l'heure de début ou l'heure de fin d'un créneau, à condition qu'aucune personne ne soit inscrite à l'activité pour l'année scolaire concernée. Le créneau modifié doit rester entièrement compris dans la plage horaire d'ouverture configurée.

#### Scenario: Modification réussie
- **WHEN** l'activité "Poterie" n'a aucun inscrit pour 2025-2026 et l'utilisateur modifie l'heure de fin d'un créneau en restant dans la plage configurée
- **THEN** le système met à jour le créneau

#### Scenario: Modification hors plage
- **WHEN** l'utilisateur tente de modifier un créneau pour le sortir de la plage d'ouverture configurée
- **THEN** le système refuse avec un message explicite

#### Scenario: Modification refusée si inscrits
- **WHEN** l'activité "Poterie" a au moins un inscrit pour 2025-2026 et l'utilisateur tente de modifier un créneau
- **THEN** le système refuse avec un message indiquant qu'il faut d'abord retirer les inscrits

### Requirement: Afficher la grille hebdomadaire
Le système SHALL afficher le planning sous forme d'une grille hebdomadaire allant du lundi au dimanche, bornée par la plage horaire d'ouverture configurée des activités (par défaut 8h à 20h), avec les créneaux positionnés aux horaires correspondants.

#### Scenario: Affichage de la grille
- **WHEN** l'utilisateur consulte un planning avec des créneaux
- **THEN** la grille s'affiche entre l'heure d'ouverture et l'heure de fermeture configurées
- **THEN** les créneaux sont affichés comme des blocs positionnés sur la grille à l'intersection du jour et de l'heure
- **THEN** chaque bloc affiche le nom de l'activité et le rôle de la personne

#### Scenario: Grille sur plage modifiée
- **WHEN** la plage d'ouverture a été modifiée (ex : 09:00–18:00)
- **THEN** la grille s'affiche entre 09:00 et 18:00
