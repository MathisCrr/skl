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
├── profile.rs       ← read skl.toml from a skill repository (SklToml)
├── types.rs         ← Tool, AssetType, Only, SklError, resolve_path
├── ui.rs            ← shared UI helpers: spinner, success, removed, warning, action
└── commands/
    ├── repo.rs      ← shared: normalize_repo_id, find_files, copy_dir
    ├── init.rs      ← wizard to detect/select tools (dialoguer MultiSelect)
    ├── install.rs   ← clone + deploy skills/agents, profile support
    ├── list.rs      ← list installed skills/agents grouped by repo, with descriptions
    ├── show.rs      ← display full content of a skill or agent by name
    ├── update.rs    ← fetch + reset --hard, re-apply profiles
    ├── uninstall.rs ← remove skills/agents + source dir, profile/skill granularity
    ├── new.rs       ← scaffold a new skill repository
    └── config.rs    ← get/set/list/locate config values
```

## Dependencies
```toml
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
toml = "0.8"
dirs = "5"
dialoguer = "0.11"
colored = "2"
indicatif = "0.17"
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
    IoError(io::Error),
    RepoNotFound(String),
    InvalidArguments(String),
    ProfileNotFound(String, Vec<String>),  // (requested, available)
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
    pub name: String,              // "author/repo"
    pub url: Option<String>,       // original git URL
    pub profiles: Vec<LockedProfile>, // profiles used during install
    pub skills: Vec<String>,       // skill names installed from this repo
    pub agents: Vec<String>,       // agent filenames installed from this repo
}
pub struct LockedProfile {
    pub name: String,
    pub skills: Vec<String>,
    pub agents: Vec<String>,
}
// stored at: ~/.config/skl/skl.lock
// used by: uninstall (knows what to remove), update (knows what to reinstall)
// list uses filesystem as source of truth, lockfile for grouping by repo
```

## Profile (src/profile.rs)
Reads `skl.toml` from a skill repository (maintainer-side).
```rust
pub struct SklToml {
    pub profiles: HashMap<String, Profile>,
}
pub struct Profile {
    pub skills: Vec<String>,
    pub agents: Vec<String>,
}
// key methods:
// load(repo_dir) → reads skl.toml, returns None if absent
// get_profile(name) → returns &Profile or ProfileNotFound error
// resolve_profiles(names) → union of skills/agents across multiple profiles
```

Example `skl.toml`:
```toml
[profiles.backend]
skills = ["sql", "pdf"]
agents = ["db-agent.md"]

[profiles.frontend]
skills = ["react", "css"]
```

## UI (src/ui.rs)
```rust
pub fn spinner(msg: &str) -> ProgressBar  // ◌ ○ ◎ ◉ ● ◉ ◎ ○ ◌ animation, cyan
pub fn success(msg: &str)   // ✓ green
pub fn removed(msg: &str)   // − red
pub fn warning(msg: &str)   // ! yellow
pub fn action(msg: &str)    // → cyan  (reserved for future use)
```

## CLI commands
```bash
skl init                                              # wizard: detect + select tools
skl install <url> [--tool <t>] [--local] [--dest <p>] [--skill <n>...] [--agent <n>...] [--only skills|agents] [--profile <name>]
skl list [--only skills|agents]                       # shows description + profiles per item
skl show <name>                                       # display full SKILL.md or agent .md
skl update [repo] [--tool <t>]
skl uninstall <repo> [--skill <name>] [--profile <name>]
skl new <name>                                        # scaffold a new skill repository
skl config <get|set|list|locate> [key] [value]
```

## install.rs logic
1. Load config; if empty → run init wizard
2. Normalize source URL → `author/repo` id
3. Spinner + `git clone --depth=1 <url> ~/.config/skl/sources/author/repo`
4. If `--profile`: load `skl.toml`, resolve profile → filter skills/agents
5. Filter skills/agents by `--skill`, `--agent`, `--only` flags
6. For each tool in config (or `--tool` override):
   - `resolve_path(tool, Skill, local, dest)` → skills_dest
   - `resolve_path(tool, Agent, local, dest)` → agents_dest
   - Copy skill dirs to skills_dest/
   - Copy agent .md files to agents_dest/
7. Save lockfile with installed names + LockedProfile if profile was used

## update.rs logic
1. For each repo in lockfile (or specific repo):
   - Spinner + `git fetch --depth=1` + `git reset --hard origin/HEAD`
   - If repo has locked profiles: re-resolve from updated `skl.toml`
   - Scan current skills/agents in source (effective list after profile filter)
   - Remove skills/agents no longer in effective list
   - Re-copy all skills/agents in effective list
   - Update LockedProfile entries from refreshed skl.toml
2. Save updated lockfile

## uninstall.rs logic
- No flag: remove everything (source dir + deployed assets + lockfile entry)
- `--skill <name>`: remove single skill from filesystem + lockfile, keep source
- `--profile <name>`: remove exclusive skills/agents (not shared with other profiles)
  - uses `locked.exclusive_skills/agents(profile_names)` on LockedRepo

## new.rs logic
`skl new <name>` or `skl new .` (current directory, must be empty):
Creates:
```
<name>/
├── skl.toml                    (profiles.default with example entries)
├── example-skill/
│   └── SKILL.md
└── agents/
    └── example-agent.md
```

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
- `skl.lock` hybrid approach: lockfile tracks remote installs, filesystem is truth for `list`
- `list` reads `description:` from SKILL.md / agent frontmatter at display time (no cache); truncated to 50 chars inline; profiles shown in `bright_yellow`
- `show` searches skills then agents across all configured tools; accepts name with or without `.md` for agents; errors with `RepoNotFound` if not found
- Local skills (not in lockfile) cannot be uninstalled via `skl uninstall` — they are managed manually
- Shallow clones (`--depth=1`) for bandwidth efficiency; update uses `fetch + reset --hard` (not `git pull`)
- git clone via `std::process::Command` (no git2 crate dependency)
- git stdout/stderr suppressed during spinners via `Stdio::null()`
- First run: `install` triggers `init` wizard automatically if config is empty
- Multi-tool: single install deploys to all configured tools simultaneously
- Profiles are maintainer-defined in `skl.toml`; users reference them with `--profile`
