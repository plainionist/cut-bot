mod concat;
mod silence;

use concat::concat_mkv;
use silence::silence;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 3 && args[1] == "concat" {
        let input_folder = &args[2];
        concat_mkv(input_folder);
    } else if args.len() == 3 && args[1] == "silence" {
        let input_file = &args[2];
        silence(input_file);
    } else {
        eprintln!("Usage: cut-bot concat <input_folder>");
    }
}
