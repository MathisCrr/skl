use std::{collections::HashMap, fs};
use std::path::Path;

use crate::{
    config::Config,
    lock::Lockfile,
    types::{AssetType, Only, SklError, resolve_path},
};
use colored::Colorize;

fn read_description(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    let mut in_frontmatter = false;
    for line in content.lines() {
        if line == "---" {
            if !in_frontmatter {
                in_frontmatter = true;
                continue;
            } else {
                break;
            }
        }
        if in_frontmatter {
            if let Some(desc) = line.strip_prefix("description:") {
                return Some(desc.trim().to_string());
            }
        }
    }
    None
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let t: String = s.chars().take(max - 1).collect();
        format!("{}…", t)
    }
}

pub fn list(only: Option<Only>) -> Result<(), SklError> {
    let config = Config::load()?;
    let lockfile = Lockfile::load()?;

    for tool in &config.tools {
        if !matches!(only, Some(Only::Agent)) {
            if let Some(skills_dir) = resolve_path(tool, &AssetType::Skill, false, None) {
                if skills_dir.is_dir() {
                    let mut grouped: HashMap<String, Vec<(String, Option<String>, Vec<String>)>> = HashMap::new();
                    let mut ungrouped: Vec<(String, Option<String>)> = Vec::new();

                    for entry in fs::read_dir(&skills_dir)? {
                        let entry = entry?;
                        let path = entry.path();
                        if path.is_dir() && path.join("SKILL.md").exists() {
                            let name = entry.file_name().to_string_lossy().to_string();
                            let desc = read_description(&path.join("SKILL.md"));
                            if let Some(repo) = lockfile.repos.iter().find(|r| r.skills.contains(&name)) {
                                let profiles: Vec<String> = repo.profiles.iter()
                                    .filter(|p| p.skills.contains(&name))
                                    .map(|p| p.name.clone())
                                    .collect();
                                grouped.entry(repo.name.clone()).or_default().push((name, desc, profiles));
                            } else {
                                ungrouped.push((name, desc));
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
                            let max_name_len = skills.iter().map(|(n, _, _)| n.len()).max().unwrap_or(0);
                            for (skill, desc, profiles) in skills {
                                let desc_part = desc
                                    .as_deref()
                                    .map(|d| format!("  {}", truncate(d, 50).dimmed()))
                                    .unwrap_or_default();
                                let profiles_part = if profiles.is_empty() {
                                    String::new()
                                } else {
                                    format!("  {} {}", "·".dimmed(), profiles.join(", ").bright_yellow())
                                };
                                println!("    {} {:<width$}{}{}", "›".dimmed(), skill, desc_part, profiles_part, width = max_name_len);
                            }
                        }
                        if !ungrouped.is_empty() {
                            ungrouped.sort_by(|a, b| a.0.cmp(&b.0));
                            println!("  {}", "(local)".dimmed());
                            for (skill, desc) in ungrouped {
                                let desc_part = desc
                                    .as_deref()
                                    .map(|d| format!("  {}", truncate(d, 50).dimmed()))
                                    .unwrap_or_default();
                                println!("    {} {}{}", "›".dimmed(), skill, desc_part);
                            }
                        }
                    }
                }
            }
        }

        if !matches!(only, Some(Only::Skill)) {
            if let Some(agents_dir) = resolve_path(tool, &AssetType::Agent, false, None) {
                if agents_dir.is_dir() {
                    let mut grouped: HashMap<String, Vec<(String, Option<String>, Vec<String>)>> = HashMap::new();
                    let mut ungrouped: Vec<(String, Option<String>)> = Vec::new();

                    for entry in fs::read_dir(&agents_dir)? {
                        let entry = entry?;
                        let path = entry.path();
                        if path.extension().map_or(false, |e| e == "md") {
                            let name = entry.file_name().to_string_lossy().to_string();
                            let desc = read_description(&path);
                            if let Some(repo) = lockfile.repos.iter().find(|r| r.agents.contains(&name)) {
                                let profiles: Vec<String> = repo.profiles.iter()
                                    .filter(|p| p.agents.contains(&name))
                                    .map(|p| p.name.clone())
                                    .collect();
                                grouped.entry(repo.name.clone()).or_default().push((name, desc, profiles));
                            } else {
                                ungrouped.push((name, desc));
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
                            let max_name_len = agents.iter().map(|(n, _, _)| n.len()).max().unwrap_or(0);
                            for (agent, desc, profiles) in agents {
                                let desc_part = desc
                                    .as_deref()
                                    .map(|d| format!("  {}", truncate(d, 50).dimmed()))
                                    .unwrap_or_default();
                                let profiles_part = if profiles.is_empty() {
                                    String::new()
                                } else {
                                    format!("  {} {}", "·".dimmed(), profiles.join(", ").bright_yellow())
                                };
                                println!("    {} {:<width$}{}{}", "›".dimmed(), agent, desc_part, profiles_part, width = max_name_len);
                            }
                        }
                        if !ungrouped.is_empty() {
                            ungrouped.sort_by(|a, b| a.0.cmp(&b.0));
                            println!("  {}", "(local)".dimmed());
                            for (agent, desc) in ungrouped {
                                let desc_part = desc
                                    .as_deref()
                                    .map(|d| format!("  {}", truncate(d, 50).dimmed()))
                                    .unwrap_or_default();
                                println!("    {} {}{}", "›".dimmed(), agent, desc_part);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
