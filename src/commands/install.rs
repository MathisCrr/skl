use crate::{
    commands::{init::init, repo::{copy_dir, find_files, normalize_repo_id}},
    config::{Config, source_path},
    lock::{Lockfile, LockedProfile, LockedRepo},
    profile::SklToml,
    types::{AssetType, Only, SklError, Tool, resolve_path},
    ui,
};
use colored::Colorize;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

pub fn install(
    source: String,
    tool: Option<Tool>,
    local: bool,
    dest: Option<PathBuf>,
    skills: Option<Vec<String>>,
    agents: Option<Vec<String>>,
    only: Option<Only>,
    profile: Option<String>,
) -> Result<(), SklError> {
    let config = match Config::load() {
        Ok(c) if !c.is_empty() => c,
        _ => init()?,
    };

    let tools: Vec<Tool> = match tool {
        Some(t) => vec![t],
        None => config.tools,
    };

    if !source.starts_with("https://") && !source.starts_with("http://") {
        return Err(SklError::InvalidArguments(
            "source must be a URL (https://...)".to_string(),
        ));
    }

    if only.is_some() && (skills.is_some() || agents.is_some()) {
        return Err(SklError::InvalidArguments(
            "--only cannot be combined with --skill or --agent".to_string(),
        ));
    }
    if profile.is_some() && (skills.is_some() || agents.is_some() || only.is_some()) {
        return Err(SklError::InvalidArguments(
            "--profile cannot be combined with --skill, --agent or --only".to_string(),
        ));
    }

    let repo_id = normalize_repo_id(&source);
    let local_source = source_path()?.join(&repo_id);
    fs::create_dir_all(local_source.parent().unwrap())?;
    if local_source.exists() {
        ui::warning(&format!("{} already cloned — run {} to refresh", repo_id.bold(), "skl update".cyan()));
    } else {
        let sp = ui::spinner(&format!("Cloning {}", repo_id.bold()));
        let status = Command::new("git")
            .args(["clone", "--depth=1", &source, local_source.to_str().unwrap()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;
        sp.finish_and_clear();
        if !status.success() {
            return Err(SklError::GitCloneFailed);
        }
    }

    // Resolve profile filter if --profile was specified
    let (skills, agents, locked_profile) = if let Some(ref profile_name) = profile {
        let skl_toml = SklToml::load(&local_source)?
            .ok_or_else(|| SklError::ProfileNotFound(profile_name.clone(), vec![]))?;
        let p = skl_toml.get_profile(profile_name)?;
        let locked = LockedProfile {
            name: profile_name.clone(),
            skills: p.skills.clone(),
            agents: p.agents.clone(),
        };
        (
            Some(p.skills.clone()),
            Some(p.agents.clone()),
            Some(locked),
        )
    } else {
        (skills, agents, None)
    };

    let mut installed_skills = Vec::new();
    let mut installed_agents = Vec::new();

    for tool in &tools {
        let skills_dest = resolve_path(tool, &AssetType::Skill, local, dest.as_ref())
            .ok_or(SklError::InvalidArguments("Could not resolve destination path".to_string()))?;
        let agents_dest = resolve_path(tool, &AssetType::Agent, local, dest.as_ref())
            .ok_or(SklError::InvalidArguments("Could not resolve destination path".to_string()))?;

        match &only {
            Some(Only::Skill) => {
                installed_skills = deploy_skills(&local_source, &skills_dest, None)?;
            }
            Some(Only::Agent) => {
                installed_agents = deploy_agents(&local_source, &agents_dest, None)?;
            }
            None => {
                let has_skill_filter = skills.is_some();
                let has_agent_filter = agents.is_some();

                if has_skill_filter || !has_agent_filter {
                    installed_skills = deploy_skills(&local_source, &skills_dest, skills.clone())?;
                }
                if has_agent_filter || !has_skill_filter {
                    installed_agents = deploy_agents(&local_source, &agents_dest, agents.clone())?;
                }
            }
        }
    }

    let mut lockfile = Lockfile::load()?;
    lockfile.add_repo(LockedRepo {
        name: repo_id,
        url: Some(source),
        profiles: locked_profile.into_iter().collect(),
        skills: installed_skills,
        agents: installed_agents,
    });
    lockfile.save()?;

    Ok(())
}

fn deploy_skills(source: &Path, dest: &Path, skills: Option<Vec<String>>) -> Result<Vec<String>, SklError> {
    let found_skills = find_files(source, "SKILL.md")?;

    if found_skills.is_empty() {
        ui::warning("no skills found");
        return Ok(vec![]);
    }

    fs::create_dir_all(dest)?;
    let mut installed_skills: Vec<String> = Vec::new();

    for found_skill in &found_skills {
        let found_skill_name = found_skill.file_name().unwrap().to_str().unwrap();

        if let Some(ref filter) = skills {
            if !filter.iter().any(|s| s == found_skill_name) {
                continue;
            }
        }

        copy_dir(found_skill, &dest.join(found_skill_name))?;
        installed_skills.push(found_skill_name.to_string());
        ui::success(&format!("skill  {}", found_skill_name.bold()));
    }

    if let Some(filter) = skills {
        let not_found: Vec<String> = filter
            .into_iter()
            .filter(|s| !installed_skills.contains(s))
            .collect();
        if !not_found.is_empty() {
            ui::warning(&format!("skills not found: {}", not_found.join(", ")));
        }
    }

    Ok(installed_skills)
}

fn deploy_agents(source: &Path, dest: &Path, agents: Option<Vec<String>>) -> Result<Vec<String>, SklError> {
    let agents_dir = source.join("agents");
    let found_agents = find_files(&agents_dir, ".md")?;

    if found_agents.is_empty() {
        ui::warning("no agents found");
        return Ok(vec![]);
    }

    fs::create_dir_all(dest)?;
    let mut installed_agents: Vec<String> = Vec::new();

    for found_agent in &found_agents {
        let agent_name = found_agent.file_name().unwrap().to_str().unwrap();

        if let Some(ref filter) = agents {
            if !filter.iter().any(|s| s == agent_name) {
                continue;
            }
        }

        fs::copy(found_agent, dest.join(agent_name))?;
        installed_agents.push(agent_name.to_string());
        ui::success(&format!("agent  {}", agent_name.bold()));
    }

    if let Some(filter) = agents {
        let not_found: Vec<String> = filter
            .into_iter()
            .filter(|a| !installed_agents.contains(a))
            .collect();
        if !not_found.is_empty() {
            ui::warning(&format!("agents not found: {}", not_found.join(", ")));
        }
    }

    Ok(installed_agents)
}
