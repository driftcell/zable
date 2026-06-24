use std::path::PathBuf;

use anyhow::anyhow;

pub(crate) fn project_config_path() -> Result<PathBuf, anyhow::Error> {
    let project_dir = directories::ProjectDirs::from("com", "zable", "zable");

    project_dir
        .map(|dir| dir.config_dir().join("config.json"))
        .ok_or(anyhow!("Failed to get project config path"))
}
