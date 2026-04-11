use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::types::SklError;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SklToml {
    #[serde(default)]
    pub profiles: HashMap<String, Profile>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Profile {
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub agents: Vec<String>,
}

impl SklToml {
    pub fn load(repo_dir: &Path) -> Result<Option<Self>, SklError> {
        let path = repo_dir.join("skl.toml");
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&path).map_err(SklError::IoError)?;
        let parsed: SklToml =
            toml::from_str(&content).map_err(|e| SklError::ConfigParseError(e.to_string()))?;
        Ok(Some(parsed))
    }

    pub fn list_profiles(&self) -> Vec<String> {
        let mut names: Vec<String> = self.profiles.keys().cloned().collect();
        names.sort();
        names
    }

    pub fn get_profile(&self, name: &str) -> Result<&Profile, SklError> {
        self.profiles
            .get(name)
            .ok_or_else(|| SklError::ProfileNotFound(name.to_string(), self.list_profiles()))
    }

    /// Resolve the union of skills and agents from a list of locked profile names.
    /// Profiles that no longer exist in skl.toml are skipped with a warning.
    pub fn resolve_profiles(&self, profile_names: &[String]) -> (Vec<String>, Vec<String>) {
        let mut skills: Vec<String> = Vec::new();
        let mut agents: Vec<String> = Vec::new();
        for name in profile_names {
            match self.get_profile(name) {
                Ok(p) => {
                    for s in &p.skills {
                        if !skills.contains(s) {
                            skills.push(s.clone());
                        }
                    }
                    for a in &p.agents {
                        if !agents.contains(a) {
                            agents.push(a.clone());
                        }
                    }
                }
                Err(_) => crate::ui::warning(&format!(
                    "profile '{}' no longer exists in skl.toml, skipping",
                    name
                )),
            }
        }
        (skills, agents)
    }
}
