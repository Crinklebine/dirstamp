use std::fs;
use std::io;
use std::path::Path;
use std::time::SystemTime;

use filetime::FileTime;
use walkdir::WalkDir;

/// Return the newest modification time of any *file* directly inside `dir`.
/// If the folder has no files, return `Ok(None)`.
fn latest_file_mtime(dir: &Path) -> io::Result<Option<SystemTime>> {
    let mut newest: Option<SystemTime> = None;

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            let mtime = entry.metadata()?.modified()?;
            if newest.map_or(true, |n| mtime > n) {
                newest = Some(mtime);
            }
        }
    }
    Ok(newest)
}

/// If the newest file inside `folder` is more recent than the folder’s own
/// modified time (by >1 s), update the folder’s mtime to match.
fn sync_folder_mtime(folder: &Path) -> io::Result<()> {
    if let Some(file_mtime) = latest_file_mtime(folder)? {
        let folder_mtime = fs::metadata(folder)?.modified()?;
        let diff = file_mtime
            .duration_since(folder_mtime)
            .unwrap_or_else(|e| e.duration());

        if diff.as_secs() > 1 {
            filetime::set_file_mtime(folder, FileTime::from_system_time(file_mtime))?;
            println!("updated {:?}", folder.display());
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    // Default to current directory; allow optional path argument.
    let root = std::env::args().nth(1).unwrap_or_else(|| ".".into());
    let root_path = Path::new(&root);

    // Collect every directory (recursive), then sort descending like the PowerShell script.
    let mut folders: Vec<_> = WalkDir::new(root_path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_dir())
        .map(|e| e.into_path())
        .collect();

    folders.sort_by(|a, b| b.to_string_lossy().cmp(&a.to_string_lossy()));

    for folder in folders {
        if let Err(err) = sync_folder_mtime(&folder) {
            eprintln!("skipped {:?}: {}", folder.display(), err);
        }
    }

    Ok(())
}
