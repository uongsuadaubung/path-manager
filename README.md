# 🚀 Path Manager (path-mgr)

A blazingly fast, lightweight Terminal User Interface (TUI) and CLI tool for managing Windows User & System PATH environment variables, built with minimal dependencies (`ratatui` & `crossterm`).

## ✨ Features

### 🖥️ Terminal User Interface (TUI)
- **Extreme Simplicity & Performance:** Only 2 runtime dependencies (`ratatui` & `crossterm`), no heavy GUI or web engines.
- **Visual PATH Management:** Instant switching between **User PATH** (`[1]`) and **System PATH** (`[2]`).
- **Real-Time Diagnostics:**
  - `✓ OK`: Directory exists and contains executable binaries.
  - `✖ BROKEN`: Directory does not exist on disk.
  - `▲ NO-EXEC`: Directory exists but contains no executable files (.exe, .bat, .cmd, etc.).
  - `🛡️ SHADOWED`: User PATH entry that is already present in System PATH.
- **Interactive Reordering:** Move entries up or down (`K` / `J` or `Ctrl+Up` / `Ctrl+Down`) to adjust priority.
- **Search & Filter:** Press `/` to search and filter paths instantly.
- **Quick Modals:** Add new paths (`a`), edit paths (`e`), confirm deletions (`d`), view help (`?`).
- **Deduplication:** One-key cleanup (`c`) to remove duplicates and redundant shadowed entries.
- **Live Registry Sync:** Automatically refreshes the UI when Registry changes are detected from other apps.
- **Zero-Delay Notification:** Broadcasts `WM_SETTINGCHANGE` on every modification so terminal and apps update without a reboot.

### ⌨️ Command-Line Interface (CLI)
- `path-mgr list [--system]`: List all PATH entries.
- `path-mgr add <PATH> [--system]`: Add a new directory.
- `path-mgr remove <INDEX> [--system]`: Remove a directory by 1-based index.
- `path-mgr set <INDEX> <PATH> [--system]`: Update a directory entry.
- `path-mgr dedupe [--system]`: Deduplicate and clean paths.
- `path-mgr backup [-f file] [--system]`: Backup current PATH list to text file.
- `path-mgr restore [-f file] [-o] [--system]`: Restore or merge PATH entries from backup.

## 🛠️ Build & Installation

### Requirements
- Windows 10/11
- [Rust](https://rustup.rs/) (2024 edition)

### Build from source
```bash
cargo build --release
```

The resulting binary in `target/release/path-mgr.exe` is completely self-contained, tiny, and starts in less than 5ms.
