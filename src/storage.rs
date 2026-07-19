use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use directories::ProjectDirs;

use crate::model::Board;

fn project_dirs() -> io::Result<ProjectDirs> {
    ProjectDirs::from("", "", "cardstack").ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "no config directory for this platform",
        )
    })
}

/// ST-R-002 — the platform config directory itself (parent of `boards/`).
pub fn config_dir() -> io::Result<PathBuf> {
    let dir = project_dirs()?.config_dir().to_path_buf();
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// ST-R-002 — board files live under the platform config directory, in a
/// `boards` subdirectory.
pub fn boards_dir() -> io::Result<PathBuf> {
    let dir = project_dirs()?.config_dir().join("boards");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

const ORDER_FILE: &str = "order.toml";

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct OrderFile {
    order: Vec<String>,
}

/// ST-R-022 — the persisted tab order (board names), or empty if none saved yet.
pub fn load_order(config_dir: &Path) -> Vec<String> {
    fs::read_to_string(config_dir.join(ORDER_FILE))
        .ok()
        .and_then(|s| toml::from_str::<OrderFile>(&s).ok())
        .map(|o| o.order)
        .unwrap_or_default()
}

/// ST-R-022 — persist the current tab order (board names) immediately.
pub fn save_order(config_dir: &Path, names: &[String]) -> io::Result<()> {
    let toml = toml::to_string_pretty(&OrderFile {
        order: names.to_vec(),
    })
    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(config_dir.join(ORDER_FILE), toml)
}

/// ST-R-013 — released by removing its lock file when dropped, so the lock
/// is freed on normal exit, error exit, and (via stack unwinding) panic.
pub struct LockGuard {
    path: PathBuf,
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// ST-R-013 — acquire the single-instance lock in `dir`; fails if another
/// instance already holds it.
pub fn acquire_lock(dir: &Path) -> io::Result<LockGuard> {
    let path = dir.join("cardstack.lock");
    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
    {
        Ok(mut file) => {
            let _ = write!(file, "{}", std::process::id());
            Ok(LockGuard { path })
        }
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!(
                "another instance of cardstack is already running (remove {} if this is stale)",
                path.display()
            ),
        )),
        Err(e) => Err(e),
    }
}

/// ST-R-003 — sanitize a board name into a filesystem-safe filename.
fn sanitize(name: &str) -> String {
    let s: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if s.is_empty() { "board".to_string() } else { s }
}

fn board_path(dir: &Path, name: &str) -> PathBuf {
    dir.join(format!("{}.toml", sanitize(name)))
}

/// ST-R-010, ST-R-011 — write a board's file immediately.
pub fn save(dir: &Path, board: &Board) -> io::Result<()> {
    let toml =
        toml::to_string_pretty(board).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(board_path(dir, &board.name), toml)
}

/// ST-R-014 — remove a board's file from disk.
pub fn delete(dir: &Path, name: &str) -> io::Result<()> {
    let path = board_path(dir, name);
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}

/// ST-R-012 — write the board under its new name and remove the old file.
pub fn rename(dir: &Path, old_name: &str, board: &Board) -> io::Result<()> {
    save(dir, board)?;
    let old_path = board_path(dir, old_name);
    if old_path != board_path(dir, &board.name) {
        let _ = fs::remove_file(old_path);
    }
    Ok(())
}

/// ST-R-020 — load every board file in `dir`; a file that fails to parse is
/// skipped (not returned, not deleted) rather than aborting the load.
pub fn load_all(dir: &Path) -> io::Result<Vec<Board>> {
    let mut boards = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        let Ok(contents) = fs::read_to_string(&path) else {
            continue;
        };
        if let Ok(board) = toml::from_str::<Board>(&contents) {
            boards.push(board);
        }
    }
    Ok(boards)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Status;

    /// ST-R-021
    #[test]
    fn ut_save_load_round_trip_preserves_state() {
        let dir = std::env::temp_dir().join(format!("cardstack-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();

        let mut board = Board::new("Round Trip");
        board.create_category("urgent");
        let id = board.create_task("write tests", Status::Open);
        board.move_status(id, Status::InProgress);
        board.tasks[0].labels.push("backend".to_string());
        board.tasks[0].category = Some("urgent".to_string());

        save(&dir, &board).unwrap();
        let loaded = load_all(&dir).unwrap();

        fs::remove_dir_all(&dir).unwrap();

        assert_eq!(loaded.len(), 1);
        let l = &loaded[0];
        assert_eq!(l.name, board.name);
        assert_eq!(l.categories.len(), 1);
        assert_eq!(l.tasks.len(), 1);
        assert_eq!(l.tasks[0].status, Status::InProgress);
        assert_eq!(l.tasks[0].labels, vec!["backend".to_string()]);
        assert_eq!(l.tasks[0].category.as_deref(), Some("urgent"));
    }

    /// ST-R-020
    #[test]
    fn ut_load_all_skips_unparseable_file() {
        let dir = std::env::temp_dir().join(format!("cardstack-test-bad-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("broken.toml"), "not valid toml {{{").unwrap();

        let loaded = load_all(&dir).unwrap();
        fs::remove_dir_all(&dir).unwrap();

        assert!(loaded.is_empty());
    }
}
