## Purpose

Enregistre systématiquement le nom d'utilisateur et l'horodatage de chaque création ou modification de données — dans les deux modes de fonctionnement (mono-utilisateur et multi-utilisateurs) — sans jamais afficher ces informations dans l'interface.

## ADDED Requirements

### Requirement: Enregistrer l'auteur et l'horodatage de chaque création ou modification
Le système SHALL enregistrer le nom d'utilisateur et l'horodatage (ISO-8601, UTC) à chaque création ou modification d'une ligne, dans les colonnes d'audit de la table concernée.

#### Scenario: Création d'une personne
- **WHEN** l'utilisateur « Marie » crée une personne
- **THEN** la ligne créée contient `modifie_par` = « Marie » et `modifie_le` = horodatage courant

#### Scenario: Modification d'une activité
- **WHEN** l'utilisateur « Paul » modifie le nom d'une activité existante
- **THEN** la ligne est mise à jour avec `modifie_par` = « Paul » et `modifie_le` = horodatage courant

#### Scenario: Suppression d'une ligne
- **WHEN** l'utilisateur supprime une ligne
- **THEN** la ligne est supprimée sans conserver de trace d'audit (les colonnes d'audit disparaissent avec elle)

### Requirement: Fournir le nom d'utilisateur aux écritures
Le système SHALL utiliser le nom d'utilisateur de la configuration pour alimenter l'audit de chaque écriture, et refuser une écriture sans nom d'utilisateur.

#### Scenario: Écriture avec nom d'utilisateur
- **WHEN** une commande d'écriture est appelée avec le nom d'utilisateur de la configuration
- **THEN** le nom d'utilisateur est enregistré dans les colonnes d'audit

#### Scenario: Écriture en mode mono-utilisateur
- **WHEN** une commande d'écriture est appelée avec le nom d'utilisateur en mode mono-utilisateur
- **THEN** le nom d'utilisateur est enregistré dans les colonnes d'audit de la base locale

#### Scenario: Écriture sans nom d'utilisateur
- **WHEN** une commande d'écriture est appelée sans nom d'utilisateur
- **THEN** le système refuse l'écriture avec une erreur explicite

### Requirement: Détecter les modifications concurrentes lors d'une mise à jour
Le système SHALL refuser une mise à jour si la fiche a été modifiée par un autre utilisateur entre son chargement et l'enregistrement, et informer l'utilisateur qu'il doit recharger la fiche.

#### Scenario: Mise à jour sans modification concurrente
- **WHEN** un utilisateur modifie une fiche que personne n'a modifiée depuis son chargement
- **THEN** la mise à jour est appliquée

#### Scenario: Mise à jour concurrente
- **WHEN** un utilisateur enregistre une fiche qui a été modifiée par un autre utilisateur depuis son chargement
- **THEN** le système refuse la mise à jour avec un message indiquant que la fiche a été modifiée entre-temps et qu'il faut la recharger

#### Scenario: Conflit détecté dans les deux modes
- **WHEN** une fiche ouverte est enregistrée alors qu'elle a déjà été modifiée depuis son chargement
- **THEN** le système refuse la mise à jour, quel que soit le mode de fonctionnement

#### Scenario: Version transmise sans être affichée
- **WHEN** l'utilisateur consulte une fiche
- **THEN** la version utilisée pour la détection de conflit est transmise au formulaire sans être affichée à l'utilisateur

### Requirement: Ne jamais afficher l'audit
Le système SHALL ne pas exposer les colonnes d'audit (nom d'utilisateur, horodatage) dans les données renvoyées à l'interface.

#### Scenario: Lecture d'une personne
- **WHEN** l'utilisateur consulte le détail d'une personne
- **THEN** la réponse ne contient ni nom d'utilisateur ni horodatage d'audit
