#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::infrastructure::hrana_guard;

    // Régression « stream not found » (mode multi) : la connexion distante Hrana
    // partage un stream unique. Un curseur abandonné (ligne lue puis `Rows`
    // droppé sans consommer la fin) ou un stream expiré côté serveur laisse un
    // baton obsolète côté client ; la requête suivante échoue avec
    // `status=404 ... stream not found`. La garde `query_avec_retry` /
    // `execute_avec_retry` réinitialise le stream (`conn.reset()`) et réessaie.
    //
    // Nécessite les variables TURSO_URL et TURSO_TOKEN (base Turso `cadence-dev`,
    // table dédiée `_e2e_stream`). Skippé si elles sont absentes.
    #[test]
    fn e2e_stream_apres_curseur_abandonne() {
        let Ok(token) = std::env::var("TURSO_TOKEN") else {
            eprintln!("TURSO_TOKEN absent : test e2e stream ignoré");
            return;
        };
        let Ok(url) = std::env::var("TURSO_URL") else {
            eprintln!("TURSO_URL absent : test e2e stream ignoré");
            return;
        };

        // Chemin TLS/hyper distant : thread à grande pile (design.md, décision 5).
        let worker = std::thread::Builder::new()
            .name("e2e-stream".into())
            .stack_size(512 * 1024 * 1024)
            .spawn(move || {
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .expect("runtime");
                rt.block_on(async {
                    let url = url
                        .strip_prefix("turso://")
                        .map(|reste| format!("libsql://{reste}"))
                        .unwrap_or(url);
                    let db = libsql::Builder::new_remote(url, token)
                        .build()
                        .await
                        .expect("connexion distante");
                    let conn = db.connect().expect("connect");

                    hrana_guard::execute_avec_retry(
                        &conn,
                        "CREATE TABLE IF NOT EXISTS _e2e_stream (id INTEGER PRIMARY KEY AUTOINCREMENT, nom TEXT)",
                        libsql::params![],
                    )
                    .await
                    .expect("création table de test");

                    // 1) Curseur abandonné : une ligne lue puis `Rows` droppé sans drain.
                    let mut rows = conn
                        .query(
                            "INSERT INTO _e2e_stream (nom) VALUES ('a') RETURNING id, nom",
                            libsql::params![],
                        )
                        .await
                        .expect("insert avec RETURNING");
                    rows.next().await.expect("row").expect("row");
                    drop(rows);

                    // 2) La requête suivante doit réussir via la garde (reset + retry).
                    let mut count = hrana_guard::query_avec_retry(
                        &conn,
                        "SELECT COUNT(*) AS count FROM _e2e_stream",
                        libsql::params![],
                    )
                    .await
                    .expect("requête après curseur abandonné");
                    let row = count
                        .next()
                        .await
                        .expect("row après curseur abandonné")
                        .expect("row après curseur abandonné");
                    let total = libsql::de::from_row::<CompteurRow>(&row)
                        .expect("décodage count")
                        .count;
                    assert!(total >= 1, "la table doit contenir la ligne insérée");

                    // 3) Stream expiré côté serveur (inactivité) : doit réussir aussi.
                    tokio::time::sleep(Duration::from_secs(7)).await;
                    let mut count2 = hrana_guard::query_avec_retry(
                        &conn,
                        "SELECT COUNT(*) AS count FROM _e2e_stream",
                        libsql::params![],
                    )
                    .await
                    .expect("requête après inactivité");
                    let row2 = count2
                        .next()
                        .await
                        .expect("row après inactivité")
                        .expect("row après inactivité");
                    let total2 = libsql::de::from_row::<CompteurRow>(&row2)
                        .expect("décodage count2")
                        .count;
                    assert!(total2 >= 1, "données intactes après inactivité");

                    hrana_guard::execute_avec_retry(
                        &conn,
                        "DROP TABLE IF EXISTS _e2e_stream",
                        libsql::params![],
                    )
                    .await
                    .expect("nettoyage table de test");
                });
            })
            .expect("thread");

        worker.join().expect("thread e2e stream a paniqué");
    }

    #[derive(Debug, Clone, serde::Deserialize)]
    struct CompteurRow {
        count: i64,
    }
}
