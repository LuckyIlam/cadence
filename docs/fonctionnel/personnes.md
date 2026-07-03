# Gestion des personnes

Ce module permet de gérer l'ensemble des personnes physiques liées à l'association : adhérents, parents, représentants légaux, intervenants.

## Créer une personne

Renseigner les champs suivants :

| Champ | Obligatoire | Description |
|-------|-------------|-------------|
| Nom | Oui | Nom de famille |
| Prénom | Oui | Prénom |
| Date de naissance | Oui | Au format JJ/MM/AAAA |
| Email | Non | Adresse email |
| Téléphone | Non | Numéro de téléphone |
| Responsable légal | Non | À sélectionner dans la liste des personnes majeures |

### Règle : mineur et responsable légal

- Si la personne est **mineure** (moins de 18 ans), un responsable léval **doit** être désigné.
- Le responsable doit être **majeur**.
- La sélection du responsable se fait via un champ de recherche avec autocomplétion (nom + prénom + âge) — pas de saisie d'identifiant technique.

### Règle : validité de la date

- La date de naissance ne peut pas être **antérieure à 1920**.
- La date de naissance ne peut pas être **dans le futur**.

## Modifier une personne

Tous les champs sont modifiables après la création. Les mêmes règles de validation s'appliquent (date, âge, responsable légal).

Si l'âge ou le responsable légal est modifié, la cohérence est revérifiée : un mineur doit toujours avoir un responsable majeur.

## Rechercher une personne

La recherche s'effectue sur le **nom**, le **prénom**, l'**email** et le **téléphone**, sans tenir compte de la casse (majuscules/minuscules).

Exemples :
- taper `dup` trouve *Dupont*, *Dupuis*
- taper `gmail` trouve les personnes dont l'email contient *gmail*
- taper `0612` trouve les personnes dont le téléphone contient *0612*

### Filtrer par adhésion

Une case à cocher **« Adhérent·e·s uniquement »** permet de filtrer la liste pour ne voir que les personnes ayant une adhésion pour l'année scolaire en cours.

### Pagination

La liste des résultats est paginée (20 résultats par page). Une barre de navigation en bas de page permet de circuler entre les pages :

| Élément | Comportement |
|---------|-------------|
| Numéros de page | Cliquer sur un numéro affiche la page correspondante |
| ← Précédent | Page précédente (désactivé sur la première page) |
| Suivant → | Page suivante (désactivé sur la dernière page) |
| Indicateur | Affiche « Page X/Y — Z résultats » |

La pagination se réinitialise à la page 1 à chaque changement de critère de recherche.

## Lister les personnes

La liste est triée par **nom** puis **prénom**, dans l'ordre alphabétique. Par défaut, toutes les personnes sont affichées (page 1, 20 résultats par page).

## Consulter le détail d'une personne

La vue détail affiche toutes les informations de la personne :

- Identité (nom, prénom, date de naissance)
- Coordonnées (email, téléphone)
- Responsable légal (si applicable)
- Liste des adhésions (voir [Adhésions](adhesions.md))

Toutes les dates sont affichées au format **JJ/MM/AAAA**.
