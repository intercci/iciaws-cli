use anyhow::{Context, Result, bail};
use dialoguer::{theme::ColorfulTheme, Input, MultiSelect};
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

include!(concat!(env!("OUT_DIR"), "/template_files.rs"));

// ── Helpers ──────────────────────────────────────────────────────────────

/// Minimal base64 decoder (avoids external dependency).
fn base64_decode(input: &str) -> Vec<u8> {
    fn char_value(c: u8) -> Option<u32> {
        match c {
            b'A'..=b'Z' => Some((c - b'A') as u32),
            b'a'..=b'z' => Some((c - b'a' + 26) as u32),
            b'0'..=b'9' => Some((c - b'0' + 52) as u32),
            b'+' => Some(62),
            b'/' => Some(63),
            b'=' => None, // padding
            _ => None,
        }
    }

    let input = input.trim_end_matches('=');
    let mut bits: u32 = 0;
    let mut bit_count: u32 = 0;
    let mut output = Vec::with_capacity(input.len() * 3 / 4);

    for c in input.bytes() {
        if let Some(val) = char_value(c) {
            bits = (bits << 6) | val;
            bit_count += 6;
            if bit_count >= 8 {
                bit_count -= 8;
                output.push((bits >> bit_count) as u8);
            }
        }
    }
    output
}

/// Replace all known placeholders in file content.
fn substitute(content: &str, project_name: &str, description: &str) -> String {
    let mut result = content.to_string();
    result = result.replace("$NAME$", project_name);
    result = result.replace("__PROJECT_NAME__", project_name);
    result = result.replace("__DESCRIPTION__", description);
    result
}

/// Remove lines from content that correspond to unselected services.
/// Handles Cargo.toml dependency lines and src/main.rs use/init/addon lines.
fn prune_services(content: &str, selected: &[&str]) -> String {
    let selected_set: HashMap<&str, ()> = selected.iter().copied().map(|s| (s, ())).collect();
    let crate_names = [
        ("iciaws_s3", "s3", "get_s3_client"),
        ("iciaws_ses", "ses", "get_ses_client"),
        ("iciaws_sns", "sns", "get_sns_client"),
    ];

    let mut lines: Vec<String> = Vec::new();
    for line in content.lines() {
        // In Cargo.toml: skip dependency lines for unselected services
        let is_dep_line = crate_names.iter().any(|(name, _, _)| {
            line.trim().starts_with(name) && !selected_set.contains_key(name)
        });
        if is_dep_line {
            continue;
        }

        // In src/main.rs: skip use / init / addon lines for unselected services
        let should_skip = crate_names.iter().any(|(crate_name, addon_name, fn_name)| {
            !selected_set.contains_key(crate_name)
                && (line.contains(fn_name)
                    || line.contains(&format!("\x22{}\x22", addon_name))
                    || line.contains(&format!("iciaws_{}/", crate_name.replacen("iciaws_", "", 1))))
        });
        if should_skip {
            continue;
        }

        lines.push(line.to_string());
    }
    lines.join("\n")
}

/// Write content to a file, creating parent dirs as needed.
fn write_file(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
}

// ── Interactive Prompts ──────────────────────────────────────────────────

fn prompt_project_name() -> Result<String> {
    let theme = ColorfulTheme::default();
    let re = Regex::new(r"^[\w][\w\-]{0,29}$").unwrap();

    loop {
        let name: String = Input::<String>::with_theme(&theme)
            .with_prompt("Project name")
            .interact_text()?;

        // Validate: no spaces, alphanumeric + underscore/hyphen only, 1-30 chars
        if name.trim().is_empty() {
            eprintln!("  Project name cannot be empty.");
            continue;
        }
        if !re.is_match(&name) {
            eprintln!(
                "  Name must be 1-30 characters: letters, numbers, hyphens, underscores only (no spaces)."
            );
            continue;
        }

        // Check if folder already exists in CWD
        if Path::new(&name).exists() {
            eprintln!("\n  Folder \"{}\" already exists. Please choose another name.\n", name);
            continue;
        }

        return Ok(name);
    }
}

fn prompt_description() -> Result<String> {
    let theme = ColorfulTheme::default();
    Ok(Input::<String>::with_theme(&theme)
        .with_prompt("Short description")
        .default("An AWS serverless API".into())
        .interact_text()?)
}

fn prompt_services() -> Result<Vec<&'static str>> {
    let theme = ColorfulTheme::default();
    let items = vec!["S3", "SES", "SNS"];

    let selected = MultiSelect::with_theme(&theme)
        .with_prompt("Select services to include")
        .items(&items)
        .interact()?;

    if selected.is_empty() {
        bail!("Please select at least one service.");
    }

    let services: Vec<&str> = selected
        .iter()
        .map(|&i| match i {
            0 => "iciaws_s3",
            1 => "iciaws_ses",
            2 => "iciaws_sns",
            _ => unreachable!(),
        })
        .collect();
    Ok(services)
}

// ── Core Orchestration ───────────────────────────────────────────────────

fn scaffold(project_name: &str, description: &str, services: &[&str]) -> Result<()> {
    let target = PathBuf::from(project_name);

    // Build a lookup: relative_path -> processed content
    let files: HashMap<&str, String> = get_template_files()
        .into_iter()
        .map(|(rel, b64)| {
            // Decode base64 bytes → string
            let bytes = base64_decode(b64);
            let content = String::from_utf8(bytes)
                .expect("template content is valid UTF-8");

            // Step 1: variable substitution
            let mut processed = substitute(&content, project_name, description);

            // Step 2: prune unselected service code
            processed = prune_services(&processed, services);

            (rel, processed)
        })
        .collect();

    // Write all files to target directory
    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} files")
            .unwrap(),
    );

    for (rel, content) in &files {
        let dst_path = target.join(rel);
        write_file(&dst_path, content)
            .with_context(|| format!("writing: {}", rel))?;
        pb.inc(1);
    }
    pb.finish_and_clear();
    println!();

    // Post-generation message.
    println!("  ✓  Project \"{}\" created successfully!", project_name);
    println!();
    println!("  Next steps:");
    println!("    cd {}", project_name);
    println!("    cargo lambda watch          # run locally");
    println!("    ./deploy.sh                 # deploy to AWS");
    println!();
    println!("  To generate data models and endpoint handlers with AI:");
    println!("    • Open the project in Claude Code (or your preferred AI agent)");
    println!("    • The CLAUDE.md file contains instructions for extending the API");
    println!("    • Use the model and handler patterns described there");
    println!();

    Ok(())
}

// ── Entry Point ──────────────────────────────────────────────────────────

fn run() -> Result<()> {
    let project_name = prompt_project_name()?;
    let description = prompt_description()?;
    let services = prompt_services()?;

    println!("\n  Scaffolding project \"{}\" …\n", project_name);
    scaffold(&project_name, &description, &services)?;

    Ok(())
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substitute_name() {
        let input = "$NAME$ __PROJECT_NAME__";
        assert_eq!(substitute(input, "myapi", "desc"), "myapi myapi");
    }

    #[test]
    fn test_substitute_description() {
        let input = "__DESCRIPTION__";
        assert_eq!(substitute(input, "x", "hello world"), "hello world");
    }

    #[test]
    fn test_prune_cargo_toml_all_selected() {
        let content = "iciaws_s3 = { git = \"...\", package = \"iciaws_s3\" }\niciaws_ses = { git = \"...\", package = \"iciaws_ses\" }";
        let result = prune_services(content, &["iciaws_s3", "iciaws_ses"]);
        assert!(result.contains("iciaws_s3"));
        assert!(result.contains("iciaws_ses"));
    }

    #[test]
    fn test_prune_cargo_toml_one_selected() {
        let content = "iciaws_s3 = { git = \"...\", package = \"iciaws_s3\" }\niciaws_ses = { git = \"...\", package = \"iciaws_ses\" }";
        let result = prune_services(content, &["iciaws_s3"]);
        assert!(result.contains("iciaws_s3"));
        assert!(!result.contains("iciaws_ses"));
    }

    #[test]
    fn test_prune_main_rs_services() {
        let content = r#"use iciaws_s3::get_s3_client;
use iciaws_ses::get_ses_client;
    let s3_client = get_s3_client().await;
    let ses_client = get_ses_client().await;
    addon_map.put_addon("s3", s3_client);
    addon_map.put_addon("ses", ses_client);"#;
        let result = prune_services(content, &["iciaws_s3"]);
        assert!(result.contains("iciaws_s3"));
        assert!(!result.contains("iciaws_ses"));
        assert!(!result.contains("get_ses_client"));
        assert!(!result.contains(r#""ses""#));
    }

    /// Verify base64 roundtrip using the same algorithm in both modules.
    #[test]
    fn test_base64_roundtrip() {
        let original = "Hello, world!\nLine 2\r\nWith #hashes##";
        // Encode using build.rs algorithm (same logic)
        const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut result = String::new();
        for chunk in original.as_bytes().chunks(3) {
            let b0 = chunk[0] as u32;
            let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
            let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
            let triple = (b0 << 16) | (b1 << 8) | b2;
            result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
            result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
            if chunk.len() > 1 { result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char); } else { result.push('='); }
            if chunk.len() > 2 { result.push(CHARS[(triple & 0x3F) as usize] as char); } else { result.push('='); }
        }
        let decoded = String::from_utf8(base64_decode(&result)).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_embedded_template_has_files() {
        let files = get_template_files();
        assert!(!files.is_empty(), "Template should have at least one file");
        let names: Vec<_> = files.iter().map(|(name, _)| *name).collect();
        // Handle both forward and backward slashes (Windows vs cross-platform)
        assert!(names.iter().any(|n| n.replace('\\', "/").ends_with("Cargo.toml")));
        assert!(names.iter().any(|n| n.replace('\\', "/").ends_with("src/main.rs")));
    }

    #[test]
    fn test_extracted_template_is_valid_utf8() {
        for (_name, b64) in get_template_files() {
            let bytes = base64_decode(b64);
            assert!(String::from_utf8(bytes).is_ok(), "Template {} must be valid UTF-8", b64);
        }
    }

    #[test]
    fn test_substitution_on_template_with_placeholders() {
        // Only files that actually contain placeholders should be checked
        for (name, b64) in get_template_files() {
            let bytes = base64_decode(b64);
            let content = String::from_utf8(bytes).unwrap();
            let has_placeholder = content.contains("$NAME$") || content.contains("__PROJECT_NAME__") || content.contains("__DESCRIPTION__");
            if has_placeholder {
                let processed = substitute(&content, "myproject", "A cool API");
                assert!(processed.contains("myproject"), "File {} should contain substituted project name", name);
            }
        }
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
