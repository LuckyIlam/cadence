# Connexion et mode de fonctionnement

## Description

Cadence fonctionne selon **deux modes de fonctionnement**, exclusifs entre eux :

- **Mono-utilisateur** : les données sont stockées dans une base locale, sur cet ordinateur. Utilisation individuelle, hors ligne.
- **Multi-utilisateurs** : les données sont stockées dans une **base partagée** hébergée chez **Turso** (libSQL, région UE), accessible par plusieurs bénévoles à la fois (y compris à distance).

Le mode est choisi lors du **premier lancement** de l'application, puis modifiable à tout moment depuis la page **Paramètres**.

## Premier lancement

Au premier lancement, l'application n'est pas encore configurée : elle affiche un écran **« Bienvenue dans Cadence »** qui vous demande de choisir le mode de fonctionnement avant d'autoriser l'accès aux autres écrans.

1. Choisissez **Mono-utilisateur** ou **Multi-utilisateurs**.
2. Renseignez les champs demandés selon le mode (voir ci-dessous).
3. Cliquez sur **Enregistrer**.

## Choisir son mode dans les Paramètres

La carte **« Connexion à la base »** de la page Paramètres permet de consulter et de modifier le mode de fonctionnement et ses paramètres.

### Mode mono-utilisateur

Seul le **nom d'utilisateur** est demandé. L'application utilise la base locale de cet ordinateur.

### Mode multi-utilisateurs

Les champs suivants sont requis :

- **URL de la base** : l'adresse de la base partagée (ex. `libsql://...`).
- **Clé d'accès** : le jeton fourni par Turso. La clé n'est jamais réaffichée en clair : si le champ est laissé vide lors d'une modification, la clé existante est conservée.
- **Nom d'utilisateur** : votre nom, enregistré avec chaque modification (audit).

> ℹ️ **Format de l'URL** : Turso affiche des adresses commençant par `turso://` (ex. `turso://cadence-dev-turso-luckyilam.aws-eu-west-1.turso.io`). L'application accepte indifféremment `turso://…` et `libsql://…` (le préfixe `turso://` est converti automatiquement). **Conservez le nom d'hôte tel quel** : `libsql://cadence-dev-luckyilam.aws-eu-west-1.turso.io` ne correspond pas à l'exemple ci-dessus et la connexion échouerait. Reprenez simplement l'adresse exacte de votre tableau de bord Turso.

> ⚠️ L'URL et la clé d'accès sont nécessaires : l'enregistrement est refusé si elles manquent en mode multi-utilisateurs.

## Tester la connexion

En mode **multi-utilisateurs**, un bouton **« Tester la connexion »** vérifie avant enregistrement que la base distante est joignable et que la clé d'accès est valide :

- **Succès** → un message vert « Connexion établie » s'affiche.
- **Échec** → un message explicite décrit l'erreur (URL incorrecte, clé invalide, base injoignable).

> ℹ️ Le test n'est disponible qu'en mode multi-utilisateurs.

## Redémarrage requis

Un changement de **mode**, d'**URL** ou de **clé d'accès** ne peut être appliqué qu'au prochain démarrage de l'application : une fenêtre **« Redémarrage requis »** vous propose alors de **redémarrer maintenant** ou **plus tard**.

- **Redémarrer maintenant** → l'application se relance et se connecte selon la nouvelle configuration.
- **Plus tard** → l'application continue de fonctionner avec l'ancienne configuration jusqu'au prochain lancement.

> ℹ️ Une simple modification du **nom d'utilisateur** est appliquée immédiatement, sans redémarrage.

## Données indépendantes

Les données du mode mono-utilisateur et du mode multi-utilisateurs sont **indépendantes** : aucun échange ni synchronisation automatique n'existe entre les deux bases. Changer de mode n'importe aucune donnée :

- en passant du local au partagé, les données locales ne sont **pas** transférées vers la base partagée ;
- en passant du partagé au local, les données de la base partagée ne sont **pas** copiées sur cet ordinateur.

> ⚠️ Avant de changer de mode, assurez-vous d'avoir saisi les bonnes informations : le mode utilisé affiche les données de **sa propre** base.

## Nom d'utilisateur et traçabilité

Le nom d'utilisateur saisi est enregistré avec chaque création ou modification d'une fiche (personne, adhésion, activité, créneau), dans les deux modes. Cette information est stockée pour la **traçabilité** des modifications et n'est **jamais affichée** dans l'application. Votre nom apparaît en haut de l'écran pour vous rappeler le compte utilisé.
