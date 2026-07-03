## 1. Base de données

- [x] 1.1 Créer la migration `create_activites` (id, nom, description?, capacite_max?)
- [x] 1.2 Créer la migration `create_tarifs_activite` (activite_id, annee_scolaire, tarif + UNIQUE)
- [x] 1.3 Créer la migration `create_activite_personnes` (activite_id, personne_id, annee_scolaire, role + UNIQUE)

## 2. Couche domaine

- [x] 2.1 Définir le struct `Activite` avec les champs et sérialisation
- [x] 2.2 Définir `CreateActivite` / `UpdateActivite`
- [x] 2.3 Définir `TarifActivite`
- [x] 2.4 Définir `LiaisonActivitePersonne` avec le rôle (enum ou string)
- [x] 2.5 Définir `DetailActivite` (activité + liste encadrants + liste participants + tarif année)
- [x] 2.6 Ajouter `activite` au `domain/mod.rs`
- [x] 2.7 Tests unitaires : validation capacité max, validation rôle unique (pas à la fois encadrant et participant pour la même activité)

## 3. Repository

- [x] 3.1 Implémenter `ActiviteRepository` : créer, modifier, lister, trouver_par_id
- [x] 3.2 Implémenter : ajouter_tarif, obtenir_tarif(activite_id, annee_scolaire)
- [x] 3.3 Implémenter : ajouter_personne, retirer_personne, lister_personnes(activite_id)
- [x] 3.4 Implémenter : lister_activites_personne(personne_id)
- [x] 3.5 Implémenter : lister_activites_par_annee(annee_scolaire)
- [x] 3.6 Tests d'intégration

## 4. Tauri commands

- [x] 4.1 Créer `creer_activite`, `modifier_activite`, `obtenir_detail_activite`
- [x] 4.2 Créer `lister_activites` avec filtre année scolaire
- [x] 4.3 Créer `definir_tarif_activite`
- [x] 4.4 Créer `ajouter_personne_activite`, `retirer_personne_activite`
- [x] 4.5 Créer `lister_activites_personne`
- [x] 4.6 Enregistrer les nouvelles commands dans `lib.rs`
- [x] 4.7 Ajouter validation métier dans les commands (capacité max, rôle unique)

## 5. Frontend — Navigation

- [x] 5.1 Créer un composant de navigation (menu avec "Personnes" et "Activités")
- [x] 5.2 Modifier `App.tsx` pour intégrer le menu et les nouvelles routes
- [x] 5.3 Mettre en évidence la route active

## 6. Frontend — Liste des activités

- [x] 6.1 Créer la page `Activites.tsx` avec sélecteur d'année scolaire (années provenant de `tarifs_activite`)
- [x] 6.2 Intégrer l'appel Tauri `lister_activites_par_annee` avec filtre année
- [x] 6.3 Afficher les résultats (nom, tarif pour l'année, nombre de participants)
- [x] 6.4 Ajouter le bouton "Nouvelle activité" avec formulaire (nom, description, capacité, année, tarif)
- [x] 6.5 Ajouter un bouton "Voir le détail" sur chaque activité

## 7. Frontend — Détail d'une activité

- [x] 7.1 Créer la page `DetailActivite.tsx`
- [x] 7.2 Afficher les informations (nom, description, capacité, tarif)
- [x] 7.3 Afficher deux sections : "Encadrants" et "Participants" avec listes
- [x] 7.4 Ajouter/modifier le tarif pour l'année scolaire
- [x] 7.5 Ajouter/retirer des personnes (avec recherche de personnes existantes)
- [x] 7.6 Gérer les messages d'erreur (capacité atteinte, double rôle, doublon)

## 8. Frontend — Section activités dans le détail personne

- [x] 8.1 Modifier `DetailPersonne.tsx` pour afficher les activités de la personne
- [x] 8.2 Deux sous-listes : "En tant qu'encadrant·e" et "En tant que participant·e"
- [x] 8.3 Chaque entrée est un lien vers le détail de l'activité

## 9. Documentation

- [x] 9.1 Ajouter `docs/fonctionnel/activites.md` avec la documentation utilisateur
- [x] 9.2 Mettre à jour `docs/fonctionnel/README.md` pour inclure le module activités

## 10. Vérifications finales

- [x] 10.1 `cargo check` (dans `src-tauri/`)
- [x] 10.2 `cargo clippy -- -D warnings`
- [x] 10.3 `cargo fmt --check`
- [x] 10.4 `npm run typecheck`
- [x] 10.5 `npm run lint`
- [x] 10.6 `npm run build`
- [x] 10.7 `cargo test`
