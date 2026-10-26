use crate::{OfficeDocument, FileResult, open_document, write_document};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
pub struct DocumentSession {
    pub id: String,
    pub document: OfficeDocument,
    pub path: Option<String>,
    pub dirty: bool,
    pub updated_at: u64,
}
fn now() -> u64 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() }
fn dir(app: &AppHandle) -> Result<PathBuf, String> {
    let p = app.path().app_data_dir().map_err(|e|e.to_string())?.join("recovery");
    fs::create_dir_all(&p).map_err(|e|e.to_string())?;
    Ok(p)
}
fn file(app: &AppHandle, id: &str) -> Result<PathBuf, String> {
    Uuid::parse_str(id).map_err(|e|e.to_string())?;
    Ok(dir(app)?.join(format!("{id}.json")))
}
fn persist(app: &AppHandle, session: &DocumentSession) -> Result<(), String> {
    let dest = file(app, &session.id)?;
    let tmp = dest.with_extension("tmp");
    let data = serde_json::to_vec(session).map_err(|e|e.to_string())?;
    fs::write(&tmp, data).map_err(|e|e.to_string())?;
    // On Windows rename cannot replace an existing destination. Keep the previous
    // snapshot intact until the new snapshot has been written successfully.
    if dest.exists() { fs::remove_file(&dest).map_err(|e|e.to_string())?; }
    fs::rename(tmp, dest).map_err(|e|e.to_string())
}
#[tauri::command]
pub fn create_session(app: AppHandle, mode: String) -> Result<DocumentSession, String> {
    if !matches!(mode.as_str(), "word" | "excel" | "powerpoint") { return Err("Unknown editor".into()); }
    let s = DocumentSession { id: Uuid::new_v4().to_string(), document: OfficeDocument::empty(&mode), path: None, dirty: false, updated_at: now() };
    persist(&app, &s)?;
    Ok(s)
}
#[tauri::command]
pub fn import_session(app: AppHandle) -> Result<Option<DocumentSession>, String> {
    let Some(FileResult {path, document}) = open_document()? else { return Ok(None) };
    let s = DocumentSession { id: Uuid::new_v4().to_string(), document, path: Some(path), dirty: false, updated_at: now() };
    persist(&app, &s)?;
    Ok(Some(s))
}
#[tauri::command]
pub fn get_session(app: AppHandle, id: String) -> Result<DocumentSession, String> {
    let data = fs::read(file(&app, &id)?).map_err(|e|e.to_string())?;
    serde_json::from_slice(&data).map_err(|e|e.to_string())
}
#[tauri::command]
pub fn list_sessions(app: AppHandle) -> Result<Vec<DocumentSession>, String> {
    let mut all = Vec::new();
    for entry in fs::read_dir(dir(&app)?).map_err(|e|e.to_string())? {
        let entry = entry.map_err(|e|e.to_string())?;
        if entry.path().extension().and_then(|s|s.to_str()) != Some("json") { continue; }
        if let Ok(bytes) = fs::read(entry.path()) {
            if let Ok(s) = serde_json::from_slice::<DocumentSession>(&bytes) { all.push(s); }
        }
    }
    all.sort_by(|a,b| b.updated_at.cmp(&a.updated_at));
    Ok(all)
}
#[tauri::command]
pub fn snapshot_session(app: AppHandle, id: String, document: OfficeDocument, path: Option<String>, dirty: bool) -> Result<(), String> {
    // Validate that the session already exists, so a stale window cannot recreate
    // a discarded document after its recovery file was removed.
    let mut s = get_session(app.clone(), id)?;
    if s.document.mode != document.mode { return Err("Editor type cannot change".into()); }
    s.document = document;
    s.path = path;
    s.dirty = dirty;
    s.updated_at = now();
    persist(&app, &s)
}
#[tauri::command]
pub fn save_session(app: AppHandle, id: String, document: OfficeDocument, path: Option<String>, save_as: bool) -> Result<Option<String>, String> {
    let mut s = get_session(app.clone(), id)?;
    if s.document.mode != document.mode { return Err("Editor type cannot change".into()); }
    let target = if save_as { None } else { path.map(PathBuf::from) };
    let target = match target {
        Some(p) => p,
        None => {
            let (ext, label) = match document.mode.as_str() { "word" => ("docx", "Word"), "excel" => ("xlsx", "Excel"), _ => ("pptx", "PowerPoint") };
            let Some(p) = rfd::FileDialog::new().add_filter(label, &[ext]).set_file_name(&format!("Untitled.{ext}")).save_file() else { return Ok(None) };
            p
        }
    };
    write_document(&target, &document)?;
    let saved_path = target.to_string_lossy().to_string();
    s.document = document;
    s.path = Some(saved_path.clone());
    s.dirty = false;
    s.updated_at = now();
    persist(&app, &s)?;
    Ok(Some(saved_path))
}
#[tauri::command]
pub fn discard_session(app: AppHandle, id: String) -> Result<(), String> {
    let p = file(&app, &id)?;
    if p.exists() { fs::remove_file(p).map_err(|e|e.to_string())?; }
    Ok(())
}
#[tauri::command]
pub fn open_editor_window(app: AppHandle, id: String) -> Result<(), String> {
    let s = get_session(app.clone(), id.clone())?;
    let label = format!("editor-{id}");
    if let Some(w) = app.get_webview_window(&label) {
        w.show().map_err(|e|e.to_string())?;
        w.set_focus().map_err(|e|e.to_string())?;
        return Ok(());
    }
    let title = match s.document.mode.as_str() { "word" => "Rust Word", "excel" => "Rust Excel", _ => "Rust PowerPoint" };
    WebviewWindowBuilder::new(&app, &label, WebviewUrl::App(format!("index.html?view=editor&session={id}").into()))
        .title(title).inner_size(1440.0, 900.0).min_inner_size(950.0, 600.0)
        .build().map_err(|e|e.to_string())?;
    Ok(())
}
#[tauri::command]
pub fn focus_manager(app: AppHandle) -> Result<(), String> {
    let w = app.get_webview_window("main").ok_or("Manager window not found")?;
    w.show().map_err(|e|e.to_string())?;
    w.set_focus().map_err(|e|e.to_string())
}
