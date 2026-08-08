#[cfg(test)]
mod tests {
    use crate::domain::personne::{CreatePersonne, UpdatePersonne};
    use crate::infrastructure::config::{ConnexionConfig, Driver, ModeConnexion};
    use crate::infrastructure::db::{init_app_state, init_connection};
    use crate::repositories::personne_repo::PersonneRepository;

    // Validation end-to-end du mode mono-utilisateur sur un vrai fichier local
    // (pas `:memory:`), avec persistance après réouverture.
    #[test]
    fn e2e_mono_fichier_crud_persiste() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        rt.block_on(async {
            let dir = std::env::temp_dir().join(format!("cadence_e2e_mono_{}", std::process::id()));
            std::fs::create_dir_all(&dir).unwrap();
            let fichier = dir.join("cadence.db");
            let _ = std::fs::remove_file(&fichier);

            let config = ConnexionConfig {
                driver: Driver::Sqlite,
                mode: ModeConnexion::Mono,
                url: None,
                token: None,
                utilisateur: "E2E mono".into(),
            };

            let conn = init_connection(&config, &dir)
                .await
                .expect("connexion locale");
            let state = init_app_state(conn);

            let nom = format!("E2E mono {}", std::process::id());
            let personne = state
                .personne_repo
                .create(
                    CreatePersonne {
                        nom: nom.clone(),
                        prenom: "Mono".into(),
                        date_naissance: chrono::NaiveDate::from_ymd_opt(1995, 5, 5).unwrap(),
                        email: None,
                        telephone: None,
                        responsable_id: None,
                    },
                    "E2E mono",
                )
                .await
                .expect("creer");
            assert!(personne.id > 0);

            state
                .personne_repo
                .update(
                    personne.id,
                    UpdatePersonne {
                        nom: nom.clone(),
                        prenom: "MonoMaj".into(),
                        date_naissance: chrono::NaiveDate::from_ymd_opt(1995, 5, 5).unwrap(),
                        email: Some("e2e@mono.test".into()),
                        telephone: None,
                        responsable_id: None,
                        version: personne.version,
                    },
                    "E2E mono",
                )
                .await
                .expect("modifier");

            drop(state);

            // Réouverture : les données doivent persister sur le fichier.
            let conn2 = init_connection(&config, &dir).await.expect("réouverture");
            let state2 = init_app_state(conn2);
            let relue = state2
                .personne_repo
                .find_by_id(personne.id)
                .await
                .expect("trouver")
                .expect("personne");
            assert_eq!(relue.prenom, "MonoMaj");
            assert_eq!(relue.email.as_deref(), Some("e2e@mono.test"));
            assert_eq!(relue.version, personne.version + 1);

            let _ = std::fs::remove_dir_all(&dir);
        });
    }
}
