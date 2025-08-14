[![Crates.io](https://img.shields.io/crates/v/dirstamp.svg)](https://crates.io/crates/dirstamp)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

# dirstamp

**dirstamp** updates the *modification timestamp* of each folder so that it matches the newest file (or folder) directly inside it.

This is useful when migrating or restoring folders, as some operating systems and tools do not preserve folder timestamps accurately.


> **Example**  
> You copy a folder on **2025-08-03**, so Windows shows its *Date modified* as **2025-08-03**. 
> The newest file inside was actually last edited on  **2024-07-28**. 
> After running `dirstamp -C`, the folder’s *Date modified* is
> reset to **2024-07-28**, matching that newest file.

## Usage

```
dirstamp [PATH] [OPTIONS]
```
If `PATH` is not specified, it defaults to the current directory.

### Options

| Flag          | Description                                |
|---------------|--------------------------------------------|
| `-C`, `--confirm` | Apply timestamp changes (dry-run is default) |
| `-D`, `--show-dates` | Show the human-readable timestamp each folder would be updated to |
| `-V`, `--version` | Show version info                       |
| `-h`, `--help`    | Show usage info                         |


---

## Console Output

Example dry-run output with -D:

```
would update ".\\test\\dirstamp_test\\media\\photos" (from 2025-08-03 08:07:08 UTC to 2025-08-01 08:07:08 UTC, -2.0 days)
would update ".\\test\\dirstamp_test\\projects\\alpha" (from 2025-08-03 08:07:08 UTC to 2025-07-19 08:07:08 UTC, -15.0 days)
would update ".\\test\\dirstamp_test\\projects\\beta" (from 2025-08-03 08:07:08 UTC to 2025-07-31 08:07:08 UTC, -3.0 days)
would update ".\\test\\dirstamp_test\\docs" (from 2025-08-03 08:07:08 UTC to 2025-07-04 08:07:08 UTC, -30.0 days)
would update ".\\test\\dirstamp_test\\media" (from 2025-08-03 08:07:08 UTC to 2025-07-24 08:07:08 UTC, -10.0 days)
would update ".\\test\\dirstamp_test\\" (from 2025-08-14 13:46:06 UTC to 2025-08-03 08:07:08 UTC, -11.2 days)

Note: this was a dry run. Use -C to confirm and apply changes.
```

If no folders need updating:

```
No folder timestamps need updating.
```

## Features

| Feature               | Description                                                                 |
|----------------------|-----------------------------------------------------------------------------|
| **File-first logic**      | Uses the newest file in a directory; if no files exist, it uses the newest subfolder. |
| **Recursive**             | Processes the specified folder and all subfolders.                         |
| **Dry run (default)**     | By default, runs without modifying anything. Shows what *would* change.     |
| **Show dates**     | Use -D or --show-dates to display the human-readable timestamp each folder would be updated to     |
| **Confirm mode**         | Use `-C` or `--confirm` to actually apply the timestamp updates.           |
| **Simple CLI**            | Easy to use, Unix-style tool.                                               |
| **Cross-platform**        | Works on Windows, Linux, and macOS.  
| **Single binary**     | No runtime, no PowerShell execution-policy fuss—just run the EXE (or ELF/Mach-O).           |

---

## Installation

### From [crates.io](https://crates.io/crates/dirstamp):

```sh
cargo install dirstamp
```

---

## Building from Source

Requires Rust 1.70+:

```sh
git clone https://github.com/Crinklebine/dirstamp
cd dirstamp
cargo build --release
```

Run it from the build output:

```sh
./target/release/dirstamp(.exe)
```

---

### Algorithm

For each directory (depth-first traversal):

1. Find the newest modification time (`mtime`) of any file directly inside the directory (not recursively).
2. If at least one file exists **and** the folder’s `mtime` differs by more than 1 second:
    - Update the folder’s `mtime` to match the newest file’s `mtime`.

**Notes:**

- If no files exist, the newest immediate subfolder is used instead.
- Empty directories are left unchanged.
- Only the **modification time (`mtime`)** is updated; creation or birth time remains untouched.
- Changes are applied only with `--confirm` (`-C`). By default, it's a dry run.


---


## Contributing

Pull requests and issues are welcome!

    cargo fmt
    cargo clippy -- -D warnings

Please run the above before submitting.

---

## License

Licensed under the **MIT License** – see [`LICENSE`](LICENSE).

---
<br>
<p align="center">
  <img src="assets/dirstamp.png" alt="dirstamp poster" width="400">
</p>