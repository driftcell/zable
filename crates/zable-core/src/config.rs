use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::ConnectionConfig;

#[derive(Serialize, Deserialize, Clone)]
pub struct ConnectionEntry {
    pub name: String,
    pub label: String,
    #[serde(flatten)]
    pub config: ConnectionConfig,
}

#[derive(Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub entries: Vec<ConnectionEntry>,
}

impl AppConfig {
    pub fn insert_entry(&mut self, name: &str, label: &str, config: &ConnectionConfig) {
        self.entries.push(ConnectionEntry {
            name: name.to_string(),
            label: label.to_string(),
            config: config.clone(),
        });
    }

    pub fn read(path: impl AsRef<Path>) -> Result<Self, anyhow::Error> {
        let content = std::fs::read_to_string(path)?;
        let entries: Self = serde_json::from_str(&content)?;
        Ok(entries)
    }

    pub fn merge(&mut self, other: &Self) {
        for entry in &other.entries {
            if let Some(existing) = self
                .entries
                .iter_mut()
                .find(|existing| existing.name == entry.name)
            {
                *existing = entry.clone();
            } else {
                self.entries.push(entry.clone());
            }
        }
    }

    pub fn write(&self, path: impl AsRef<Path>) -> Result<(), anyhow::Error> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(&self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
