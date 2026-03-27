use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, ExitCode};

const SCAFFOLD_MAIN: &str = include_str!("../../scaffold/main.rs");

const PANTRY_TOML: &str = "\
[config]
theme = \"dark\"

[ingredients]
source = \"my_crate\"  # replace with your crate name
modules = []
# Add module paths as you create .ingredient.rs files:
# modules = [\"widgets::button\", \"panes::header\"]
";

fn cargo_alias(feature: &str) -> String {
    format!("\n[alias]\npantry = \"run --example widget_preview --features {feature}\"\n")
}

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

    match forward.first().map(|s| s.as_str()) {
        Some("init") => return init(),
        Some("dump") => return headless(forward, &["--dump"]),
        Some("list") => return headless(forward, &["--list"]),
        _ => {}
    }

    let (package, rest) = extract_package(forward);
    require_pantry_toml(&package)?;

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

    let status = Command::new("cargo")
        .args(["add", "tui-pantry", "--optional"])
        .status()
        .map_err(|e| format!("failed to run cargo add: {e}"))?;

    if !status.success() {
        return Err("cargo add tui-pantry --optional failed".into());
    }

    let feature = detect_feature_name(&cwd)?;

    ensure_file(&cwd.join("pantry.toml"), PANTRY_TOML)?;
    ensure_file(&cwd.join("examples/widget_preview/main.rs"), SCAFFOLD_MAIN)?;
    ensure_cargo_alias(&cwd, &feature)?;

    eprintln!("\ncargo-pantry: initialized.");
    eprintln!();
    eprintln!("Next steps:");
    eprintln!("  1. Create .ingredient.rs files in src/ with #[cfg(feature = \"{feature}\")]");
    eprintln!("  2. Add module paths to pantry.toml under [ingredients] modules");
    eprintln!("  3. Run `cargo pantry` to preview");
    eprintln!();
    eprintln!("See https://docs.taho.is/tui-pantry/getting-started for the full guide.");

    Ok(ExitCode::SUCCESS)
}

/// When no `-p` flag, validate that pantry.toml exists locally.
/// With `-p`, let cargo handle resolution — it knows the workspace.
fn require_pantry_toml(package: &Option<String>) -> Result<(), String> {
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

    Ok(())
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
            || stderr.contains(&format!("does not have feature `{feature}`"))
            || stderr.contains(&format!("does not contain this feature: {feature}"));
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

/// Run the example binary in headless mode, forwarding args after `--`.
///
/// `prefix` is the flag(s) injected before the forwarded arguments
/// (e.g. `["--dump"]` or `["--list"]`).
fn headless(forward: &[String], prefix: &[&str]) -> Result<ExitCode, String> {
    let after_sub = &forward[1..];
    let (package, rest) = extract_package(after_sub);
    require_pantry_toml(&package)?;

    let feature = resolve_feature(&package, &[])?;

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--example", "widget_preview", "--features", &feature]);

    if let Some(ref pkg) = package {
        cmd.args(["-p", pkg]);
    }

    cmd.arg("--");
    cmd.args(prefix);
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

/// Detect whether the crate uses `pantry` or `tui-pantry` as the feature name.
///
/// Prefers an explicit `pantry` feature (the conventional rename) over the
/// implicit `tui-pantry` feature created by `cargo add --optional`.
fn detect_feature_name(cwd: &Path) -> Result<String, String> {
    let cargo_toml = fs::read_to_string(cwd.join("Cargo.toml"))
        .map_err(|e| format!("cannot read Cargo.toml: {e}"))?;

    let manifest: toml::Table =
        toml::from_str(&cargo_toml).map_err(|e| format!("cannot parse Cargo.toml: {e}"))?;

    let has_pantry = manifest
        .get("features")
        .and_then(|f| f.as_table())
        .is_some_and(|features| features.contains_key("pantry"));

    Ok(if has_pantry { "pantry" } else { "tui-pantry" }.into())
}

/// Create or append the `pantry` alias to `.cargo/config.toml`.
///
/// Skips if the alias already exists. Appends rather than overwrites
/// because `.cargo/config.toml` may contain other user configuration.
fn ensure_cargo_alias(cwd: &Path, feature: &str) -> Result<(), String> {
    let alias = cargo_alias(feature);
    let config_path = cwd.join(".cargo/config.toml");

    if config_path.exists() {
        let content = fs::read_to_string(&config_path)
            .map_err(|e| format!("cannot read {}: {e}", config_path.display()))?;

        if content.contains("pantry") {
            return Ok(());
        }

        fs::write(&config_path, format!("{content}{alias}"))
            .map_err(|e| format!("cannot write {}: {e}", config_path.display()))?;

        eprintln!("cargo-pantry: appended alias to {}", config_path.display());
    } else {
        ensure_file(&config_path, alias.trim_start())?;
    }

    Ok(())
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
