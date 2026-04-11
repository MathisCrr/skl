# skl

**A package manager for AI skills and agents.**

Install and share skills and agents across your team — regardless of the AI tool you use.

---

## What is skl?

AI tools support custom instructions called **skills** and **agents**. Teams often want to share these across projects and teammates, but there's no standard way to do it.

`skl` solves this by treating skill repositories like packages — install, update, and uninstall them just like you would npm packages.

---

## Installation

```sh
cargo install skl-cli
```

Or with the install script (Linux and MacOs):

```sh
curl -fsSL https://raw.githubusercontent.com/MathisCrr/skl/main/install.sh | bash
```

---

## Quick start

```sh
# 1. Set up skl and select your AI tools
skl init

# 2. Install a skill repository
skl install https://github.com/my-org/skills

# 3. See what's installed
skl list

# 4. Update all repositories (or a specific one)
skl update
skl update my-org/skills

# 6. Uninstall a repository
skl uninstall my-org/skills
```

For all available options, use `skl --help` or `skl <command> --help`.

---

## Repository format & profiles

Any Git repository works. Use `skl new <name>` to scaffold a boilerplate. `skl` scans for:

- **Skills** — any folder containing a `SKILL.md` file (the folder name becomes the skill name)
- **Agents** — any `.md` file inside an `agents/` directory

Optionally, a `skl.toml` at the root defines **profiles** — named subsets of skills and agents:

```toml
[profile.backend]
skills = ["commit", "review-pr"]
agents = ["code-reviewer.md"]

[profile.frontend]
skills = ["commit", "css-helper"]
```

Profiles let you install only what's relevant to a context. A large shared repository can have dozens of skills — profiles let each role (backend, frontend, devops) install only what they need, without touching the rest.

```sh
# Install only the backend profile
skl install https://github.com/my-org/skills --profile backend

# Uninstall a profile
skl uninstall my-org/skills --profile backend
```

---

## Supported tools

| Tool | Skills path | Agents path |
|------|-------------|-------------|
| Claude Code | `~/.claude/skills/` | `~/.claude/agents/` |

More tools coming soon (Cursor, VS Code Copilot, Windsurf).

---

## How it works

- Skills are installed globally by default (`~/.claude/skills/`)
- `--local` installs into `.claude/` relative to the current directory
- `skl.lock` at `~/.config/skl/skl.lock` tracks installed repositories and profiles
- Updates use `git fetch` + `git reset --hard` for clean, reproducible installs

---

## License

MIT
