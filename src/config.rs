use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

const CONFIG_FILE_NAME: &str = "cut-bot.conf";

pub fn get_value(key: &str) -> Result<String, Box<dyn Error>> {
    let path = config_path()?;
    let values = load_values()?;

    values
        .get(key)
        .cloned()
        .ok_or_else(|| format!("Missing '{}' in {}", key, path.display()).into())
}

pub fn get_optional_value(key: &str) -> Result<Option<String>, Box<dyn Error>> {
    let values = load_values()?;

    Ok(values.get(key).cloned())
}

fn load_values() -> Result<HashMap<String, String>, Box<dyn Error>> {
    let path = config_path()?;
    let content = fs::read_to_string(&path)?;

    Ok(parse_config(&content))
}

pub fn config_path() -> Result<PathBuf, Box<dyn Error>> {
    let executable = std::env::current_exe()?;
    let executable_dir = executable
        .parent()
        .ok_or("Failed to determine executable directory")?;
    Ok(executable_dir.join(CONFIG_FILE_NAME))
}

fn parse_config(content: &str) -> HashMap<String, String> {
    let mut values = HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            values.insert(key.trim().to_string(), value.trim().to_string());
        }
    }

    values
}