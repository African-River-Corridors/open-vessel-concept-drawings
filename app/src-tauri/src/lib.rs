//! The OVCD desktop app shell.
//!
//! Thin by design (spec 0002): each command forwards to the Design session in `ovcd-format`. The
//! shell owns the native open and save dialogs, the unsaved-changes guard and the window title,
//! and nothing else. It holds no calculation, validation or default logic.

mod notice;

use ovcd_shell::AppState;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State, Window, WindowEvent};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};

type Shared = Mutex<AppState>;
type Reply = Result<serde_json::Value, String>;

fn lock(state: &Shared) -> Result<std::sync::MutexGuard<'_, AppState>, String> {
    state
        .lock()
        .map_err(|_| "the app state is unusable".to_string())
}

/// Set the window title from the state and return the snapshot for the front end.
fn finish(window: &Window, app: &AppState) -> Reply {
    window.set_title(&app.title()).map_err(|e| e.to_string())?;
    Ok(app.snapshot())
}

/// Smoke-test hook, debug builds only. A native dialog cannot be driven by WebDriver, so when
/// `OVCD_TEST_DIALOG_DIR` is set, each dialog is answered from a file in that folder instead:
/// `open.txt`, `save.txt` and `profile.txt` hold the path to pick (no file means the user
/// cancelled), and `discard.txt` holds `no` to refuse a discard (anything else, or no file,
/// agrees). Release builds have no such hook.
#[cfg(debug_assertions)]
fn test_answer(file: &str) -> Option<Option<String>> {
    let dir = std::env::var_os("OVCD_TEST_DIALOG_DIR")?;
    let text = std::fs::read_to_string(Path::new(&dir).join(file)).ok();
    Some(text.map(|t| t.trim().to_string()).filter(|t| !t.is_empty()))
}

#[cfg(not(debug_assertions))]
fn test_answer(_file: &str) -> Option<Option<String>> {
    None
}

/// Ask whether to discard unsaved changes. `true` means go ahead.
fn ask_discard(app: &AppHandle, text: &str) -> bool {
    if let Some(answer) = test_answer("discard.txt") {
        return answer.as_deref() != Some("no");
    }
    app.dialog()
        .message(text)
        .title("Unsaved changes")
        .buttons(MessageDialogButtons::OkCancelCustom(
            "Discard".into(),
            "Cancel".into(),
        ))
        .blocking_show()
}

/// Ask before the unsaved Design is lost. `true` means go ahead.
fn confirm_discard(app: &AppHandle, state: &Shared) -> Result<bool, String> {
    if !lock(state)?.is_dirty() {
        return Ok(true);
    }
    Ok(ask_discard(
        app,
        "The Design has unsaved changes. Discard them?",
    ))
}

/// Ask for a file to read. `None` when the user cancels.
fn pick_file(
    app: &AppHandle,
    test_file: &str,
    filter: &str,
    extensions: &[&str],
) -> Result<Option<PathBuf>, String> {
    if let Some(answer) = test_answer(test_file) {
        return Ok(answer.map(PathBuf::from));
    }
    match app
        .dialog()
        .file()
        .add_filter(filter, extensions)
        .blocking_pick_file()
    {
        Some(picked) => Ok(Some(picked.into_path().map_err(|e| e.to_string())?)),
        None => Ok(None),
    }
}

/// Ask where to save. `None` when the user cancels.
fn pick_save(app: &AppHandle) -> Result<Option<PathBuf>, String> {
    let picked = match test_answer("save.txt") {
        Some(answer) => answer.map(PathBuf::from),
        None => match app
            .dialog()
            .file()
            .add_filter("OVCD Design", &["vdesign"])
            .set_file_name("design.vdesign")
            .blocking_save_file()
        {
            Some(p) => Some(p.into_path().map_err(|e| e.to_string())?),
            None => None,
        },
    };
    Ok(picked.map(|mut p| {
        if p.extension().is_none() {
            p.set_extension("vdesign");
        }
        p
    }))
}

fn write_file(path: &Path, text: &str) -> Result<(), String> {
    // Write beside the target and rename, so a failed write cannot leave half a file.
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    std::fs::write(&tmp, text).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    std::fs::rename(&tmp, path).map_err(|e| format!("cannot write {}: {e}", path.display()))
}

#[tauri::command]
fn snapshot(window: Window, state: State<Shared>) -> Reply {
    let guard = lock(&state)?;
    finish(&window, &guard)
}

#[tauri::command]
fn input_schema(state: State<Shared>) -> Reply {
    Ok(lock(&state)?.session().input_schema())
}

#[tauri::command]
fn evaluate(state: State<Shared>) -> Reply {
    Ok(lock(&state)?.evaluate_json())
}

/// `null` when the user cancels.
#[tauri::command]
async fn new_file(app: AppHandle, window: Window, state: State<'_, Shared>) -> Reply {
    if !confirm_discard(&app, &state)? {
        return Ok(serde_json::Value::Null);
    }
    let mut guard = lock(&state)?;
    *guard = AppState::new_design()?;
    finish(&window, &guard)
}

/// `null` when the user cancels.
#[tauri::command]
async fn open_file(app: AppHandle, window: Window, state: State<'_, Shared>) -> Reply {
    if !confirm_discard(&app, &state)? {
        return Ok(serde_json::Value::Null);
    }
    let Some(path) = pick_file(&app, "open.txt", "OVCD Design", &["vdesign"])? else {
        return Ok(serde_json::Value::Null);
    };
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let opened = AppState::open(&path, &text)?;
    let mut guard = lock(&state)?;
    *guard = opened;
    finish(&window, &guard)
}

fn save_to(app: &AppHandle, window: &Window, state: &Shared, ask_for_path: bool) -> Reply {
    let (known, text) = {
        let guard = lock(state)?;
        (guard.path().map(Path::to_path_buf), guard.save_text()?)
    };
    let path = match known {
        Some(p) if !ask_for_path => p,
        _ => {
            let Some(p) = pick_save(app)? else {
                return Ok(serde_json::Value::Null);
            };
            p
        }
    };
    write_file(&path, &text)?;
    let mut guard = lock(state)?;
    guard.saved(&path);
    finish(window, &guard)
}

/// `null` when the user cancels the file dialog.
#[tauri::command]
async fn save(app: AppHandle, window: Window, state: State<'_, Shared>) -> Reply {
    save_to(&app, &window, &state, false)
}

#[tauri::command]
async fn save_as(app: AppHandle, window: Window, state: State<'_, Shared>) -> Reply {
    save_to(&app, &window, &state, true)
}

#[tauri::command]
fn set_input(
    window: Window,
    state: State<Shared>,
    option: String,
    name: String,
    value: serde_json::Value,
) -> Reply {
    let mut guard = lock(&state)?;
    guard.set_input(&option, &name, value)?;
    finish(&window, &guard)
}

#[tauri::command]
fn reset_input(window: Window, state: State<Shared>, option: String, name: String) -> Reply {
    let mut guard = lock(&state)?;
    guard.reset_input(&option, &name)?;
    finish(&window, &guard)
}

#[tauri::command]
fn set_formation(
    window: Window,
    state: State<Shared>,
    option: String,
    kind: String,
    abreast: Option<u64>,
    in_line: Option<u64>,
) -> Reply {
    let mut guard = lock(&state)?;
    guard.set_formation(&option, &kind, abreast, in_line)?;
    finish(&window, &guard)
}

#[tauri::command]
fn add_option(window: Window, state: State<Shared>) -> Reply {
    let mut guard = lock(&state)?;
    guard.add_option()?;
    finish(&window, &guard)
}

#[tauri::command]
fn duplicate_option(window: Window, state: State<Shared>, option: String) -> Reply {
    let mut guard = lock(&state)?;
    guard.duplicate_option(&option)?;
    finish(&window, &guard)
}

#[tauri::command]
fn rename_option(window: Window, state: State<Shared>, option: String, new_name: String) -> Reply {
    let mut guard = lock(&state)?;
    guard.rename_option(&option, &new_name)?;
    finish(&window, &guard)
}

#[tauri::command]
fn delete_option(window: Window, state: State<Shared>, option: String) -> Reply {
    let mut guard = lock(&state)?;
    guard.delete_option(&option)?;
    finish(&window, &guard)
}

/// `null` when the user cancels the file dialog. An `Err` carries the session's reason, for
/// example that the profile loosens a hard range.
#[tauri::command]
async fn load_profile(app: AppHandle, window: Window, state: State<'_, Shared>) -> Reply {
    let Some(path) = pick_file(&app, "profile.txt", "Defaults Profile", &["json"])? else {
        return Ok(serde_json::Value::Null);
    };
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let mut guard = lock(&state)?;
    guard
        .load_profile(&text)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    finish(&window, &guard)
}

#[tauri::command]
fn use_built_in_profile(window: Window, state: State<Shared>) -> Reply {
    let mut guard = lock(&state)?;
    guard.use_built_in_profile()?;
    finish(&window, &guard)
}

#[tauri::command]
fn set_active_option(window: Window, state: State<Shared>, option: String) -> Reply {
    let mut guard = lock(&state)?;
    guard.set_active_option(&option)?;
    finish(&window, &guard)
}

/// The folder that holds the first-run marker: the app's config folder.
fn notice_dir(app: &AppHandle) -> Result<PathBuf, String> {
    match notice::test_config_dir() {
        Some(dir) => Ok(dir),
        None => app.path().app_config_dir().map_err(|e| e.to_string()),
    }
}

/// Has the user accepted the first-run notice?
#[tauri::command]
fn notice_seen(app: AppHandle) -> Result<bool, String> {
    Ok(notice::seen(&notice_dir(&app)?))
}

/// Remember that the user accepted the first-run notice.
#[tauri::command]
fn accept_notice(app: AppHandle) -> Result<(), String> {
    notice::accept(&notice_dir(&app)?, env!("CARGO_PKG_VERSION"))
}

pub fn run() {
    let initial = AppState::new_design().expect("the starter Design is valid");
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Mutex::new(initial))
        .invoke_handler(tauri::generate_handler![
            snapshot,
            input_schema,
            evaluate,
            new_file,
            open_file,
            save,
            save_as,
            set_input,
            reset_input,
            set_active_option,
            set_formation,
            add_option,
            duplicate_option,
            rename_option,
            delete_option,
            load_profile,
            use_built_in_profile,
            notice_seen,
            accept_notice,
        ])
        .on_window_event(|window, event| {
            // The unsaved-changes guard on close: ask first, then close for real.
            if let WindowEvent::CloseRequested { api, .. } = event {
                let app = window.app_handle().clone();
                let dirty = app
                    .state::<Shared>()
                    .lock()
                    .map(|s| s.is_dirty())
                    .unwrap_or(false);
                if dirty {
                    api.prevent_close();
                    let window = window.clone();
                    std::thread::spawn(move || {
                        let discard = ask_discard(
                            &app,
                            "The Design has unsaved changes. Discard them and close?",
                        );
                        if discard {
                            let _ = window.destroy();
                        }
                    });
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running the OVCD app");
}
