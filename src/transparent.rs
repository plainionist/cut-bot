use crate::config;
use glob::glob;
use std::path::Path;
use std::process::Command;

pub fn make_transparent(pattern: &str) {
    let magick_executable = match config::get_optional_value("magick_executable") {
        Ok(Some(value)) => value,
        Ok(None) => "magick".to_string(),
        Err(error) => {
            eprintln!("Error reading magick config: {}", error);
            std::process::exit(1);
        }
    };

    let entries = match glob(pattern) {
        Ok(entries) => entries,
        Err(error) => {
            eprintln!("Invalid file pattern '{}': {}", pattern, error);
            std::process::exit(1);
        }
    };

    let mut matched_files = 0;

    for entry in entries {
        let path = match entry {
            Ok(path) => path,
            Err(error) => {
                eprintln!("Failed to read match for '{}': {}", pattern, error);
                std::process::exit(1);
            }
        };

        matched_files += 1;

        let output_file = transparent_output_path(&path);
        let status = Command::new(&magick_executable)
            .arg(&path)
            .arg("-transparent")
            .arg("#1e1e1e")
            .arg(&output_file)
            .status();

        match status {
            Ok(status) if status.success() => {
                println!("Created {}", output_file.display());
            }
            Ok(status) => {
                eprintln!(
                    "magick failed for '{}' with exit code {:?}",
                    path.display(),
                    status.code()
                );
                std::process::exit(1);
            }
            Err(error) => {
                eprintln!(
                    "Failed to execute '{}' for '{}': {}",
                    magick_executable,
                    path.display(),
                    error
                );
                std::process::exit(1);
            }
        }
    }

    if matched_files == 0 {
        eprintln!("No files matched pattern '{}'", pattern);
        std::process::exit(1);
    }
}

fn transparent_output_path(path: &Path) -> std::path::PathBuf {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let base_name = path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("output");

    parent.join(format!("{}.transparent.png", base_name))
}