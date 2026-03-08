use crate::image_command;
use crate::tool;

pub fn add_borders(pattern: &str) {
    if let Err(error) = run_add_borders(pattern) {
        eprintln!("{}", error);
        std::process::exit(1);
    }
}

fn run_add_borders(pattern: &str) -> Result<(), Box<dyn std::error::Error>> {
    let magick_executable = tool::optional_executable("magick_executable", "magick")?;
    tool::validate_executable(&magick_executable, "magick")?;

    image_command::run_glob_transform(
        &magick_executable,
        pattern,
        |path| image_command::output_with_suffix(path, "border.png"),
        |path, output_file| {
            vec![
                path.display().to_string(),
                "-bordercolor".to_string(),
                "black".to_string(),
                "-border".to_string(),
                "8".to_string(),
                output_file.display().to_string(),
            ]
        },
    )
}