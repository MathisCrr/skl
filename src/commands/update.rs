use crate::{
    commands::repo::{copy_dir, find_files, normalize_repo_id},
    config::{Config, source_path},
    lock::{Lockfile, LockedRepo},
    profile::SklToml,
    types::{AssetType, SklError, Tool, resolve_path},
    ui,
};
use colored::Colorize;
use std::{fs, path::Path, process::{Command, Stdio}};

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
        ui::warning("no repositories to update");
        return Ok(());
    }

    for repo_id in &repo_ids {
        let locked = lockfile
            .repos
            .iter()
            .find(|r| r.name == *repo_id)
            .ok_or_else(|| SklError::RepoNotFound(repo_id.clone()))?
            .clone();

        let source_dir = source_path()?.join(repo_id);
        let dir = source_dir.to_str().unwrap();

        let sp = ui::spinner(&format!("Updating {}", repo_id.bold()));

        let fetch = Command::new("git")
            .args(["-C", dir, "fetch", "--depth=1"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;
        if !fetch.success() {
            sp.finish_and_clear();
            ui::warning(&format!("failed to fetch {}, skipping", repo_id.bold()));
            continue;
        }

        let reset = Command::new("git")
            .args(["-C", dir, "reset", "--hard", "origin/HEAD"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;
        if !reset.success() {
            sp.finish_and_clear();
            ui::warning(&format!("failed to reset {}, skipping", repo_id.bold()));
            continue;
        }

        sp.finish_and_clear();

        // Re-apply profiles if any were used during install
        let profile_filter: Option<(Vec<String>, Vec<String>)> = if !locked.profiles.is_empty() {
            match SklToml::load(&source_dir)? {
                Some(skl_toml) => {
                    let names: Vec<String> = locked.profiles.iter().map(|p| p.name.clone()).collect();
                    Some(skl_toml.resolve_profiles(&names))
                }
                None => {
                    ui::warning(&format!("skl.toml not found in {}, installing all skills", repo_id.bold()));
                    None
                }
            }
        } else {
            None
        };

        let current_skills = scan_skills(&source_dir)?;
        let current_agents = scan_agents(&source_dir)?;

        // Effective lists after applying profile filter
        let effective_skills: Vec<String> = match &profile_filter {
            Some((ps, _)) => current_skills.iter().filter(|s| ps.contains(s)).cloned().collect(),
            None => current_skills.clone(),
        };
        let effective_agents: Vec<String> = match &profile_filter {
            Some((_, pa)) => current_agents.iter().filter(|a| pa.contains(a)).cloned().collect(),
            None => current_agents.clone(),
        };

        let mut installed_skills = Vec::new();
        let mut installed_agents = Vec::new();

        for tool in &tools {
            let skills_dest = resolve_path(tool, &AssetType::Skill, false, None)
                .ok_or(SklError::InvalidArguments("Could not resolve destination path".to_string()))?;
            let agents_dest = resolve_path(tool, &AssetType::Agent, false, None)
                .ok_or(SklError::InvalidArguments("Could not resolve destination path".to_string()))?;

            for skill in &locked.skills {
                if !effective_skills.contains(skill) {
                    let path = skills_dest.join(skill);
                    if path.exists() {
                        fs::remove_dir_all(&path)?;
                    }
                    ui::removed(&format!("skill  {}", skill.bold()));
                }
            }

            for agent in &locked.agents {
                if !effective_agents.contains(agent) {
                    let path = agents_dest.join(agent);
                    if path.exists() {
                        fs::remove_file(&path)?;
                    }
                    ui::removed(&format!("agent  {}", agent.bold()));
                }
            }

            for skill in &effective_skills {
                if let Some(path) = find_skill_path(&source_dir, skill)? {
                    fs::create_dir_all(&skills_dest)?;
                    copy_dir(&path, &skills_dest.join(skill))?;
                    ui::success(&format!("skill  {}", skill.bold()));
                    installed_skills.push(skill.clone());
                }
            }

            let agents_dir = source_dir.join("agents");
            for agent in &effective_agents {
                let agent_path = agents_dir.join(agent);
                fs::create_dir_all(&agents_dest)?;
                fs::copy(&agent_path, agents_dest.join(agent))?;
                ui::success(&format!("agent  {}", agent.bold()));
                installed_agents.push(agent.clone());
            }
        }

        // Update locked profiles with refreshed skill/agent lists from skl.toml
        let updated_profiles = if !locked.profiles.is_empty() {
            match SklToml::load(&source_dir)? {
                Some(skl_toml) => locked.profiles.iter().map(|lp| {
                    match skl_toml.get_profile(&lp.name) {
                        Ok(p) => crate::lock::LockedProfile {
                            name: lp.name.clone(),
                            skills: p.skills.clone(),
                            agents: p.agents.clone(),
                        },
                        Err(_) => lp.clone(),
                    }
                }).collect(),
                None => locked.profiles.clone(),
            }
        } else {
            vec![]
        };

        lockfile.add_repo(LockedRepo {
            name: repo_id.clone(),
            url: locked.url.clone(),
            profiles: updated_profiles,
            skills: installed_skills,
            agents: installed_agents,
        });
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
