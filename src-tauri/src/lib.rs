mod commands;
mod domain;
mod infrastructure;
mod repositories;

use tauri::Manager;
use infrastructure::db::{init_pool, AppState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("failed to get app data dir");
            std::fs::create_dir_all(&app_dir).expect("failed to create app data dir");

            let db_path = app_dir.join("cadence.db");
            let database_url = format!("sqlite:{}?mode=rwc", db_path.to_string_lossy());

            let pool = tauri::async_runtime::block_on(init_pool(&database_url))
                .expect("failed to initialize database");

            app.manage(AppState { pool });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::personne_commands::creer_personne,
            commands::personne_commands::modifier_personne,
            commands::personne_commands::obtenir_personne,
            commands::personne_commands::lister_personnes,
            commands::personne_commands::rechercher_personnes,
            commands::adhesion_commands::creer_adhesion,
            commands::adhesion_commands::modifier_adhesion,
            commands::adhesion_commands::lister_adhesions_personne,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
