use std::path::{Path, PathBuf};
use crate::ffmpeg;
use crate::mlt_builder;

fn find_audio_chunks(
    loud_periods: &[f64],
    silent_periods: &[f64],
    total_duration: f64,
) -> Vec<(f64, f64)> {
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

fn generate_mlt(timestamps: &Vec<(f64, f64)>, duration: f64, input_file: &str, output_file: &str) {
    mlt_builder::MltBuilder::new()
        .timestamps(timestamps.clone())
        .duration(duration)
        .input_file(input_file)
        .output_file(output_file)
        .build();
}

fn generate_output_mlt_path(input_path: &Path) -> PathBuf {
    let parent_dir = input_path.parent().unwrap_or_else(|| Path::new("."));
    parent_dir.join("output.mlt")
}

pub fn silence(input_video: &str) {
    let input_path = Path::new(input_video);
    let output_mlt = generate_output_mlt_path(input_path);

    let duration = ffmpeg::extract_duration(input_video).unwrap_or_default();

    let loud_periods = ffmpeg::extract_loud_starts(input_video).unwrap_or_default();

    let silent_periods = ffmpeg::extract_silence_starts(input_video).unwrap_or_default();

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
