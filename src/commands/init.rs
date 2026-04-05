use crate::{config::Config, types::{SklError, Tool}, ui};
use colored::Colorize;
use dialoguer::MultiSelect;

pub fn init() -> Result<Config, SklError> {
    println!("Welcome to {}! Let's set up your tools.", "skl".cyan().bold());

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
        ui::warning("no tools selected, aborting");
        return Err(SklError::InvalidArguments("No tools selected".to_string()));
    }

    let tools: Vec<Tool> = selected
        .iter()
        .map(|&i| available_tools[i].parse::<Tool>().unwrap())
        .collect();

    let config = Config { tools };
    config.save()?;
    ui::success("config saved");

    Ok(config)
}
