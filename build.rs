use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("Missing CARGO_MANIFEST_DIR"));
    let source_config = manifest_dir.join("cut-bot.conf");

    println!("cargo:rerun-if-changed={}", source_config.display());

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("Missing OUT_DIR"));
    let target_dir = out_dir
        .ancestors()
        .nth(3)
        .expect("Failed to determine target directory from OUT_DIR");
    let target_config = target_dir.join("cut-bot.conf");

    fs::copy(&source_config, &target_config).expect("Failed to copy cut-bot.conf next to the executable");
}