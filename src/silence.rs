use regex::Regex;
use std::fs::File;
use std::io::Write;
use std::process::{Command, Stdio};
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct SilenceEvent {
    start: f64,
    end: f64,
}

const FFMPEG_EXECUTABLE: &str = r"C:\Program Files\ShotCut\ffmpeg.exe";

fn run_ffmpeg(input_file: &str) -> Vec<SilenceEvent> {
    // Run ffmpeg to detect silence
    let ffmpeg_output = Command::new(FFMPEG_EXECUTABLE)
        .arg("-i")
        .arg(input_file)
        .arg("-af")
        .arg("silencedetect=noise=-30dB:d=0.5")
        .arg("-f")
        .arg("null")
        .arg("-")
        .stderr(Stdio::piped()) // FFmpeg writes to stderr, so we capture that
        .output()
        .expect("Failed to execute FFmpeg");

    let output = String::from_utf8_lossy(&ffmpeg_output.stderr);
    parse_ffmpeg_output(&output)
}

fn parse_ffmpeg_output(output: &str) -> Vec<SilenceEvent> {
    let mut silence_events = Vec::new();

    let silence_start_re = Regex::new(r"silence_start:\s*(\d+\.?\d*)").unwrap();
    let silence_end_re = Regex::new(r"silence_end:\s*(\d+\.?\d*)").unwrap();

    let mut current_start: Option<f64> = None;
    for line in output.lines() {
        if let Some(start_cap) = silence_start_re.captures(line) {
            current_start = Some(start_cap[1].parse::<f64>().unwrap());
        }

        if let Some(end_cap) = silence_end_re.captures(line) {
            if let Some(start) = current_start {
                let end = end_cap[1].parse::<f64>().unwrap();
                silence_events.push(SilenceEvent { start, end });
                current_start = None;
            }
        }
    }

    silence_events
}

fn generate_mlt(timestamps: &[SilenceEvent], input_file: &str, output_file: &str) {
  let mut mlt_content = String::from(
      r#"<?xml version="1.0" standalone="no"?>
<mlt LC_NUMERIC="C" version="7.27.0" producer="main_bin">
<profile width="2560" height="1440" progressive="1" sample_aspect_num="1" sample_aspect_den="1" display_aspect_num="16" display_aspect_den="9" frame_rate_num="60000000" frame_rate_den="1000000" colorspace="709"/>
<playlist id="main_bin">
</playlist>
<producer id="black" in="00:00:00.000" out="00:02:00.667">
</producer>
<playlist id="background">
  <entry producer="black" in="00:00:00.000" out="00:02:00.667"/>
</playlist>
"#,
  );

  // Add chain elements and corresponding playlist entries
  for (i, _) in timestamps.iter().enumerate() {
      // Create a chain for each segment
      mlt_content.push_str(&format!(
          r#"  <chain id="chain{}" out="00:02:00.667">
  <property name="resource">{}</property>
</chain>
"#,
          i, input_file
      ));
  }

  // Start the playlist definition
  mlt_content.push_str(r#"  <playlist id="playlist0">
"#);

  // Add entries to the playlist, linking each chain
  for (i, event) in timestamps.iter().enumerate() {
      let start_time = format_time(event.start);
      let end_time = format_time(event.end);
      mlt_content.push_str(&format!(
          r#"    <entry producer="chain{}" in="{}" out="{}"/>
"#,
          i, start_time, end_time
      ));
  }

  // Close the playlist
  mlt_content.push_str(r#"  </playlist>
"#);

  // Add tractor element with background and playlist tracks
  mlt_content.push_str(
      r#"  <tractor id="tractor0" title="Shotcut version 24.08.29" in="00:00:00.000" out="00:02:00.667">
  <property name="shotcut">1</property>
  <property name="shotcut:projectAudioChannels">2</property>
  <property name="shotcut:projectFolder">0</property>
  <track producer="background"/>
  <track producer="playlist0"/>
  <transition id="transition0">
  </transition>
  <transition id="transition1">
  </transition>
</tractor>
</mlt>
"#,
  );

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

    let silence_events = run_ffmpeg(input_video);
    println!("Parsed silence events: {:?}", silence_events);

    generate_mlt(&silence_events, input_video, &output_mlt.to_string_lossy());

    println!("MLT file generated: {}", output_mlt.to_string_lossy());
}
