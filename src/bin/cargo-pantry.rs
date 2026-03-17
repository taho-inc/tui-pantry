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
    let (package, rest) = extract_package(forward);

    // When no -p flag, we can validate locally and scaffold the example.
    // With -p, let cargo handle resolution — it knows the workspace.
    if package.is_none() {
        let cwd = env::current_dir().map_err(|e| format!("cannot read working directory: {e}"))?;

        if !cwd.join("pantry.toml").exists() {
            return Err(format!(
                "no pantry.toml in {}\n\
                 Create one with [ingredients] to get started.",
                cwd.display()
            ));
        }

        ensure_example(&cwd.join("examples/widget_preview.rs"))?;
    }

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--example", "widget_preview", "--features", "pantry"]);
    if let Some(ref pkg) = package {
        cmd.args(["-p", pkg]);
    }
    cmd.args(&rest);

    let status = cmd
        .status()
        .map_err(|e| format!("failed to run cargo: {e}"))?;

    Ok(if status.success() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(status.code().unwrap_or(1) as u8)
    })
}

/// Extract `-p <pkg>` or `--package <pkg>` from args, returning the
/// package name and remaining args with the flag pair removed.
fn extract_package(args: &[String]) -> (Option<String>, Vec<String>) {
    let mut i = 0;
    while i < args.len() {
        if (args[i] == "-p" || args[i] == "--package") && i + 1 < args.len() {
            let pkg = args[i + 1].clone();
            let mut rest = args[..i].to_vec();
            rest.extend_from_slice(&args[i + 2..]);
            return (Some(pkg), rest);
        }
        if args[i] == "--" {
            break;
        }
        i += 1;
    }
    (None, args.to_vec())
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

    fs::write(path, EXAMPLE_SOURCE).map_err(|e| format!("cannot write {}: {e}", path.display()))?;

    eprintln!("cargo-pantry: created {}", path.display());

    Ok(())
}
