use crate::types::{SklError, SklError::ConfigDirectoryNotFound};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

pub fn lock_path() -> Result<PathBuf, SklError> {
    dirs::config_dir()
        .ok_or(ConfigDirectoryNotFound)
        .map(|config| config.join("skl").join("skl.lock"))
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Lockfile {
    #[serde(default)]
    pub repos: Vec<LockedRepo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedProfile {
    pub name: String,
    pub skills: Vec<String>,
    pub agents: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedRepo {
    pub name: String,
    pub url: Option<String>,
    pub profiles: Vec<LockedProfile>,
    pub skills: Vec<String>,
    pub agents: Vec<String>,
}

impl LockedRepo {
    /// Returns skills that belong exclusively to the given profiles (not shared with any other profile).
    pub fn exclusive_skills(&self, profile_names: &[String]) -> Vec<String> {
        let other_skills: Vec<&String> = self
            .profiles
            .iter()
            .filter(|p| !profile_names.contains(&p.name))
            .flat_map(|p| p.skills.iter())
            .collect();
        self.profiles
            .iter()
            .filter(|p| profile_names.contains(&p.name))
            .flat_map(|p| p.skills.iter())
            .filter(|s| !other_skills.contains(s))
            .cloned()
            .collect()
    }

    /// Returns agents that belong exclusively to the given profiles (not shared with any other profile).
    pub fn exclusive_agents(&self, profile_names: &[String]) -> Vec<String> {
        let other_agents: Vec<&String> = self
            .profiles
            .iter()
            .filter(|p| !profile_names.contains(&p.name))
            .flat_map(|p| p.agents.iter())
            .collect();
        self.profiles
            .iter()
            .filter(|p| profile_names.contains(&p.name))
            .flat_map(|p| p.agents.iter())
            .filter(|a| !other_agents.contains(a))
            .cloned()
            .collect()
    }
}

impl Lockfile {
    pub fn load() -> Result<Self, SklError> {
        let path = lock_path()?;
        if !path.exists() {
            return Ok(Lockfile::default());
        }
        let content = fs::read_to_string(&path).map_err(SklError::IoError)?;
        toml::from_str(&content).map_err(|e| SklError::ConfigParseError(e.to_string()))
    }

    pub fn save(&self) -> Result<(), SklError> {
        let path = lock_path()?;
        fs::create_dir_all(path.parent().unwrap())?;
        let content =
            toml::to_string(self).map_err(|e| SklError::ConfigParseError(e.to_string()))?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn add_repo(&mut self, repo: LockedRepo) {
        self.repos.retain(|r| r.name != repo.name);
        self.repos.push(repo);
    }

    pub fn remove_repo(&mut self, name: &str) -> Option<LockedRepo> {
        if let Some(pos) = self.repos.iter().position(|r| r.name == name) {
            Some(self.repos.remove(pos))
        } else {
            None
        }
    }
}
