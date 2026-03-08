use crate::tool;
use glob::glob;
use std::error::Error;
use std::path::{Path, PathBuf};

pub fn run_glob_transform<F, G>(
    executable: &str,
    pattern: &str,
    output_path: F,
    arguments: G,
) -> Result<(), Box<dyn Error>>
where
    F: Fn(&Path) -> PathBuf,
    G: Fn(&Path, &Path) -> Vec<String>,
{
    let entries = glob(pattern)?;
    let mut matched_files = 0;

    for entry in entries {
        let path = entry?;
        matched_files += 1;

        let output_file = output_path(&path);
        let args = arguments(&path, &output_file);
        tool::run_command(executable, &args, &format!("'{}'", path.display()))?;
        println!("Created {}", output_file.display());
    }

    if matched_files == 0 {
        return Err(format!("No files matched pattern '{}'", pattern).into());
    }

    Ok(())
}

pub fn output_with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let base_name = path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("output");

    parent.join(format!("{}.{}", base_name, suffix))
}