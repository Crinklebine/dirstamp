// build.rs
use std::process::Command;

fn main() {
    // ----- git hash (short) -----
    let git_hash = Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_else(|| "unknown".into())
        .trim()
        .to_owned();

    // ----- build date + time (UTC) -----
    // Keep UTC for reproducible logs.
    let build_date = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();

    // Export as env vars for rustc
    println!("cargo:rustc-env=GIT_HASH={git_hash}");
    println!("cargo:rustc-env=BUILD_DATE={build_date}");
}
