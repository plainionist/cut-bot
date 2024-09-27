use regex::Regex;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug)]
struct SilenceEvent {
    start: f64,
    end: f64,
}

const FFMPEG_EXECUTABLE: &str = r"C:\Program Files\ShotCut\ffmpeg.exe";

fn run_ffmpeg(input_file: &str) -> (Vec<SilenceEvent>, f64) {
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

    let ffmpeg_output = Command::new(FFMPEG_EXECUTABLE)
        .arg("-i")
        .arg(input_file)
        .arg("-af")
        .arg("silencedetect=noise=-40dB:d=0.9")
        .arg("-f")
        .arg("null")
        .arg("-")
        .stderr(Stdio::piped()) // FFmpeg writes to stderr, so we capture that
        .output()
        .expect("Failed to execute FFmpeg");

    let output = String::from_utf8_lossy(&ffmpeg_output.stderr);
    let silence_events = parse_ffmpeg_output(&output, duration);

    (silence_events, duration)
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

fn parse_ffmpeg_output(output: &str, total_duration: f64) -> Vec<SilenceEvent> {
    let mut silence_events = Vec::new();

    let silence_start_re = Regex::new(r"silence_start:\s*(\d+\.?\d*)").unwrap();
    let silence_end_re = Regex::new(r"silence_end:\s*(\d+\.?\d*)").unwrap();

    let mut current_position = 0.0;
    let mut last_silence_end: Option<f64> = None;

    for line in output.lines() {
        // Detect silence start
        if let Some(start_cap) = silence_start_re.captures(line) {
            let silence_start = start_cap[1].parse::<f64>().unwrap();

            // Add non-silent part before the silence if it exists
            if let Some(last_end) = last_silence_end {
                if last_end < silence_start {
                    silence_events.push(SilenceEvent {
                        start: last_end,
                        end: silence_start,
                    });
                }
            } else if current_position < silence_start {
                // This handles the non-silent part before the first silence
                silence_events.push(SilenceEvent {
                    start: current_position,
                    end: silence_start,
                });
            }

            current_position = silence_start;
        }

        // Detect silence end
        if let Some(end_cap) = silence_end_re.captures(line) {
            let silence_end = end_cap[1].parse::<f64>().unwrap();

            // Add the silent part as well
            silence_events.push(SilenceEvent {
                start: current_position,
                end: silence_end,
            });

            current_position = silence_end;
            last_silence_end = Some(silence_end);
        }
    }

    // Add the last non-silent part after the last silence, if any
    if let Some(last_end) = last_silence_end {
        if last_end < total_duration {
            silence_events.push(SilenceEvent {
                start: last_end,
                end: total_duration,
            });
        }
    } else if current_position < total_duration {
        // If there was no silence, add the entire remaining part
        silence_events.push(SilenceEvent {
            start: current_position,
            end: total_duration,
        });
    }

    silence_events
}

fn generate_mlt(timestamps: &[SilenceEvent], duration: f64, input_file: &str, output_file: &str) {
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
"#,total_duration)
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

    let (silence_events, duration) = run_ffmpeg(input_video);
    println!("Parsed silence events: {:?}", silence_events);

    generate_mlt(
        &silence_events,
        duration,
        input_video,
        &output_mlt.to_string_lossy(),
    );

    println!("MLT file generated: {}", output_mlt.to_string_lossy());
}
