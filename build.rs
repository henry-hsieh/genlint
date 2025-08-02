use std::fs;
use std::path::Path;

use clap::Command;
use clap_complete::{generate_to, Shell};

fn build_cli() -> Command {
    Command::new("genlint")
        .about("Generic and configurable linter")
        .arg_required_else_help(true)
}

fn main() {
    let completions_dir = Path::new("completions");
    fs::create_dir_all(completions_dir).expect("Could not create completions directory");

    let mut cli = build_cli();
    for &shell in &[Shell::Bash, Shell::Zsh, Shell::Fish, Shell::PowerShell] {
        let path = generate_to(shell, &mut cli, "genlint", completions_dir)
            .expect("Failed to generate completion script");
        println!("cargo:warning=Completion file generated: {}", path.display());
    }
}
