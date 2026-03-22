use crate::{
    commands::repo::normalize_repo_id,
    config::{Config, source_path},
    lock::Lockfile,
    types::{AssetType, SklError, resolve_path},
};
use std::fs;

pub fn uninstall(repo: String) -> Result<(), SklError> {
    let config = Config::load()?;
    let repo_id = normalize_repo_id(&repo);

    let mut lockfile = Lockfile::load()?;

    let locked = lockfile
        .remove_repo(&repo_id)
        .ok_or_else(|| SklError::RepoNotFound(repo_id.clone()))?;

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

    lockfile.save()?;
    println!("✅ Uninstalled repo: {}", repo_id);

    Ok(())
}
