# Gestion des adhésions

Une adhésion représente une inscription annuelle à l'association. Elle est valable une **année scolaire** (ex. 2025-2026) et peut être réglée ou en attente de paiement.

## Ajouter une adhésion

Depuis la fiche détail d'une personne, un bouton permet d'ajouter une adhésion.

Champs à renseigner :

| Champ | Obligatoire | Description |
|-------|-------------|-------------|
| Année scolaire | Oui | Format "YYYY-YYYY" (ex. 2025-2026) |
| Réglée | Oui | Coche si la cotisation a été payée |
| Note de paiement | Non | Information libre : numéro de chèque, virement, espèces, etc. (max 255 caractères) |

### Règle : une seule adhésion par année

Une personne ne peut avoir qu'une **seule** adhésion par année scolaire. Si une adhésion existe déjà pour l'année en cours, le bouton d'ajout est désactivé avec le message :

> *"Une adhésion existe déjà pour l'année XXXX-XXXX"*

### Format de l'année scolaire

Le format attendu est **"YYYY-YYYY"** (quatre chiffres, tiret, quatre chiffres). Exemples :
- `2025-2026` ✅ valide
- `2025/2026` ❌ invalide
- `2025-26` ❌ invalide

## Modifier une adhésion

Il est possible de modifier le statut **Réglée** et la **Note de paiement** d'une adhésion existante. Cela permet par exemple de marquer comme réglée une adhésion qui était en attente.

## Lister les adhésions d'une personne

Les adhésions sont affichées dans la vue détail de la personne, triées de la **plus récente** à la **plus ancienne**.

Si la personne n'a jamais adhéré, le message suivant s'affiche :

> *"Aucune adhésion"*

## Suivi des règlements

Le statut **Réglée** permet de suivre les cotisations :
- **En attente** : la personne a adhéré mais n'a pas encore payé
- **Réglée** : le paiement a été reçu

La **note de paiement** permet de conserver une trace (numéro de chèque, référence virement, etc.).
