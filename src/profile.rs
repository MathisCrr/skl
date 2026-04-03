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

    pub fn save(&self, repo_dir: &Path) -> Result<(), SklError> {
        let path = repo_dir.join("skl.toml");
        let content =
            toml::to_string_pretty(self).map_err(|e| SklError::ConfigParseError(e.to_string()))?;
        std::fs::write(&path, content).map_err(SklError::IoError)?;
        Ok(())
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

    /// Create or replace a profile entirely. Returns true if a profile was replaced.
    pub fn add_profile(&mut self, name: String, skills: Vec<String>, agents: Vec<String>) -> bool {
        let replaced = self.profiles.contains_key(&name);
        self.profiles.insert(name, Profile { skills, agents });
        replaced
    }

    /// Add a skill to an existing profile. No-op if the skill is already present.
    pub fn add_skill_to_profile(&mut self, profile_name: &str, skill: String) -> Result<(), SklError> {
        let available = self.list_profiles();
        let profile = self
            .profiles
            .get_mut(profile_name)
            .ok_or_else(|| SklError::ProfileNotFound(profile_name.to_string(), available))?;
        if !profile.skills.contains(&skill) {
            profile.skills.push(skill);
        }
        Ok(())
    }

    /// Resolve the union of skills and agents from a list of locked profile names.
    /// Profiles that no longer exist in skl.toml are skipped with a warning.
    pub fn resolve_profiles(&self, profile_names: &[String]) -> (Vec<String>, Vec<String>) {
        let mut skills: Vec<String> = Vec::new();
        let mut agents: Vec<String> = Vec::new();
        for name in profile_names {
            match self.get_profile(name) {
                Ok(p) => {
                    for s in &p.skills { if !skills.contains(s) { skills.push(s.clone()); } }
                    for a in &p.agents { if !agents.contains(a) { agents.push(a.clone()); } }
                }
                Err(_) => println!("⚠️  Profile '{}' no longer exists in skl.toml, skipping.", name),
            }
        }
        (skills, agents)
    }

    /// Add an agent to an existing profile. No-op if the agent is already present.
    pub fn add_agent_to_profile(&mut self, profile_name: &str, agent: String) -> Result<(), SklError> {
        let available = self.list_profiles();
        let profile = self
            .profiles
            .get_mut(profile_name)
            .ok_or_else(|| SklError::ProfileNotFound(profile_name.to_string(), available))?;
        if !profile.agents.contains(&agent) {
            profile.agents.push(agent);
        }
        Ok(())
    }
}
