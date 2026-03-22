# skl — AI Skills Package Manager

## What is skl?
A Rust CLI tool to install, share and manage AI skills and agents across a team.
Like npm but for AI skills. No Node.js required — standalone binary.

## Core concept
A "skill" is any Markdown file that changes AI behavior:
- **Skills** → folders with `SKILL.md` (Claude Code slash command format)
- **Agents** → `.md` files in an `agents/` directory

## Project structure
```
src/
├── main.rs          ← CLI definition with clap (derive)
├── config.rs        ← read/write ~/.config/skl/config.toml
├── lock.rs          ← read/write ~/.config/skl/skl.lock
├── types.rs         ← Tool, AssetType, Only, SklError, resolve_path
└── commands/
    ├── repo.rs      ← shared: normalize_repo_id, find_files, copy_dir
    ├── init.rs      ← wizard to detect/select tools (dialoguer MultiSelect)
    ├── install.rs   ← clone + deploy skills/agents
    ├── list.rs      ← list installed skills/agents grouped by repo
    ├── update.rs    ← fetch + reset --hard for each locked repo
    ├── uninstall.rs ← remove skills/agents + source dir
    └── config.rs    ← get/set/list/locate config values
```

## Dependencies
```toml
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
toml = "0.8"
dirs = "5"
dialoguer = "0.11"
```

## Types (src/types.rs)
```rust
pub enum Tool {
    #[serde(rename = "claude")] Claude,
    // more tools planned: cursor, vscode-copilot, windsurf
}

pub enum AssetType { Skill, Agent }

pub enum Only { Skill, Agent }  // for --only flag

pub enum SklError {
    ConfigDirectoryNotFound,
    ConfigReadError(String),
    ConfigParseError(String),
    ConfigWriteError(String),
    LockReadError(String),
    LockParseError(String),
    LockWriteError(String),
    IoError(String),
    RepoNotFound(String),
    InvalidArguments(String),
}

pub fn resolve_path(tool: &Tool, asset: &AssetType, local: bool, dest: Option<&PathBuf>) -> Option<PathBuf>
// dest provided → dest/skills/ or dest/agents/
// local=true    → .claude/skills/ (or ./skills/ if .claude/ not found)
// global        → ~/.claude/skills/ or ~/.claude/agents/
```

## Config (src/config.rs)
```rust
pub struct Config {
    pub tools: Vec<Tool>,  // multi-tool support
}
// stored at: ~/.config/skl/config.toml (via dirs::config_dir())
// sources stored at: ~/.config/skl/sources/<author>/<repo>/
```

## Lockfile (src/lock.rs)
```rust
pub struct Lockfile {
    pub repos: Vec<LockedRepo>,
}
pub struct LockedRepo {
    pub name: String,        // "author/repo"
    pub url: Option<String>, // original git URL
    pub skills: Vec<String>, // skill names installed from this repo
    pub agents: Vec<String>, // agent filenames installed from this repo
}
// stored at: ~/.config/skl/skl.lock
// used by: uninstall (knows what to remove), update (knows what to reinstall)
// list uses filesystem as source of truth, lockfile for grouping by repo
```

## CLI commands
```bash
skl init                                              # wizard: detect + select tools
skl install <url> [--tool <t>] [--local] [--dest <p>] [--skill <n>...] [--agent <n>...] [--only skills|agents]
skl list [--only skills|agents]
skl update [repo] [--tool <t>]
skl uninstall <repo>
skl config <get|set|list|locate> [key] [value]
```

## install.rs logic
1. Load config; if empty → run init wizard
2. Normalize source URL → `author/repo` id
3. `git clone --depth=1 <url> ~/.config/skl/sources/author/repo`
4. Filter skills/agents by `--skill`, `--agent`, `--only` flags
5. For each tool in config (or `--tool` override):
   - `resolve_path(tool, Skill, local, dest)` → skills_dest
   - `resolve_path(tool, Agent, local, dest)` → agents_dest
   - Copy skill dirs to skills_dest/
   - Copy agent .md files to agents_dest/
6. Save lockfile with installed names

## update.rs logic
1. For each repo in lockfile (or specific repo):
   - `git -C <source_dir> fetch --depth=1`
   - `git -C <source_dir> reset --hard origin/HEAD`
   - Scan current skills/agents in source
   - Remove skills/agents no longer in source
   - Re-copy skills/agents still present
2. Save updated lockfile

## Source repo format
No specific structure required. skl searches recursively for:
- Any folder containing `SKILL.md` → treated as a skill (folder name = skill name)
- Any `agents/*.md` → treated as an agent (filename = agent name)

Example:
```
my-org/skills/
├── review-pr/SKILL.md
├── commit/SKILL.md
└── agents/
    ├── code-reviewer.md
    └── tester.md
```

## Key decisions
- No `Scope` enum — replaced by `--local` bool flag and `--dest` path
- No `add`/`remove` commands (v1 scope: repo-level install/uninstall only)
- No `new` command (AI generates skills natively)
- `skl.lock` hybrid approach: lockfile tracks remote installs, filesystem is truth for `list`
- Shallow clones (`--depth=1`) for bandwidth efficiency; update uses `fetch + reset --hard` (not `git pull`)
- git clone via `std::process::Command` (no git2 crate dependency)
- First run: `install` triggers `init` wizard automatically if config is empty
- Multi-tool: single install deploys to all configured tools simultaneously
