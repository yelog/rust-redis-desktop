use crate::connection::ConnectionConfig;
use crate::theme::ThemePreference;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;
use uuid::Uuid;

const MAX_COMMAND_HISTORY: usize = 100;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub command: String,
    pub timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_time_ms: Option<u64>,
}

impl Default for HistoryEntry {
    fn default() -> Self {
        Self {
            command: String::new(),
            timestamp: Utc::now(),
            execution_time_ms: None,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CommandHistory {
    #[serde(default)]
    pub entries: Vec<HistoryEntry>,
    #[serde(default)]
    pub favorites: Vec<String>,
}

impl CommandHistory {
    pub fn add(&mut self, entry: HistoryEntry) {
        if self.entries.len() >= MAX_COMMAND_HISTORY {
            self.entries.remove(0);
        }
        self.entries.push(entry);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn toggle_favorite(&mut self, command: &str) {
        if let Some(pos) = self.favorites.iter().position(|c| c == command) {
            self.favorites.remove(pos);
        } else {
            self.favorites.push(command.to_string());
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default)]
    pub auto_refresh_interval: u32,
    #[serde(default, alias = "theme_mode")]
    pub theme_preference: ThemePreference,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            auto_refresh_interval: 0,
            theme_preference: ThemePreference::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ConfigFile {
    connections: Vec<ConnectionConfig>,
    #[serde(default)]
    settings: AppSettings,
    #[serde(default)]
    command_history: CommandHistory,
}

#[derive(Clone, PartialEq)]
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

    pub fn load_command_history(&self) -> io::Result<CommandHistory> {
        let file = self.load_or_create_config_file()?;
        Ok(file.command_history)
    }

    pub fn add_command_history(&self, entry: HistoryEntry) -> io::Result<()> {
        let mut file = self.load_or_create_config_file()?;
        file.command_history.add(entry);
        self.save_config_file(&file)
    }

    pub fn clear_command_history(&self) -> io::Result<()> {
        let mut file = self.load_or_create_config_file()?;
        file.command_history.clear();
        self.save_config_file(&file)
    }

    pub fn toggle_favorite(&self, command: &str) -> io::Result<()> {
        let mut file = self.load_or_create_config_file()?;
        file.command_history.toggle_favorite(command);
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
                command_history: CommandHistory::default(),
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
