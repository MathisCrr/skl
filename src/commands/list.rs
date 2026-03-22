use std::{collections::HashMap, fs};

use crate::{
    config::Config,
    lock::Lockfile,
    types::{AssetType, Only, SklError, resolve_path},
};

pub fn list(only: Option<Only>) -> Result<(), SklError> {
    let config = Config::load()?;

    let lockfile = Lockfile::load()?;

    let mut skill_repo: HashMap<String, String> = HashMap::new();
    let mut agent_repo: HashMap<String, String> = HashMap::new();
    for repo in &lockfile.repos {
        for skill in &repo.skills {
            skill_repo.insert(skill.clone(), repo.name.clone());
        }
        for agent in &repo.agents {
            agent_repo.insert(agent.clone(), repo.name.clone());
        }
    }

    for tool in &config.tools {
        if !matches!(only, Some(Only::Agent)) {
            println!("Skills:");
            if let Some(skills_dir) = resolve_path(tool, &AssetType::Skill, false, None) {
                if skills_dir.is_dir() {
                    let mut grouped: HashMap<String, Vec<String>> = HashMap::new();
                    let mut ungrouped: Vec<String> = Vec::new();

                    for entry in fs::read_dir(&skills_dir)? {
                        let entry = entry?;
                        let path = entry.path();
                        if path.is_dir() && path.join("SKILL.md").exists() {
                            let name = entry.file_name().to_string_lossy().to_string();
                            if let Some(repo) = skill_repo.get(&name) {
                                grouped.entry(repo.clone()).or_default().push(name);
                            } else {
                                ungrouped.push(name);
                            }
                        }
                    }

                    if grouped.is_empty() && ungrouped.is_empty() {
                        println!("  (none)");
                    } else {
                        let mut repos: Vec<String> = grouped.keys().cloned().collect();
                        repos.sort();
                        for repo in repos {
                            println!("  {}", repo);
                            let mut skills = grouped[&repo].clone();
                            skills.sort();
                            for skill in skills {
                                println!("    - {}", skill);
                            }
                        }
                        ungrouped.sort();
                        for skill in ungrouped {
                            println!("  - {}", skill);
                        }
                    }
                } else {
                    println!("  (none)");
                }
            }
        }

        if !matches!(only, Some(Only::Skill)) {
            println!("Agents:");
            if let Some(agents_dir) = resolve_path(tool, &AssetType::Agent, false, None) {
                if agents_dir.is_dir() {
                    let mut grouped: HashMap<String, Vec<String>> = HashMap::new();
                    let mut ungrouped: Vec<String> = Vec::new();

                    for entry in fs::read_dir(&agents_dir)? {
                        let entry = entry?;
                        let path = entry.path();
                        if path.extension().map_or(false, |e| e == "md") {
                            let name = entry.file_name().to_string_lossy().to_string();
                            if let Some(repo) = agent_repo.get(&name) {
                                grouped.entry(repo.clone()).or_default().push(name);
                            } else {
                                ungrouped.push(name);
                            }
                        }
                    }

                    if grouped.is_empty() && ungrouped.is_empty() {
                        println!("  (none)");
                    } else {
                        let mut repos: Vec<String> = grouped.keys().cloned().collect();
                        repos.sort();
                        for repo in repos {
                            println!("  {}", repo);
                            let mut agents = grouped[&repo].clone();
                            agents.sort();
                            for agent in agents {
                                println!("    - {}", agent);
                            }
                        }
                        ungrouped.sort();
                        for agent in ungrouped {
                            println!("  - {}", agent);
                        }
                    }
                } else {
                    println!("  (none)");
                }
            }
        }
    }

    Ok(())
}
