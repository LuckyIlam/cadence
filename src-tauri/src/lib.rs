mod commands;
mod domain;
#[cfg(test)]
mod e2e_mono;
#[cfg(test)]
mod e2e_multi;
#[cfg(test)]
mod e2e_stream;
mod error;
mod infrastructure;
mod repositories;
mod services;

use infrastructure::config::ConnexionConfig;
use infrastructure::db::{init_app_state, init_connection};
use tauri::Manager;

fn write_crash_log(msg: &str) {
    let paths = [
        std::env::current_dir()
            .ok()
            .map(|p| p.join("cadence_crash.log")),
        Some(std::env::temp_dir().join("cadence_crash.log")),
    ];
    for path in paths.into_iter().flatten() {
        let _ = std::fs::write(&path, msg);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    std::env::set_var("RUST_MIN_STACK", "536870912");

    let result = tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("échec du dossier de données : {e}"))?;
            std::fs::create_dir_all(&app_dir)
                .map_err(|e| format!("échec création dossier {} : {e}", app_dir.display()))?;

            let config = infrastructure::config::load_config(&app_dir)?.unwrap_or_default();
            let mut config: ConnexionConfig = config;
            if config.utilisateur.trim().is_empty() {
                config.utilisateur = "local".to_string();
            }

            // Le chemin distant (TLS/hyper) en build debug consomme ~256 MiB de pile
            // (design.md, décision 5). Le setup s'exécute sur le thread main (pile
            // par défaut ~1 Mo) : on passe par un thread dédié à grande pile.
            let conn = std::thread::Builder::new()
                .name("cadence-db".into())
                .stack_size(512 * 1024 * 1024)
                .spawn(move || tauri::async_runtime::block_on(init_connection(&config, &app_dir)))
                .map_err(|e| format!("échec création du thread base de données : {e}"))?
                .join()
                .map_err(|_| "le thread base de données a paniqué".to_string())?
                .map_err(|e| format!("échec base de données : {e}"))?;

            app.manage(init_app_state(conn));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::personne_commands::creer_personne,
            commands::personne_commands::modifier_personne,
            commands::personne_commands::obtenir_personne,
            commands::personne_commands::obtenir_detail_personne,
            commands::personne_commands::rechercher_personnes,
            commands::adhesion_commands::creer_adhesion,
            commands::adhesion_commands::modifier_adhesion,
            commands::adhesion_commands::lister_adhesions_personne,
            commands::activite_commands::creer_activite,
            commands::activite_commands::modifier_activite,
            commands::activite_commands::obtenir_activite,
            commands::activite_commands::obtenir_detail_activite,
            commands::activite_commands::lister_annees_activites,
            commands::activite_commands::lister_activites,
            commands::activite_commands::definir_tarif_activite,
            commands::activite_commands::ajouter_personne_activite,
            commands::activite_commands::retirer_personne_activite,
            commands::activite_commands::lister_activites_personne,
            commands::planning_commands::ajouter_creneau,
            commands::planning_commands::supprimer_creneau,
            commands::planning_commands::modifier_creneau,
            commands::planning_commands::lister_creneaux,
            commands::planning_commands::ajouter_semaine_banalisee,
            commands::planning_commands::supprimer_semaine_banalisee,
            commands::planning_commands::lister_semaines_banalisees,
            commands::planning_commands::planning_personne,
            commands::planning_commands::verifier_collision,
            commands::parametre_commands::obtenir_parametres_planning,
            commands::parametre_commands::apercu_creneaux_hors_plage,
            commands::parametre_commands::modifier_plage_horaire,
            commands::connexion_commands::obtenir_config,
            commands::connexion_commands::sauvegarder_config,
            commands::connexion_commands::tester_connexion,
        ])
        .run(tauri::generate_context!());

    if let Err(e) = result {
        let msg = format!("Cadence — erreur fatale : {e}");
        write_crash_log(&msg);
        eprintln!("{msg}");
        panic!("{msg}");
    }
}
