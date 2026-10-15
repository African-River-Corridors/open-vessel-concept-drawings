//! What the shell holds besides the Design session: the file path, the unsaved flag, the active
//! Option (for the window title), and the hash warnings and the notes of a changed old file from the
//! last open.
//!
//! This module has no Tauri types, so it is tested like any other Rust. It holds no calculation,
//! validation or default logic: every answer comes from [`DesignSession`].

use crate::views;
use ovcd_format::{DesignSession, HashStatus, Profile, RunOutcome};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

pub struct AppState {
    session: DesignSession,
    path: Option<PathBuf>,
    dirty: bool,
    active_option: String,
    /// Names of the Options whose stored results hash differs from this engine's, at open.
    hash_changed: Vec<String>,
    /// What changed when an old (version 1) file was read as the current inputs, at open.
    migration_notes: Vec<String>,
}

fn first_option(session: &DesignSession) -> String {
    session
        .design()
        .options
        .first()
        .map(|o| o.name.clone())
        .unwrap_or_default()
}

impl AppState {
    /// A new Design: the starter Design `ovcd new` writes.
    pub fn new_design() -> Result<AppState, String> {
        let session = DesignSession::new_design(Profile::built_in())?;
        let active_option = first_option(&session);
        Ok(AppState {
            session,
            path: None,
            dirty: false,
            active_option,
            hash_changed: Vec::new(),
            migration_notes: Vec::new(),
        })
    }

    /// Open a `.vdesign` text that was read from `path`.
    pub fn open(path: &Path, text: &str) -> Result<AppState, String> {
        let (session, report) = DesignSession::open(text, Profile::built_in())?;
        let active_option = first_option(&session);
        let hash_changed = report
            .hash_status
            .into_iter()
            .filter(|(_, status)| matches!(status, HashStatus::Differs { .. }))
            .map(|(name, _)| name)
            .collect();
        Ok(AppState {
            session,
            path: Some(path.to_path_buf()),
            dirty: false,
            active_option,
            hash_changed,
            migration_notes: report.report.migration_notes,
        })
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn session(&self) -> &DesignSession {
        &self.session
    }

    /// The evaluation as JSON for the front end: the session's own, with the hull views (SVG) added
    /// to the outcome of each Option that was computed.
    pub fn evaluate_json(&self) -> Value {
        let evaluation = self.session.evaluate();
        let mut json = evaluation.to_json();
        for (i, option) in evaluation.options.iter().enumerate() {
            if let Some(outcome @ RunOutcome::Results { .. }) = &option.outcome {
                json["options"][i]["outcome"]["views"] = views::hull_views(outcome);
            }
        }
        json
    }

    pub fn set_input(&mut self, option: &str, name: &str, value: Value) -> Result<(), String> {
        self.session.set_input(option, name, value)?;
        self.dirty = true;
        Ok(())
    }

    pub fn reset_input(&mut self, option: &str, name: &str) -> Result<(), String> {
        self.session.reset_input(option, name)?;
        self.dirty = true;
        Ok(())
    }

    pub fn set_formation(
        &mut self,
        option: &str,
        kind: &str,
        abreast: Option<u64>,
        in_line: Option<u64>,
    ) -> Result<(), String> {
        self.session.set_formation(option, kind, abreast, in_line)?;
        self.dirty = true;
        Ok(())
    }

    /// Add an Option and make it the active one.
    pub fn add_option(&mut self) -> Result<(), String> {
        self.active_option = self.session.add_option(None)?;
        self.dirty = true;
        Ok(())
    }

    /// Copy an Option and make the copy the active one.
    pub fn duplicate_option(&mut self, option: &str) -> Result<(), String> {
        self.active_option = self.session.duplicate_option(option, None)?;
        self.dirty = true;
        Ok(())
    }

    pub fn rename_option(&mut self, option: &str, new_name: &str) -> Result<(), String> {
        self.session.rename_option(option, new_name)?;
        if self.active_option == option {
            self.active_option = new_name.trim().to_string();
        }
        self.dirty = true;
        Ok(())
    }

    /// Delete an Option. The session refuses to delete the last one.
    pub fn delete_option(&mut self, option: &str) -> Result<(), String> {
        self.session.delete_option(option)?;
        if self.active_option == option {
            self.active_option = first_option(&self.session);
        }
        self.dirty = true;
        Ok(())
    }

    /// Swap in a Defaults Profile from its text. The session refuses one that loosens a hard range.
    pub fn load_profile(&mut self, text: &str) -> Result<(), String> {
        self.session.load_profile(text)?;
        self.dirty = true;
        Ok(())
    }

    pub fn use_built_in_profile(&mut self) -> Result<(), String> {
        self.load_profile(Profile::built_in_text())
    }

    pub fn set_active_option(&mut self, option: &str) -> Result<(), String> {
        if self
            .session
            .design()
            .options
            .iter()
            .any(|o| o.name == option)
        {
            self.active_option = option.to_string();
            Ok(())
        } else {
            Err(format!("no Option named '{option}'"))
        }
    }

    /// The `.vdesign` text to write. Refused while the Design has a hard error.
    pub fn save_text(&self) -> Result<String, String> {
        self.session.save()
    }

    /// Record that the Design was written to `path`.
    pub fn saved(&mut self, path: &Path) {
        self.path = Some(path.to_path_buf());
        self.dirty = false;
        // A saved file carries fresh hashes and the current inputs, so the open-time warnings are over.
        self.hash_changed.clear();
        self.migration_notes.clear();
    }

    pub fn file_name(&self) -> String {
        self.path
            .as_deref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Untitled".into())
    }

    /// The window title: unsaved marker, file name, active Option.
    pub fn title(&self) -> String {
        let marker = if self.dirty { "\u{25cf} " } else { "" };
        format!(
            "{marker}{} \u{2014} {} \u{2014} OVCD",
            self.file_name(),
            self.active_option
        )
    }

    /// What the front end shows. The Design is the session's own `.vdesign` text, parsed.
    pub fn snapshot(&self) -> Value {
        let design: Value =
            serde_json::from_str(&self.session.design().to_text()).unwrap_or(Value::Null);
        json!({
            "engine_version": ovcd_format::vdesign::ENGINE_VERSION,
            "profile": self.session.profile().name(),
            "design": design,
            "file_name": self.file_name(),
            "path": self.path.as_ref().map(|p| p.to_string_lossy().into_owned()),
            "dirty": self.dirty,
            "title": self.title(),
            "active_option": self.active_option,
            "hash_changed": self.hash_changed,
            "migration_notes": self.migration_notes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ovcd_format::{RunOutcome, check, new_design};

    fn starter_text() -> String {
        new_design(&Profile::built_in()).unwrap().to_text()
    }

    #[test]
    fn new_is_the_starter_design_ovcd_new_writes() {
        let state = AppState::new_design().unwrap();
        let expected: Value = serde_json::from_str(&starter_text()).unwrap();
        assert_eq!(state.snapshot()["design"], expected);
        assert_eq!(state.file_name(), "Untitled");
        assert!(!state.is_dirty());
    }

    #[test]
    fn title_shows_file_option_and_the_unsaved_marker() {
        let mut state = AppState::open(Path::new("/x/barge.vdesign"), &starter_text()).unwrap();
        assert_eq!(
            state.title(),
            "barge.vdesign \u{2014} Option 1 \u{2014} OVCD"
        );
        state.set_input("Option 1", "beam_m", json!(12.0)).unwrap();
        assert_eq!(
            state.title(),
            "\u{25cf} barge.vdesign \u{2014} Option 1 \u{2014} OVCD"
        );
        state.saved(Path::new("/x/other.vdesign"));
        assert_eq!(
            state.title(),
            "other.vdesign \u{2014} Option 1 \u{2014} OVCD"
        );
    }

    #[test]
    fn an_edit_marks_the_design_unsaved_and_a_refused_edit_does_not() {
        let mut state = AppState::new_design().unwrap();
        assert!(
            state
                .set_input("Option 1", "no_such_input", json!(1))
                .is_err()
        );
        assert!(state.set_input("No such", "beam_m", json!(1)).is_err());
        assert!(!state.is_dirty());
        state.reset_input("Option 1", "max_speed_kn").unwrap();
        assert!(state.is_dirty());
    }

    #[test]
    fn active_option_must_exist() {
        let mut state = AppState::new_design().unwrap();
        assert!(state.set_active_option("Option 9").is_err());
        assert_eq!(state.snapshot()["active_option"], "Option 1");
    }

    #[test]
    fn a_saved_file_passes_validate_and_reopens_unchanged() {
        let mut state = AppState::new_design().unwrap();
        state.set_input("Option 1", "beam_m", json!(12.5)).unwrap();
        let text = state.save_text().unwrap();
        let checked = check(&text, &Profile::built_in());
        assert!(checked.report.is_ok(), "{:?}", checked.report.errors);
        let reopened = AppState::open(Path::new("a.vdesign"), &text).unwrap();
        assert_eq!(reopened.save_text().unwrap(), text);
        assert!(
            reopened.snapshot()["hash_changed"]
                .as_array()
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn a_saved_file_carries_each_options_results_hash() {
        let state = AppState::new_design().unwrap();
        let text = state.save_text().unwrap();
        let v: Value = serde_json::from_str(&text).unwrap();
        let hash = v["options"][0]["results_hash"].as_str().unwrap();
        let eval = state.session().evaluate();
        let Some(RunOutcome::Results { hash: computed, .. }) = &eval.options[0].outcome else {
            panic!("the starter Design should compute");
        };
        assert_eq!(hash, computed);
    }

    #[test]
    fn opening_a_file_whose_results_hash_changed_names_the_options_affected() {
        let mut v: Value = serde_json::from_str(&starter_text()).unwrap();
        v["options"][0]["results_hash"] = json!("0000000000000000deadbeef");
        let state = AppState::open(Path::new("old.vdesign"), &v.to_string()).unwrap();
        assert_eq!(state.snapshot()["hash_changed"], json!(["Option 1"]));
        // Saving writes fresh hashes, which ends the warning.
        let mut state = state;
        state.saved(Path::new("old.vdesign"));
        assert_eq!(state.snapshot()["hash_changed"], json!([]));
    }

    #[test]
    fn opening_a_version_1_adn_g_file_shows_what_changed_until_it_is_saved() {
        // Issue #175: a version 1 file that names ADN_G opens as IGC_2G, and the window says so.
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../crates/ovcd-format/tests/legacy/v1_adn_g_self_propelled.vdesign"
        );
        let text = std::fs::read_to_string(path).unwrap();
        let mut state = AppState::open(Path::new("old.vdesign"), &text).unwrap();
        let notes = state.snapshot()["migration_notes"].clone();
        let notes = notes.as_array().unwrap();
        assert_eq!(notes.len(), 3, "{notes:?}");
        assert!(
            notes[0]
                .as_str()
                .unwrap()
                .contains("ADN_G is no longer offered")
        );
        assert!(
            notes[1]
                .as_str()
                .unwrap()
                .contains("`adn_tank_cap_m3` is no longer an input")
        );
        let ruleset = &state.snapshot()["design"]["options"][0]["inputs"]["ruleset"];
        assert_eq!(ruleset["value"], "IGC_2G");
        // The saved file is the current format and needs no notes.
        state.saved(Path::new("old.vdesign"));
        assert_eq!(state.snapshot()["migration_notes"], json!([]));
        let saved: Value = serde_json::from_str(&state.save_text().unwrap()).unwrap();
        assert_eq!(saved["format_version"], 2);
        // A file that needs no change has none.
        let fresh = AppState::open(Path::new("a.vdesign"), &starter_text()).unwrap();
        assert_eq!(fresh.snapshot()["migration_notes"], json!([]));
    }

    #[test]
    fn option_commands_keep_the_active_option_in_step() {
        let mut state = AppState::new_design().unwrap();
        state.add_option().unwrap();
        assert_eq!(state.snapshot()["active_option"], "Option 2");
        state.rename_option("Option 2", "  Wide ").unwrap();
        assert_eq!(state.snapshot()["active_option"], "Wide");
        state.duplicate_option("Wide").unwrap();
        assert_eq!(state.snapshot()["active_option"], "Wide copy");
        state.delete_option("Wide copy").unwrap();
        assert_eq!(state.snapshot()["active_option"], "Option 1");
        state.delete_option("Wide").unwrap();
        assert!(state.delete_option("Option 1").is_err());
        assert!(state.rename_option("Option 1", "").is_err());
    }

    #[test]
    fn a_profile_that_loosens_a_hard_range_is_refused_and_the_old_one_stays() {
        let mut v: Value = serde_json::from_str(Profile::built_in_text()).unwrap();
        v["name"] = json!("loose");
        let text = serde_json::to_string(&v).unwrap();
        let mut state = AppState::new_design().unwrap();
        state.load_profile(&text).unwrap();
        assert_eq!(state.snapshot()["profile"], "loose");
        state.use_built_in_profile().unwrap();
        assert_eq!(state.snapshot()["profile"], "built-in");
        v["inputs"]["beam_m"]["hard"]["le"] = json!(5000);
        let err = state.load_profile(&v.to_string()).unwrap_err();
        assert!(err.contains("loosens the hard range of `beam_m`"), "{err}");
        assert!(state.load_profile("not json").is_err());
        assert_eq!(state.snapshot()["profile"], "built-in");
    }

    #[test]
    fn the_evaluation_carries_the_three_hull_views_of_a_computed_option() {
        let state = AppState::new_design().unwrap();
        let eval = state.evaluate_json();
        let outcome = &eval["options"][0]["outcome"];
        assert_eq!(outcome["status"], "results");
        for key in ["side", "plan", "midship"] {
            assert!(
                outcome["views"][key].as_str().unwrap().contains("<svg"),
                "{key}"
            );
        }
        // the rest of the evaluation is the session's own
        assert_eq!(eval["options"][0]["name"], "Option 1");
        assert!(outcome["results"]["dims"].is_object());
    }

    #[test]
    fn an_option_with_a_hard_error_gets_no_views() {
        let mut state = AppState::new_design().unwrap();
        state.set_input("Option 1", "beam_m", json!(-1.0)).unwrap();
        let eval = state.evaluate_json();
        assert_eq!(eval["options"][0]["outcome"]["status"], "not_run");
        assert!(eval["options"][0]["outcome"].get("views").is_none());
    }

    #[test]
    fn a_file_that_is_not_a_design_is_refused() {
        assert!(AppState::open(Path::new("x.vdesign"), "not json").is_err());
    }
}
