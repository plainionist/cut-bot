use crate::config;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::fs;
use std::process::Command;

pub fn export_drawio(drawio_file: &str) {
    let drawio_executable = match config::get_value("drawio_executable") {
        Ok(value) => value,
        Err(error) => {
            eprintln!("Error reading draw.io config: {}", error);
            std::process::exit(1);
        }
    };

    let page_names = match extract_page_names(drawio_file) {
        Ok(page_names) => page_names,
        Err(error) => {
            eprintln!("Failed to read draw.io file '{}': {}", drawio_file, error);
            std::process::exit(1);
        }
    };

    if page_names.is_empty() {
        eprintln!("No pages found in '{}'", drawio_file);
        std::process::exit(1);
    }

    for (page_index, page_name) in page_names.iter().enumerate() {
        let output_file = format!("{}.png", page_name);
        println!("Exporting {}", page_name);

        let status = Command::new(&drawio_executable)
            .arg("--width")
            .arg("2560")
            .arg("--height")
            .arg("1440")
            .arg("--export")
            .arg("--output")
            .arg(&output_file)
            .arg("--page-index")
            .arg(page_index.to_string())
            .arg("--transparent")
            .arg(drawio_file)
            .status();

        match status {
            Ok(status) if status.success() => {}
            Ok(status) => {
                eprintln!(
                    "draw.io failed for page '{}' with exit code {:?}",
                    page_name,
                    status.code()
                );
                std::process::exit(1);
            }
            Err(error) => {
                eprintln!(
                    "Failed to execute '{}' for '{}': {}",
                    drawio_executable,
                    drawio_file,
                    error
                );
                std::process::exit(1);
            }
        }
    }
}

fn extract_page_names(drawio_file: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let xml = fs::read_to_string(drawio_file)?;
    let mut reader = Reader::from_str(&xml);
    reader.trim_text(true);

    let mut buffer = Vec::new();
    let mut page_names = Vec::new();

    loop {
        match reader.read_event(&mut buffer)? {
            Event::Start(event) if event.name() == b"diagram" => {
                for attribute in event.attributes() {
                    let attribute = attribute?;
                    if attribute.key == b"name" {
                        page_names.push(attribute.unescape_and_decode_value(&reader)?);
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }

        buffer.clear();
    }

    Ok(page_names)
}