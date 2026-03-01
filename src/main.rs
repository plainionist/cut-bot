mod silence;
mod ffmpeg;
mod mlt_builder;

use silence::silence;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 4 && args[1] == "silence" {
        let input = &args[2];
        let output = &args[3];
        silence(input, output);
    } else {
        eprintln!("Usage: cut-bot <command> <input> <output>");
        eprintln!("");
        eprintln!("Commands: ");
        eprintln!("    silence <input> <output.mlt>  - creates ShotCut project with silent parts marked");
        eprintln!("                                   input: folder with .mkv files or single file");
        eprintln!("");
    }
}
