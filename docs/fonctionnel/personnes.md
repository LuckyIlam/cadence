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

La recherche s'effectue sur le **nom** et le **prénom**, sans tenir compte de la casse (majuscules/minuscules) ni des accents.

Exemples :
- taper `dup` trouve *Dupont*, *Dupuis*
- taper `DUP` donne les mêmes résultats que `dup`

Si aucun résultat ne correspond, la liste est vide.

## Lister les personnes

La liste complète est triée par **nom** puis **prénom**, dans l'ordre alphabétique.

## Consulter le détail d'une personne

La vue détail affiche toutes les informations de la personne :

- Identité (nom, prénom, date de naissance)
- Coordonnées (email, téléphone)
- Responsable légal (si applicable)
- Liste des adhésions (voir [Adhésions](adhesions.md))

Toutes les dates sont affichées au format **JJ/MM/AAAA**.
