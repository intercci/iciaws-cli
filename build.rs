// Build script: walk ./template and emit a Rust source file that embeds every
// template file as base64 bytes keyed by its relative path.
// Base64 avoids all escaping issues (raw strings break on content containing #).

use std::fs;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest = Path::new(&out_dir).join("template_files.rs");

    let template_dir = Path::new("template");
    if !template_dir.is_dir() {
        panic!("template/ directory not found — required for embedding");
    }

    // Collect all file paths, sorted for determinism.
    let mut files: Vec<PathBuf> = Vec::new();
    for entry in WalkDir::new(template_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let p = entry.path().to_path_buf();
        if let Ok(rel) = p.strip_prefix(template_dir) {
            if rel.starts_with(".claude") {
                continue;
            }
        }
        files.push(p);
    }
    files.sort();

    let mut f = BufWriter::new(fs::File::create(&dest).expect("failed to create template_files.rs"));

    writeln!(f, "/// Return a list of (relative_path, base64_content) pairs from the embedded template.")
        .unwrap();
    writeln!(
        f,
        "pub fn get_template_files() -> Vec<(&'static str, &'static str)> {{ "
    )
    .unwrap();
    writeln!(f, "    vec![").unwrap();

    for path in &files {
        let rel = path.strip_prefix(template_dir).unwrap();
        let rel_str = rel.to_string_lossy();
        let content = fs::read(path).expect("failed to read template file");
        let b64 = base64_encode(&content);
        writeln!(f, "        ({:?}, {:?}),", rel_str, b64).unwrap();
    }

    writeln!(f, "    ]").unwrap();
    writeln!(f, "}}").unwrap();
}

/// Minimal base64 encoder (avoids external dependency).
fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;

        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}
