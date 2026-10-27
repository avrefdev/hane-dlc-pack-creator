pub mod pack;
pub mod rpf;
mod settings;

use pack::{Created, Kind};
use serde::Serialize;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_dialog::DialogExt;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Gta {
    path: String,
    valid: bool,
    dlcpacks: String,
}

fn config_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path().app_config_dir().map_err(|e| e.to_string())
}

fn gta_state(path: &str) -> Gta {
    let valid = !path.is_empty() && pack::is_gta_root(Path::new(path));
    let dlcpacks = if valid { pack::dlcpacks_dir(Path::new(path)).display().to_string() } else { String::new() };
    Gta { path: path.to_string(), valid, dlcpacks }
}

fn saved_gta(app: &AppHandle) -> Result<String, String> {
    let dir = config_dir(app)?;
    let mut s = settings::load(&dir);
    if s.gta_path.is_empty() && !s.output_folder.is_empty() {
        if let Some(root) = pack::find_gta_root(Path::new(&s.output_folder)) {
            s.gta_path = root.display().to_string();
            settings::save(&dir, &s)?;
        }
    }
    Ok(s.gta_path)
}

#[tauri::command]
fn start(app: AppHandle) -> Result<Gta, String> {
    Ok(gta_state(&saved_gta(&app)?))
}

#[tauri::command]
async fn pick_gta(app: AppHandle) -> Result<Option<Gta>, String> {
    let current = saved_gta(&app)?;
    let mut dialog = app
        .dialog()
        .file()
        .set_title("Select GTA5.exe")
        .add_filter("GTA V", &["exe"]);
    if !current.is_empty() && Path::new(&current).is_dir() {
        dialog = dialog.set_directory(&current);
    }
    let Some(picked) = dialog.blocking_pick_file() else { return Ok(None) };
    let exe = picked.into_path().map_err(|e| e.to_string())?;
    let name = exe.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    if !pack::GTA_EXES.iter().any(|e| e.eq_ignore_ascii_case(&name)) {
        return Err("That is not GTA5.exe. Select GTA5.exe from your GTA V folder.".into());
    }
    let root = exe.parent().ok_or("Invalid location.")?.display().to_string();
    let dir = config_dir(&app)?;
    let mut s = settings::load(&dir);
    s.gta_path = root.clone();
    settings::save(&dir, &s)?;
    Ok(Some(gta_state(&root)))
}

fn dlcpacks(app: &AppHandle) -> Result<PathBuf, String> {
    let gta = saved_gta(app)?;
    if !pack::is_gta_root(Path::new(&gta)) {
        return Err("GTA V was not found. Select GTA5.exe again in Settings.".into());
    }
    Ok(pack::dlcpacks_dir(Path::new(&gta)))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LinkInfo {
    name: String,
    resource: String,
    synced: String,
    resource_exists: bool,
}

fn link_infos(s: &settings::Settings) -> Vec<LinkInfo> {
    s.links
        .iter()
        .map(|l| LinkInfo {
            name: l.name.clone(),
            resource: l.resource.clone(),
            synced: l.synced.clone(),
            resource_exists: Path::new(&l.resource).is_dir(),
        })
        .collect()
}

fn save_link(app: &AppHandle, name: &str, resource: &str) -> Result<(), String> {
    let dir = config_dir(app)?;
    let mut s = settings::load(&dir);
    s.links.retain(|l| l.name != name);
    s.links.insert(
        0,
        settings::Link { name: name.to_string(), resource: resource.to_string(), synced: pack::timestamp() },
    );
    settings::save(&dir, &s)
}

#[tauri::command]
async fn pick_resource(app: AppHandle) -> Result<Option<pack::Scan>, String> {
    let Some(picked) = app.dialog().file().set_title("Select the FiveM map resource folder").blocking_pick_folder() else {
        return Ok(None);
    };
    let path = picked.into_path().map_err(|e| e.to_string())?;
    pack::scan(&path).map(Some)
}

#[tauri::command]
async fn create_pack(app: AppHandle, kind: Kind, name: String, resource: Option<String>) -> Result<Created, String> {
    let out = dlcpacks(&app)?;
    let resource = resource.filter(|r| !r.is_empty() && kind == Kind::Map);
    let created = pack::create(kind, &name, &out, resource.as_deref().map(Path::new))?;
    if let Some(r) = resource {
        save_link(&app, &pack::normalize_name(&name)?, &r)?;
    }
    Ok(created)
}

#[tauri::command]
fn links(app: AppHandle) -> Result<Vec<LinkInfo>, String> {
    Ok(link_infos(&settings::load(&config_dir(&app)?)))
}

#[tauri::command]
async fn sync_link(app: AppHandle, name: String) -> Result<Vec<LinkInfo>, String> {
    let out = dlcpacks(&app)?;
    let dir = config_dir(&app)?;
    let s = settings::load(&dir);
    let link = s.links.iter().find(|l| l.name == name).cloned().ok_or("This link no longer exists.")?;
    pack::sync(&link.name, &out, Path::new(&link.resource))?;
    let mut s = settings::load(&dir);
    if let Some(l) = s.links.iter_mut().find(|l| l.name == name) {
        l.synced = pack::timestamp();
    }
    settings::save(&dir, &s)?;
    Ok(link_infos(&s))
}

#[tauri::command]
fn unlink(app: AppHandle, name: String) -> Result<Vec<LinkInfo>, String> {
    let dir = config_dir(&app)?;
    let mut s = settings::load(&dir);
    s.links.retain(|l| l.name != name);
    settings::save(&dir, &s)?;
    Ok(link_infos(&s))
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
        return Err("That location does not exist yet.".into());
    }
    cmd.spawn().map(|_| ()).map_err(|e| e.to_string())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            start,
            pick_gta,
            pick_resource,
            create_pack,
            links,
            sync_link,
            unlink,
            copy_text,
            reveal
        ])
        .run(tauri::generate_context!())
        .expect("error while running Hane DLC Pack Creator");
}
