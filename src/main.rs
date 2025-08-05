//! src/main.rs
//! dirstamp — update each directory’s modification time so it matches the
//! newest item directly inside it. Priority is:
//!   1. Newest file
//!   2. If no files exist, newest immediate sub-folder
//! Empty directories are left unchanged.

use std::fs;
use std::io;
use std::path::Path;
use std::time::SystemTime;

use filetime::FileTime;
use walkdir::WalkDir;

/// Program version, commit hash, and build date embedded at compile time.
/// - VERSION comes from Cargo.toml
/// - GIT_HASH and BUILD_DATE are set by build.rs
const VERSION: &str = env!("CARGO_PKG_VERSION");
const GIT_HASH: &str = env!("GIT_HASH");
const BUILD_DATE: &str = env!("BUILD_DATE");

/// One-paragraph help string shown with -h / --help.
const USAGE: &str = r#"dirstamp [OPTIONS] [PATH]

Synchronise each folder’s modified time to its newest item
(files take priority; if none, newest immediate sub-folder).

Options
  -h, --help       Show this help message and exit
  -V, --version    Show version and exit

Arguments
  PATH             Root directory to process (default: current dir)
"#;

/// Return the newest modification time among direct children of `dir`.
/// Files take priority; if no files are present, fall back to sub-folders.
/// Returns `None` for an entirely empty directory.
fn latest_child_mtime(dir: &Path) -> io::Result<Option<SystemTime>> {
    let mut newest_file: Option<SystemTime> = None;
    let mut newest_dir:  Option<SystemTime> = None;

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let meta  = entry.metadata()?;
        let mtime = meta.modified()?;

        if meta.is_file() {
            if newest_file.map_or(true, |t| mtime > t) {
                newest_file = Some(mtime);
            }
        } else if meta.is_dir() {
            if newest_dir.map_or(true, |t| mtime > t) {
                newest_dir = Some(mtime);
            }
        }
    }
    Ok(newest_file.or(newest_dir))
}

/// Update `folder`'s mtime if the chosen child time differs by >1 s.
fn sync_folder_mtime(folder: &Path) -> io::Result<()> {
    if let Some(child_mtime) = latest_child_mtime(folder)? {
        let folder_mtime = fs::metadata(folder)?.modified()?;
        let diff = child_mtime
            .duration_since(folder_mtime)
            .unwrap_or_else(|e| e.duration());

        if diff.as_secs() > 1 {
            filetime::set_file_mtime(folder, FileTime::from_system_time(child_mtime))?;
            println!("updated {:?}", folder.display());
        }
    }
    Ok(())
}


fn main() -> io::Result<()> {
    // Parse command-line arguments.
    let mut args = std::env::args().skip(1);          // iterator of remaining CLI args
    let mut root   = ".".to_string();                 // default path

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                println!("{USAGE}");
                return Ok(());
            }
            "-V" | "--version" => {
            // Show full version info only if built from source with commit hash.
            // Avoid showing a potentially confusing build date when installed via crates.io.    
            if GIT_HASH.is_empty() {
                    println!("dirstamp {VERSION}");
                } else {
                    println!("dirstamp {VERSION} ({GIT_HASH} {BUILD_DATE})");
                }
                    return Ok(());
            }
            _ if arg.starts_with('-') => {
                eprintln!("Unknown option: {arg}\n\n{USAGE}");
                std::process::exit(1);
            }
            _ => {
                // first non-flag = path argument
                root = arg;
                break;                 // stop option parsing
            }
        }
    }

    // build remaining program using `root`
    let root_path = Path::new(&root);

    // Collect every directory, children before parents (deep-first).
    let mut folders: Vec<_> = WalkDir::new(root_path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_dir())
        .map(|e| e.into_path())
        .collect();

    // Reverse alphabetic order ensures deepest paths first.
    folders.sort_by(|a, b| b.to_string_lossy().cmp(&a.to_string_lossy()));

    for folder in folders {
        if let Err(err) = sync_folder_mtime(&folder) {
            eprintln!("skipped {:?}: {}", folder.display(), err);
        }
    }
    Ok(())
}
