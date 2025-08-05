//! src/main.rs
//! dirstamp — update each directory’s modification time so it matches the
//! newest item directly inside it. Priority is:
//!   1. Newest file
//!   2. If no files exist, newest immediate sub-folder
//! Empty directories are left unchanged.

use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::time::SystemTime;

use filetime::{set_file_mtime, FileTime};
use walkdir::WalkDir;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const GIT_HASH: &str = env!("GIT_HASH");
const BUILD_DATE: &str = env!("BUILD_DATE");

const USAGE: &str = "\
dirstamp {VERSION}

Usage:
  dirstamp [PATH] [OPTIONS]

Options:
  -C, --confirm     Actually apply changes (default is dry-run)
  -V, --version     Show version information
  -h, --help        Show this help message
";

fn main() -> io::Result<()> {
    let mut args = env::args();
    let _program = args.next(); // skip program name

    let mut path_arg = None;
    let mut confirm = false;
    let mut updated_count = 0;


    for arg in args {
        match arg.as_str() {
            "-h" | "--help" => {
                println!("{}", USAGE.replace("{VERSION}", VERSION));
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
            "-C" | "--confirm" => {
                confirm = true;
            }
            _ if path_arg.is_none() => {
                path_arg = Some(arg);
            }
            _ => {
                eprintln!("Unknown argument: {}", arg);
                return Ok(());
            }
        }
    }

    let root = Path::new(path_arg.as_deref().unwrap_or("."));
    let mut dirs: Vec<_> = WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
        .collect();

    dirs.sort_by_key(|e| std::cmp::Reverse(e.path().to_path_buf()));

    for entry in dirs {
        if let Ok(metadata) = fs::metadata(entry.path()) {
            let dir_mtime = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);

            if let Some(latest) = find_latest_mtime(entry.path()).unwrap_or(None) {
                if dir_mtime != latest {
                    if confirm {
                        set_folder_mtime(entry.path(), latest)?;
                        println!("updated {:?}", entry.path());
                    } else {
                        println!("would update {:?}", entry.path());
                    }
                    updated_count += 1;
                }
            }
        }
    }

    if updated_count == 0 {
        println!("No folder timestamps need updating.");
    } else if !confirm {
        println!("\nNote: this was a dry run. Use -C to confirm and apply changes.");
    }

    Ok(())
}

fn find_latest_mtime(path: &Path) -> io::Result<Option<SystemTime>> {
    let mut newest_file: Option<SystemTime> = None;
    let mut newest_dir: Option<SystemTime> = None;

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let meta = entry.metadata()?;

        let modified = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);

        if meta.is_file() {
            newest_file = Some(newest_file.map_or(modified, |curr| curr.max(modified)));
        } else if meta.is_dir() {
            newest_dir = Some(newest_dir.map_or(modified, |curr| curr.max(modified)));
        }
    }

    Ok(newest_file.or(newest_dir))
}

fn set_folder_mtime(path: &Path, mtime: SystemTime) -> io::Result<()> {
    let ft = FileTime::from_system_time(mtime);
    set_file_mtime(path, ft)
}
