use std::fs;

use crate::{
    config::Config,
    types::{AssetType, SklError, resolve_path},
};
use colored::Colorize;

pub fn show(name: String) -> Result<(), SklError> {
    let config = Config::load()?;

    for tool in &config.tools {
        // Check skills first
        if let Some(skills_dir) = resolve_path(tool, &AssetType::Skill, false, None) {
            let skill_file = skills_dir.join(&name).join("SKILL.md");
            if skill_file.exists() {
                let content = fs::read_to_string(&skill_file).map_err(SklError::IoError)?;
                println!("{} {}", "skill".dimmed(), name.bold());
                println!();
                print!("{}", content);
                return Ok(());
            }
        }

        // Check agents
        if let Some(agents_dir) = resolve_path(tool, &AssetType::Agent, false, None) {
            let agent_name = if name.ends_with(".md") {
                name.clone()
            } else {
                format!("{}.md", name)
            };
            let agent_file = agents_dir.join(&agent_name);
            if agent_file.exists() {
                let content = fs::read_to_string(&agent_file).map_err(SklError::IoError)?;
                println!("{} {}", "agent".dimmed(), name.bold());
                println!();
                print!("{}", content);
                return Ok(());
            }
        }
    }

    Err(SklError::RepoNotFound(name))
}
