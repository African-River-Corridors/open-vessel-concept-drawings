//! The first-run notice: shown once per user. The app remembers that it was accepted with a small
//! file in its config folder. The file is the only state; the text of the notice is in the front end.

use std::path::{Path, PathBuf};

const MARKER: &str = "first-run-notice-accepted.txt";

/// The folder that holds the marker. A debug build reads `OVCD_TEST_CONFIG_DIR` first, so the smoke
/// tests never touch the real config folder. Release builds have no such hook.
#[cfg(debug_assertions)]
pub fn test_config_dir() -> Option<PathBuf> {
    std::env::var_os("OVCD_TEST_CONFIG_DIR").map(PathBuf::from)
}

#[cfg(not(debug_assertions))]
pub fn test_config_dir() -> Option<PathBuf> {
    None
}

/// Has the notice been accepted in this folder?
pub fn seen(dir: &Path) -> bool {
    dir.join(MARKER).is_file()
}

/// Remember that the notice was accepted. The file holds the app version that showed it.
pub fn accept(dir: &Path, version: &str) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    let path = dir.join(MARKER);
    std::fs::write(&path, format!("accepted in OVCD {version}\n"))
        .map_err(|e| format!("cannot write {}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_notice_is_shown_until_it_is_accepted_and_then_never_again() {
        let dir = std::env::temp_dir().join(format!("ovcd-notice-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        assert!(!seen(&dir));
        accept(&dir, "0.2.1").unwrap();
        assert!(seen(&dir));
        // accepting again changes nothing and is not an error
        accept(&dir, "0.2.1").unwrap();
        assert!(seen(&dir));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
