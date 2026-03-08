use crate::tool;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::fs;

pub fn export_drawio(drawio_file: &str) {
    if let Err(error) = run_export_drawio(drawio_file) {
        eprintln!("{}", error);
        std::process::exit(1);
    }
}

fn run_export_drawio(drawio_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let drawio_executable = tool::required_executable("drawio_executable")?;
    tool::validate_executable(&drawio_executable, "draw.io")?;

    let page_names = extract_page_names(drawio_file)?;
    if page_names.is_empty() {
        return Err(format!("No pages found in '{}'", drawio_file).into());
    }

    for (page_index, page_name) in page_names.iter().enumerate() {
        let output_file = format!("{}.png", sanitize_file_name(page_name));
        println!("Exporting {}", page_name);

        let args = vec![
            "--width".to_string(),
            "2560".to_string(),
            "--height".to_string(),
            "1440".to_string(),
            "--export".to_string(),
            "--output".to_string(),
            output_file,
            "--page-index".to_string(),
            page_index.to_string(),
            "--transparent".to_string(),
            drawio_file.to_string(),
        ];

        tool::run_command(&drawio_executable, &args, &format!("page '{}'", page_name))?;
    }

    Ok(())
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

fn sanitize_file_name(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|character| match character {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            _ => character,
        })
        .collect();

    let trimmed = sanitized.trim().trim_end_matches('.').to_string();
    if trimmed.is_empty() {
        "page".to_string()
    } else {
        trimmed
    }
}