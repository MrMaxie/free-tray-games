use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use toml;
use std::{env, fs, sync::Arc};

const CONFIG_FILE: &str = "config.toml";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InnerConfig {
    pub is_notifications_enabled: bool,
}

impl Default for InnerConfig {
    fn default() -> Self {
        Self {
            is_notifications_enabled: true,
        }
    }
}

#[derive(Clone)]
pub struct Config {
    inner_config: Arc<Mutex<InnerConfig>>,
}

impl Config {
    pub fn new() -> Self {
        let inner_config = Arc::new(Mutex::new(InnerConfig::default()));
        Config { inner_config }
    }

    pub async fn is_notifications_enabled(&self) -> bool {
        self.inner_config.lock().await.is_notifications_enabled
    }

    pub async fn toggle_notifications(&mut self) {
        let mut inner_config = self.inner_config.lock().await;
        inner_config.is_notifications_enabled = !inner_config.is_notifications_enabled;
    }

    pub async fn load(&mut self) {
        let mut config_path = env::current_exe().unwrap();
        config_path.set_file_name(CONFIG_FILE);

        if !config_path.exists() {
            let default_config = InnerConfig::default();
            let toml = toml::to_string(&default_config).unwrap();
            fs::write(&config_path, toml).unwrap();
            self.inner_config = Arc::new(Mutex::new(default_config));
        } else {
            let config_str = fs::read_to_string(&config_path).unwrap();
            let new_config = toml::from_str(&config_str).unwrap_or_default();
            self.inner_config = Arc::new(Mutex::new(new_config));
        }
    }

    pub async fn save(&self) {
        let mut config_path = env::current_exe().unwrap();
        config_path.set_file_name(CONFIG_FILE);

        let inner_config = self.inner_config.lock().await.clone();
        let toml = toml::to_string(&inner_config).unwrap();
        fs::write(&config_path, toml).unwrap();
    }

    pub fn get_app_id() -> String {
        "FreeTrayGames.App".to_string()
    }
}