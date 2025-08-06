use std::fs;
use std::path::Path;

use clap::Command;
use clap_complete::{Shell, generate_to};
use clap_mangen::Man;

fn build_cli() -> Command {
    Command::new("genlint")
        .about("Generic and configurable linter")
        .arg_required_else_help(true)
}

fn main() {
    let completions_dir = Path::new("completions");
    let man_dir = Path::new("man");
    fs::create_dir_all(completions_dir).expect("Could not create completions directory");
    fs::create_dir_all(man_dir).expect("Could not create man directory");

    // Generate completions to `completions/*`
    let mut cli = build_cli();
    for &shell in &[Shell::Bash, Shell::Zsh, Shell::Fish, Shell::PowerShell] {
        let path = generate_to(shell, &mut cli, "genlint", completions_dir)
            .expect("Failed to generate completion script");
        println!(
            "cargo:warning=Completion file generated: {}",
            path.display()
        );
    }

    // Generate man page to `man/genlint.1`
    let man = Man::new(cli);
    let mut buffer: Vec<u8> = Vec::new();
    man.render(&mut buffer).expect("Failed to render man page");
    let man_path = man_dir.join("genlint.1");
    fs::write(&man_path, buffer).expect("Failed to write man page");
    println!("cargo:warning=Man page generated: {}", man_path.display());
}
