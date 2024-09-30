use regex::Regex;
use std::error::Error;
use std::process::{Command, Stdio};

const FFMPEG_EXECUTABLE: &str = r"C:\Program Files\ShotCut\ffmpeg.exe"; 

pub fn extract_silence_starts(input_file: &str) -> Result<Vec<f64>, Box<dyn Error>> {
    let pattern = Regex::new(r"silence_start:\s*(\d+\.?\d*)")?;
    run_silence_detect(input_file, "-60dB", "0.1", pattern)
}

pub fn extract_loud_starts(input_file: &str) -> Result<Vec<f64>, Box<dyn Error>> {
    let pattern = Regex::new(r"silence_end:\s*(\d+\.?\d*)")?;
    run_silence_detect(input_file, "-30dB", "0.5", pattern)
}

pub fn extract_duration(input_file: &str) -> Result<f64, Box<dyn Error>> {
    let output = Command::new(FFMPEG_EXECUTABLE)
        .arg("-i")
        .arg(input_file)
        .arg("-f")
        .arg("null")
        .arg("-")
        .stderr(Stdio::piped())
        .output()?;

    parse_duration(&String::from_utf8_lossy(&output.stderr))
}

fn run_silence_detect(
    input_file: &str,
    noise: &str,
    duration: &str,
    pattern: Regex,
) -> Result<Vec<f64>, Box<dyn Error>> {
    let output = Command::new(FFMPEG_EXECUTABLE)
        .arg("-i")
        .arg(input_file)
        .arg("-af")
        .arg(&format!("silencedetect=noise={}:d={}", noise, duration))
        .arg("-f")
        .arg("null")
        .arg("-")
        .stderr(Stdio::piped())
        .output()?;

    Ok(parse_output(&String::from_utf8_lossy(&output.stderr), pattern))
}

fn parse_output(output: &str, pattern: Regex) -> Vec<f64> {
    output
        .lines()
        .filter_map(|line| pattern.captures(line))
        .filter_map(|cap| cap[1].parse::<f64>().ok())
        .collect()
}

fn parse_duration(output: &str) -> Result<f64, Box<dyn Error>> {
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
