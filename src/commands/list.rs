use std::{collections::HashMap, fs};

use crate::{
    config::Config,
    lock::Lockfile,
    types::{AssetType, Only, SklError, resolve_path},
};
use colored::Colorize;

pub fn list(only: Option<Only>) -> Result<(), SklError> {
    let config = Config::load()?;
    let lockfile = Lockfile::load()?;

    for tool in &config.tools {
        if !matches!(only, Some(Only::Agent)) {
            if let Some(skills_dir) = resolve_path(tool, &AssetType::Skill, false, None) {
                if skills_dir.is_dir() {
                    let mut grouped: HashMap<String, Vec<(String, Vec<String>)>> = HashMap::new();
                    let mut ungrouped: Vec<String> = Vec::new();

                    for entry in fs::read_dir(&skills_dir)? {
                        let entry = entry?;
                        let path = entry.path();
                        if path.is_dir() && path.join("SKILL.md").exists() {
                            let name = entry.file_name().to_string_lossy().to_string();
                            if let Some(repo) = lockfile.repos.iter().find(|r| r.skills.contains(&name)) {
                                let profiles: Vec<String> = repo.profiles.iter()
                                    .filter(|p| p.skills.contains(&name))
                                    .map(|p| p.name.clone())
                                    .collect();
                                grouped.entry(repo.name.clone()).or_default().push((name, profiles));
                            } else {
                                ungrouped.push(name);
                            }
                        }
                    }

                    if !grouped.is_empty() || !ungrouped.is_empty() {
                        println!("{}", "Skills".bold());
                        let mut repos: Vec<String> = grouped.keys().cloned().collect();
                        repos.sort();
                        for repo in repos {
                            let repo_profiles: Vec<String> = lockfile.repos.iter()
                                .find(|r| r.name == repo)
                                .map(|r| r.profiles.iter().map(|p| p.name.clone()).collect())
                                .unwrap_or_default();
                            if repo_profiles.is_empty() {
                                println!("  {}", repo.cyan());
                            } else {
                                println!("  {}  {}", repo.cyan(), format!("[{}]", repo_profiles.join(", ")).dimmed());
                            }

                            let mut skills = grouped[&repo].clone();
                            skills.sort_by(|a, b| a.0.cmp(&b.0));
                            let max_len = skills.iter().map(|(n, _)| n.len()).max().unwrap_or(0);
                            for (skill, profiles) in skills {
                                if profiles.is_empty() {
                                    println!("    {} {}", "›".dimmed(), skill);
                                } else {
                                    println!("    {} {:<width$}  {} {}", "›".dimmed(), skill, "·".dimmed(), profiles.join(", ").dimmed(), width = max_len);
                                }
                            }
                        }
                        if !ungrouped.is_empty() {
                            ungrouped.sort();
                            println!("  {}", "(local)".dimmed());
                            for skill in ungrouped {
                                println!("    {} {}", "›".dimmed(), skill);
                            }
                        }
                    }
                }
            }
        }

        if !matches!(only, Some(Only::Skill)) {
            if let Some(agents_dir) = resolve_path(tool, &AssetType::Agent, false, None) {
                if agents_dir.is_dir() {
                    let mut grouped: HashMap<String, Vec<(String, Vec<String>)>> = HashMap::new();
                    let mut ungrouped: Vec<String> = Vec::new();

                    for entry in fs::read_dir(&agents_dir)? {
                        let entry = entry?;
                        let path = entry.path();
                        if path.extension().map_or(false, |e| e == "md") {
                            let name = entry.file_name().to_string_lossy().to_string();
                            if let Some(repo) = lockfile.repos.iter().find(|r| r.agents.contains(&name)) {
                                let profiles: Vec<String> = repo.profiles.iter()
                                    .filter(|p| p.agents.contains(&name))
                                    .map(|p| p.name.clone())
                                    .collect();
                                grouped.entry(repo.name.clone()).or_default().push((name, profiles));
                            } else {
                                ungrouped.push(name);
                            }
                        }
                    }

                    if !grouped.is_empty() || !ungrouped.is_empty() {
                        println!("{}", "Agents".bold());
                        let mut repos: Vec<String> = grouped.keys().cloned().collect();
                        repos.sort();
                        for repo in repos {
                            let repo_profiles: Vec<String> = lockfile.repos.iter()
                                .find(|r| r.name == repo)
                                .map(|r| r.profiles.iter().map(|p| p.name.clone()).collect())
                                .unwrap_or_default();
                            if repo_profiles.is_empty() {
                                println!("  {}", repo.cyan());
                            } else {
                                println!("  {}  {}", repo.cyan(), format!("[{}]", repo_profiles.join(", ")).dimmed());
                            }

                            let mut agents = grouped[&repo].clone();
                            agents.sort_by(|a, b| a.0.cmp(&b.0));
                            let max_len = agents.iter().map(|(n, _)| n.len()).max().unwrap_or(0);
                            for (agent, profiles) in agents {
                                if profiles.is_empty() {
                                    println!("    {} {}", "›".dimmed(), agent);
                                } else {
                                    println!("    {} {:<width$}  {} {}", "›".dimmed(), agent, "·".dimmed(), profiles.join(", ").dimmed(), width = max_len);
                                }
                            }
                        }
                        if !ungrouped.is_empty() {
                            ungrouped.sort();
                            println!("  {}", "(local)".dimmed());
                            for agent in ungrouped {
                                println!("    {} {}", "›".dimmed(), agent);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
