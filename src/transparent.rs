use crate::image_command;
use crate::tool;

pub fn make_transparent(pattern: &str) {
    if let Err(error) = run_make_transparent(pattern) {
        eprintln!("{}", error);
        std::process::exit(1);
    }
}

fn run_make_transparent(pattern: &str) -> Result<(), Box<dyn std::error::Error>> {
    let magick_executable = tool::optional_executable("magick_executable", "magick")?;
    tool::validate_executable(&magick_executable, "magick")?;

    image_command::run_glob_transform(
        &magick_executable,
        pattern,
        |path| image_command::output_with_suffix(path, "transparent.png"),
        |path, output_file| {
            vec![
                path.display().to_string(),
                "-transparent".to_string(),
                "#1e1e1e".to_string(),
                output_file.display().to_string(),
            ]
        },
    )
}