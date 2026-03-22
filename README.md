# skl

**A package manager for AI skills and agents.**

Install and share skills and agents across your team — regardless of the AI tool you use.

```sh
skl install https://github.com/my-org/skills
skl list
skl update
skl uninstall my-org/skills
```

---

## What is skl?

AI tools like Claude Code, Cursor, and VS Code Copilot support custom instructions called **skills** (slash commands) and **agents** (sub-agents). Teams often want to share these across projects and teammates, but there's no standard way to do it.

`skl` solves this by treating skill repositories like packages — clone, install, update, and uninstall them just like you would npm packages.

---

## Installation

```sh
cargo install skl
```

Or build from source:

```sh
git clone https://github.com/your-org/skl
cd skl
cargo build --release
```

---

## Quick Start

Install skills from a GitHub repository:

```sh
skl install https://github.com/my-org/skills
```

On first run, `skl` will detect which AI tools are installed on your machine and ask which ones to install for.

List what's installed:

```sh
skl list
```

Update all repositories:

```sh
skl update
```

Uninstall a repository:

```sh
skl uninstall my-org/skills
```

---

## Commands

### `skl install <source>`

Install skills and agents from a GitHub repository.

```sh
skl install https://github.com/my-org/skills
```

**Options:**

| Flag | Description |
|------|-------------|
| `--tool <tool>` | Install only for a specific tool (e.g. `claude`) |
| `--local` | Install in the current directory instead of globally |
| `--dest <path>` | Install to a custom destination folder |
| `--skill <name>...` | Install only specific skills by name |
| `--agent <name>...` | Install only specific agents by name |
| `--only skills\|agents` | Install only skills or only agents |

**Examples:**

```sh
# Install everything globally
skl install https://github.com/my-org/skills

# Install locally (into .claude/ or ./skills/ in the current directory)
skl install https://github.com/my-org/skills --local

# Install only specific skills
skl install https://github.com/my-org/skills --skill review-pr commit

# Install to a custom path
skl install https://github.com/my-org/skills --dest ./my-project/.claude

# Install only for Claude Code
skl install https://github.com/my-org/skills --tool claude
```

### `skl list`

List all installed skills and agents, grouped by repository.

```sh
skl list
skl list --only skills
skl list --only agents
```

### `skl update [repo]`

Update installed repositories by fetching the latest changes.

```sh
# Update all repositories
skl update

# Update a specific repository
skl update my-org/skills

# Update only for a specific tool
skl update --tool claude
```

### `skl uninstall <repo>`

Uninstall a repository and remove all its skills and agents.

```sh
skl uninstall my-org/skills
skl uninstall https://github.com/my-org/skills
```

### `skl init`

Run the setup wizard to configure which AI tools to install for.

```sh
skl init
```

### `skl config`

Manage skl configuration.

```sh
skl config list              # Show current configuration
skl config get tools         # Get a specific key
skl config set tools claude  # Set a value
skl config locate            # Show the config file path
```

---

## Skill Repository Format

Any Git repository works. `skl` scans recursively for:

- **Skills** — any folder containing a `SKILL.md` file (the folder name becomes the skill name)
- **Agents** — any `.md` file inside an `agents/` directory

---

## Supported Tools

| Tool | Skills path | Agents path |
|------|-------------|-------------|
| Claude Code | `~/.claude/skills/` | `~/.claude/agents/` |

More tools coming soon (Cursor, VS Code Copilot, Windsurf).

---

## How It Works

- **Global install** — skills go into `~/.claude/skills/`, agents into `~/.claude/agents/`
- **Local install** (`--local`) — installs into `.claude/skills/` or `./skills/` relative to the current directory
- **Custom dest** (`--dest`) — installs into `<dest>/skills/` and `<dest>/agents/`
- A `skl.lock` file at `~/.config/skl/skl.lock` tracks what's installed and from which repositories
- `skl update` uses shallow clones (`--depth=1`) for fast, bandwidth-efficient updates

---

## Configuration

The config file lives at `~/.config/skl/config.toml`:

```toml
tools = ["claude"]
```

Use `skl config locate` to find it on your system.

---

## License

MIT
