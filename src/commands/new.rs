use crate::types::SklError;
use std::fs;
use std::path::Path;

pub fn new(name: String) -> Result<(), SklError> {
    let root = Path::new(&name);
    let in_place = name == ".";

    if !in_place && root.exists() {
        return Err(SklError::InvalidArguments(format!(
            "Directory '{}' already exists",
            name
        )));
    }

    if in_place && root.read_dir().map_or(false, |mut d| d.next().is_some()) {
        return Err(SklError::InvalidArguments(
            "Current directory is not empty".to_string(),
        ));
    }

    // Create directory structure
    fs::create_dir_all(root.join("example-skill"))?;
    fs::create_dir_all(root.join("agents"))?;

    // skl.toml
    fs::write(
        root.join("skl.toml"),
        r#"[profiles.default]
skills = ["example-skill"]
agents = ["example-agent.md"]
"#,
    )?;

    // Example skill — format reference: https://code.claude.com/docs/en/slash-commands
    fs::write(
        root.join("example-skill").join("SKILL.md"),
        r#"---
description: Describe what this skill does and when to use it.
argument-hint: [your-argument]
---

> Format reference: https://code.claude.com/docs/en/slash-commands

Add your skill instructions here. Use $ARGUMENTS to refer to the arguments passed to the skill.
"#,
    )?;

    // Example agent — format reference: https://code.claude.com/docs/en/sub-agents
    fs::write(
        root.join("agents").join("example-agent.md"),
        r#"---
name: example-agent
description: Describe what this agent does and when Claude should delegate to it.
---

> Format reference: https://code.claude.com/docs/en/sub-agents

You are an expert in... Describe the agent's role and instructions here.
"#,
    )?;

    if in_place {
        println!("✅ Scaffolded in current directory");
    } else {
        println!("✅ Created '{}'", name);
    }
    println!("   {}/skl.toml", name);
    println!("   {}/example-skill/SKILL.md", name);
    println!("   {}/agents/example-agent.md", name);

    Ok(())
}
