# Planning

## Description

Le module Planning permet de gérer les horaires des activités et de consulter l'emploi du temps d'une personne.

Chaque activité peut avoir un ou plusieurs **créneaux horaires** récurrents dans la semaine (ex : "Poterie le lundi de 14h à 16h"). On peut également marquer des **semaines banalisées** (vacances, ponts) où une activité n'a pas lieu.

## Concepts

- **Créneau** : un jour de la semaine (lundi à dimanche) avec une heure de début et une heure de fin, associé à une activité pour une année scolaire.
- **Semaine banalisée** : une semaine où une activité spécifique n'a pas lieu (ex : "Vacances de Noël", "Pont de l'Ascension").
- **Collision** : lorsqu'une personne est inscrite à deux activités dont les créneaux se chevauchent (même jour et horaires qui se superposent).

## Fonctionnalités

### Gérer les créneaux d'une activité

Dans le détail d'une activité, une section **Créneaux** affiche la liste des créneaux pour l'année scolaire sélectionnée, triés par jour puis par heure.

**Ajouter un créneau** :
1. Cliquez sur "Ajouter un créneau".
2. Sélectionnez le jour de la semaine.
3. Renseignez l'heure de début et l'heure de fin.
4. Validez.

**Modifier un créneau** : cliquez sur le créneau et modifiez le jour ou les horaires.

**Supprimer un créneau** : cliquez sur la corbeille à côté du créneau.

> ⚠️ Si des personnes sont déjà inscrites à l'activité pour l'année, les créneaux existants ne peuvent plus être modifiés ni supprimés. Vous pouvez toujours ajouter de nouveaux créneaux. Pour modifier ou supprimer un créneau, retirez d'abord toutes les personnes inscrites à l'activité pour cette année.

### Gérer les semaines banalisées

Dans le détail d'une activité, une section **Semaines banalisées** liste les semaines où l'activité n'a pas lieu.

**Ajouter une semaine banalisée** :
1. Cliquez sur "Ajouter une semaine".
2. Renseignez la date du lundi de la semaine concernée.
3. Ajoutez un motif si souhaité (optionnel).
4. Validez.

**Supprimer une semaine banalisée** : cliquez sur la corbeille à côté de la semaine concernée.

### Consulter le planning d'une personne

Deux façons d'accéder au planning :

- **Depuis le détail d'une personne** : une section "Planning" affiche la grille hebdomadaire de la personne. Naviguez entre les semaines avec les boutons "Semaine précédente" et "Semaine suivante". Le numéro de semaine ISO et la date du lundi sont affichés.
- **Depuis la page Planning** (`/planning`) : sélectionnez une personne pour voir son planning. Utile pour les secrétaires qui consultent le planning de différents adhérents.

La grille hebdomadaire s'affiche du lundi au dimanche, de 8h à 20h. Chaque créneau apparaît comme un bloc coloré positionné à l'intersection du jour et de l'heure. Le bloc indique le nom de l'activité et le rôle de la personne (Encadrant ou Participant). Les couleurs diffèrent selon le rôle (bleu pour encadrant, vert pour participant).

Les activités dont la semaine est banalisée n'apparaissent pas dans la grille.

### Détection des collisions

Lors de l'inscription d'une personne à une activité, le système vérifie automatiquement que les créneaux de cette activité ne chevauchent pas ceux d'une autre activité où la personne est déjà inscrite, quel que soit son rôle (participant ou encadrant). En cas de collision, l'inscription est refusée avec un message indiquant l'activité en conflit.

## Flux

1. **Créer une activité** → définir ses créneaux horaires
2. **Ajouter des semaines banalisées** si nécessaire
3. **Inscrire des personnes** → la collision est vérifiée automatiquement
4. **Consulter le planning** d'une personne pour vérifier son emploi du temps
