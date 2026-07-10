## Purpose

Gérer les personnes physiques de l'association : création, modification, recherche, consultation. Chaque personne peut être adhérente, intervenante, ou représentant légal (rôles gérés ultérieurement).

## Requirements

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

#### Scenario: Modification refusée si date invalide
- **WHEN** l'utilisateur modifie la date de naissance avec une valeur avant 1920 ou dans le futur
- **THEN** le système refuse la modification avec un message d'erreur explicite

### Requirement: Lister les personnes
Le système SHALL retourner la liste des personnes physiques, triée par nom puis prénom, avec pagination.

#### Scenario: Première page
- **WHEN** l'utilisateur consulte la liste des personnes
- **THEN** le système retourne les 20 premières personnes triées par nom, prénom
- **THEN** le système indique le nombre total de résultats et le nombre de pages

#### Scenario: Navigation entre pages
- **WHEN** l'utilisateur clique sur un numéro de page ou sur « Suivant » / « Précédent »
- **THEN** le système retourne les résultats de la page demandée

### Requirement: Rechercher une personne
Le système SHALL permettre une recherche textuelle insensible à la casse sur le nom, le prénom, l'email et le téléphone.

#### Scenario: Recherche par nom
- **WHEN** l'utilisateur tape "dup" dans le champ de recherche
- **THEN** le système retourne les personnes dont le nom ou prénom contient "dup" (ex: Dupont, Dupuis)

#### Scenario: Recherche par email
- **WHEN** l'utilisateur tape "gmail" dans le champ de recherche
- **THEN** le système retourne les personnes dont l'email contient "gmail"

#### Scenario: Recherche par téléphone
- **WHEN** l'utilisateur tape "0612" dans le champ de recherche
- **THEN** le système retourne les personnes dont le téléphone contient "0612"

#### Scenario: Recherche insensible à la casse
- **WHEN** l'utilisateur tape "DUP" ou "dup"
- **THEN** le système retourne les mêmes résultats dans les deux cas

#### Scenario: Aucun résultat
- **WHEN** la recherche ne correspond à aucune personne
- **THEN** le système retourne une liste vide

#### Scenario: Pagination réinitialisée sur recherche
- **WHEN** l'utilisateur modifie le texte de recherche
- **THEN** le système repasse à la page 1 des résultats

### Requirement: Filtrer par statut d'adhésion
Le système SHALL permettre de filtrer la liste pour n'afficher que les personnes ayant une adhésion pour l'année scolaire courante.

#### Scenario: Filtre adhérent activé
- **WHEN** l'utilisateur coche « Adhérent·e·s uniquement »
- **THEN** le système ne retourne que les personnes avec une adhésion pour l'année scolaire en cours

#### Scenario: Filtre adhérent désactivé
- **WHEN** l'utilisateur décoche « Adhérent·e·s uniquement »
- **THEN** le système retourne toutes les personnes correspondant aux autres critères

#### Scenario: Combinaison texte libre + filtre adhérent
- **WHEN** l'utilisateur tape un texte de recherche ET coche « Adhérent·e·s uniquement »
- **THEN** le système applique les deux filtres simultanément

### Requirement: Consulter le détail d'une personne
Le système SHALL afficher toutes les informations d'une personne, y compris son responsable légal si applicable et ses activités (participations et encadrements). Les dates MUST être affichées au format JJ/MM/AAAA.

#### Scenario: Consultation personne sans responsable
- **WHEN** l'utilisateur clique sur une personne majeure
- **THEN** le système affiche ses informations sans section responsable, les dates au format JJ/MM/AAAA

#### Scenario: Consultation personne avec responsable
- **WHEN** l'utilisateur clique sur un mineur
- **THEN** le système affiche ses informations avec le nom et prénom du responsable, les dates au format JJ/MM/AAAA

#### Scenario: Consultation avec activités
- **WHEN** l'utilisateur consulte le détail d'une personne
- **THEN** le système affiche une section "Activités" avec deux sous-listes : "En tant qu'encadrant·e" et "En tant que participant·e"

### Requirement: Validation âge et responsable
Le système MUST valider côté Rust et côté base de données (trigger SQLite) qu'une personne mineure a un responsable_id renseigné pointant vers une personne majeure.

#### Scenario: Validation à la création
- **WHEN** création d'une personne mineure
- **THEN** le système vérifie responsable_id non nul et responsable majeur

#### Scenario: Validation à la modification
- **WHEN** modification de l'âge ou du responsable_id
- **THEN** le système revalide la contrainte mineur → responsable

### Requirement: Affichage des dates en français
Le système SHALL afficher toutes les dates au format JJ/MM/AAAA dans l'interface utilisateur (liste, détail, formulaires).

#### Scenario: Format dans la liste
- **WHEN** la liste des personnes s'affiche
- **THEN** les dates de naissance sont formatées JJ/MM/AAAA

#### Scenario: Format dans le détail
- **WHEN** la vue détail s'affiche
- **THEN** la date de naissance est au format JJ/MM/AAAA

### Requirement: Navigation entre personnes et activités
Le système SHALL offrir un menu de navigation permettant de basculer entre la liste des personnes et la liste des activités.

#### Scenario: Menu visible
- **WHEN** l'utilisateur ouvre l'application
- **THEN** un menu de navigation est affiché avec les entrées "Personnes" et "Activités"
- **THEN** l'entrée active est mise en évidence
