use std::fs;
use std::process::Command;
use std::time::SystemTime;

const MELT_EXECUTABLE: &str = r"C:\Program Files\ShotCut\melt.exe";

fn concat_mkv(input_folder: &str, output_file: &str) {
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
    for file in &sorted_files {
        melt_command.arg(file);
    }

    melt_command
        .arg("-consumer")
        .arg(format!("avformat:{}\\{}", input_folder, output_file))
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
            println!("Successfully merged files into {}", output_file)
        }
        Ok(status) => eprintln!("melt.exe failed with exit code: {}", status),
        Err(err) => eprintln!("Failed to execute melt.exe: {}", err),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 3 && args[1] == "concat" {
        let input_folder = &args[2];
        concat_mkv(input_folder, "output.mkv");
    } else {
        eprintln!("Usage: cut-bot concat <input_folder>");
    }
}
