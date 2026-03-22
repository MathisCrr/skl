use crate::{
    commands::repo::{copy_dir, find_files, normalize_repo_id},
    config::{Config, source_path},
    lock::{Lockfile, LockedRepo},
    types::{AssetType, SklError, Tool, resolve_path},
};
use std::{fs, path::Path, process::Command};

pub fn update(repo: Option<String>, tool: Option<Tool>) -> Result<(), SklError> {
    let config = Config::load()?;

    let tools: Vec<Tool> = match tool {
        Some(t) => vec![t],
        None => config.tools,
    };

    let mut lockfile = Lockfile::load()?;

    let repo_ids: Vec<String> = match repo {
        Some(r) => vec![normalize_repo_id(&r)],
        None => lockfile.repos.iter().map(|r| r.name.clone()).collect(),
    };

    if repo_ids.is_empty() {
        println!("No repositories to update.");
        return Ok(());
    }

    for repo_id in &repo_ids {
        let locked = lockfile
            .repos
            .iter()
            .find(|r| r.name == *repo_id)
            .ok_or_else(|| SklError::RepoNotFound(repo_id.clone()))?
            .clone();

        println!("Updating {}...", repo_id);

        let source_dir = source_path()?.join(repo_id);
        let dir = source_dir.to_str().unwrap();

        let fetch = Command::new("git")
            .args(["-C", dir, "fetch", "--depth=1"])
            .status()?;
        if !fetch.success() {
            println!("⚠️  Failed to fetch {}, skipping.", repo_id);
            continue;
        }

        let reset = Command::new("git")
            .args(["-C", dir, "reset", "--hard", "origin/HEAD"])
            .status()?;
        if !reset.success() {
            println!("⚠️  Failed to reset {}, skipping.", repo_id);
            continue;
        }

        let current_skills = scan_skills(&source_dir)?;
        let current_agents = scan_agents(&source_dir)?;

        let mut installed_skills = Vec::new();
        let mut installed_agents = Vec::new();

        for tool in &tools {
            let skills_dest = resolve_path(tool, &AssetType::Skill, false, None)
                .ok_or(SklError::InvalidArguments("Could not resolve destination path".to_string()))?;
            let agents_dest = resolve_path(tool, &AssetType::Agent, false, None)
                .ok_or(SklError::InvalidArguments("Could not resolve destination path".to_string()))?;

            for skill in &locked.skills {
                if !current_skills.contains(skill) {
                    let path = skills_dest.join(skill);
                    if path.exists() {
                        fs::remove_dir_all(&path)?;
                    }
                    println!("🗑️  Removed skill: {}", skill);
                }
            }

            for agent in &locked.agents {
                if !current_agents.contains(agent) {
                    let path = agents_dest.join(agent);
                    if path.exists() {
                        fs::remove_file(&path)?;
                    }
                    println!("🗑️  Removed agent: {}", agent);
                }
            }

            for skill in &locked.skills {
                if current_skills.contains(skill) {
                    if let Some(path) = find_skill_path(&source_dir, skill)? {
                        fs::create_dir_all(&skills_dest)?;
                        copy_dir(&path, &skills_dest.join(skill))?;
                        println!("🔄 Updated skill: {}", skill);
                        installed_skills.push(skill.clone());
                    }
                }
            }

            let agents_dir = source_dir.join("agents");
            for agent in &locked.agents {
                if current_agents.contains(agent) {
                    let agent_path = agents_dir.join(agent);
                    fs::create_dir_all(&agents_dest)?;
                    fs::copy(&agent_path, agents_dest.join(agent))?;
                    println!("🔄 Updated agent: {}", agent);
                    installed_agents.push(agent.clone());
                }
            }
        }

        lockfile.add_repo(LockedRepo {
            name: repo_id.clone(),
            url: locked.url.clone(),
            skills: installed_skills,
            agents: installed_agents,
        });

        println!("✅ Updated {}", repo_id);
    }

    lockfile.save()?;
    Ok(())
}

fn scan_skills(source: &Path) -> Result<Vec<String>, SklError> {
    find_files(source, "SKILL.md").map(|paths| {
        paths
            .iter()
            .filter_map(|p| p.file_name()?.to_str().map(|s| s.to_string()))
            .collect()
    })
}

fn scan_agents(source: &Path) -> Result<Vec<String>, SklError> {
    let agents_dir = source.join("agents");
    if !agents_dir.is_dir() {
        return Ok(vec![]);
    }
    let mut agents = Vec::new();
    for entry in fs::read_dir(&agents_dir)? {
        let entry = entry?;
        if entry.path().extension().map_or(false, |e| e == "md") {
            agents.push(entry.file_name().to_string_lossy().to_string());
        }
    }
    Ok(agents)
}

fn find_skill_path(source: &Path, skill_name: &str) -> Result<Option<std::path::PathBuf>, SklError> {
    Ok(find_files(source, "SKILL.md")?
        .into_iter()
        .find(|p| p.file_name().map_or(false, |n| n == skill_name)))
}
