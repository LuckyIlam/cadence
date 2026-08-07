use serde::{Deserialize, Serialize};

use crate::error::AppError;

const FICHIER_CONFIG: &str = "cadence_config.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModeConnexion {
    Mono,
    Multi,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnexionConfig {
    pub mode: ModeConnexion,
    pub url: Option<String>,
    pub token: Option<String>,
    pub utilisateur: String,
}

impl Default for ConnexionConfig {
    fn default() -> Self {
        Self {
            mode: ModeConnexion::Mono,
            url: None,
            token: None,
            utilisateur: String::new(),
        }
    }
}

pub fn config_path(app_dir: &std::path::Path) -> std::path::PathBuf {
    app_dir.join(FICHIER_CONFIG)
}

pub fn load_config(app_dir: &std::path::Path) -> Result<Option<ConnexionConfig>, AppError> {
    let path = config_path(app_dir);
    if !path.exists() {
        return Ok(None);
    }
    let contenu = std::fs::read_to_string(&path).map_err(|e| AppError::Database(format!("{e}")))?;
    let config = serde_json::from_str(&contenu)
        .map_err(|e| AppError::Database(format!("configuration invalide : {e}")))?;
    Ok(Some(config))
}

pub fn save_config(app_dir: &std::path::Path, config: &ConnexionConfig) -> Result<(), AppError> {
    let contenu = serde_json::to_string_pretty(config)
        .map_err(|e| AppError::Database(format!("sérialisation config : {e}")))?;
    std::fs::write(config_path(app_dir), contenu)
        .map_err(|e| AppError::Database(format!("sauvegarde config : {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_dir(nom: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "cadence_config_test_{}_{}",
            std::process::id(),
            nom
        ))
    }

    #[test]
    fn test_aucune_config_retourne_none() {
        let dir = tmp_dir("aucune");
        std::fs::create_dir_all(&dir).unwrap();
        let path = config_path(&dir);
        let _ = std::fs::remove_file(&path);
        let config = load_config(&dir).unwrap();
        assert!(config.is_none());
    }

    #[test]
    fn test_sauvegarde_et_relecture_mono() {
        let dir = tmp_dir("mono");
        std::fs::create_dir_all(&dir).unwrap();
        let path = config_path(&dir);
        let _ = std::fs::remove_file(&path);

        let config = ConnexionConfig {
            mode: ModeConnexion::Mono,
            url: None,
            token: None,
            utilisateur: "Jean".into(),
        };
        save_config(&dir, &config).unwrap();

        let relu = load_config(&dir).unwrap().unwrap();
        assert_eq!(relu.mode, ModeConnexion::Mono);
        assert_eq!(relu.utilisateur, "Jean");
        assert!(relu.url.is_none());
    }

    #[test]
    fn test_sauvegarde_et_relecture_multi() {
        let dir = tmp_dir("multi");
        std::fs::create_dir_all(&dir).unwrap();
        let path = config_path(&dir);
        let _ = std::fs::remove_file(&path);

        let config = ConnexionConfig {
            mode: ModeConnexion::Multi,
            url: Some("https://example.turso.io".into()),
            token: Some("secret".into()),
            utilisateur: "Marie".into(),
        };
        save_config(&dir, &config).unwrap();

        let relu = load_config(&dir).unwrap().unwrap();
        assert_eq!(relu.mode, ModeConnexion::Multi);
        assert_eq!(relu.url.as_deref(), Some("https://example.turso.io"));
        assert_eq!(relu.token.as_deref(), Some("secret"));
    }
}
