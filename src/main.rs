// src/main.rs
// dirstamp — set each directory's mtime to match its newest immediate child
// Priority: newest file; if no files, newest immediate subdir. Empty dirs unchanged.

use std::cmp::Reverse;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use filetime::{set_file_mtime, FileTime};
use walkdir::{DirEntry, WalkDir};

// For human-readable UTC timestamps when -D/--show-dates is used.
use time::{format_description::parse as parse_format, OffsetDateTime};

const VERSION: &str = env!("CARGO_PKG_VERSION");

// These may be provided by build.rs; use option_env! so builds still work if absent.
const GIT_HASH_OPT: Option<&'static str> = option_env!("GIT_HASH");
const BUILD_DATE_OPT: Option<&'static str> = option_env!("BUILD_DATE");

const USAGE: &str = "\
dirstamp {VERSION}

Usage:
  dirstamp [PATH] [OPTIONS]

Options:
  -C, --confirm     Apply changes (default is dry run)
  -D, --show-dates  Show from → to timestamps and ±days for each change
  -V, --version     Show version information
  -h, --help        Show this help message
";

fn print_help_and_exit() -> ! {
    println!("{}", USAGE.replace("{VERSION}", VERSION));
    std::process::exit(0)
}

fn print_version_and_exit() -> ! {
    match (GIT_HASH_OPT, BUILD_DATE_OPT) {
        (Some(hash), Some(date)) if !hash.is_empty() => {
            println!("dirstamp {} ({} {})", VERSION, hash, date);
        }
        (Some(hash), None) if !hash.is_empty() => {
            println!("dirstamp {} ({})", VERSION, hash);
        }
        _ => {
            println!("dirstamp {}", VERSION);
        }
    }
    std::process::exit(0)
}

fn is_dir(entry: &DirEntry) -> bool {
    entry.file_type().is_dir()
}

fn depth_of(path: &Path) -> usize {
    path.components().count()
}

/// Find newest mtime among *immediate* children of `path`.
/// Priority: newest file; if none, newest immediate subdir; None if no children.
fn find_latest_mtime(path: &Path) -> io::Result<Option<SystemTime>> {
    let mut newest_file: Option<SystemTime> = None;
    let mut newest_dir: Option<SystemTime> = None;

    for item in fs::read_dir(path)? {
        let entry = item?;
        let meta = entry.metadata()?;
        let modified = match meta.modified() {
            Ok(m) => m,
            Err(_) => continue,
        };

        if meta.is_file() {
            newest_file = Some(match newest_file {
                Some(curr) => curr.max(modified),
                None => modified,
            });
        } else if meta.is_dir() {
            newest_dir = Some(match newest_dir {
                Some(curr) => curr.max(modified),
                None => modified,
            });
        }
    }

    Ok(newest_file.or(newest_dir))
}

fn set_folder_mtime(path: &Path, mtime: SystemTime) -> io::Result<()> {
    let ft = FileTime::from_system_time(mtime);
    set_file_mtime(path, ft)
}

fn main() -> io::Result<()> {
    // ---- parse args (simple hand-rolled flags) ----
    let mut confirm = false;
    let mut show_dates = false;
    let mut path_arg: Option<String> = None;

    let mut args = env::args().skip(1).peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => print_help_and_exit(),
            "-V" | "--version" => print_version_and_exit(),
            "-C" | "--confirm" => confirm = true,
            "-D" | "--show-dates" => show_dates = true,
            s if s.starts_with('-') => {
                eprintln!("Unknown option: {s}");
                print_help_and_exit();
            }
            _ => {
                path_arg = Some(arg);
                // First non-flag is the root; ignore any further args.
                break;
            }
        }
    }

    let root: PathBuf = PathBuf::from(path_arg.unwrap_or_else(|| ".".to_string()));
    if !root.exists() {
        eprintln!("Path does not exist: {}", root.display());
        std::process::exit(2);
    }

    // ---- collect directories and process child-before-parent ----
    let mut dirs: Vec<DirEntry> = Vec::new();
    for entry in WalkDir::new(&root).follow_links(true) {
        match entry {
            Ok(e) if is_dir(&e) => dirs.push(e),
            Ok(_) => {}
            Err(err) => eprintln!("skipped (walk error): {err}"),
        }
    }
    // Deeper paths first ⇒ children stamped before parents.
    dirs.sort_by_key(|e| Reverse(depth_of(&e.path())));

    let one_sec = Duration::from_secs(1);
    let mut updated_count = 0usize;

    // Prepare formatter for dates if requested.
    let fmt = if show_dates {
        Some(parse_format("[year]-[month]-[day] [hour]:[minute]:[second] UTC").expect("valid time format"))
    } else {
        None
    };

    for entry in dirs {
        let path = entry.path();

        // Current dir mtime
        let dir_mtime = match fs::metadata(&path).and_then(|m| m.modified()) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("skipped (mtime read failed): {:?} ({e})", path);
                continue;
            }
        };

        // Newest immediate child (file preferred, else subdir)
        let latest = match find_latest_mtime(&path) {
            Ok(Some(t)) => t,
            Ok(None) => continue, // empty dir
            Err(e) => {
                eprintln!("skipped (child scan failed): {:?} ({e})", path);
                continue;
            }
        };

        // Tolerance: only act if delta > 1s to avoid noisy rewrites on coarse FS
        let needs_change = latest > dir_mtime + one_sec || latest + one_sec < dir_mtime;
        if !needs_change {
            continue;
        }

        // Optional verbose strings
        let maybe_dates = if let Some(f) = &fmt {
            let from_s = OffsetDateTime::from(dir_mtime)
                .format(f)
                .unwrap_or_else(|_| "<bad time>".into());
            let to_s = OffsetDateTime::from(latest)
                .format(f)
                .unwrap_or_else(|_| "<bad time>".into());
            let days = match latest.duration_since(dir_mtime) {
                Ok(d) => d.as_secs_f64() / 86_400.0,
                Err(e) => -(e.duration().as_secs_f64() / 86_400.0),
            };
            Some((from_s, to_s, days))
        } else {
            None
        };

        if confirm {
            if let Err(e) = set_folder_mtime(&path, latest) {
                eprintln!("skipped (set mtime failed): {:?} ({e})", path);
                continue;
            }
            if let Some((from_s, to_s, days)) = &maybe_dates {
                println!(
                    "updated {:?} (from {} to {}, {:+.1} days)",
                    path, from_s, to_s, days
                );
            } else {
                println!("updated {:?}", path);
            }
        } else {
            if let Some((from_s, to_s, days)) = &maybe_dates {
                println!(
                    "would update {:?} (from {} to {}, {:+.1} days)",
                    path, from_s, to_s, days
                );
            } else {
                println!("would update {:?}", path);
            }
        }

        updated_count += 1;
    }

    if updated_count == 0 {
        println!("No folder timestamps needed updating.");
    } else if !confirm {
        println!("\nNote: this was a dry run. Use -C to confirm and apply changes.");
    }

    Ok(())
}