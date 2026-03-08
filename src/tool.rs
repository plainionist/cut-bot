use crate::config;
use std::error::Error;
use std::path::Path;
use std::process::Command;

pub fn required_executable(config_key: &str) -> Result<String, Box<dyn Error>> {
    config::get_value(config_key)
}

pub fn optional_executable(config_key: &str, default: &str) -> Result<String, Box<dyn Error>> {
    Ok(config::get_optional_value(config_key)?.unwrap_or_else(|| default.to_string()))
}

pub fn validate_executable(executable: &str, tool_name: &str) -> Result<(), Box<dyn Error>> {
    let has_path_separator = executable.contains(std::path::MAIN_SEPARATOR) || executable.contains('/');

    if has_path_separator && !Path::new(executable).exists() {
        return Err(format!("{} not found at '{}'", tool_name, executable).into());
    }

    Ok(())
}

pub fn run_command(executable: &str, args: &[String], context: &str) -> Result<(), Box<dyn Error>> {
    let status = Command::new(executable).args(args).status()?;

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "{} failed for {} with exit code {:?}",
            executable,
            context,
            status.code()
        )
        .into())
    }
}