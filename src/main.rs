mod silence;
mod border;
mod config;
mod ffmpeg;
mod mlt_builder;

use border::add_borders;
use silence::silence;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 4 && args[1] == "silence" {
        let input = &args[2];
        let output = &args[3];
        silence(input, output);
    } else if args.len() == 3 && args[1] == "border" {
        let pattern = &args[2];
        add_borders(pattern);
    } else {
        eprintln!("Usage: cut-bot <command> [args]");
        eprintln!("");
        eprintln!("Commands: ");
        eprintln!("    silence <input> <output.mlt> - creates ShotCut project with silent parts marked");
        eprintln!("                                   input: folder with .mkv files or single file");
        eprintln!("    border <pattern>             - adds an 8px black border using ImageMagick");
        eprintln!("");
    }
}
