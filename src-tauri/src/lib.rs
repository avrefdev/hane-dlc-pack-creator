pub mod pack;
pub mod rpf;
mod settings;

use pack::{Created, Kind};
use serde::Serialize;
use settings::Settings;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_dialog::DialogExt;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FolderItem {
    path: String,
    exists: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Folders {
    current: String,
    lost: String,
    items: Vec<FolderItem>,
}

fn config_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path().app_config_dir().map_err(|e| e.to_string())
}

fn is_dir(path: &str) -> bool {
    !path.is_empty() && Path::new(path).is_dir()
}

fn folders(s: &Settings) -> Folders {
    let items: Vec<FolderItem> = s.folders.iter().map(|p| FolderItem { path: p.clone(), exists: is_dir(p) }).collect();
    let (current, lost) = if is_dir(&s.output_folder) {
        (s.output_folder.clone(), String::new())
    } else {
        let fallback = items.iter().find(|i| i.exists).map(|i| i.path.clone()).unwrap_or_default();
        (fallback, s.output_folder.clone())
    };
    Folders { current, lost, items }
}

fn update(app: &AppHandle, change: impl FnOnce(&mut Settings)) -> Result<Folders, String> {
    let dir = config_dir(app)?;
    let mut s = settings::load(&dir);
    change(&mut s);
    settings::save(&dir, &s)?;
    let mut f = folders(&s);
    f.lost.clear();
    Ok(f)
}

#[tauri::command]
fn start(app: AppHandle) -> Result<Folders, String> {
    Ok(folders(&settings::load(&config_dir(&app)?)))
}

#[tauri::command]
async fn pick_folder(app: AppHandle, current: String) -> Result<Option<Folders>, String> {
    let mut dialog = app.dialog().file().set_title("Choose where to save DLC packs");
    if is_dir(&current) {
        dialog = dialog.set_directory(&current);
    }
    let Some(picked) = dialog.blocking_pick_folder() else { return Ok(None) };
    let folder = picked.into_path().map_err(|e| e.to_string())?.display().to_string();
    update(&app, |s| s.use_folder(&folder)).map(Some)
}

#[tauri::command]
fn select_folder(app: AppHandle, path: String) -> Result<Folders, String> {
    if !is_dir(&path) {
        return Err("This folder no longer exists.".into());
    }
    update(&app, |s| s.use_folder(&path))
}

#[tauri::command]
fn remove_folder(app: AppHandle, path: String) -> Result<Folders, String> {
    update(&app, |s| s.remove_folder(&path))
}

#[tauri::command]
fn create_pack(app: AppHandle, kind: Kind, name: String, output: String) -> Result<Created, String> {
    let created = pack::create(kind, &name, Path::new(&output))?;
    let _ = update(&app, |s| s.use_folder(&output));
    Ok(created)
}

#[tauri::command]
fn copy_text(app: AppHandle, text: String) -> Result<(), String> {
    app.clipboard().write_text(text).map_err(|e| e.to_string())
}

#[tauri::command]
fn reveal(path: String) -> Result<(), String> {
    let target = Path::new(&path);
    let mut cmd = std::process::Command::new("explorer.exe");
    if target.is_file() && path.encode_utf16().count() < 260 {
        cmd.raw_arg(format!("/select,\"{}\"", target.display()));
    } else if target.is_file() {
        cmd.arg(target.parent().unwrap_or(target));
    } else if target.is_dir() {
        cmd.arg(target);
    } else {
        return Err("That location no longer exists.".into());
    }
    cmd.spawn().map(|_| ()).map_err(|e| e.to_string())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            start,
            pick_folder,
            select_folder,
            remove_folder,
            create_pack,
            copy_text,
            reveal
        ])
        .run(tauri::generate_context!())
        .expect("error while running Hane DLC Pack Creator");
}
