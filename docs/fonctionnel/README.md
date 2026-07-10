# Documentation fonctionnelle — Cadence

Cadence est une application de bureau conçue pour aider les associations à gérer leurs adhérents et leurs activités.

## Public visé

Cette documentation s'adresse aux **utilisateurs de l'application** (bénévoles, secrétaires, trésoriers, responsables d'association). Elle décrit ce que l'application permet de faire et comment l'utiliser.

## Modules fonctionnels

| Module | Description |
|--------|-------------|
| [Personnes](personnes.md) | Gestion des personnes physiques (adhérents, responsables légaux) |
| [Adhésions](adhesions.md) | Gestion des adhésions annuelles |
| [Activités](activites.md) | Gestion des activités de l'association (participants, encadrants, tarifs, créneaux horaires) |
| [Planning](planning.md) | Gestion des créneaux horaires, semaines banalisées, planning hebdomadaire par personne |

## Concepts généraux

- **Personne physique** : tout individu inscrit dans l'application (adhérent, parent, intervenant).
- **Responsable légal** : personne majeure liée à un mineur.
- **Adhésion** : inscription annuelle à l'association, valable une année scolaire (ex. 2025-2026).
- **Année scolaire** : période de septembre à août, notée sous la forme "YYYY-YYYY".

## Flux principaux

1. **Créer une personne** → renseigner son identité
2. **Ajouter une adhésion** → associer une cotisation pour l'année en cours
3. **Suivre les règlements** → marquer les adhésions comme réglées et ajouter des notes de paiement
4. **Créer une activité** → définir un nom, un tarif pour l'année
5. **Créer des créneaux** → ajouter les horaires des activités (jour, heure début, heure fin)
6. **Inscrire des personnes** → ajouter des participants et des encadrants aux activités (la collision horaire est vérifiée automatiquement)
7. **Consulter le planning** → visualiser l'emploi du temps hebdomadaire d'une personne
