use regex::Regex;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const FFMPEG_EXECUTABLE: &str = r"C:\Program Files\ShotCut\ffmpeg.exe";

fn run_ffmpeg(input_file: &str) -> (Vec<f64>, Vec<f64>, f64) {
    let duration_output = Command::new(FFMPEG_EXECUTABLE)
        .arg("-i")
        .arg(input_file)
        .arg("-f")
        .arg("null")
        .arg("-")
        .stderr(Stdio::piped()) // FFmpeg writes to stderr, so we capture that
        .output()
        .expect("Failed to execute FFmpeg for duration");

    let duration = parse_duration(&String::from_utf8_lossy(&duration_output.stderr));

    let loud_ffmpeg_output = Command::new(FFMPEG_EXECUTABLE)
        .arg("-i")
        .arg(input_file)
        .arg("-af")
        .arg("silencedetect=noise=-30dB:d=1")
        .arg("-f")
        .arg("null")
        .arg("-")
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to execute FFmpeg for loud detection");

    let loud_periods =
        parse_ffmpeg_output_for_loud(&String::from_utf8_lossy(&loud_ffmpeg_output.stderr));

    let silence_ffmpeg_output = Command::new(FFMPEG_EXECUTABLE)
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

    let silent_periods =
        parse_ffmpeg_output_for_silence(&String::from_utf8_lossy(&silence_ffmpeg_output.stderr));

    (loud_periods, silent_periods, duration)
}

fn parse_ffmpeg_output_for_loud(output: &str) -> Vec<f64> {
    let mut loud_periods = Vec::new();
    let silence_end_re = Regex::new(r"silence_end:\s*(\d+\.?\d*)").unwrap();

    for line in output.lines() {
        if let Some(end_cap) = silence_end_re.captures(line) {
            let silence_end = end_cap[1].parse::<f64>().unwrap();
            loud_periods.push(silence_end); // Non-silent period starts after silence ends
        }
    }

    loud_periods
}

fn parse_ffmpeg_output_for_silence(output: &str) -> Vec<f64> {
    let mut silent_periods = Vec::new();
    let silence_start_re = Regex::new(r"silence_start:\s*(\d+\.?\d*)").unwrap();

    for line in output.lines() {
        if let Some(start_cap) = silence_start_re.captures(line) {
            let silence_start = start_cap[1].parse::<f64>().unwrap();
            silent_periods.push(silence_start); // Silence starts here
        }
    }

    silent_periods
}

fn find_audio_chunks(loud_periods: &[f64], silent_periods: &[f64], total_duration: f64) -> Vec<(f64, f64)> {
    let mut all_chunks = Vec::new();
    let mut current_time = 0.0;

    for &start in loud_periods {
        // Add silent period before the loud period if necessary
        if current_time < start {
            all_chunks.push((current_time, start)); // Silent chunk before loud period
        }

        // Find the next silent period after the loud start
        let end = silent_periods
            .iter()
            .find(|&&silence_start| silence_start > start)
            .cloned()
            .unwrap_or(total_duration); // If no silence is found, go until the end

        all_chunks.push((start, end)); // Loud chunk
        current_time = end; // Move to the end of this loud chunk
    }

    // Add any remaining silent chunk if necessary
    if current_time < total_duration {
        all_chunks.push((current_time, total_duration)); // Remaining silence
    }

    all_chunks
}

fn parse_duration(output: &str) -> f64 {
    let duration_re = Regex::new(r"Duration: (\d+):(\d+):(\d+)\.(\d+)").unwrap();
    if let Some(cap) = duration_re.captures(output) {
        let hours: f64 = cap[1].parse().unwrap();
        let minutes: f64 = cap[2].parse().unwrap();
        let seconds: f64 = cap[3].parse().unwrap();
        let millis: f64 = cap[4].parse().unwrap();
        return hours * 3600.0 + minutes * 60.0 + seconds + (millis / 100.0);
    }
    0.0
}

fn generate_mlt(timestamps: &Vec<(f64, f64)>, duration: f64, input_file: &str, output_file: &str) {
    let total_duration = format_time(duration);

    let mut mlt_content = String::from(format!(
        r#"<?xml version="1.0" standalone="no"?>
<mlt LC_NUMERIC="C" version="7.27.0" producer="main_bin">
<profile width="2560" height="1440" progressive="1" sample_aspect_num="1" sample_aspect_den="1" display_aspect_num="16" display_aspect_den="9" frame_rate_num="60000000" frame_rate_den="1000000" colorspace="709"/>
<playlist id="main_bin">
</playlist>
<producer id="black" in="00:00:00.000" out="{}">
</producer>
<playlist id="background">
  <entry producer="black" in="00:00:00.000" out="{}"/>
</playlist>
"#,
        total_duration, total_duration
    ));

    // Add chain elements and corresponding playlist entries
    for (i, _) in timestamps.iter().enumerate() {
        // Create a chain for each segment
        mlt_content.push_str(&format!(
            r#"  <chain id="chain{}" out="{}">
  <property name="resource">{}</property>
</chain>
"#,
            i, total_duration, input_file
        ));
    }

    // Start the playlist definition
    mlt_content.push_str(
        r#"  <playlist id="playlist0">
"#,
    );

    // Add entries to the playlist, linking each chain
    for (i, (start, end)) in timestamps.iter().enumerate() {
        let start_time = format_time(*start);
        let end_time = format_time(*end);
        mlt_content.push_str(&format!(
            r#"    <entry producer="chain{}" in="{}" out="{}"/>
"#,
            i, start_time, end_time
        ));
    }

    // Close the playlist
    mlt_content.push_str(
        r#"  </playlist>
"#,
    );

    // Add tractor element with background and playlist tracks
    mlt_content.push_str(&format!(
        r#"  <tractor id="tractor0" in="00:00:00.000" out="{}">
  <property name="shotcut">1</property>
  <property name="shotcut:projectAudioChannels">2</property>
  <property name="shotcut:projectFolder">0</property>
  <property name="shotcut:skipConvert">0</property>
  <track producer="background"/>
  <track producer="playlist0"/>
</tractor>
</mlt>
"#,
        total_duration
    ));

    let mut file = File::create(output_file).expect("Unable to create file");
    file.write_all(mlt_content.as_bytes())
        .expect("Unable to write data");
}

fn format_time(seconds: f64) -> String {
    let hours = (seconds / 3600.0).floor() as u32;
    let minutes = ((seconds % 3600.0) / 60.0).floor() as u32;
    let seconds = seconds % 60.0;
    format!("{:02}:{:02}:{:06.3}", hours, minutes, seconds)
}

fn generate_output_mlt_path(input_path: &Path) -> PathBuf {
    let parent_dir = input_path.parent().unwrap_or_else(|| Path::new("."));
    let output_mlt = parent_dir.join("output.mlt");
    output_mlt
}

pub fn silence(input_video: &str) {
    let input_path = Path::new(input_video);
    let output_mlt = generate_output_mlt_path(input_path);

    let (loud_periods, silent_periods, duration) = run_ffmpeg(input_video);

    println!(
        "Loud starts at: {}",
        loud_periods
            .iter()
            .map(|x| format!("{:.2}", x))
            .collect::<Vec<_>>()
            .join(", ")
    );

    println!(
        "Silent starts at: {}",
        silent_periods
            .iter()
            .map(|x| format!("{:.2}", x))
            .collect::<Vec<_>>()
            .join(", ")
    );

    let audio_chunks = find_audio_chunks(&loud_periods, &silent_periods, duration);

    println!(
        "Audio chunks: {}",
        audio_chunks
            .iter()
            .map(|(start, end)| format!("({:.2}, {:.2})", start, end))
            .collect::<Vec<_>>()
            .join(", ")
    );

    println!("Total duration: {:.2}", duration);

    generate_mlt(
        &audio_chunks,
        duration,
        input_video,
        &output_mlt.to_string_lossy(),
    );

    println!("MLT file generated: {}", output_mlt.to_string_lossy());
}
