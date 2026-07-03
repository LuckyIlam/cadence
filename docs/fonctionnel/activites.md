# Activités

## Description

Le module Activités permet de gérer les activités proposées par l'association : ateliers, cours, entraînements, sorties, etc.

Chaque activité peut avoir :
- Un nom (obligatoire)
- Une description (optionnelle)
- Une capacité maximale (optionnelle)
- Un tarif différent chaque année scolaire

## Concepts

- **Activité** : ce que l'association propose (ex: "Poterie", "Théâtre", "Couture").
- **Tarif** : le prix de l'activité pour une année scolaire donnée. L'historique des tarifs est conservé (`tarifs_activite`).
- **Participant** : une personne inscrite à l'activité.
- **Encadrant** : une personne qui encadre l'activité (animateur, professeur, bénévole).
- Une personne ne peut pas être à la fois encadrante ET participante pour la même activité (même année).
- Une personne peut participer à une même activité sur plusieurs années (via des enregistrements distincts dans `activite_personnes`).

## Fonctionnalités

### Lister les activités

La liste des activités affiche les activités ayant un tarif défini pour l'année scolaire sélectionnée. Le sélecteur d'année ne propose que les années présentes dans la table `tarifs_activite`. Par défaut, la première année disponible est sélectionnée.

Pour chaque activité, la liste indique :
- Le nom
- Le tarif pour l'année
- Le nombre de participants (et la capacité maximale si définie)

### Créer une activité

Cliquez sur "Nouvelle activité" et remplissez :
- **Nom** (obligatoire)
- **Description** (optionnelle)
- **Capacité max** (optionnelle)
- **Année scolaire** (requise, choix entre l'année courante et la suivante)
- **Tarif** (optionnel, 0€ par défaut si non saisi)

Un tarif est automatiquement créé pour l'année choisie (0€ par défaut). Il peut être modifié ensuite dans le détail.

### Modifier une activité

Dans le détail d'une activité, cliquez sur le nom, la description ou la capacité maximale pour les modifier directement.

### Gérer les tarifs

Dans le détail d'une activité, sélectionnez l'année scolaire puis cliquez sur le tarif pour le modifier. Le tarif est enregistré avec l'historique : changer le tarif pour 2025-2026 n'affecte pas le tarif de 2024-2025.

Le sélecteur d'année dans la page de détail propose les 4 années autour de l'année courante (N-2 à N+1) pour faciliter la navigation.

### Ajouter une personne à une activité

Dans le détail d'une activité, cliquez sur "Ajouter" dans la section Participants ou Encadrants. Un panneau de recherche s'ouvre dans le bloc concerné :

1. Tapez le nom ou prénom de la personne dans le champ de recherche
2. Les résultats apparaissent dynamiquement après 300ms
3. Cliquez sur une personne pour l'ajouter avec le rôle correspondant au bloc (participant ou encadrant)

Le rôle est automatiquement déterminé par le bloc où se trouve le bouton "Ajouter" — il n'y a pas de sélecteur de rôle.

### Retirer une personne d'une activité

Cliquez sur "Retirer" à côté du nom de la personne dans la liste.

### Consulter les activités d'une personne

Dans le détail d'une personne, une section "Activités" affiche les activités auxquelles elle participe ou qu'elle encadre, avec deux sous-listes : "En tant qu'encadrant·e" et "En tant que participant·e".
