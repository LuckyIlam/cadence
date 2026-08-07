## Purpose

Permet de choisir le mode de fonctionnement de l'application (mono-utilisateur sur base locale ou multi-utilisateurs sur base partagée Turso), de configurer la connexion selon le mode choisi, de tester la connexion et de guider le premier lancement.

## Requirements

### Requirement: Choisir le mode de fonctionnement
Le système SHALL permettre à l'utilisateur de choisir entre deux modes de fonctionnement — mono-utilisateur (base locale) et multi-utilisateurs (base partagée) — avec un seul mode actif à la fois.

#### Scenario: Choix du mode mono-utilisateur
- **WHEN** l'utilisateur choisit le mode mono-utilisateur puis enregistre
- **THEN** l'application utilise la base SQLite locale

#### Scenario: Choix du mode multi-utilisateurs
- **WHEN** l'utilisateur choisit le mode multi-utilisateurs puis enregistre
- **THEN** l'application utilise la base partagée Turso

#### Scenario: Exclusion des deux modes
- **WHEN** le mode multi-utilisateurs est actif
- **THEN** le mode mono-utilisateur est inactif, et réciproquement

### Requirement: Configurer la connexion selon le mode choisi
Le système SHALL présenter des champs de configuration adaptés au mode : en mode multi-utilisateurs, l'URL de la base, la clé d'accès et le nom d'utilisateur sont requis ; en mode mono-utilisateur, seul le nom d'utilisateur est requis.

#### Scenario: Configuration en mode multi-utilisateurs
- **WHEN** l'utilisateur configure le mode multi-utilisateurs avec une URL, une clé d'accès et un nom d'utilisateur
- **THEN** le système enregistre la configuration et utilise la base distante

#### Scenario: URL manquante en mode multi-utilisateurs
- **WHEN** l'utilisateur tente d'enregistrer le mode multi-utilisateurs sans URL
- **THEN** le système refuse avec un message explicite

#### Scenario: Configuration en mode mono-utilisateur sans URL
- **WHEN** l'utilisateur configure le mode mono-utilisateur avec seulement un nom d'utilisateur
- **THEN** le système enregistre la configuration et utilise la base locale (l'URL et la clé ne sont pas demandées)

#### Scenario: Nom d'utilisateur manquant
- **WHEN** l'utilisateur tente d'enregistrer une configuration sans nom d'utilisateur
- **THEN** le système refuse avec un message explicite

### Requirement: Afficher un écran de premier lancement si la base n'est pas configurée
Le système SHALL bloquer l'accès aux fonctionnalités tant qu'aucun mode de fonctionnement n'est configuré, et présenter l'écran de configuration avec le choix du mode.

#### Scenario: Premier lancement sans configuration
- **WHEN** l'utilisateur lance l'application pour la première fois sans configuration existante
- **THEN** le système affiche l'écran de configuration avec le choix du mode et ne permet pas d'accéder aux autres écrans

#### Scenario: Lancement avec configuration existante
- **WHEN** l'utilisateur lance l'application alors qu'une configuration a déjà été enregistrée
- **THEN** le système accède directement aux fonctionnalités selon le mode configuré

### Requirement: Tester la connexion en mode multi-utilisateurs
Le système SHALL permettre, en mode multi-utilisateurs, de vérifier que la base distante est joignable et que la clé d'accès est valide avant d'enregistrer la configuration.

#### Scenario: Connexion valide
- **WHEN** l'utilisateur teste une URL et une clé d'accès valides
- **THEN** le système confirme que la connexion est établie

#### Scenario: Connexion invalide
- **WHEN** l'utilisateur teste une URL ou une clé d'accès invalides
- **THEN** le système affiche une erreur explicite

#### Scenario: Test indisponible en mode mono-utilisateur
- **WHEN** le mode mono-utilisateur est sélectionné
- **THEN** le test de connexion n'est pas disponible

### Requirement: Conserver et relire la configuration localement
Le système SHALL stocker la configuration (mode et valeurs) en local, la recharger au démarrage, et ne pas exposer la clé d'accès à l'affichage.

#### Scenario: Redémarrage de l'application
- **WHEN** l'utilisateur redémarre l'application après avoir configuré la connexion
- **THEN** le système recharge la configuration et se connecte selon le mode configuré

#### Scenario: Consultation de la configuration
- **WHEN** l'utilisateur consulte l'écran de configuration après l'avoir enregistrée
- **THEN** le système affiche le mode, l'URL et le nom d'utilisateur mais pas la clé d'accès en clair

### Requirement: Appliquer un changement de mode ou de connexion
Le système SHALL inviter à redémarrer l'application lorsqu'un changement porte sur le mode, l'URL ou la clé d'accès, et appliquer immédiatement une modification du nom d'utilisateur seul.

#### Scenario: Changement de mode avec redémarrage
- **WHEN** l'utilisateur change de mode ou modifie l'URL ou la clé d'accès puis enregistre
- **THEN** le système enregistre la configuration et propose de redémarrer l'application pour appliquer le changement

#### Scenario: Redémarrage différé
- **WHEN** l'utilisateur choisit de reporter le redémarrage
- **THEN** l'application continue avec l'ancien mode jusqu'au prochain lancement

#### Scenario: Changement de nom d'utilisateur seul
- **WHEN** l'utilisateur modifie uniquement son nom d'utilisateur puis enregistre
- **THEN** le système applique immédiatement la modification, sans redémarrage
