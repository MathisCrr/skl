use crate::config::Config;
use crate::types::{SklError, Tool};
use dialoguer::MultiSelect;

pub fn init() -> Result<Config, SklError> {
    println!("Welcome to skl! Let's set up your tools.");

    let available_tools = vec!["claude"];
    let detected: Vec<bool> = vec![
        dirs::home_dir().map_or(false, |h| h.join(".claude").exists()),
    ];

    let selected = MultiSelect::new()
        .with_prompt("Select tools to install skills for (space to select, enter to confirm)")
        .items(&available_tools)
        .defaults(&detected)
        .interact()
        .unwrap();

    if selected.is_empty() {
        println!("No tools selected, aborting.");
        return Err(SklError::InvalidArguments("No tools selected".to_string()));
    }

    let tools: Vec<Tool> = selected
        .iter()
        .map(|&i| available_tools[i].parse::<Tool>().unwrap())
        .collect();

    let config = Config { tools };
    config.save()?;
    println!("✅ Config saved!");

    Ok(config)
}
