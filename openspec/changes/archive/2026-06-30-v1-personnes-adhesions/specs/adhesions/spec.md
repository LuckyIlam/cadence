## ADDED Requirements

### Requirement: Ajouter une adhésion
Le système SHALL permettre d'ajouter une adhésion pour une personne existante, avec une année scolaire (format "YYYY-YYYY"), un booléen reglee, et une note optionnelle (VARCHAR(255)).

#### Scenario: Ajout réussi
- **WHEN** l'utilisateur ajoute une adhésion pour l'année "2025-2026" à une personne existante
- **THEN** le système crée l'adhésion et la retourne

#### Scenario: Année scolaire invalide
- **WHEN** l'utilisateur saisit un format d'année non conforme
- **THEN** le système refuse avec un message d'erreur

#### Scenario: Note de paiement
- **WHEN** l'utilisateur ajoute une note comme "Chèque n°123"
- **THEN** le système enregistre la note avec l'adhésion

#### Scenario: Ajout désactivé si adhésion existante pour l'année en cours
- **WHEN** l'utilisateur consulte une personne ayant déjà une adhésion pour l'année scolaire en cours
- **THEN** le bouton "Ajouter une adhésion" est désactivé avec une infobulle "Une adhésion existe déjà pour l'année XXXX-XXXX"

### Requirement: Adhésion unique par an
Le système SHALL garantir qu'une personne ne peut avoir qu'une seule adhésion par année scolaire (contrainte UNIQUE sur personne_id + annee_scolaire).

#### Scenario: Doublon refusé
- **WHEN** l'utilisateur tente d'ajouter une deuxième adhésion pour la même personne et la même année
- **THEN** le système refuse avec un message d'erreur

### Requirement: Modifier une adhésion
Le système SHALL permettre de modifier le statut reglee et la note de paiement d'une adhésion existante.

#### Scenario: Marquer comme réglée
- **WHEN** l'utilisateur passe reglee de false à true pour une adhésion
- **THEN** le système met à jour et retourne l'adhésion

### Requirement: Lister les adhésions d'une personne
Le système SHALL retourner toutes les adhésions d'une personne, triées par année scolaire décroissante.

#### Scenario: Consultation des adhésions
- **WHEN** l'utilisateur consulte la vue détail d'une personne
- **THEN** le système affiche ses adhésions triées de la plus récente à la plus ancienne

#### Scenario: Aucune adhésion
- **WHEN** la personne n'a jamais adhéré
- **THEN** le système affiche une liste vide avec un message "Aucune adhésion"
