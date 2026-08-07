## Why

Cadence est aujourd'hui une application monoposte : chaque installation (bénévole, secrétaire, trésorier) gère sa propre base SQLite locale dans son dossier de données applicatives. Les données ne sont jamais partagées entre les bénévoles de l'association. Le passage au multi-utilisateurs — avec des données partagées, dont des données de mineurs — est nécessaire pour permettre aux bénévoles à distance de travailler sur les mêmes données, dans le respect du RGPD UE (données hébergées dans l'UE, traçabilité des modifications). Le mode mono-utilisateur existant (base locale) reste néanmoins utile (usage individuel, hors ligne) : les deux modes de fonctionnement cohabitent et l'utilisateur en choisit un seul à la fois.

## What Changes

- Ajout d'un mode **multi-utilisateurs** (base partagée hébergée chez Turso, libSQL, région UE) à côté du mode **mono-utilisateur** existant (base SQLite locale). Le domaine métier, le SQL et les commandes restent inchangés.
- Les deux modes sont **exclusifs** : l'utilisateur en choisit un seul via l'écran de configuration ; les deux ne peuvent pas être actifs en même temps.
- Écran de configuration (choix du mode et, selon le mode, URL / clé d'accès / nom d'utilisateur) intégré à la page Paramètres, ainsi qu'un écran de premier lancement lorsque la base n'est pas encore configurée.
- Enregistrement systématique du nom d'utilisateur et de l'horodatage à chaque création ou modification (stocké en base, jamais affiché, actif dans les deux modes), avec détection des modifications concurrentes (optimistic locking).
- Chaque mode utilise sa propre base de données, **sans synchronisation** entre les deux.
- Rafraîchissement des données à chaque appel de commande (rafraîchissement suffisant, pas de temps réel).

## Capabilities

### New Capabilities
- `connexion-distance`: choix du mode de fonctionnement (mono-utilisateur sur base locale ou multi-utilisateurs sur base partagée Turso), configuration de la connexion selon le mode (URL, clé d'accès, nom d'utilisateur), stockage local, écran de premier lancement, test de connexion et application des changements de mode.
- `audit-modifications`: enregistrement du nom d'utilisateur et de l'horodatage ISO-8601 à chaque création ou modification (audit stocké uniquement, jamais affiché), actif dans les deux modes, et détection des modifications concurrentes lors d'une mise à jour.

### Modified Capabilities
<!-- Aucun requirement de niveau spec existant ne change : l'application conserve les mêmes comportements, le changement de backend de stockage est une modification d'implémentation. -->

## Impact

- Backend Rust : remplacement de SQLx par la crate `libsql` 0.9 (features remote/core/tls, local pour le mode fichier), qui pilote **les deux modes** (`new_local` / `new_remote`) ; nouveau runner de migrations, pools et transactions adaptés, audit dans les repositories, extension d'`AppError`.
- Migration de schéma : colonnes `modifie_par` / `modifie_le` / `version` sur les tables existantes (dans les deux modes).
- Commandes Tauri : nouvelles commandes `obtenir_config`, `sauvegarder_config`, `tester_connexion` (multi uniquement) ; le nom d'utilisateur est passé aux commandes d'écriture.
- Frontend React : écran de premier lancement (App.tsx) avec choix du mode, carte « Connexion à la base » dans ParametresPage (sélecteur de mode, redémarrage requis au changement), saisie du nom d'utilisateur.
- Exigence de développement : `RUST_MIN_STACK` 512 MiB en debug **uniquement en mode multi** (récursion hyper/rustls distante), release OK à 2 MiB.
- Déploiement : Turso (sous-traitant UE), base de test `cadence-dev` fournie.
