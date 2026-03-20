use crate::connection::ConnectionConfig;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default)]
    pub auto_refresh_interval: u32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            auto_refresh_interval: 0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ConfigFile {
    connections: Vec<ConnectionConfig>,
    #[serde(default)]
    settings: AppSettings,
}

#[derive(Clone)]
pub struct ConfigStorage {
    config_path: PathBuf,
}

impl ConfigStorage {
    pub fn new() -> io::Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Config directory not found"))?
            .join("rust-redis-desktop");

        fs::create_dir_all(&config_dir)?;

        let config_path = config_dir.join("config.json");

        Ok(Self { config_path })
    }

    pub fn new_temp() -> io::Result<Self> {
        let temp_dir = std::env::temp_dir().join("rust-redis-desktop-test");
        fs::create_dir_all(&temp_dir)?;

        let config_path = temp_dir.join("config.json");

        Ok(Self { config_path })
    }

    pub fn save_connection(&self, config: ConnectionConfig) -> io::Result<()> {
        let mut file = self.load_or_create_config_file()?;

        if let Some(pos) = file.connections.iter().position(|c| c.id == config.id) {
            file.connections[pos] = config;
        } else {
            file.connections.push(config);
        }

        self.save_config_file(&file)
    }

    pub fn load_connections(&self) -> io::Result<Vec<ConnectionConfig>> {
        let file = self.load_or_create_config_file()?;
        Ok(file.connections)
    }

    pub fn delete_connection(&self, id: Uuid) -> io::Result<()> {
        let mut file = self.load_or_create_config_file()?;
        file.connections.retain(|c| c.id != id);
        self.save_config_file(&file)
    }

    pub fn load_settings(&self) -> io::Result<AppSettings> {
        let file = self.load_or_create_config_file()?;
        Ok(file.settings)
    }

    pub fn save_settings(&self, settings: &AppSettings) -> io::Result<()> {
        let mut file = self.load_or_create_config_file()?;
        file.settings = settings.clone();
        self.save_config_file(&file)
    }

    fn load_or_create_config_file(&self) -> io::Result<ConfigFile> {
        if self.config_path.exists() {
            let content = fs::read_to_string(&self.config_path)?;
            serde_json::from_str(&content)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
        } else {
            Ok(ConfigFile {
                connections: Vec::new(),
                settings: AppSettings::default(),
            })
        }
    }

    fn save_config_file(&self, file: &ConfigFile) -> io::Result<()> {
        let content = serde_json::to_string_pretty(file)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

        fs::write(&self.config_path, content)
    }
}

impl Default for ConfigStorage {
    fn default() -> Self {
        Self::new().expect("Failed to create config storage")
    }
}
