use std::{collections::HashMap, fs};

use crate::{
    config::Config,
    lock::Lockfile,
    types::{AssetType, Only, SklError, resolve_path},
};

pub fn list(only: Option<Only>) -> Result<(), SklError> {
    let config = Config::load()?;
    let lockfile = Lockfile::load()?;

    for tool in &config.tools {
        if !matches!(only, Some(Only::Agent)) {
            println!("Skills:");
            if let Some(skills_dir) = resolve_path(tool, &AssetType::Skill, false, None) {
                if skills_dir.is_dir() {
                    // repo → [(skill_name, profiles)]
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

                    if grouped.is_empty() && ungrouped.is_empty() {
                        println!("  (none)");
                    } else {
                        let mut repos: Vec<String> = grouped.keys().cloned().collect();
                        repos.sort();
                        for repo in repos {
                            let repo_profiles: Vec<String> = lockfile.repos.iter()
                                .find(|r| r.name == repo)
                                .map(|r| r.profiles.iter().map(|p| p.name.clone()).collect())
                                .unwrap_or_default();
                            if repo_profiles.is_empty() {
                                println!("  {}", repo);
                            } else {
                                println!("  {}  [{}]", repo, repo_profiles.join(", "));
                            }

                            let mut skills = grouped[&repo].clone();
                            skills.sort_by(|a, b| a.0.cmp(&b.0));
                            let max_len = skills.iter().map(|(n, _)| n.len()).max().unwrap_or(0);
                            for (skill, profiles) in skills {
                                if profiles.is_empty() {
                                    println!("    - {}", skill);
                                } else {
                                    println!("    - {:<width$}  · {}", skill, profiles.join(", "), width = max_len);
                                }
                            }
                        }
                        if !ungrouped.is_empty() {
                            ungrouped.sort();
                            println!("  (no repo)");
                            for skill in ungrouped {
                                println!("    - {}", skill);
                            }
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

                    if grouped.is_empty() && ungrouped.is_empty() {
                        println!("  (none)");
                    } else {
                        let mut repos: Vec<String> = grouped.keys().cloned().collect();
                        repos.sort();
                        for repo in repos {
                            let repo_profiles: Vec<String> = lockfile.repos.iter()
                                .find(|r| r.name == repo)
                                .map(|r| r.profiles.iter().map(|p| p.name.clone()).collect())
                                .unwrap_or_default();
                            if repo_profiles.is_empty() {
                                println!("  {}", repo);
                            } else {
                                println!("  {}  [{}]", repo, repo_profiles.join(", "));
                            }

                            let mut agents = grouped[&repo].clone();
                            agents.sort_by(|a, b| a.0.cmp(&b.0));
                            let max_len = agents.iter().map(|(n, _)| n.len()).max().unwrap_or(0);
                            for (agent, profiles) in agents {
                                if profiles.is_empty() {
                                    println!("    - {}", agent);
                                } else {
                                    println!("    - {:<width$}  · {}", agent, profiles.join(", "), width = max_len);
                                }
                            }
                        }
                        if !ungrouped.is_empty() {
                            ungrouped.sort();
                            println!("  (no repo)");
                            for agent in ungrouped {
                                println!("    - {}", agent);
                            }
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
