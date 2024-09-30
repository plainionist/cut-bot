use regex::Regex;
use std::process::{Command, Stdio};

const FFMPEG_EXECUTABLE: &str = r"C:\Program Files\ShotCut\ffmpeg.exe";

pub fn extract_silence_starts(input_file: &str) -> Vec<f64> {
    let output = Command::new(FFMPEG_EXECUTABLE)
        .arg("-i")
        .arg(input_file)
        .arg("-af")
        .arg("silencedetect=noise=-60dB:d=0.1")
        .arg("-f")
        .arg("null")
        .arg("-")
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to execute FFmpeg for silence detection");

    parse_output(
        &String::from_utf8_lossy(&output.stderr),
        Regex::new(r"silence_start:\s*(\d+\.?\d*)").unwrap(),
    )
}

pub fn extract_loud_starts(input_file: &str) -> Vec<f64> {
    let output = Command::new(FFMPEG_EXECUTABLE)
        .arg("-i")
        .arg(input_file)
        .arg("-af")
        .arg("silencedetect=noise=-30dB:d=0.5")
        .arg("-f")
        .arg("null")
        .arg("-")
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to execute FFmpeg for loud detection");
    
    parse_output(
        &String::from_utf8_lossy(&output.stderr),
        Regex::new(r"silence_end:\s*(\d+\.?\d*)").unwrap(),
    )
}

pub fn extract_duration(input_file: &str) -> f64 {
    let output = Command::new(FFMPEG_EXECUTABLE)
        .arg("-i")
        .arg(input_file)
        .arg("-f")
        .arg("null")
        .arg("-")
        .stderr(Stdio::piped()) // FFmpeg writes to stderr, so we capture that
        .output()
        .expect("Failed to execute FFmpeg for duration");

    match parse_duration(&String::from_utf8_lossy(&output.stderr)) {
      Ok(duration) => duration,
      Err(_) => 0.0
    }
}

fn parse_output(output: &str, pattern: Regex) -> Vec<f64> {
    output
        .lines()
        .filter_map(|line| pattern.captures(line))
        .filter_map(|cap| cap[1].parse::<f64>().ok())
        .collect()
}

fn parse_duration(output: &str) -> Result<f64, Box<dyn std::error::Error>> {
    let duration_re = Regex::new(r"Duration: (\d+):(\d+):(\d+)\.(\d+)")?;

    if let Some(cap) = duration_re.captures(output) {
        let hours: f64 = cap[1].parse()?;
        let minutes: f64 = cap[2].parse()?;
        let seconds: f64 = cap[3].parse()?;
        let millis: f64 = cap[4].parse()?;
        Ok(hours * 3600.0 + minutes * 60.0 + seconds + (millis / 100.0))
    } else {
        Err("Failed to capture duration".into())
    }
}
