use std::fs;
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

fn find_mkv_files(folder: &str) -> Vec<String> {
    let mut files: Vec<String> = fs::read_dir(folder)
        .expect("Failed to read input folder")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("mkv") {
                Some(path.to_string_lossy().to_string())
            } else {
                None
            }
        })
        .collect();
    files.sort();
    files
}

pub fn silence(input_folder: &str) {
    let mkv_files = find_mkv_files(input_folder);
    if mkv_files.is_empty() {
        eprintln!("No .mkv files found in folder: {}", input_folder);
        return;
    }

    println!("Found {} .mkv file(s):", mkv_files.len());
    for f in &mkv_files {
        println!("  {}", f);
    }

    let output_mlt = Path::new(input_folder)
        .join("output.mlt")
        .to_string_lossy()
        .to_string();

    let mut all_chunks: Vec<(String, f64, f64)> = Vec::new();
    let mut total_duration = 0.0;

    for file in &mkv_files {
        let duration = ffmpeg::extract_duration(file).unwrap_or_default();
        let loud_periods = ffmpeg::extract_loud_starts(file).unwrap_or_default();
        let silent_periods = ffmpeg::extract_silence_starts(file).unwrap_or_default();

        let audio_chunks = find_audio_chunks(&loud_periods, &silent_periods, duration);

        println!(
            "Audio chunks for {}: {}",
            file,
            audio_chunks
                .iter()
                .map(|(start, end)| format!("({:.2}, {:.2})", start, end))
                .collect::<Vec<_>>()
                .join(", ")
        );

        for (start, end) in &audio_chunks {
            all_chunks.push((file.clone(), *start, *end));
        }

        if duration > total_duration {
            total_duration = duration;
        }
    }

    MltBuilder::new()
        .chunks(all_chunks)
        .duration(total_duration)
        .output_file(&output_mlt)
        .build();

    println!("MLT file generated: {}", output_mlt);
}
