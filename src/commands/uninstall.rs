use crate::{
    commands::repo::normalize_repo_id,
    config::{Config, source_path},
    lock::Lockfile,
    types::{AssetType, SklError, resolve_path},
};
use std::fs;

pub fn uninstall(repo: String, skill: Option<String>, profile: Option<String>) -> Result<(), SklError> {
    if skill.is_some() && profile.is_some() {
        return Err(SklError::InvalidArguments(
            "--skill and --profile cannot be combined".to_string(),
        ));
    }

    let config = Config::load()?;
    let repo_id = normalize_repo_id(&repo);

    let mut lockfile = Lockfile::load()?;

    let locked = lockfile
        .repos
        .iter()
        .find(|r| r.name == repo_id)
        .ok_or_else(|| SklError::RepoNotFound(repo_id.clone()))?
        .clone();

    if let Some(skill_name) = skill {
        // Remove a single skill
        uninstall_skill(&config.tools, &repo_id, &skill_name, &mut lockfile)?;
    } else if let Some(profile_name) = profile {
        // Remove all skills belonging to this profile (not shared with other profiles)
        uninstall_profile(&config.tools, &repo_id, &profile_name, &locked, &mut lockfile)?;
    } else {
        // Remove everything
        lockfile.remove_repo(&repo_id);
        for tool in &config.tools {
            let skills_dest = resolve_path(tool, &AssetType::Skill, false, None)
                .ok_or(SklError::InvalidArguments("Could not resolve destination path".to_string()))?;
            let agents_dest = resolve_path(tool, &AssetType::Agent, false, None)
                .ok_or(SklError::InvalidArguments("Could not resolve destination path".to_string()))?;

            for skill in &locked.skills {
                let path = skills_dest.join(skill);
                if path.exists() {
                    fs::remove_dir_all(&path)?;
                    println!("🗑️  Removed skill: {}", skill);
                } else {
                    println!("⚠️  Skill '{}' not found on filesystem, skipping", skill);
                }
            }
            for agent in &locked.agents {
                let path = agents_dest.join(agent);
                if path.exists() {
                    fs::remove_file(&path)?;
                    println!("🗑️  Removed agent: {}", agent);
                } else {
                    println!("⚠️  Agent '{}' not found on filesystem, skipping", agent);
                }
            }
        }
        let source = source_path()?.join(&repo_id);
        if source.exists() {
            fs::remove_dir_all(&source)?;
        }
        println!("✅ Uninstalled repo: {}", repo_id);
    }

    lockfile.save()?;
    Ok(())
}

fn uninstall_skill(tools: &[crate::types::Tool], repo_id: &str, skill_name: &str, lockfile: &mut Lockfile) -> Result<(), SklError> {
    let locked = lockfile.repos.iter_mut().find(|r| r.name == repo_id)
        .ok_or_else(|| SklError::RepoNotFound(repo_id.to_string()))?;

    if !locked.skills.contains(&skill_name.to_string()) {
        return Err(SklError::InvalidArguments(format!("Skill '{}' not found in repo '{}'", skill_name, repo_id)));
    }

    for tool in tools {
        let skills_dest = resolve_path(tool, &AssetType::Skill, false, None)
            .ok_or(SklError::InvalidArguments("Could not resolve destination path".to_string()))?;
        let path = skills_dest.join(skill_name);
        if path.exists() {
            fs::remove_dir_all(&path)?;
            println!("🗑️  Removed skill: {}", skill_name);
        }
    }

    locked.skills.retain(|s| s != skill_name);
    // Remove skill from any profile it belongs to
    for profile in &mut locked.profiles {
        profile.skills.retain(|s| s != skill_name);
    }

    Ok(())
}

fn uninstall_profile(tools: &[crate::types::Tool], repo_id: &str, profile_name: &str, locked: &crate::lock::LockedRepo, lockfile: &mut Lockfile) -> Result<(), SklError> {
    if !locked.profiles.iter().any(|p| p.name == profile_name) {
        return Err(SklError::ProfileNotFound(
            profile_name.to_string(),
            locked.profiles.iter().map(|p| p.name.clone()).collect(),
        ));
    }

    let profile_names = vec![profile_name.to_string()];
    let skills_to_remove = locked.exclusive_skills(&profile_names);
    let agents_to_remove = locked.exclusive_agents(&profile_names);

    for tool in tools {
        let skills_dest = resolve_path(tool, &AssetType::Skill, false, None)
            .ok_or(SklError::InvalidArguments("Could not resolve destination path".to_string()))?;
        let agents_dest = resolve_path(tool, &AssetType::Agent, false, None)
            .ok_or(SklError::InvalidArguments("Could not resolve destination path".to_string()))?;

        for skill in &skills_to_remove {
            let path = skills_dest.join(skill);
            if path.exists() {
                fs::remove_dir_all(&path)?;
                println!("🗑️  Removed skill: {}", skill);
            }
        }
        for agent in &agents_to_remove {
            let path = agents_dest.join(agent);
            if path.exists() {
                fs::remove_file(&path)?;
                println!("🗑️  Removed agent: {}", agent);
            }
        }
    }

    // Update lockfile
    let locked_mut = lockfile.repos.iter_mut().find(|r| r.name == repo_id).unwrap();
    locked_mut.profiles.retain(|p| p.name != profile_name);
    locked_mut.skills.retain(|s| !skills_to_remove.contains(s));
    locked_mut.agents.retain(|a| !agents_to_remove.contains(a));

    println!("✅ Removed profile '{}' from {}", profile_name, repo_id);
    Ok(())
}
