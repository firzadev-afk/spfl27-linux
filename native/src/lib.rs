//! SPFL27 library — pure logic + dependency detection.
//! Separated from the UI (src/ui.rs) so it can be tested without opening a window.

pub mod core;
pub mod deps;

use std::path::PathBuf;

/// Path to the Lutris installer YAML (walks up from the executable).
pub fn yaml_path() -> PathBuf {
    let exe = std::env::current_exe().ok();
    let base = exe
        .and_then(|p| p.parent().map(|x| x.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    for up in [2usize, 3, 4, 5] {
        if let Some(c) = base.ancestors().nth(up) {
            let y = c.join("lutris/sp-football-life-2027.yaml");
            if y.is_file() {
                return y;
            }
        }
    }
    base.join("lutris/sp-football-life-2027.yaml")
}

/// Path to the docs/ folder (walks up from the executable, same pattern).
/// None if not found — the UI must handle it, not panic.
pub fn docs_dir() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok();
    let base = exe
        .and_then(|p| p.parent().map(|x| x.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    for up in [2usize, 3, 4, 5] {
        if let Some(c) = base.ancestors().nth(up) {
            let d = c.join("docs");
            if d.join("01-installation.md").is_file() {
                return Some(d);
            }
        }
    }
    None
}
