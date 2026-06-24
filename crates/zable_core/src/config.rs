use serde::{Deserialize, Serialize};

use crate::{ConnectionConfig, path::project_config_path};

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to get config path: {0}")]
    Path(#[from] anyhow::Error),

    #[error("Failed to read config file: {0}")]
    Read(#[source] std::io::Error),

    #[error("Failed to write config file: {0}")]
    Write(#[source] std::io::Error),

    #[error("Failed to parse config file: {0}")]
    Parse(#[from] serde_json::Error),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ConnectionEntry {
    pub name: String,
    pub label: String,
    #[serde(flatten)]
    pub config: ConnectionConfig,
}

impl ConnectionEntry {
    pub fn new(
        name: impl Into<String>,
        label: impl Into<String>,
        config: &ConnectionConfig,
    ) -> Self {
        Self {
            name: name.into(),
            label: label.into(),
            config: config.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub entries: Vec<ConnectionEntry>,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let path = project_config_path()?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let raw = std::fs::read_to_string(&path).map_err(ConfigError::Read)?;

        if raw.trim().is_empty() {
            return Ok(Self::default());
        }

        let config = serde_json::from_str(&raw)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let path = project_config_path()?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(ConfigError::Write)?;
        }

        let json = serde_json::to_string_pretty(self)?;

        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, json).map_err(ConfigError::Write)?;
        std::fs::rename(&tmp, &path).map_err(ConfigError::Write)?;

        Ok(())
    }
}

impl AppConfig {
    pub fn find(&self, name: &str) -> Option<&ConnectionEntry> {
        self.entries.iter().find(|entry| entry.name == name)
    }

    pub fn upsert(&mut self, entry: ConnectionEntry) {
        match self.find_index(&entry.name) {
            Some(index) => self.entries[index] = entry,
            None => self.entries.push(entry),
        }
    }

    pub fn remove(&mut self, name: &str) -> bool {
        let Some(index) = self.find_index(name) else {
            return false;
        };

        self.entries.remove(index);
        true
    }

    fn find_index(&self, name: &str) -> Option<usize> {
        self.entries.iter().position(|entry| entry.name == name)
    }
}
