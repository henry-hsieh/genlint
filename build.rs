use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use clap_complete::{Generator, Shell, generate};
use clap_mangen::Man;

#[path = "src/enums.rs"]
mod enums;

#[path = "src/args.rs"]
mod args;

fn main() {
    let update_docs = cfg!(feature = "doc");
    let is_ci = env::var("CI").is_ok();

    let completions_dir = Path::new("completions");
    let man_dir = Path::new("man");
    let help_path = Path::new("HELP.md");

    // Only create directories if we intend to write
    if update_docs {
        fs::create_dir_all(completions_dir).expect("Could not create completions directory");
        fs::create_dir_all(man_dir).expect("Could not create man directory");
    }

    let mut cli = args::build_cli();
    let mut artifacts: Vec<(PathBuf, Vec<u8>)> = Vec::new();

    // 1. Generate Completions
    for &shell in &[Shell::Bash, Shell::Zsh, Shell::Fish, Shell::PowerShell] {
        let mut buffer = Vec::new();
        generate(shell, &mut cli, "genlint", &mut buffer);

        let file_name = shell.file_name("genlint");
        artifacts.push((completions_dir.join(file_name), buffer));
    }

    // 2. Generate Man Page
    let man = Man::new(cli.clone());
    let mut man_buffer: Vec<u8> = Vec::new();
    man.render(&mut man_buffer)
        .expect("Failed to render man page");
    artifacts.push((man_dir.join("genlint.1"), man_buffer));

    // 3. Generate Help Message (HELP.md)
    let help_message = cli.render_help().to_string();
    let help_markdown = format!("```text\n{}\n```\n", help_message);
    artifacts.push((help_path.to_path_buf(), help_markdown.into_bytes()));

    if update_docs {
        // Feature "doc" enabled: Force update files
        for (path, content) in artifacts {
            fs::write(&path, content)
                .unwrap_or_else(|_| panic!("Failed to write {}", path.display()));
            println!("cargo:warning=Updated {}", path.display());
        }
    } else {
        // Feature "doc" disabled: Verify files match
        let mut out_of_sync_files = Vec::new();

        for (path, content) in artifacts {
            // Normalize EOL before comparison to avoid CRLF vs LF issues on Windows
            let generated_str = String::from_utf8_lossy(&content).replace("\r\n", "\n");
            let existing_content = match fs::read(&path) {
                Ok(content) => content,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
                Err(e) => panic!(
                    "Failed to read existing documentation file {}: {}",
                    path.display(),
                    e
                ),
            };
            let existing_str = String::from_utf8_lossy(&existing_content).replace("\r\n", "\n");
            if existing_str != generated_str {
                // Convert path separators to forward slashes for consistent display/checking
                let path_str = path.to_string_lossy().replace('\\', "/");
                out_of_sync_files.push(path_str.to_owned());
            }
        }

        if !out_of_sync_files.is_empty() {
            if is_ci {
                panic!(
                    "Documentation is out of sync with CLI definition. \
                    Files requiring update: {:?}. \
                    Run 'cargo build --features doc' locally to update them.",
                    out_of_sync_files
                );
            } else {
                for f in out_of_sync_files {
                    println!(
                        "cargo:warning=Documentation file '{}' is out of sync with CLI.",
                        f
                    );
                }
                println!("cargo:warning=Run 'cargo build --features doc' to update documentation.");
            }
        }
    }
}
