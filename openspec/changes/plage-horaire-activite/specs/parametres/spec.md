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
