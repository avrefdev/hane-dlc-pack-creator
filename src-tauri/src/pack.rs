use crate::rpf::{self, Dir};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

pub const LANGUAGES: [&str; 13] = [
    "american",
    "chinese",
    "chinesesimp",
    "french",
    "german",
    "italian",
    "japanese",
    "korean",
    "mexican",
    "polish",
    "portuguese",
    "russian",
    "spanish",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Map,
    Vehicle,
}

#[derive(Serialize)]
pub struct Created {
    pub folder: String,
    pub rpf: String,
    pub item: String,
    pub meta: usize,
    pub main: usize,
}

pub fn normalize_name(raw: &str) -> Result<String, String> {
    let name = raw.to_lowercase();
    if name.is_empty() || name.contains(' ') {
        return Err("Please enter a DLC name without spaces.".into());
    }
    if !name.is_ascii() {
        return Err("Use only English letters, numbers, - and _ in the DLC name.".into());
    }
    if name.chars().any(|c| matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') || c.is_ascii_control()) {
        return Err("The DLC name contains characters Windows does not allow.".into());
    }
    if name.ends_with('.') {
        return Err("The DLC name cannot end with a dot.".into());
    }
    let stem = name.split('.').next().unwrap_or("");
    let reserved = matches!(stem, "con" | "prn" | "aux" | "nul")
        || (stem.len() == 4 && (stem.starts_with("com") || stem.starts_with("lpt")) && stem.as_bytes()[3].is_ascii_digit());
    if reserved {
        return Err(format!("\"{name}\" is a reserved Windows name. Pick another one."));
    }
    Ok(name)
}

pub fn dlclist_item(name: &str) -> String {
    format!("<Item>dlcpacks:\\{name}\\</Item>")
}

fn lines(list: &[String]) -> Vec<u8> {
    let mut s = String::new();
    for l in list {
        s.push_str(l);
        s.push_str("\r\n");
    }
    s.into_bytes()
}

fn data_item(out: &mut Vec<String>, filename: &str, file_type: &str, disabled: bool, persistent: bool, contents: Option<&str>) {
    out.push("\t\t<Item>".into());
    out.push(format!("\t\t\t<filename>{filename}</filename>"));
    out.push(format!("\t\t\t<fileType>{file_type}</fileType>"));
    out.push("\t\t\t<overlay value=\"false\" />".into());
    out.push(format!("\t\t\t<disabled value=\"{disabled}\" />"));
    out.push(format!("\t\t\t<persistent value=\"{persistent}\" />"));
    if let Some(c) = contents {
        out.push(format!("\t\t\t<contents>{c}</contents>"));
    }
    out.push("\t\t</Item>".into());
}

pub fn content_xml(kind: Kind, name: &str) -> Vec<u8> {
    let mut x: Vec<String> = vec![
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>".into(),
        "<CDataFileMgr__ContentsOfDataFileXml>".into(),
        "\t<disabledFiles />".into(),
        "\t<includedXmlFiles />".into(),
        "\t<includedDataFiles />".into(),
        "\t<dataFiles>".into(),
    ];
    let enable: Vec<String>;
    let change_set: String;
    let tail: Vec<String>;
    match kind {
        Kind::Map => {
            let level = format!("dlc_{name}:/%PLATFORM%/levels/gta5/{name}.rpf");
            let meta = format!("dlc_{name}:/%PLATFORM%/levels/gta5/{name}_metadata.rpf");
            data_item(&mut x, &level, "RPF_FILE", false, false, None);
            data_item(&mut x, &meta, "RPF_FILE", false, false, Some("CONTENTS_DLC_MAP_DATA"));
            change_set = format!("CCS_{name}_NG_STREAMING_MAP");
            enable = vec![level, meta];
            tail = vec![
                "\t\t\t<executionConditions>".into(),
                "\t\t\t\t<activeChangesetConditions>".into(),
                "\t\t\t\t</activeChangesetConditions>".into(),
                "\t\t\t\t<genericConditions>$level=MO_JIM_L11</genericConditions>".into(),
                "\t\t\t</executionConditions>".into(),
            ];
        }
        Kind::Vehicle => {
            let data = format!("dlc_{name}:/data/");
            let vehicles_rpf = format!("dlc_{name}:/%PLATFORM%/vehicles.rpf");
            data_item(&mut x, &format!("{data}vehicles.meta"), "VEHICLE_METADATA_FILE", true, false, None);
            data_item(&mut x, &format!("{data}carcols.meta"), "CARCOLS_FILE", true, false, None);
            data_item(&mut x, &format!("{data}carvariations.meta"), "VEHICLE_VARIATION_FILE", true, false, None);
            data_item(&mut x, &format!("{data}dlctext.meta"), "TEXTFILE_METAFILE", true, false, None);
            data_item(&mut x, &format!("{data}handling.meta"), "HANDLING_FILE", true, false, None);
            data_item(&mut x, &vehicles_rpf, "RPF_FILE", true, true, None);
            change_set = format!("{name}_AUTOGEN");
            enable = vec![
                format!("{data}handling.meta"),
                format!("{data}vehicles.meta"),
                format!("{data}carcols.meta"),
                format!("{data}carvariations.meta"),
                format!("{data}dlctext.meta"),
                vehicles_rpf,
            ];
            tail = vec![
                "\t\t\t<txdToLoad />".into(),
                "\t\t\t<txdToUnLoad />".into(),
                "\t\t\t<residentResources />".into(),
                "\t\t\t<unregisterResources />".into(),
            ];
        }
    }
    x.push("\t</dataFiles>".into());
    x.push("\t<contentChangeSets>".into());
    x.push("\t\t<Item>".into());
    x.push(format!("\t\t\t<changeSetName>{change_set}</changeSetName>"));
    x.push("\t\t\t<filesToEnable>".into());
    for e in enable {
        x.push(format!("\t\t\t\t<Item>{e}</Item>"));
    }
    x.push("\t\t\t</filesToEnable>".into());
    x.extend(tail);
    x.push("\t\t</Item>".into());
    x.push("\t</contentChangeSets>".into());
    x.push("\t<patchFiles />".into());
    x.push("</CDataFileMgr__ContentsOfDataFileXml>".into());
    lines(&x)
}

fn group(x: &mut Vec<String>, hash: &str, sets: &[String]) {
    x.push("\t\t<Item>".into());
    x.push(format!("\t\t\t<NameHash>{hash}</NameHash>"));
    x.push("\t\t\t<ContentChangeSets>".into());
    for s in sets {
        x.push(format!("\t\t\t\t<Item>{s}</Item>"));
    }
    x.push("\t\t\t</ContentChangeSets>".into());
    x.push("\t\t</Item>".into());
}

pub fn setup2_xml(kind: Kind, name: &str, timestamp: &str) -> Vec<u8> {
    let mut x: Vec<String> = vec![
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>".into(),
        "<SSetupData>".into(),
        format!("\t<deviceName>dlc_{name}</deviceName>"),
        "\t<datFile>content.xml</datFile>".into(),
        format!("\t<timeStamp>{timestamp}</timeStamp>"),
        format!("\t<nameHash>{name}</nameHash>"),
        "\t<contentChangeSets />".into(),
        "\t<contentChangeSetGroups>".into(),
    ];
    match kind {
        Kind::Map => {
            group(&mut x, "GROUP_EARLY_ON", &[format!("CCS_{name}_NG_INIT")]);
            group(
                &mut x,
                "GROUP_UPDATE_STREAMING",
                &[format!("CCS_{name}_NG_STREAMING"), format!("CCS_{name}_NG_STREAMING_MAP")],
            );
            group(&mut x, "GROUP_UPDATE_TEXT", &[format!("CCS_{name}_NG_TEXT")]);
            x.extend(
                [
                    "\t</contentChangeSetGroups>",
                    "\t<startupScript />",
                    "\t<scriptCallstackSize value=\"0\" />",
                    "\t<type>EXTRACONTENT_LEVEL_PACK</type>",
                    "\t<order value=\"2\" />",
                    "\t<minorOrder value=\"0\" />",
                    "\t<isLevelPack value=\"true\" />",
                    "\t<dependencyPackHash />",
                    "\t<requiredVersion />",
                    "\t<subPackCount value=\"0\" />",
                ]
                .map(String::from),
            );
        }
        Kind::Vehicle => {
            group(&mut x, "GROUP_STARTUP", &[format!("{name}_AUTOGEN")]);
            x.extend(
                ["\t</contentChangeSetGroups>", "\t<type>EXTRACONTENT_COMPAT_PACK</type>", "\t<order value=\"9\" />"]
                    .map(String::from),
            );
        }
    }
    x.push("</SSetupData>".into());
    lines(&x)
}

pub fn empty_gxt2() -> Vec<u8> {
    let mut b = Vec::with_capacity(16);
    b.extend_from_slice(b"2TXG");
    b.extend_from_slice(&0u32.to_le_bytes());
    b.extend_from_slice(b"2TXG");
    b.extend_from_slice(&16u32.to_le_bytes());
    b
}

pub const META_EXTS: [&str; 6] = ["ymt", "ytd", "ytyp", "ymap", "ynv", "ymf"];
pub const MAIN_EXTS: [&str; 13] = [
    "ydr", "ydd", "ybn", "yft", "ycd", "ypt", "yld", "yed", "ynd", "yvr", "ywr", "ypdb", "yfd",
];

#[derive(Default)]
pub struct Assets {
    pub meta: Vec<(String, PathBuf)>,
    pub main: Vec<(String, PathBuf)>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Scan {
    pub path: String,
    pub meta: usize,
    pub main: usize,
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("Could not read {}: {e}", dir.display()))?;
    for entry in entries.flatten() {
        let path = entry.path();
        let hidden = entry.file_name().to_string_lossy().starts_with('.');
        match entry.file_type() {
            Ok(t) if t.is_dir() && !hidden => walk(&path, out)?,
            Ok(t) if t.is_file() => out.push(path),
            _ => {}
        }
    }
    Ok(())
}

pub fn collect(resource: &Path) -> Result<Assets, String> {
    if !resource.is_dir() {
        return Err("The FiveM resource folder no longer exists.".into());
    }
    let mut files = Vec::new();
    walk(resource, &mut files)?;
    files.sort();
    let mut assets = Assets::default();
    let mut seen: std::collections::HashMap<String, PathBuf> = std::collections::HashMap::new();
    let mut conflicts: Vec<String> = Vec::new();
    for path in files {
        let ext = path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
        let is_meta = META_EXTS.contains(&ext.as_str());
        if !is_meta && !MAIN_EXTS.contains(&ext.as_str()) {
            continue;
        }
        let name = path.file_name().map(|n| n.to_string_lossy().to_lowercase()).unwrap_or_default();
        if !name.is_ascii() {
            return Err(format!("\"{name}\" has a non English character in its name."));
        }
        if let Some(first) = seen.get(&name) {
            let rel = |p: &Path| p.strip_prefix(resource).unwrap_or(p).display().to_string();
            conflicts.push(format!("{} and {}", rel(first), rel(&path)));
            continue;
        }
        seen.insert(name.clone(), path.clone());
        if is_meta {
            assets.meta.push((name, path));
        } else {
            assets.main.push((name, path));
        }
    }
    if !conflicts.is_empty() {
        let shown: Vec<String> = conflicts.iter().take(3).cloned().collect();
        let more = if conflicts.len() > 3 { format!(" and {} more", conflicts.len() - 3) } else { String::new() };
        return Err(format!("Files with the same name: {}{more}.", shown.join(", ")));
    }
    if assets.meta.is_empty() && assets.main.is_empty() {
        return Err("No GTA files (ymap, ytyp, ydr...) were found in this folder.".into());
    }
    Ok(assets)
}

pub fn scan(resource: &Path) -> Result<Scan, String> {
    let a = collect(resource)?;
    Ok(Scan { path: resource.display().to_string(), meta: a.meta.len(), main: a.main.len() })
}

fn add_files(dir: &mut Dir, files: &[(String, PathBuf)]) -> Result<(), String> {
    for (name, path) in files {
        let data = std::fs::read(path).map_err(|e| format!("Could not read {}: {e}", path.display()))?;
        dir.file(name, data);
    }
    Ok(())
}

pub fn build_tree(kind: Kind, name: &str, timestamp: &str, assets: Option<&Assets>) -> Result<Dir, String> {
    let mut root = Dir::root();
    root.file("content.xml", content_xml(kind, name));
    root.file("setup2.xml", setup2_xml(kind, name, timestamp));
    match kind {
        Kind::Map => {
            let gta5 = root.dir("x64").dir("levels").dir("gta5");
            let main = gta5.archive(&format!("{name}.rpf"));
            if let Some(a) = assets {
                add_files(main, &a.main)?;
            }
            let meta = gta5.archive(&format!("{name}_metadata.rpf"));
            if let Some(a) = assets {
                add_files(meta, &a.meta)?;
            }
        }
        Kind::Vehicle => {
            root.dir("data");
            let x64 = root.dir("x64");
            let lang = x64.dir("data").dir("lang");
            for l in LANGUAGES {
                lang.archive(&format!("{l}dlc.rpf")).file("global.gxt2", empty_gxt2());
            }
            x64.archive("vehicles.rpf");
        }
    }
    Ok(root)
}

pub fn timestamp() -> String {
    chrono::Local::now().format("%m/%d/%Y %H:%M:%S").to_string()
}

pub const GTA_EXES: [&str; 3] = ["GTA5.exe", "GTA5_Enhanced.exe", "PlayGTAV.exe"];

pub fn is_gta_root(dir: &Path) -> bool {
    GTA_EXES.iter().any(|exe| dir.join(exe).is_file())
}

pub fn find_gta_root(path: &Path) -> Option<PathBuf> {
    path.ancestors().find(|d| is_gta_root(d)).map(Path::to_path_buf)
}

pub fn dlcpacks_dir(gta_root: &Path) -> PathBuf {
    gta_root.join("mods").join("update").join("x64").join("dlcpacks")
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("Could not write {}: {e}", path.display()))?;
    if let Err(e) = f.write_all(bytes).and_then(|_| f.sync_all()) {
        drop(f);
        let _ = std::fs::remove_file(path);
        return Err(format!("Could not write {}: {e}", path.display()));
    }
    Ok(())
}

fn created(folder: &Path, rpf_path: &Path, name: &str, assets: Option<&Assets>) -> Created {
    Created {
        folder: folder.display().to_string(),
        rpf: rpf_path.display().to_string(),
        item: dlclist_item(name),
        meta: assets.map_or(0, |a| a.meta.len()),
        main: assets.map_or(0, |a| a.main.len()),
    }
}

pub fn create(kind: Kind, raw_name: &str, output: &Path, resource: Option<&Path>) -> Result<Created, String> {
    let name = normalize_name(raw_name)?;
    let folder: PathBuf = output.join(&name);
    let rpf_path = folder.join("dlc.rpf");
    if rpf_path.exists() {
        return Err(format!("A pack named \"{name}\" already exists in dlcpacks."));
    }
    let assets = match (kind, resource) {
        (Kind::Map, Some(r)) => Some(collect(r)?),
        _ => None,
    };
    let bytes = rpf::build(&build_tree(kind, &name, &timestamp(), assets.as_ref())?)?;
    std::fs::create_dir_all(&folder).map_err(|e| format!("Could not create {}: {e}", folder.display()))?;
    write_new(&rpf_path, &bytes)?;
    Ok(created(&folder, &rpf_path, &name, assets.as_ref()))
}

pub fn sync(name: &str, output: &Path, resource: &Path) -> Result<Created, String> {
    let name = normalize_name(name)?;
    let assets = collect(resource)?;
    let bytes = rpf::build(&build_tree(Kind::Map, &name, &timestamp(), Some(&assets))?)?;
    let folder: PathBuf = output.join(&name);
    std::fs::create_dir_all(&folder).map_err(|e| format!("Could not create {}: {e}", folder.display()))?;
    let rpf_path = folder.join("dlc.rpf");
    let tmp = folder.join("dlc.rpf.new");
    let bak = folder.join("dlc.rpf.bak");
    let _ = std::fs::remove_file(&tmp);
    write_new(&tmp, &bytes)?;
    if rpf_path.exists() {
        let _ = std::fs::remove_file(&bak);
        std::fs::rename(&rpf_path, &bak).map_err(|e| {
            let _ = std::fs::remove_file(&tmp);
            format!("Could not replace {}. Close OpenIV or the game if it is open. ({e})", rpf_path.display())
        })?;
    }
    std::fs::rename(&tmp, &rpf_path).map_err(|e| format!("Could not write {}: {e}", rpf_path.display()))?;
    Ok(created(&folder, &rpf_path, &name, Some(&assets)))
}
