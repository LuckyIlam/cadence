#[cfg(test)]
mod tests {
    use crate::domain::personne::{CreatePersonne, UpdatePersonne};
    use crate::infrastructure::config::{ConnexionConfig, ModeConnexion};
    use crate::infrastructure::db::{init_app_state, init_connection};
    use crate::repositories::personne_repo::PersonneRepository;

    // Test de validation end-to-end : mode multi-utilisateurs sur la base de test
    // Turso `cadence-dev`. Nécessite les variables d'environnement TURSO_URL et
    // TURSO_TOKEN (le token n'est jamais committé). Skippé si elles sont absentes.
    #[test]
    fn e2e_multi_crud() {
        let Ok(token) = std::env::var("TURSO_TOKEN") else {
            eprintln!("TURSO_TOKEN absent : test e2e multi ignoré");
            return;
        };
        let Ok(url) = std::env::var("TURSO_URL") else {
            eprintln!("TURSO_URL absent : test e2e multi ignoré");
            return;
        };

        // Chemin TLS/hyper distant : thread à grande pile (design.md, décision 5).
        let worker = std::thread::Builder::new()
            .name("e2e-multi".into())
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
                    let config = ConnexionConfig {
                        mode: ModeConnexion::Multi,
                        url: Some(url),
                        token: Some(token),
                        utilisateur: "E2E test".into(),
                    };
                    let tmp = std::env::temp_dir().join("cadence_e2e_multi_tmp");
                    std::fs::create_dir_all(&tmp).unwrap();

                    let conn = init_connection(&config, &tmp)
                        .await
                        .expect("connexion + migrations distantes");
                    let state = init_app_state(conn);

                    // SELECT 1 direct (équivalent tester_connexion).
                    let mut rows = state
                        .conn
                        .query("SELECT 1", libsql::params![])
                        .await
                        .expect("SELECT 1");
                    rows.next().await.expect("row").expect("row");

                    // CRUD via le vrai repository.
                    let nom = format!("E2E {}", std::process::id());
                    let personne = state
                        .personne_repo
                        .create(
                            CreatePersonne {
                                nom: nom.clone(),
                                prenom: "Multi".into(),
                                date_naissance: chrono::NaiveDate::from_ymd_opt(2000, 1, 1)
                                    .unwrap(),
                                email: None,
                                telephone: None,
                                responsable_id: None,
                            },
                            "E2E test",
                        )
                        .await
                        .expect("creer");
                    let relue = state
                        .personne_repo
                        .find_by_id(personne.id)
                        .await
                        .expect("trouver")
                        .expect("personne");
                    assert_eq!(relue.nom, nom);

                    state
                        .personne_repo
                        .update(
                            personne.id,
                            UpdatePersonne {
                                nom: nom.clone(),
                                prenom: "MultiMaj".into(),
                                date_naissance: chrono::NaiveDate::from_ymd_opt(2000, 1, 1)
                                    .unwrap(),
                                email: None,
                                telephone: None,
                                responsable_id: None,
                                version: personne.version,
                            },
                            "E2E test",
                        )
                        .await
                        .expect("modifier");

                    let relue2 = state
                        .personne_repo
                        .find_by_id(personne.id)
                        .await
                        .expect("trouver2")
                        .expect("personne2");
                    assert_eq!(relue2.prenom, "MultiMaj");
                    assert_eq!(relue2.version, personne.version + 1);

                    // Nettoyage direct (aucune commande de suppression personne).
                    state
                        .conn
                        .execute(
                            "DELETE FROM personnes_physiques WHERE id = ?",
                            libsql::params![personne.id],
                        )
                        .await
                        .expect("nettoyage");

                    let plus = state
                        .personne_repo
                        .find_by_id(personne.id)
                        .await
                        .expect("trouver3");
                    assert!(plus.is_none());
                });
            })
            .expect("thread");

        worker.join().expect("thread e2e a paniqué");
    }
}
