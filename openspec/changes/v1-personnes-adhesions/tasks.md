## 1. Initialisation du projet

- [ ] 1.1 Créer le projet Tauri avec React + TypeScript + Vite
- [ ] 1.2 Configurer Tailwind CSS
- [ ] 1.3 Ajouter les dépendances Rust (sqlx, tauri, chrono, serde)
- [ ] 1.4 Créer la structure de dossiers (domain/, repositories/, commands/, infrastructure/)

## 2. Base de données

- [ ] 2.1 Créer la migration `create_personnes_physiques` (id, nom, prenom, date_naissance, email, telephone, responsable_id)
- [ ] 2.2 Créer la migration `create_adhesions` (id, personne_id, annee_scolaire, reglee, note_paiement + UNIQUE)
- [ ] 2.3 Configurer SQLx avec SQLite et lancer les migrations au démarrage
- [ ] 2.4 Ajouter la contrainte CHECK âge/responsable côté SQLite

## 3. Couche domaine

- [ ] 3.1 Définir le struct `Personne` avec les champs et sérialisation
- [ ] 3.2 Définir le struct `Adhesion` avec les champs et sérialisation
- [ ] 3.3 Implémenter la validation âge (est_mineur, validation_responsable)

## 4. Repositories

- [ ] 4.1 Implémenter `PersonneRepository` (créer, modifier, lister, rechercher, trouver_par_id)
- [ ] 4.2 Implémenter `AdhesionRepository` (créer, modifier, lister_par_personne, trouver_par_id)
- [ ] 4.3 Implémenter la recherche textuelle avec `LIKE %...%` insensible à la casse

## 5. Tauri commands

- [ ] 5.1 Créer les commands pour `Personne` (créer, modifier, lister, rechercher, obtenir)
- [ ] 5.2 Créer les commands pour `Adhesion` (créer, modifier, lister_par_personne)
- [ ] 5.3 Ajouter la validation âge/responsable dans les commands de création/modification personne

## 6. Frontend — Liste des personnes

- [ ] 6.1 Créer la page liste avec champ de recherche
- [ ] 6.2 Intégrer l'appel Tauri `lister_personnes` / `rechercher_personnes`
- [ ] 6.3 Afficher les résultats (nom, prénom, âge, email)
- [ ] 6.4 Ajouter le bouton "Nouvelle personne"

## 7. Frontend — Détail d'une personne

- [ ] 7.1 Créer la page détail personne
- [ ] 7.2 Afficher les informations de la personne + responsable
- [ ] 7.3 Afficher la liste des adhésions
- [ ] 7.4 Ajouter le formulaire d'ajout/modification d'adhésion
- [ ] 7.5 Ajouter le formulaire de modification des informations personne

## 8. Validation et finitions

- [ ] 8.1 Vérifier la validation âge/responsable à la création et modification
- [ ] 8.2 Gérer les erreurs côté frontend (messages d'erreur explicites)
- [ ] 8.3 Responsive design de base (mobile/desktop avec Tailwind)
- [ ] 8.4 Nettoyage et vérification finale
