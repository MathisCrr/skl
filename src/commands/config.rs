use crate::config::{Config, config_path};
use crate::types::SklError;

pub fn get(key: &str) -> Result<(), SklError> {
    let config = Config::load()?;

    match key {
        "tools" => println!("{:?}", config.tools),
        _ => println!("Unknown key: '{}'. Valid keys: tools", key),
    }

    Ok(())
}

pub fn set(key: &str, value: &str) -> Result<(), SklError> {
    let mut config = Config::load()?;

    match key {
        "tools" => config.tools = value.split(',').map(|t| t.trim().parse().unwrap()).collect(),
        _ => {
            println!("Unknown key: '{}'. Valid keys: tools", key);
            return Ok(());
        }
    }

    config.save()?;
    println!("✅ {} = {}", key, value);

    Ok(())
}

pub fn list() -> Result<(), SklError> {
    let config = Config::load()?;
    println!("tools = {:?}", config.tools);

    Ok(())
}

pub fn locate() -> Result<(), SklError> {
    let path = config_path()?;
    println!("{}", path.display());

    Ok(())
}
