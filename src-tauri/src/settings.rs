use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const MAX_FOLDERS: usize = 8;

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub output_folder: String,
    pub folders: Vec<String>,
}

pub fn same(a: &str, b: &str) -> bool {
    let key = |p: &str| p.trim_end_matches(['\\', '/']).to_lowercase();
    key(a) == key(b)
}

impl Settings {
    fn tidy(mut self) -> Self {
        if !self.output_folder.is_empty() && !self.folders.iter().any(|f| same(f, &self.output_folder)) {
            self.folders.insert(0, self.output_folder.clone());
        }
        let mut kept: Vec<String> = Vec::new();
        for f in self.folders.drain(..) {
            if !f.trim().is_empty() && !kept.iter().any(|k| same(k, &f)) {
                kept.push(f);
            }
        }
        kept.truncate(MAX_FOLDERS);
        self.folders = kept;
        self
    }

    pub fn use_folder(&mut self, path: &str) {
        self.folders.retain(|f| !same(f, path));
        self.folders.insert(0, path.to_string());
        self.folders.truncate(MAX_FOLDERS);
        self.output_folder = path.to_string();
    }

    pub fn remove_folder(&mut self, path: &str) {
        self.folders.retain(|f| !same(f, path));
        if same(&self.output_folder, path) {
            self.output_folder = self.folders.first().cloned().unwrap_or_default();
        }
    }
}

fn file(dir: &PathBuf) -> PathBuf {
    dir.join("settings.json")
}

pub fn load(dir: &PathBuf) -> Settings {
    std::fs::read_to_string(file(dir))
        .ok()
        .and_then(|s| serde_json::from_str::<Settings>(&s).ok())
        .unwrap_or_default()
        .tidy()
}

pub fn save(dir: &PathBuf, s: &Settings) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let tmp = dir.join("settings.json.tmp");
    let json = serde_json::to_string_pretty(s).map_err(|e| e.to_string())?;
    std::fs::write(&tmp, json).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, file(dir)).map_err(|e| e.to_string())
}
