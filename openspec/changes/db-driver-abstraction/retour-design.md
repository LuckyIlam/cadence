Diagnostic global : le design est solide et bien informé

Les décisions structurantes (D1-D9) sont cohérentes et le plan en 3 PR est raisonnable. Points que je valide après lecture du code :

Connection clonée × 5 dans init_app_state (db.rs:63-72) : confirmé exactement.
libsql::params! / libsql::de::from_row omniprésents : confirmé, avec des chiffres réels proches de ceux du design (94 occurrences de libsql::params!, 58 de from_row, 14 de libsql::Value).
AppError::From<libsql::Error> qui écrase tout dans Database(String) (R4) : confirmé mot pour mot dans error.rs.
Vec<libsql::Value> pour la recherche paginée dynamique (R3, personne_repo.rs:191-243) : confirmé, c'est exactement ce pattern.
Le choix dyn Db plutôt que generics ou enum (D1) est justifié et argumenté honnêtement (alternatives écartées avec de vraies raisons, pas de la figuration).
D9 (RETURNING piloté par le repo, pas par le trait Db) est la bonne décision — imposer insert_returning dans le trait aurait forcé une abstraction avec le plus petit dénominateur commun (MySQL).
⚠️ Divergence factuelle importante : la fuite de libsql est plus large que ce que dit le design

C'est le point le plus important de ma relecture. Le design (Context, ligne 14) affirme :

&mut libsql::Transaction fuit dans 2 signatures de traits (planning_repo.rs:38-46, parametre_repo.rs:13,51)

En grepant le code réel :

repositories/activite_repo.rs   → trait ActiviteRepository, 9 méthodes _tx avec &mut libsql::Transaction
repositories/planning_repo.rs   → trait PlanningRepository, plusieurs méthodes _tx
repositories/parametre_repo.rs  → trait ParametreRepository, 1 méthode _tx

C'est donc 3 traits, pas 2 (ActiviteRepository est oublié).

Plus grave : libsql::Transaction et libsql::TransactionBehavior apparaissent aussi dans :

services/activite_service.rs (12 occurrences de &mut libsql::Transaction, + transaction_with_behavior(libsql::TransactionBehavior::Immediate) ligne 197)
services/parametre_service.rs (9 occurrences)
commands/planning_commands.rs (3 occurrences, dont un appel direct state.conn.transaction_with_behavior(libsql::TransactionBehavior::Immediate) ligne 27)

Or D1 affirme explicitement : « La couche commands/ ignore tout du driver ». Ce n'est pas vrai aujourd'hui — commands/planning_commands.rs importe et manipule libsql::TransactionBehavior directement, et state.conn (le champ pub conn: Connection de AppState) est utilisé directement dans commands/activite_commands.rs (10 sites) et commands/parametre_commands.rs (3 sites) pour construire les services.

Conséquence concrète pour le plan :

Le champ pub conn: Connection sur AppState doit disparaître ou devenir Arc<dyn Db> — mais ça touche directement les fichiers commands/*.rs, pas seulement repositories/ et services/. PR 2 dit « AppState contient Arc<dyn Db> » mais ne mentionne nulle part la modification des commands/*.rs qui consomment state.conn aujourd'hui. C'est un angle mort du plan.
Le pattern transaction_with_behavior(Immediate) posé dans les commands doit soit remonter dans Db::begin_immediate() (variante du trait), soit Db::begin() doit exposer un moyen de choisir le mode BEGIN — sinon PR 2 casse la garantie d'atomicité documentée en commentaire (« BEGIN IMMEDIATE... pour éviter un conflit non détecté »).
Le chiffrage « ~1 010 lignes ajoutées / ~360 supprimées » (R9) risque d'être sous-estimé compte tenu de ce périmètre commands/ non compté explicitement.

Je recommande d'ajouter une clarification factuelle dans le Context (corriger « 2 signatures » → au minimum 3 traits + commands) et un item de scope explicite pour commands/*.rs dans PR 2.

Autres points techniques à challenger

1. Le retry Hrana sur execute (mutation) mérite d'être documenté comme risque, pas juste déplacé tel quel dans RetryPolicy (D4).
Dans hrana_guard.rs, execute_avec_retry réexécute une commande d'écriture après un « stream not found ». Le commentaire du code affirme que l'échec survient toujours « au moment du prepare/describe, avant toute exécution côté serveur » — c'est l'hypothèse qui rend le retry sûr pour une écriture. Le design D4 dit vouloir généraliser ce mécanisme (RetryPolicy) à d'autres drivers (SqlxRetryPolicy pour serialization_failure/deadlock). Or pour Postgres, un retry aveugle sur serialization_failure après exécution partielle n'est sûr que si toute la transaction est rejouée, pas une requête isolée. Le design ne distingue pas retry-au-niveau-requête vs retry-au-niveau-transaction dans le trait RetryPolicy. À clarifier avant que ça devienne un piège pour le driver Postgres futur (même si « hors scope » aujourd'hui, l'interface posée dans cette PR doit anticiper le bon grain de retry).

2. Non-atomicité de cadence_migrations (migrations.rs:58-83, pas mentionné dans les Risks).
Chaque migration fait execute_batch("BEGIN; ...; COMMIT;") puis, dans un appel séparé, INSERT INTO _cadence_migrations. Si le process crashe entre les deux, la migration sera rejouée au prochain boot (CREATE TABLE IF NOT EXISTS protège les créations de table, mais pas forcément les ALTER TABLE comme dans add_audit.sql). C'est un risque préexistant, indépendant du driver, mais comme D7/PR3 touche justement ce runner (refinery), ce serait le bon moment de le noter en Risk plutôt que de le découvrir pendant le spike.

3. R1 (compat refinery/libsql) est le vrai risque bloquant, et le design le traite bien — mais le mitigant (« on garde le runner actuel derrière &dyn Db ») dépend d'un Db::execute_batch ou équivalent qui n'apparaît pas dans le trait Db de D1 (seulement execute, fetch_one/all/optional, begin). cadence_migrations utilise execute_batch avec un BEGIN;...;COMMIT; littéral — si Db n'expose pas de batch, PR 1 devra soit l'ajouter au trait, soit le runner de migration reste en dehors de l'abstraction. À trancher explicitement, sinon PR 3 butera dessus.

4. Le chiffrage historique (~3 100 lignes touchées par sqlx→libsql, ~2 200 lignes cible) n'est pas vérifiable depuis le code actuel seul (c'est un delta de commit, pas un état). Je ne peux pas le confirmer ni l'infirmer sans l'historique git — à mentionner si vous voulez que je vérifie via git log/git diff si le dépôt complet (avec .git) est disponible.

En résumé

Le design est bien pensé sur le fond (trait Db minimal, séparation driver/domaine, RETURNING non imposé, migrations en Rust). Le point à corriger avant de figer le plan : le périmètre réel de la fuite libsql est plus large que ce que dit le Context — commands/*.rs manipule directement Connection et TransactionBehavior, pas seulement 2 repositories. Ça vaut la peine de relire le chiffrage de PR 2 à la lumière de ça, et de clarifier le grain du RetryPolicy avant qu'il devienne un contrat figé pour les futurs drivers.