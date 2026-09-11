use tauri::Manager;

use crate::error::AppError;
use crate::infrastructure::compat::Compatibilite;
use crate::infrastructure::config::{
    load_config, save_config, ConnexionConfig, Driver, ModeConnexion,
};

#[derive(Debug, Clone, serde::Serialize)]
pub struct ConfigAffichee {
    pub configuree: bool,
    pub mode: Option<ModeConnexion>,
    pub url: Option<String>,
    pub utilisateur: Option<String>,
    pub a_une_cle: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ResultatSauvegarde {
    pub config: ConfigAffichee,
    pub redemarrage_requis: bool,
}

fn app_dir<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> Result<std::path::PathBuf, AppError> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Database(format!("dossier de données : {e}")))?;
    std::fs::create_dir_all(&dir)
        .map_err(|e| AppError::Database(format!("création dossier {} : {e}", dir.display())))?;
    Ok(dir)
}

fn vers_config_affichee(config: Option<&ConnexionConfig>) -> ConfigAffichee {
    match config {
        None => ConfigAffichee {
            configuree: false,
            mode: None,
            url: None,
            utilisateur: None,
            a_une_cle: false,
        },
        Some(c) => ConfigAffichee {
            configuree: true,
            mode: Some(c.mode),
            url: c.url.clone(),
            utilisateur: Some(c.utilisateur.clone()),
            a_une_cle: c.token.as_deref().map(|t| !t.is_empty()).unwrap_or(false),
        },
    }
}

fn lire_config(dir: &std::path::Path) -> Result<ConfigAffichee, AppError> {
    Ok(vers_config_affichee(load_config(dir)?.as_ref()))
}

/// Turso fournit des URLs `turso://…` ; libsql n'accepte que `libsql://…`.
fn normaliser_url(url: String) -> String {
    url.strip_prefix("turso://")
        .map(|reste| format!("libsql://{reste}"))
        .unwrap_or(url)
}

/// Applique et enregistre la configuration dans `dir`. `token` absent ou vide
/// conserve la clé existante (la clé n'est jamais renvoyée au front en clair).
fn appliquer_config(
    dir: &std::path::Path,
    mode: ModeConnexion,
    url: Option<String>,
    token: Option<String>,
    utilisateur: String,
) -> Result<ResultatSauvegarde, AppError> {
    let utilisateur = utilisateur.trim().to_string();
    if utilisateur.is_empty() {
        return Err(AppError::Validation(
            "Le nom d'utilisateur est requis.".to_string(),
        ));
    }

    let url = match mode {
        ModeConnexion::Mono => None,
        ModeConnexion::Multi => {
            let url = url.map(|u| normaliser_url(u.trim().to_string()));
            match url {
                Some(u) if u.is_empty() => {
                    return Err(AppError::Validation(
                        "L'URL de la base est requise en mode multi-utilisateurs.".to_string(),
                    ))
                }
                Some(u) => Some(u),
                None => {
                    return Err(AppError::Validation(
                        "L'URL de la base est requise en mode multi-utilisateurs.".to_string(),
                    ))
                }
            }
        }
    };

    let avant = load_config(dir)?;
    let ancien_token = avant
        .as_ref()
        .and_then(|c| c.token.clone())
        .unwrap_or_default();
    let token = match token {
        Some(t) if !t.trim().is_empty() => Some(t.trim().to_string()),
        _ if !ancien_token.is_empty() => Some(ancien_token),
        _ => None,
    };

    let config = ConnexionConfig {
        driver: Driver::Sqlite,
        mode,
        url,
        token,
        utilisateur,
    };
    save_config(dir, &config)?;

    let redemarrage_requis = match &avant {
        // Premier lancement : l'application est déjà connectée à la base locale,
        // seul un passage en mode multi-utilisateurs nécessite un redémarrage.
        None => config.mode == ModeConnexion::Multi,
        Some(avant) => {
            avant.mode != config.mode || avant.url != config.url || avant.token != config.token
        }
    };

    Ok(ResultatSauvegarde {
        config: vers_config_affichee(Some(&config)),
        redemarrage_requis,
    })
}

#[tauri::command]
pub async fn obtenir_config<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<ConfigAffichee, AppError> {
    let dir = app_dir(&app)?;
    lire_config(&dir)
}

#[tauri::command]
pub async fn obtenir_compatibilite(
    state: tauri::State<'_, Compatibilite>,
) -> Result<Compatibilite, AppError> {
    Ok(state.inner().clone())
}

#[tauri::command]
pub async fn sauvegarder_config<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    mode: ModeConnexion,
    url: Option<String>,
    token: Option<String>,
    utilisateur: String,
) -> Result<ResultatSauvegarde, AppError> {
    let dir = app_dir(&app)?;
    appliquer_config(&dir, mode, url, token, utilisateur)
}

/// Vérifie qu'une base Turso distante est joignable avec l'URL et la clé fournies.
#[tauri::command]
pub async fn tester_connexion(url: String, token: String) -> Result<(), AppError> {
    let url = normaliser_url(url.trim().to_string());
    let token = token.trim();
    if url.is_empty() || token.is_empty() {
        return Err(AppError::Validation(
            "URL et clé d'accès requises pour tester la connexion.".to_string(),
        ));
    }

    let db = libsql::Builder::new_remote(url, token.to_string())
        .build()
        .await
        .map_err(|e| AppError::Database(format!("Connexion impossible : {e}")))?;
    let conn = db
        .connect()
        .map_err(|e| AppError::Database(format!("Connexion impossible : {e}")))?;
    let mut rows = conn
        .query("SELECT 1", libsql::params![])
        .await
        .map_err(|e| AppError::Database(format!("Connexion impossible : {e}")))?;
    rows.next()
        .await
        .map_err(|e| AppError::Database(format!("Connexion impossible : {e}")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_dir(nom: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("cadence_config_cmd_{}_{}", std::process::id(), nom))
    }

    #[test]
    fn test_normaliser_url_turso() {
        assert_eq!(
            normaliser_url("turso://base.turso.io".to_string()),
            "libsql://base.turso.io"
        );
        assert_eq!(
            normaliser_url("libsql://base.turso.io".to_string()),
            "libsql://base.turso.io"
        );
        assert_eq!(normaliser_url("".to_string()), "");
    }

    #[test]
    fn test_obtenir_config_absente() {
        let dir = tmp_dir("absente");
        std::fs::create_dir_all(&dir).unwrap();
        let _ = std::fs::remove_file(crate::infrastructure::config::config_path(&dir));
        let config = lire_config(&dir).unwrap();
        assert!(!config.configuree);
        assert!(!config.a_une_cle);
    }

    #[test]
    fn test_sauvegarder_config_mono_sans_redemarrage() {
        let dir = tmp_dir("mono");
        std::fs::create_dir_all(&dir).unwrap();
        let _ = std::fs::remove_file(crate::infrastructure::config::config_path(&dir));

        let resultat =
            appliquer_config(&dir, ModeConnexion::Mono, None, None, "Jean".to_string()).unwrap();
        assert!(!resultat.redemarrage_requis);
        assert_eq!(resultat.config.mode, Some(ModeConnexion::Mono));
        assert_eq!(resultat.config.utilisateur.as_deref(), Some("Jean"));

        let resultat = appliquer_config(
            &dir,
            ModeConnexion::Mono,
            None,
            None,
            "Jean-Paul".to_string(),
        )
        .unwrap();
        assert!(!resultat.redemarrage_requis);
        assert_eq!(resultat.config.utilisateur.as_deref(), Some("Jean-Paul"));
    }

    #[test]
    fn test_sauvegarder_config_sans_utilisateur_refusee() {
        let dir = tmp_dir("sans_utilisateur");
        std::fs::create_dir_all(&dir).unwrap();
        let err =
            appliquer_config(&dir, ModeConnexion::Mono, None, None, "   ".to_string()).unwrap_err();
        assert!(err.to_string().contains("utilisateur"));
    }

    #[test]
    fn test_sauvegarder_config_multi_sans_url_refusee() {
        let dir = tmp_dir("sans_url");
        std::fs::create_dir_all(&dir).unwrap();
        let err = appliquer_config(
            &dir,
            ModeConnexion::Multi,
            Some("   ".to_string()),
            Some("cle".to_string()),
            "Marie".to_string(),
        )
        .unwrap_err();
        assert!(err.to_string().contains("URL"));
    }

    #[test]
    fn test_sauvegarder_config_multi_avec_url_ok_et_cle_conservee() {
        let dir = tmp_dir("multi_cle");
        std::fs::create_dir_all(&dir).unwrap();
        let _ = std::fs::remove_file(crate::infrastructure::config::config_path(&dir));

        let resultat = appliquer_config(
            &dir,
            ModeConnexion::Multi,
            Some("https://exemple.turso.io".to_string()),
            Some("secret".to_string()),
            "Marie".to_string(),
        )
        .unwrap();
        assert!(resultat.redemarrage_requis);
        assert_eq!(resultat.config.mode, Some(ModeConnexion::Multi));
        assert_eq!(
            resultat.config.url.as_deref(),
            Some("https://exemple.turso.io")
        );
        assert!(resultat.config.a_une_cle);

        let resultat = appliquer_config(
            &dir,
            ModeConnexion::Multi,
            Some("https://exemple.turso.io".to_string()),
            None,
            "Marie-Anne".to_string(),
        )
        .unwrap();
        assert!(!resultat.redemarrage_requis);
        assert!(resultat.config.a_une_cle);
        assert_eq!(resultat.config.utilisateur.as_deref(), Some("Marie-Anne"));
    }

    #[test]
    fn test_changement_de_mode_redemarrage_requis() {
        let dir = tmp_dir("changement_mode");
        std::fs::create_dir_all(&dir).unwrap();
        let _ = std::fs::remove_file(crate::infrastructure::config::config_path(&dir));

        appliquer_config(&dir, ModeConnexion::Mono, None, None, "Jean".to_string()).unwrap();

        let resultat = appliquer_config(
            &dir,
            ModeConnexion::Multi,
            Some("https://exemple.turso.io".to_string()),
            Some("secret".to_string()),
            "Jean".to_string(),
        )
        .unwrap();
        assert!(resultat.redemarrage_requis);
    }

    #[test]
    fn test_obtenir_config_ne_renvoie_pas_la_cle() {
        let dir = tmp_dir("pas_cle");
        std::fs::create_dir_all(&dir).unwrap();
        let _ = std::fs::remove_file(crate::infrastructure::config::config_path(&dir));

        appliquer_config(
            &dir,
            ModeConnexion::Multi,
            Some("https://exemple.turso.io".to_string()),
            Some("secret".to_string()),
            "Marie".to_string(),
        )
        .unwrap();

        let config = lire_config(&dir).unwrap();
        assert!(config.configuree);
        assert!(config.a_une_cle);
        assert_eq!(config.mode, Some(ModeConnexion::Multi));
    }
}
