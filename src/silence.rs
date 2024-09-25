use std::process::{Command, Stdio};
use regex::Regex;
use std::fs::File;
use std::io::Write;

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
        r#"<mlt>
  <profile description="HD 1080p 25 fps" width="1920" height="1080" progressive="1" sample_aspect_num="1" sample_aspect_den="1" display_aspect_num="16" display_aspect_den="9" frame_rate_num="25" frame_rate_den="1" colorspace="709"/>
  <playlist id="playlist0">
"#,
    );

    for (i, event) in timestamps.iter().enumerate() {
        if i == 0 {
            mlt_content.push_str(&format!(
                r#"    <entry producer="producer1" in="0" out="{:.2}"/>
"#,
                event.start
            ));
        } else {
            let prev_end = timestamps[i - 1].end;
            mlt_content.push_str(&format!(
                r#"    <entry producer="producer1" in="{:.2}" out="{:.2}"/>
"#,
                prev_end, event.start
            ));
        }
    }

    if let Some(last) = timestamps.last() {
        mlt_content.push_str(&format!(
            r#"    <entry producer="producer1" in="{:.2}" out="end"/>
"#,
            last.end
        ));
    }

    mlt_content.push_str(&format!(
        r#"  </playlist>
  <producer id="producer1">
    <property name="resource">{}</property>
  </producer>
</mlt>
"#,
        input_file
    ));

    let mut file = File::create(output_file).expect("Unable to create file");
    file.write_all(mlt_content.as_bytes())
        .expect("Unable to write data");
}

pub fn silence(input_video:&str) {
    let output_mlt = "output.mlt";

    // Run FFmpeg to detect silence and parse the output
    let silence_events = run_ffmpeg(input_video);
    println!("Parsed silence events: {:?}", silence_events);

    // Generate MLT file based on the parsed silence events
    generate_mlt(&silence_events, input_video, output_mlt);

    println!("MLT file generated: {}", output_mlt);
}
