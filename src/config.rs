use crate::types::{AuthConfig, ConfigFile};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const CONFIG_DIR: &str = ".config/tokens";
const CONFIG_FILE: &str = "config.json";

#[derive(Default)]
pub struct ConfigManager;

impl ConfigManager {
    pub fn new() -> Self {
        Self
    }

    pub fn get_config_path(&self) -> PathBuf {
        let home = dirs::home_dir().expect("Could not determine home directory.");
        home.join(CONFIG_DIR).join(CONFIG_FILE)
    }

    pub fn read_config(&self, path: &Path) -> Option<ConfigFile> {
        if path.exists() {
            fs::read_to_string(path)
                .ok()
                .and_then(|data| serde_json::from_str(&data).ok())
        } else {
            None
        }
    }

    pub fn save_config(&self, path: &Path, config: &ConfigFile) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(config)?;
        fs::write(path, data)
    }

    pub fn add_client(&self, config: &mut ConfigFile, nickname: String, auth_config: AuthConfig) {
        config.clients.insert(nickname, auth_config);
    }

    pub fn remove_client(&self, config: &mut ConfigFile, nickname: &str) -> bool {
        config.clients.remove(nickname).is_some()
    }

    pub fn get_client_mut<'a>(
        &self,
        config: &'a mut ConfigFile,
        nickname: &str,
    ) -> Option<&'a mut AuthConfig> {
        config.clients.get_mut(nickname)
    }

    #[allow(dead_code)]
    pub fn get_client<'a>(&self, config: &'a ConfigFile, nickname: &str) -> Option<&'a AuthConfig> {
        config.clients.get(nickname)
    }

    pub fn list_clients<'a>(&self, config: &'a ConfigFile) -> Vec<(&'a String, &'a AuthConfig)> {
        config.clients.iter().collect()
    }
}
