# DiskSift 🔍

A modern, fast CLI tool to analyze disk usage and find the largest files and directories on your system.
Written in Rust.

## Features

- 🚀 **Fast**: Uses parallel directory traversal (via `walkdir`).
- 📂 **Smart calculation**: Aggregates directory sizes to find "heavy" folders.
- 🎨 **Modern UI**: Beautiful colored output and progress bars.
- 🔍 **Flexible**:
  - Filter by minimum size (e.g., `--min-size 100MB`)
  - Limit recursion depth for display (`-d` / `--depth`)
  - Limit top N results (`-n` / `--limit`)

## Installation

### From Crates.io
```bash
cargo install disksift
```

### From Source
```bash
git clone https://github.com/RUNFUNRUN/disksift.git
cd disksift
cargo install --path .
```

## Usage

```bash
# Scan current directory
disksift

# Scan a specific path
disksift /path/to/directory

# Show top 20 items
disksift -n 20

# Find items larger than 1GB
disksift --min-size 1GB

# Limit depth to 2 levels
disksift -d 2
```

## License

MIT
