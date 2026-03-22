use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::{fmt, io, path::PathBuf, str::FromStr};

#[derive(Debug, Clone, Serialize, Deserialize, ValueEnum)]
pub enum Tool {
    #[serde(rename = "claude")]
    Claude,
}

impl FromStr for Tool {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "claude" => Ok(Tool::Claude),
            _ => Err(format!("Unknown tool: '{}'. Valid tools: claude", s)),
        }
    }
}

pub enum AssetType {
    Skill,
    Agent,
}

/// Resolve the destination path for an asset.
/// If `local` is true, checks for a tool folder in the current directory first,
/// then falls back to ./skills/ or ./agents/.
/// If `dest` is provided, uses that as the base path.
pub fn resolve_path(tool: &Tool, asset: &AssetType, local: bool, dest: Option<&PathBuf>) -> Option<PathBuf> {
    if let Some(base) = dest {
        return Some(match asset {
            AssetType::Skill => base.join("skills"),
            AssetType::Agent => base.join("agents"),
        });
    }

    if local {
        let tool_dir = match tool {
            Tool::Claude => PathBuf::from(".claude"),
        };
        let base = if tool_dir.exists() { tool_dir } else { PathBuf::from(".") };
        return Some(match asset {
            AssetType::Skill => base.join("skills"),
            AssetType::Agent => base.join("agents"),
        });
    }

    match (tool, asset) {
        (Tool::Claude, AssetType::Skill) => {
            Some(dirs::home_dir()?.join(".claude").join("skills"))
        }
        (Tool::Claude, AssetType::Agent) => {
            Some(dirs::home_dir()?.join(".claude").join("agents"))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ValueEnum)]
pub enum Only {
    #[serde(rename = "skill")]
    Skill,
    #[serde(rename = "agent")]
    Agent,
}

impl FromStr for Only {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "skills" => Ok(Only::Skill),
            "agents" => Ok(Only::Agent),
            _ => Err(format!(
                "Unknown value '{}'. Valid values: skills, agents",
                s
            )),
        }
    }
}

#[derive(Debug)]
pub enum SklError {
    ConfigDirectoryNotFound,
    ConfigNotFound,
    ConfigParseError(String),
    GitCloneFailed,
    IoError(io::Error),
    InvalidArguments(String),
    RepoNotFound(String),
}

impl fmt::Display for SklError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SklError::ConfigDirectoryNotFound => write!(f, "Could not find config directory"),
            SklError::ConfigNotFound => write!(f, "Config not found. Run `skl install` to get started"),
            SklError::ConfigParseError(msg) => write!(f, "Invalid config: {}", msg),
            SklError::GitCloneFailed => write!(f, "Failed to clone repository"),
            SklError::IoError(err) => write!(f, "IO error: {}", err),
            SklError::InvalidArguments(msg) => write!(f, "Invalid arguments: {}", msg),
            SklError::RepoNotFound(name) => write!(f, "Repo '{}' not found in lockfile", name),
        }
    }
}

impl From<io::Error> for SklError {
    fn from(err: io::Error) -> Self {
        SklError::IoError(err)
    }
}
