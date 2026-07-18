mod commands;
mod domain;
mod infrastructure;
mod repositories;

use infrastructure::db::{init_pool, AppState};
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
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("échec du dossier de données : {e}"))?;
            std::fs::create_dir_all(&app_dir)
                .map_err(|e| format!("échec création dossier {} : {e}", app_dir.display()))?;

            let db_path = app_dir.join("cadence.db");
            let database_url = format!("sqlite:{}?mode=rwc", db_path.to_string_lossy());

            let pool = tauri::async_runtime::block_on(init_pool(&database_url))
                .map_err(|e| format!("échec base de données {} : {e}", db_path.display()))?;

            app.manage(AppState { pool });

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
        ])
        .run(tauri::generate_context!());

    if let Err(e) = result {
        let msg = format!("Cadence — erreur fatale : {e}");
        write_crash_log(&msg);
        eprintln!("{msg}");
        panic!("{msg}");
    }
}
