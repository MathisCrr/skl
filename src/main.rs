use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;

mod commands;
mod config;
mod lock;
mod profile;
mod types;
mod ui;

#[derive(Parser)]
#[command(
    version,
    about,
    long_about = "skl is a package manager for AI skills.

Install and share skills and agents across your team —
regardless of the AI tool you use.

    skl install https://github.com/my-org/skills
    skl list
    skl update
    skl uninstall my-org/skills

Supports Claude Code, Cursor, VS Code Copilot and more.
"
)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Setup skl and select your tools
    Init,

    /// Install a skill repository
    Install {
        /// GitHub repository URL (e.g. https://github.com/my-org/skills)
        source: String,
        /// Only install for a specific tool
        #[arg(long)]
        tool: Option<types::Tool>,
        /// Install in the current directory instead of globally
        #[arg(long)]
        local: bool,
        /// Custom destination folder
        #[arg(long)]
        dest: Option<PathBuf>,
        /// Install only specific skills by name
        #[arg(long, num_args = 1..)]
        skill: Option<Vec<String>>,
        /// Install only specific agents by name
        #[arg(long, num_args = 1..)]
        agent: Option<Vec<String>>,
        /// Install only skills or only agents
        #[arg(long)]
        only: Option<types::Only>,
        /// Install only skills and agents from a specific profile defined in skl.toml
        #[arg(long)]
        profile: Option<String>,
    },

    /// List installed skills and agents
    List {
        /// List only skills or only agents
        #[arg(long)]
        only: Option<types::Only>,
    },

    /// Update installed repositories
    Update {
        /// Specific repository to update (updates all if omitted)
        repo: Option<String>,
        /// Only update for a specific tool
        #[arg(long)]
        tool: Option<types::Tool>,
    },

    /// Uninstall a repository and all its deployed skills and agents
    Uninstall {
        /// Repository to uninstall (e.g. my-org/skills or https://github.com/my-org/skills)
        repo: String,
        /// Uninstall only a specific skill
        #[arg(long)]
        skill: Option<String>,
        /// Uninstall only a specific profile
        #[arg(long)]
        profile: Option<String>,
    },

    /// Show the content of a skill or agent
    Show {
        /// Name of the skill or agent to display
        name: String,
    },

    /// Create a new skill repository
    New {
        /// Name of the repository to create
        name: String,
    },

    /// Manage skl configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Set a configuration value
    Set { key: String, value: String },
    /// Get a configuration value
    Get { key: String },
    /// List all configuration values
    List,
    /// Show the config file location
    Locate,
}

fn main() {
    let args = Args::parse();

    let result = match args.command {
        Commands::Init => commands::init::init().map(|_| ()),
        Commands::Install {
            source,
            tool,
            local,
            dest,
            skill,
            agent,
            only,
            profile,
        } => commands::install::install(source, tool, local, dest, skill, agent, only, profile),
        Commands::List { only } => commands::list::list(only),
        Commands::Show { name } => commands::show::show(name),
        Commands::Update { repo, tool } => commands::update::update(repo, tool),
        Commands::New { name } => commands::new::new(name),
        Commands::Uninstall {
            repo,
            skill,
            profile,
        } => commands::uninstall::uninstall(repo, skill, profile),
        Commands::Config { action } => match action {
            ConfigAction::Get { key } => commands::config::get(&key),
            ConfigAction::Set { key, value } => commands::config::set(&key, &value),
            ConfigAction::List => commands::config::list(),
            ConfigAction::Locate => commands::config::locate(),
        },
    };

    if let Err(err) = result {
        eprintln!("{} {}", "error:".red().bold(), err);
        std::process::exit(1);
    }
}
