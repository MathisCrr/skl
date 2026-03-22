use crate::types::{SklError, Tool};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

pub fn config_path() -> Result<PathBuf, SklError> {
    dirs::config_dir()
        .ok_or(SklError::ConfigDirectoryNotFound)
        .map(|config| config.join("skl").join("config.toml"))
}

pub fn source_path() -> Result<PathBuf, SklError> {
    dirs::config_dir()
        .ok_or(SklError::ConfigDirectoryNotFound)
        .map(|config| config.join("skl").join("sources"))
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub tools: Vec<Tool>,
}

impl Config {
    pub fn load() -> Result<Self, SklError> {
        let path = config_path()?;
        let content = fs::read_to_string(&path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                SklError::ConfigNotFound
            } else {
                SklError::IoError(e)
            }
        })?;
        toml::from_str(&content).map_err(|e| SklError::ConfigParseError(e.to_string()))
    }

    pub fn save(&self) -> Result<(), SklError> {
        let path = config_path()?;
        fs::create_dir_all(path.parent().unwrap())?;
        let content =
            toml::to_string(self).map_err(|e| SklError::ConfigParseError(e.to_string()))?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }
}
