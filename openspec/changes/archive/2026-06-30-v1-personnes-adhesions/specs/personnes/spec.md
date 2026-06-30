## ADDED Requirements

### Requirement: Créer une personne physique
Le système SHALL permettre de créer une personne physique avec les champs : nom, prénom, date de naissance, email (optionnel), téléphone (optionnel), responsable_id (optionnel).

#### Scenario: Création réussie
- **WHEN** l'utilisateur saisit nom, prénom et date de naissance valides
- **THEN** le système crée la personne et retourne ses informations

#### Scenario: Création avec responsable mineur
- **WHEN** l'utilisateur saisit une personne de moins de 18 ans avec un responsable_id valide pointant vers une personne majeure
- **THEN** le système crée la personne avec le lien responsable

#### Scenario: Création refusée si mineur sans responsable
- **WHEN** l'utilisateur saisit une personne de moins de 18 ans sans responsable_id
- **THEN** le système refuse la création avec un message d'erreur explicite

#### Scenario: Création refusée si responsable est lui-même mineur
- **WHEN** le responsable_id pointe vers une personne de moins de 18 ans
- **THEN** le système refuse la création avec un message d'erreur

#### Scenario: Création refusée si date avant 1920
- **WHEN** l'utilisateur saisit une date de naissance antérieure à 1920
- **THEN** le système refuse la création avec un message "La date doit être après 1920"

#### Scenario: Création refusée si date future
- **WHEN** l'utilisateur saisit une date de naissance dans le futur
- **THEN** le système refuse la création avec un message "La date ne peut pas être dans le futur"

#### Scenario: Sélection du responsable via recherche
- **WHEN** l'utilisateur doit choisir un responsable légal pour un mineur
- **THEN** le système affiche un champ de recherche avec autocomplétion listant les personnes majeures (nom + prénom + âge)
- **THEN** l'utilisateur ne saisit PAS un ID technique mais sélectionne une personne dans la liste

### Requirement: Modifier une personne physique
Le système SHALL permettre de modifier les champs d'une personne existante.

#### Scenario: Modification réussie
- **WHEN** l'utilisateur modifie le prénom d'une personne existante
- **THEN** le système met à jour la personne et retourne ses nouvelles informations

### Requirement: Lister les personnes
Le système SHALL retourner la liste des personnes physiques, triée par nom puis prénom.

#### Scenario: Liste complète
- **WHEN** l'utilisateur consulte la liste des personnes
- **THEN** le système retourne toutes les personnes triées par nom, prénom

### Requirement: Rechercher une personne
Le système SHALL permettre une recherche textuelle insensible à la casse sur le nom et le prénom.

#### Scenario: Recherche par nom
- **WHEN** l'utilisateur tape "dup" dans le champ de recherche
- **THEN** le système retourne les personnes dont le nom ou prénom contient "dup" (ex: Dupont, Dupuis)

#### Scenario: Recherche insensible à la casse
- **WHEN** l'utilisateur tape "DUP" ou "dup"
- **THEN** le système retourne les mêmes résultats dans les deux cas

#### Scenario: Aucun résultat
- **WHEN** la recherche ne correspond à aucune personne
- **THEN** le système retourne une liste vide

### Requirement: Consulter le détail d'une personne
Le système SHALL afficher toutes les informations d'une personne, y compris son responsable légal si applicable. Les dates MUST être affichées au format JJ/MM/AAAA.

#### Scenario: Consultation personne sans responsable
- **WHEN** l'utilisateur clique sur une personne majeure
- **THEN** le système affiche ses informations sans section responsable

#### Scenario: Consultation personne avec responsable
- **WHEN** l'utilisateur clique sur un mineur
- **THEN** le système affiche ses informations avec le nom et prénom du responsable

### Requirement: Affichage des dates
Le système SHALL afficher toutes les dates au format JJ/MM/AAAA dans l'interface (liste, détail).

#### Scenario: Format dans la liste
- **WHEN** la liste des personnes s'affiche
- **THEN** les dates de naissance sont au format JJ/MM/AAAA

#### Scenario: Format dans le détail
- **WHEN** la vue détail s'affiche
- **THEN** la date de naissance est au format JJ/MM/AAAA

### Requirement: Validation âge et responsable
Le système MUST valider côté Rust et côté base de données (contrainte CHECK ou trigger SQLite) qu'une personne mineure a un responsable_id renseigné pointant vers une personne majeure.

#### Scenario: Validation à la création
- **WHEN** création d'une personne mineure
- **THEN** le système vérifie responsable_id non nul et responsable majeur

#### Scenario: Validation à la modification
- **WHEN** modification de l'âge ou du responsable_id
- **THEN** le système revalide la contrainte mineur → responsable
