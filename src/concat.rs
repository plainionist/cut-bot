use std::fs;
use std::process::Command;
use std::time::SystemTime;

const MELT_EXECUTABLE: &str = r"C:\Program Files\ShotCut\melt.exe";

pub fn concat_mkv(input_folder: &str) {
    let mut files: Vec<_> = fs::read_dir(input_folder)
        .expect("Could not read directory")
        .map(|entry| entry.expect("Could not read directory entry"))
        .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("mkv"))
        .map(|entry| {
            let modified_time = entry
                .metadata()
                .unwrap()
                .modified()
                .unwrap_or(SystemTime::now());
            (entry.path(), modified_time)
        })
        .collect();

    files.sort_by_key(|&(_, modified_time)| modified_time);

    let sorted_files: Vec<_> = files
        .iter()
        .map(|(path, _)| path.to_str().unwrap().to_string())
        .collect();

    let mut melt_command = Command::new(MELT_EXECUTABLE);
    melt_command.current_dir(input_folder);

    for file in &sorted_files {
        melt_command.arg(file);
    }

    melt_command
        .arg("-verbose")
        .arg("-progress 2")
        .arg("-consumer")
        .arg(format!("xml:{}\\pass1.mlt", input_folder))
        .arg("acodec=aac")
        .arg("vcodec=libx264");

    let command_str = format!(
        "{:?} {}",
        melt_command.get_program(),
        melt_command
            .get_args()
            .map(|arg| arg.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ")
    );
    println!("Executing: {}", command_str);

    match melt_command.status() {
        Ok(status) if status.success() => {
            println!("Successfully merged!")
        }
        Ok(status) => eprintln!("melt.exe failed with exit code: {}", status),
        Err(err) => eprintln!("Failed to execute melt.exe: {}", err),
    }

    // TODO: we need to add "<playlist/>"
}

