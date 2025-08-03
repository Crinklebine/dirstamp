# dirstamp

`dirstamp` is a tiny cross-platform command-line utility that **copies the
_last-modified_ time (`mtime`) of the newest *file* in each directory (folder) onto the
directory itself**.

It’s perfect after migrations, restores, or bulk copies where all your folders
suddenly say “today” even though the files inside span years. Seeing an accurate
date of your folders is useful to view data history by folder.

> **Example**  
> You copy a folder on **2025-08-03**, so Windows shows its *Date modified* as  
> **2025-08-03**. The newest file inside was actually last edited on  
> **2024-07-28**. After running `dirstamp`, the folder’s *Date modified* is
> reset to **2024-07-28**, matching that newest file.

---

## Features

| Feature               | Details                                                                                     |
|-----------------------|---------------------------------------------------------------------------------------------|
| **Deep-first walk**   | Children processed before parents so nested folders finish first.                           |
| **Child-first logic** | Uses the newest **file** in a directory; if no files exist, falls back to the newest **immediate sub-folder**.      |
| **Skip when current** | Touches a folder only if its timestamp differs by more than one second.                     |
| **Cross-platform**    | Builds on Windows, Linux, and macOS (safe Rust, no platform-specific code paths).           |
| **Single binary**     | No runtime, no PowerShell execution-policy fuss—just run the EXE (or ELF/Mach-O).           |

---

## Quick start (Windows)

    # after you build or download dirstamp.exe
    put dirstamp.exe in your path to have available in any cmd or powershell window
    dirstamp.exe "D:\MigratedDocs"
    or simply dirstamp.exe within the current directory

Console output:

```text
updated ".\projects\beta"
updated ".\projects\alpha"
updated ".\projects"
updated ".\media\photos"
updated ".\media"
updated ".\docs"
updated "."
```
---

## Building from source

1. **Install Rust** – <https://rustup.rs>  
2. **Clone & build**

       git clone https://github.com/Crinklebine/dirstamp.git
       cd dirstamp
       cargo build --release

3. The binary is in `target/release/dirstamp[.exe]`.

### Cross-compile a Windows build from Linux/macOS

       rustup target add x86_64-pc-windows-gnu
       cargo build --release --target x86_64-pc-windows-gnu
       # → target/x86_64-pc-windows-gnu/release/dirstamp.exe

---

## Usage

    dirstamp [PATH]

*`PATH`* — root directory to process (default: current directory)

### Algorithm (simple version)

    for each directory (deep-first):
        newest_file_mtime = newest file directly inside
        if such a file exists and folder.mtime differs by >1 s:
            set folder.mtime = newest_file_mtime

*Only **mtime** is updated; creation/birth time stays unchanged.*

---

## Road-map / ideas

* `--dry-run` preview mode  
* `--oldest` flag (use earliest file instead)   
* Continuous Integration build & release workflow  
* Publish on crates.io  

---

## Contributing

Pull requests and issues are welcome!

    cargo fmt
    cargo clippy -- -D warnings

Please run the above before submitting.

---

## License

Licensed under the **MIT License** – see [`LICENSE`](LICENSE).

