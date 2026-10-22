//! The Design session in the browser (web build, issue #245).
//!
//! A thin forwarder: each method calls `ovcd_shell::AppState` (or `export`) and returns the same
//! JSON the desktop app's commands return. It holds no calculation, validation or default logic.
//! Errors are thrown to JS as plain strings.

mod export;

use ovcd_shell::AppState;
use serde_json::Value;
use std::path::Path;
use wasm_bindgen::prelude::*;

type Js<T> = Result<T, JsValue>;

fn err(message: String) -> JsValue {
    JsValue::from_str(&message)
}

fn ok<T>(result: Result<T, String>) -> Js<T> {
    result.map_err(err)
}

/// The engine version, as written in a `.vdesign` file.
#[wasm_bindgen]
pub fn engine_version() -> String {
    ovcd_format::vdesign::ENGINE_VERSION.to_string()
}

/// One open Design, as in the desktop app.
#[wasm_bindgen]
pub struct Session {
    state: AppState,
}

impl Session {
    /// Edit, then return the snapshot JSON.
    fn edit(&mut self, f: impl FnOnce(&mut AppState) -> Result<(), String>) -> Js<String> {
        ok(f(&mut self.state))?;
        Ok(self.state.snapshot().to_string())
    }
}

#[wasm_bindgen]
impl Session {
    /// The starter Design.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Js<Session> {
        Ok(Session {
            state: ok(AppState::new_design())?,
        })
    }

    pub fn snapshot(&self) -> String {
        self.state.snapshot().to_string()
    }

    pub fn input_schema(&self) -> String {
        self.state.session().input_schema().to_string()
    }

    pub fn evaluate(&self) -> String {
        self.state.evaluate_json().to_string()
    }

    pub fn new_design(&mut self) -> Js<String> {
        self.state = ok(AppState::new_design())?;
        Ok(self.state.snapshot().to_string())
    }

    pub fn open_text(&mut self, name: &str, text: &str) -> Js<String> {
        self.state = ok(AppState::open(Path::new(name), text))?;
        Ok(self.state.snapshot().to_string())
    }

    pub fn save_text(&self) -> Js<String> {
        ok(self.state.save_text())
    }

    pub fn saved(&mut self, name: &str) -> String {
        self.state.saved(Path::new(name));
        self.state.snapshot().to_string()
    }

    pub fn is_dirty(&self) -> bool {
        self.state.is_dirty()
    }

    pub fn set_input(&mut self, option: &str, name: &str, value_json: &str) -> Js<String> {
        let value: Value = serde_json::from_str(value_json)
            .map_err(|e| err(format!("the value is not JSON: {e}")))?;
        self.edit(|s| s.set_input(option, name, value))
    }

    /// Issue #451: import an offsets CSV into an Option.
    pub fn import_offsets(&mut self, option: &str, csv: &str) -> Js<String> {
        self.edit(|s| s.import_offsets(option, csv))
    }

    pub fn reset_input(&mut self, option: &str, name: &str) -> Js<String> {
        self.edit(|s| s.reset_input(option, name))
    }

    pub fn set_active_option(&mut self, option: &str) -> Js<String> {
        self.edit(|s| s.set_active_option(option))
    }

    pub fn set_formation(
        &mut self,
        option: &str,
        kind: &str,
        abreast: Option<u32>,
        in_line: Option<u32>,
    ) -> Js<String> {
        self.edit(|s| s.set_formation(option, kind, abreast.map(u64::from), in_line.map(u64::from)))
    }

    pub fn add_option(&mut self) -> Js<String> {
        self.edit(|s| s.add_option())
    }

    pub fn duplicate_option(&mut self, option: &str) -> Js<String> {
        self.edit(|s| s.duplicate_option(option))
    }

    pub fn rename_option(&mut self, option: &str, new_name: &str) -> Js<String> {
        self.edit(|s| s.rename_option(option, new_name))
    }

    pub fn delete_option(&mut self, option: &str) -> Js<String> {
        self.edit(|s| s.delete_option(option))
    }

    pub fn load_profile_text(&mut self, text: &str) -> Js<String> {
        self.edit(|s| s.load_profile(text))
    }

    pub fn use_built_in_profile(&mut self) -> Js<String> {
        self.edit(|s| s.use_built_in_profile())
    }

    /// One hull view (`side`, `plan` or `midship`) as SVG with the font embedded.
    pub fn drawing_svg(&self, option: &str, view: &str) -> Js<String> {
        ok(export::svg(&self.state, option, view))
    }

    pub fn drawing_pdf(&self, option: &str, view: &str) -> Js<Vec<u8>> {
        ok(export::pdf(&self.state, option, view))
    }

    pub fn drawing_dxf(&self, option: &str, view: &str) -> Js<Vec<u8>> {
        ok(export::dxf(&self.state, option, view))
    }
}
