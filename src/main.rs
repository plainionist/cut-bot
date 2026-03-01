mod silence;
mod ffmpeg;
mod mlt_builder;

use silence::silence;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 3 && args[1] == "silence" {
        let input_file = &args[2];
        silence(input_file);
    } else {
        eprintln!("Usage: cut-bot <command> <input_folder>");
        eprintln!("");
        eprintln!("Commands: ");
        eprintln!("    silence   - creates ShotCut project with silent parts marked");
        eprintln!("");
    }
}
