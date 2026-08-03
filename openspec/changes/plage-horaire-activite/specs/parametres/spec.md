## ADDED Requirements

### Requirement: Consulter la plage horaire d'ouverture des activités
Le système SHALL exposer la configuration globale de la plage horaire d'ouverture des activités (heure d'ouverture et heure de fermeture), avec une valeur par défaut de 08:00 et 20:00.

#### Scenario: Valeurs par défaut
- **WHEN** l'utilisateur consulte la configuration pour la première fois
- **THEN** le système retourne une ouverture à 08:00 et une fermeture à 20:00

### Requirement: Modifier la plage horaire d'ouverture
Le système SHALL permettre de modifier la plage horaire d'ouverture des activités, en refusant une plage invalide (format d'heure incorrect ou ouverture après/égale à la fermeture).

#### Scenario: Modification réussie
- **WHEN** l'utilisateur définit une ouverture à 09:00 et une fermeture à 18:00
- **THEN** le système enregistre la nouvelle plage et la retourne

#### Scenario: Ouverture après fermeture
- **WHEN** l'utilisateur tente de définir une ouverture à 20:00 et une fermeture à 08:00
- **THEN** le système refuse avec un message explicite

#### Scenario: Format d'heure invalide
- **WHEN** l'utilisateur tente de définir une heure au format non HH:MM
- **THEN** le système refuse avec un message explicite

### Requirement: Apercevoir l'impact d'une réduction de plage
Le système SHALL exposer, sans modifier la base, la liste des créneaux qui sortent de la plage proposée (heure de début avant l'ouverture ou heure de fin après la fermeture), avec pour chacun l'action qui serait appliquée : suppression (créneau sans inscrit), déplacement vers la place libre la plus proche (créneau avec inscrits) ou déplacement impossible (aucune place libre dans la plage, même jour).

#### Scenario: Aperçu avec suppressions et déplacements
- **WHEN** l'utilisateur propose une réduction de plage et qu'un créneau sans inscrit sort de la plage et qu'un créneau avec inscrits sort de la plage avec une place libre disponible
- **THEN** le système retourne deux impacts : le premier en « suppression », le second en « déplacement » avec ses nouveaux horaires
- **THEN** les déplacements retenus ne chevauchent ni les créneaux de la même activité qui restent en place ni les autres créneaux déplacés de la même activité lors de la même opération, pour la même année scolaire et le même jour

#### Scenario: Aperçu sans impact
- **WHEN** l'utilisateur propose une plage qui couvre tous les créneaux existants
- **THEN** le système retourne une liste vide

### Requirement: Réduire la plage horaire en gérant les créneaux impactés
Le système SHALL appliquer une réduction de plage en transaction : suppression des créneaux sortant de la plage sans inscrit, déplacement des créneaux sortant avec inscrits vers la place libre la plus proche du même jour, puis mise à jour de la plage. Une réduction qui impacte au moins un créneau SHALL être refusée sans confirmation explicite, et une réduction SHALL être refusée si un créneau avec inscrits ne peut pas être déplacé.

#### Scenario: Réduction confirmée
- **WHEN** l'utilisateur confirme une réduction qui supprime un créneau sans inscrit et déplace un créneau avec inscrits
- **THEN** le système supprime et déplace les créneaux correspondants, puis enregistre la nouvelle plage
- **THEN** le tout est appliqué de façon atomique (en cas d'échec, aucune modification n'est conservée)

#### Scenario: Réduction refusée sans confirmation
- **WHEN** l'utilisateur tente d'appliquer une réduction qui impacte au moins un créneau sans confirmer
- **THEN** le système refuse avec un message indiquant le nombre de créneaux à supprimer et à déplacer et demandant une confirmation

#### Scenario: Réduction impossible
- **WHEN** un créneau avec inscrits sort de la plage et ne trouve aucune place libre dans la nouvelle plage pour le même jour
- **THEN** le système refuse la réduction avec un message indiquant le créneau concerné et proposant d'élargir la plage ou de retirer les inscrits

#### Scenario: Déplacement refusé si chevauchement avec une autre activité de l'adhérent
- **WHEN** un adhérent du créneau à déplacer est déjà inscrit à une autre activité (même année) dont un créneau — à l'état final, le même jour — chevauche le nouvel horaire
- **THEN** le système marque le déplacement comme impossible et refuse la réduction, avec un message mentionnant l'activité en conflit
