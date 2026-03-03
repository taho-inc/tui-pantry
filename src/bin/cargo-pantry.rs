use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, ExitCode};

const EXAMPLE_SOURCE: &str = "\
fn main() -> std::io::Result<()> {
    tui_pantry::run!()
}
";

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(msg) => {
            eprintln!("cargo-pantry: {msg}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<ExitCode, String> {
    let args: Vec<String> = env::args().collect();
    let forward = skip_subcommand_token(&args);

    let cwd = env::current_dir().map_err(|e| format!("cannot read working directory: {e}"))?;

    if !cwd.join("pantry.toml").exists() {
        return Err(format!(
            "no pantry.toml in {}\n\
             Create one with [ingredients] to get started.",
            cwd.display()
        ));
    }

    let example = cwd.join("examples/pantry.rs");
    ensure_example(&example)?;

    let status = Command::new("cargo")
        .arg("run")
        .arg("--example")
        .arg("pantry")
        .arg("--features")
        .arg("pantry")
        .args(forward)
        .status()
        .map_err(|e| format!("failed to run cargo: {e}"))?;

    Ok(if status.success() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(status.code().unwrap_or(1) as u8)
    })
}

/// Cargo subcommand convention: argv = ["cargo-pantry", "pantry", ...rest].
/// Strip the subcommand token so forwarded args are clean.
fn skip_subcommand_token(args: &[String]) -> &[String] {
    if args.len() > 1 && args[1] == "pantry" {
        &args[2..]
    } else {
        &args[1..]
    }
}

fn ensure_example(path: &Path) -> Result<(), String> {
    if path.exists() {
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
    }

    fs::write(path, EXAMPLE_SOURCE)
        .map_err(|e| format!("cannot write {}: {e}", path.display()))?;

    eprintln!("cargo-pantry: created {}", path.display());

    Ok(())
}
