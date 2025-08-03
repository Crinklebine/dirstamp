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
    // Default to current directory; allow optional path argument.
    let root = std::env::args().nth(1).unwrap_or_else(|| ".".into());
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
