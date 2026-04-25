use std::fs::File;
use std::io::{BufRead, BufReader};
use std::env;
use std::path::Path;

use csv::Writer;
use walkdir::WalkDir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: mbox_to_csv <thunderbird_dir> <output_csv>");
        std::process::exit(1);
    }

    let base_dir = &args[1];
    let output_path = &args[2];

    let mut writer = Writer::from_path(output_path)?;

    writer.write_record(&[
        "message_id",
        "source_file",
        "subject",
        "from",
        "to",
        "cc",
        "bcc",
        "recipient_type"
    ])?;

    let mut global_id: u64 = 0;

    for entry in WalkDir::new(base_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        if !is_mbox_file(path) {
            continue;
        }

        eprintln!("Processing {:?}", path);

        process_mbox(path, &mut writer, &mut global_id)?;
    }

    writer.flush()?;
    Ok(())
}

fn is_mbox_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    if path.extension().is_some() {
        return false;
    }

    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };

    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();

    if reader.read_until(b'\n', &mut buffer).is_err() {
        return false;
    }

    let line = String::from_utf8_lossy(&buffer);
    line.starts_with("From ")
}

fn process_mbox(
    path: &Path,
    writer: &mut Writer<File>,
    global_id: &mut u64,
) -> Result<(), Box<dyn std::error::Error>> {

    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    let mut buffer: Vec<u8> = Vec::new();
    let mut current_message: Vec<String> = Vec::new();

    loop {
        buffer.clear();
        let bytes_read = reader.read_until(b'\n', &mut buffer)?;

        if bytes_read == 0 {
            break;
        }

        let line = String::from_utf8_lossy(&buffer).to_string();

        if line.starts_with("From ") && !current_message.is_empty() {
            process_message(&current_message, *global_id, writer, path)?;
            current_message.clear();
            *global_id += 1;
        }

        current_message.push(line);
    }

    if !current_message.is_empty() {
        process_message(&current_message, *global_id, writer, path)?;
        *global_id += 1;
    }

    Ok(())
}

fn process_message(
    raw_lines: &Vec<String>,
    message_id: u64,
    writer: &mut Writer<File>,
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {

    let mut lines = raw_lines.clone();

    if !lines.is_empty() && lines[0].starts_with("From ") {
        lines.remove(0);
    }

    while !lines.is_empty() && lines[0].trim().is_empty() {
        lines.remove(0);
    }

    // Extract header block
    let mut header_lines = Vec::new();
    for line in &lines {
        if line.trim().is_empty() {
            break;
        }
        header_lines.push(line.clone());
    }

    if header_lines.is_empty() {
        return Ok(());
    }

    // Normalize folded headers (RFC continuation lines)
    let normalized = unfold_headers(header_lines);

    let subject = get_header(&normalized, "Subject");
    let from = get_header(&normalized, "From");
    let to = get_header(&normalized, "To");
    let cc = get_header(&normalized, "Cc");
    let bcc = get_header(&normalized, "Bcc");

    let tos = split_addresses(&to);
    let ccs = split_addresses(&cc);
    let bccs = split_addresses(&bcc);

    let source = path.to_string_lossy().to_string();
    let id_str = message_id.to_string();

    if tos.is_empty() && ccs.is_empty() && bccs.is_empty() {
        writer.write_record(&[
            id_str.as_str(),
            source.as_str(),
            subject.as_str(),
            from.as_str(),
            "",
            "",
            "",
            "",
        ])?;
    }

    for t in tos {
        writer.write_record(&[
            id_str.as_str(),
            source.as_str(),
            subject.as_str(),
            from.as_str(),
            t.as_str(),
            "",
            "",
            "to",
        ])?;
    }

    for c in ccs {
        writer.write_record(&[
            id_str.as_str(),
            source.as_str(),
            subject.as_str(),
            from.as_str(),
            "",
            c.as_str(),
            "",
            "cc",
        ])?;
    }

    for b in bccs {
        writer.write_record(&[
            id_str.as_str(),
            source.as_str(),
            subject.as_str(),
            from.as_str(),
            "",
            "",
            b.as_str(),
            "bcc",
        ])?;
    }

    Ok(())
}

fn unfold_headers(lines: Vec<String>) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();

    for line in lines {
        if line.starts_with(' ') || line.starts_with('\t') {
            current.push_str(" ");
            current.push_str(line.trim());
        } else {
            if !current.is_empty() {
                result.push(current);
            }
            current = line;
        }
    }

    if !current.is_empty() {
        result.push(current);
    }

    result
}

fn get_header(lines: &Vec<String>, name: &str) -> String {
    let name_lower = name.to_lowercase();

    for line in lines {
        if let Some(pos) = line.find(':') {
            let key = line[..pos].trim().to_lowercase();
            if key == name_lower {
                return line[pos + 1..].trim().to_string();
            }
        }
    }

    String::new()
}

fn split_addresses(field: &str) -> Vec<String> {
    field
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}