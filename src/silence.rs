use std::path::Path;
use crate::ffmpeg;
use crate::mlt_builder::MltBuilder;

fn find_audio_chunks(
    loud_periods: &[f64],
    silent_periods: &[f64],
    total_duration: f64,
) -> Vec<(f64, f64)> {
    let mut chunks = Vec::new();
    let mut current_time = 0.0;

    for &start in loud_periods {
        // Add silent period before the loud period if necessary
        if current_time < start {
            chunks.push((current_time, start)); // Silent chunk before loud period
        }

        // Find the next silent period after the loud start
        let end = silent_periods
            .iter()
            .find(|&&silence_start| silence_start > start)
            .cloned()
            .unwrap_or(total_duration); // If no silence is found, go until the end

        chunks.push((start, end)); // Loud chunk
        current_time = end; // Move to the end of this loud chunk
    }

    // Add any remaining silent chunk if necessary
    if current_time < total_duration {
        chunks.push((current_time, total_duration)); 
    }

    chunks
}

fn generate_output_mlt_path(input_video: &str) -> String {
    let input_path = Path::new(input_video);
    let parent_dir = input_path.parent().unwrap_or_else(|| Path::new("."));
    parent_dir.join("output.mlt").to_string_lossy().to_string()
}

pub fn silence(input_video: &str) {
    let output_mlt = generate_output_mlt_path(input_video);

    let duration = ffmpeg::extract_duration(input_video).unwrap_or_default();
    let loud_periods = ffmpeg::extract_loud_starts(input_video).unwrap_or_default();
    let silent_periods = ffmpeg::extract_silence_starts(input_video).unwrap_or_default();

    let audio_chunks = find_audio_chunks(&loud_periods, &silent_periods, duration);

    println!(
        "Audio chunks: {}",
        audio_chunks
            .iter()
            .map(|(start, end)| format!("({:.2}, {:.2})", start, end))
            .collect::<Vec<_>>()
            .join(", ")
    );

    MltBuilder::new()
        .timestamps(audio_chunks.clone())
        .duration(duration)
        .input_file(input_video)
        .output_file(&output_mlt)
        .build();

    println!("MLT file generated: {}", output_mlt);
}
