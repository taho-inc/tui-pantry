use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, ExitCode};

const SCAFFOLD_MAIN: &str = include_str!("../../scaffold/main.rs");
const SCAFFOLD_WIDGETS: &str = include_str!("../../scaffold/widgets.rs");
const SCAFFOLD_PANES: &str = include_str!("../../scaffold/panes.rs");
const SCAFFOLD_VIEWS: &str = include_str!("../../scaffold/views.rs");

const PANTRY_TOML: &str = "\
[config]
theme = \"dark\"
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

    if forward.first().map(|s| s.as_str()) == Some("init") {
        return init();
    }

    let (package, rest) = extract_package(forward);

    // When no -p flag, we can validate locally and scaffold the example.
    // With -p, let cargo handle resolution — it knows the workspace.
    if package.is_none() {
        let cwd = env::current_dir().map_err(|e| format!("cannot read working directory: {e}"))?;

        if !cwd.join("pantry.toml").exists() {
            return Err(format!(
                "no pantry.toml in {}\n\
                 Run `cargo pantry init` to scaffold, or create one manually.",
                cwd.display()
            ));
        }
    }

    // Probe for the right feature name with `cargo build` (captured output),
    // then launch with `cargo run` (inherited stdio so the TUI owns the terminal).
    let feature = resolve_feature(&package, &rest)?;

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--example", "widget_preview", "--features", &feature]);
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

fn init() -> Result<ExitCode, String> {
    let cwd = env::current_dir().map_err(|e| format!("cannot read working directory: {e}"))?;

    if !cwd.join("Cargo.toml").exists() {
        return Err("no Cargo.toml in current directory".into());
    }

    // Add tui-pantry as optional dependency via cargo add.
    let status = Command::new("cargo")
        .args(["add", "tui-pantry", "--optional"])
        .status()
        .map_err(|e| format!("failed to run cargo add: {e}"))?;

    if !status.success() {
        return Err("cargo add tui-pantry --optional failed".into());
    }

    ensure_file(&cwd.join("pantry.toml"), PANTRY_TOML)?;

    let example_dir = cwd.join("examples/widget_preview");
    ensure_file(&example_dir.join("main.rs"), SCAFFOLD_MAIN)?;
    ensure_file(&example_dir.join("widgets.rs"), SCAFFOLD_WIDGETS)?;
    ensure_file(&example_dir.join("panes.rs"), SCAFFOLD_PANES)?;
    ensure_file(&example_dir.join("views.rs"), SCAFFOLD_VIEWS)?;

    eprintln!("\ncargo-pantry: initialized. Run `cargo pantry` to see the sample ingredients.");
    eprintln!();
    eprintln!("Scaffolded examples/widget_preview/ with Widgets, Panes, and Views tabs.");
    eprintln!("Edit these files to replace samples with your own widgets, or delete");
    eprintln!("the directory and start fresh.");
    eprintln!();
    eprintln!("To move ingredients into your crate for reuse:");
    eprintln!("  1. Create src/ingredient.rs with your widget previews");
    eprintln!("  2. Gate it: #[cfg(feature = \"tui-pantry\")] pub mod ingredient;");
    eprintln!("  3. Update examples/widget_preview/main.rs:");
    eprintln!("       tui_pantry::run!(my_crate::ingredient::ingredients())");

    Ok(ExitCode::SUCCESS)
}

/// Probe `cargo build` to find which feature name the crate uses.
/// Tries `tui-pantry` (implicit from `cargo add --optional`) then `pantry` (alias).
/// Build output is captured so it doesn't pollute the terminal.
fn resolve_feature(package: &Option<String>, rest: &[String]) -> Result<String, String> {
    for feature in ["tui-pantry", "pantry"] {
        let mut cmd = Command::new("cargo");
        cmd.args([
            "build",
            "--example",
            "widget_preview",
            "--features",
            feature,
        ]);
        if let Some(pkg) = package {
            cmd.args(["-p", pkg]);
        }
        cmd.args(rest);

        let output = cmd
            .output()
            .map_err(|e| format!("failed to run cargo: {e}"))?;

        if output.status.success() {
            return Ok(feature.to_string());
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        let not_found = stderr.contains(&format!("feature `{feature}` is not a feature"))
            || stderr.contains(&format!("does not have feature `{feature}`"));
        if !not_found {
            // Real build error — surface it.
            std::io::Write::write_all(&mut std::io::stderr(), &output.stderr).ok();
            return Err(format!("build failed with --features {feature}"));
        }
    }

    Err("neither `tui-pantry` nor `pantry` feature found.\n\
         Run `cargo pantry init` or `cargo add tui-pantry --optional`."
        .into())
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

fn ensure_file(path: &Path, content: &str) -> Result<(), String> {
    if path.exists() {
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
    }

    fs::write(path, content).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    eprintln!("cargo-pantry: created {}", path.display());

    Ok(())
}
