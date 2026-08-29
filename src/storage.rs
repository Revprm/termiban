#[cfg(target_os = "windows")]
compile_error!("Termiban is Unix-only (Linux and macOS) — Windows is not supported");

use crate::board::Board;
use std::{fs, path::PathBuf};

/// Resolve data path — Unix-only (Linux and macOS)
///
/// Priority:
/// 1. $TERMIBAN_DATA_PATH or $TERMIBAN_DATA (portable override)
/// 2. $XDG_DATA_HOME/termiban/board.json (XDG, Linux + macOS if set)
/// 3. ~/Library/Application Support/termiban/board.json (macOS native)
/// 4. ~/.local/share/termiban/board.json (Linux + macOS fallback)
/// 5. ./board.json (dev fallback)
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

    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        if !xdg.is_empty() {
            return PathBuf::from(xdg).join("termiban").join("board.json");
        }
    }

    // macOS native location — preferred on macOS if HOME is set
    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            if !home.is_empty() {
                return PathBuf::from(home)
                    .join("Library")
                    .join("Application Support")
                    .join("termiban")
                    .join("board.json");
            }
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

    PathBuf::from("board.json")
}

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
        unsafe { std::env::set_var("TERMIBAN_DATA_PATH", "/tmp/custom_board.json") };
        assert_eq!(data_path(), PathBuf::from("/tmp/custom_board.json"));
        unsafe { std::env::remove_var("TERMIBAN_DATA_PATH") };
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
