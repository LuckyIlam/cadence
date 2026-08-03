# Paramètres

## Description

La page **Paramètres** (`/parametres`) permet de configurer la **plage horaire d'ouverture** des activités : l'heure d'ouverture et l'heure de fermeture. Par défaut, elle est fixée à **8h – 20h**.

## Modifier la plage horaire

1. Ouvrez la page **Paramètres** depuis la navigation.
2. Renseignez l'**heure d'ouverture** et l'**heure de fermeture**.
3. Cliquez sur **Enregistrer**.

> ℹ️ L'heure de fermeture doit être **strictement après** l'heure d'ouverture, sinon l'enregistrement est refusé.

## Réduire la plage horaire

Si la nouvelle plage ne couvre pas certains créneaux existants, un **avertissement** liste les créneaux impactés avant d'appliquer :

- **Créneau sans inscrit** → il est **supprimé**.
- **Créneau avec inscrits** → il est **déplacé** vers la place libre la plus proche, le **même jour**, en conservant sa durée (sans chevaucher les créneaux de la même activité ; deux activités distinctes peuvent partager un créneau).
- **Créneau avec inscrits sans place libre** → la réduction est **refusée** : élargissez la plage ou retirez d'abord les inscrits.
- **Adhérent déjà inscrit à une autre activité sur ce créneau** → la réduction est **refusée** : le système vérifie l'emploi du temps final de chaque adhérent du créneau déplacé ; si une autre activité (même année, même jour) occupe le nouvel horaire, le déplacement est impossible et le message mentionne l'activité en conflit.

Cliquez sur **Confirmer la réduction** pour appliquer (ou **Annuler** pour garder la plage actuelle). La réduction est appliquée en une seule opération : en cas d'erreur, aucun changement n'est conservé.

> ⚠️ Un créneau partiellement hors plage (ex. 7h30–9h00 avec ouverture à 8h00) est considéré comme hors plage et subit le même traitement.

## Impact de la plage

La plage configurée est utilisée à deux endroits :

- **À la création / modification d'un créneau** : un créneau doit être entièrement compris dans la plage (heure de début ≥ ouverture et heure de fin ≤ fermeture), sinon la demande est refusée.
- **À la consultation d'un planning** : la grille hebdomadaire s'affiche entre l'heure d'ouverture et l'heure de fermeture configurées.

Voir [Plage horaire d'ouverture](planning.md#plage-horaire-douverture) pour le détail.
