## 1. Base de données

- [ ] 1.1 Créer la migration SQL pour la table `creneaux_activite`
- [ ] 1.2 Créer la migration SQL pour la table `semaines_banalisees`

## 2. Backend — Domaine

- [ ] 2.1 Créer les types Rust `CreneauActivite`, `CreateCreneau` dans `domain/planning.rs`
- [ ] 2.2 Créer les types Rust `SemaineBanalisee`, `CreateSemaineBanalisee` dans `domain/planning.rs`
- [ ] 2.3 Ajouter `pub mod planning` dans `domain/mod.rs`
- [ ] 2.4 Ajouter les fonctions de validation (jour_semaine, heure, date_debut lundi) dans `domain/planning.rs`

## 3. Backend — Repository

- [ ] 3.1 Créer `repositories/planning_repo.rs` avec CRUD pour les créneaux
- [ ] 3.2 Ajouter CRUD pour les semaines banalisées dans `planning_repo.rs`
- [ ] 3.3 Ajouter la fonction `lister_creneaux_personne_semaine` pour le calcul du planning
- [ ] 3.4 Ajouter la fonction `verifier_collision` pour détecter les chevauchements
- [ ] 3.5 Ajouter la fonction `compter_inscrits_activite` pour le verrouillage des créneaux
- [ ] 3.6 Ajouter `pub mod planning_repo` dans `repositories/mod.rs`

## 4. Backend — Commandes

- [ ] 4.1 Créer `commands/planning_commands.rs` avec les commandes `ajouter_creneau`, `supprimer_creneau`, `modifier_creneau`, `lister_creneaux`
- [ ] 4.2 Ajouter les commandes `ajouter_semaine_banalisee`, `supprimer_semaine_banalisee`, `lister_semaines_banalisees`
- [ ] 4.3 Ajouter la commande `planning_personne` retournant les créneaux d'une personne pour une semaine donnée
- [ ] 4.4 Intégrer la vérification de collision dans `activite_commands::ajouter_personne_activite`
- [ ] 4.5 Enregistrer les nouvelles commandes dans `lib.rs` (invoke_handler)

## 5. Backend — Tests

- [ ] 5.1 Tests unitaires pour la validation des créneaux
- [ ] 5.2 Tests unitaires pour la validation des semaines banalisées
- [ ] 5.3 Tests d'intégration pour le CRUD des créneaux
- [ ] 5.4 Tests d'intégration pour la détection de collision
- [ ] 5.5 Tests d'intégration pour le calcul du planning d'une personne

## 6. Frontend — Types

- [ ] 6.1 Ajouter les types TypeScript `CreneauActivite`, `CreateCreneau`, `SemaineBanalisee`, `CreateSemaineBanalisee`, `PlanningCreneau`
- [ ] 6.2 Ajouter les helpers (numéro de semaine ISO, jour semaine en texte, format heure)

## 7. Frontend — Composant PlanningHebdo

- [ ] 7.1 Créer le composant `PlanningHebdo` : grille lundi→dimanche, 8h→20h
- [ ] 7.2 Gérer l'affichage des créneaux sous forme de blocs positionnés
- [ ] 7.3 Ajouter la navigation semaine (précédent/suivant, numéro de semaine, date du lundi)
- [ ] 7.4 Afficher le nom de l'activité et le rôle pour chaque bloc

## 8. Frontend — Page Planning

- [ ] 8.1 Créer la page `PlanningPage` avec sélecteur de personne
- [ ] 8.2 Charger les données via `invoke("planning_personne", {...})`
- [ ] 8.3 Ajouter la route `/planning` et `/planning/:personneId` dans `App.tsx`
- [ ] 8.4 Ajouter le lien "Planning" dans la navigation (`Nav.tsx`)

## 9. Frontend — Sections dans les pages existantes

- [ ] 9.1 Ajouter la section "Créneaux" dans `DetailActivite` (liste + ajout, verrouillage si inscrits)
- [ ] 9.2 Ajouter la section "Semaines banalisées" dans `DetailActivite` (liste + ajout/suppression)
- [ ] 9.3 Ajouter la section "Planning" dans `DetailPersonne` avec le composant `PlanningHebdo`

## 10. Vérifications finales

- [ ] 10.1 Exécuter `cargo check` dans `src-tauri/`
- [ ] 10.2 Exécuter `cargo clippy -- -D warnings`
- [ ] 10.3 Exécuter `cargo fmt --check`
- [ ] 10.4 Exécuter `cargo test` pour les tests Rust
- [ ] 10.5 Exécuter `npm run typecheck`
- [ ] 10.6 Exécuter `npm run lint`
- [ ] 10.7 Exécuter `npm run build`
