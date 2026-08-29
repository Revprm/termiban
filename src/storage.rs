//! storage.rs — Cross-platform persistence for Termiban
//!
//! Responsibilities:
//! * `data_path()` — resolve where `board.json` lives on Linux / macOS / Windows
//! * `load_board()` / `save_board()` — JSON serde with graceful fallback

use crate::board::Board;
use std::{fs, path::PathBuf};

// ---------------------------------------------------------------------------
// Path resolution — self-hosted, no external `dirs` crate needed
// ---------------------------------------------------------------------------

/// Resolve the JSON file path.
///
/// Priority:
/// 1. `$TERMIBAN_DATA_PATH` or `$TERMIBAN_DATA` (explicit override, portable mode)
/// 2. Windows: `%APPDATA%\termiban\board.json`
/// 3. `$XDG_DATA_HOME/termiban/board.json`
/// 4. `~/.local/share/termiban/board.json` (Linux/macOS default)
/// 5. `%USERPROFILE%\AppData\Roaming\termiban\board.json` (Windows fallback)
/// 6. `./board.json` (dev / fallback)
pub fn data_path() -> PathBuf {
    if let Ok(p) = std::env::var("TERMIBAN_DATA_PATH") {
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    if let Ok(p) = std::env::var("TERMIBAN_DATA") {
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }

    #[cfg(windows)]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            if !appdata.is_empty() {
                return PathBuf::from(appdata).join("termiban").join("board.json");
            }
        }
    }

    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        if !xdg.is_empty() {
            return PathBuf::from(xdg).join("termiban").join("board.json");
        }
    }

    if let Ok(home) = std::env::var("HOME") {
        if !home.is_empty() {
            return PathBuf::from(home)
                .join(".local")
                .join("share")
                .join("termiban")
                .join("board.json");
        }
    }

    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        if !userprofile.is_empty() {
            return PathBuf::from(userprofile)
                .join("AppData")
                .join("Roaming")
                .join("termiban")
                .join("board.json");
        }
    }

    PathBuf::from("board.json")
}

// ---------------------------------------------------------------------------
// Load / Save helpers
// ---------------------------------------------------------------------------

/// Load board from `data_path()`, falling back to `Board::default()` on any error.
///
/// Errors are reported to stderr but never panic — the TUI can still start.
pub fn load_board() -> Board {
    let path = data_path();
    if path.exists() {
        match fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str::<Board>(&content) {
                Ok(board) => return board,
                Err(e) => eprintln!(
                    "warn: failed to parse {}: {e} — using defaults",
                    path.display()
                ),
            },
            Err(e) => eprintln!("warn: failed to read {}: {e}", path.display()),
        }
    }
    Board::default()
}

/// Persist board to `data_path()` as pretty JSON.
///
/// Creates parent directories as needed. Errors are logged to stderr.
pub fn save_board(board: &Board) {
    let path = data_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    match serde_json::to_string_pretty(board) {
        Ok(json) => {
            if let Err(e) = fs::write(&path, json) {
                eprintln!("failed to write {}: {e}", path.display());
            }
        }
        Err(e) => eprintln!("failed to serialize board: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{Board, Task};

    #[test]
    fn data_path_respects_env_override() {
        // SAFETY: single-threaded test, env vars are process-global
        unsafe { std::env::set_var("TERMIBAN_DATA_PATH", "/tmp/custom_board.json") };
        assert_eq!(data_path(), PathBuf::from("/tmp/custom_board.json"));
        unsafe { std::env::remove_var("TERMIBAN_DATA_PATH") };
        // Should now fall back to HOME/XDG logic, not panic
        let _ = data_path();
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = std::env::temp_dir().join(format!("termiban_test_{}", std::process::id()));
        let path = dir.join("board.json");
        unsafe { std::env::set_var("TERMIBAN_DATA_PATH", &path) };

        let mut board = Board::default();
        board.next_id = 99;
        board.columns[0].tasks.push(Task {
            id: 99,
            title: "Roundtrip".into(),
            description: "ok".into(),
            deadline: Some("2026-12-31".into()),
            priority: crate::board::Priority::High,
            tags: vec!["test".into()],
            created_at: Some("2026-01-01".into()),
        });

        save_board(&board);
        let loaded = load_board();
        assert_eq!(loaded.next_id, 99);
        assert!(loaded.columns[0].tasks.iter().any(|t| t.title == "Roundtrip"));

        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir(&dir);
        unsafe { std::env::remove_var("TERMIBAN_DATA_PATH") };
    }
}
