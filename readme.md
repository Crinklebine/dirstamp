# dirstamp

`dirstamp` is a tiny cross-platform command-line utility that **copies the
_last-modified_ time (`mtime`) of the newest **file** in each directory onto
the directory itself**.

It’s perfect after migrations or bulk copies where all your folders now say
“today” even though the documents inside span years.

> **Example**  
> A folder whose newest file is from **2024-07-28** will get that same date on
> the folder, instead of the day you moved it.

---

## Features

* **Scans every directory deep-first** (children processed before parents).
* **Updates only when needed** – skips a folder if it already matches.
* **Skips empty folders** or folders that contain only sub-directories
  (they keep whatever date the OS gave them).
* **Cross-platform** (Windows, Linux, macOS) – written in safe Rust.
* **Single static binary** – no runtime, no PowerShell execution-policy fuss.

---

## Quick start (Windows)

```powershell
# After you’ve built or downloaded dirstamp.exe
dirstamp.exe "D:\MigratedDocs"
or simply dirstamp.exe within the current directory
