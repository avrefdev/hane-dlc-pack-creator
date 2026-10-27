use flate2::write::DeflateEncoder;
use flate2::Compression;
use std::collections::HashMap;
use std::io::Write;

pub const MAGIC: u32 = 0x5250_4637;
pub const ENCRYPTION_OPEN: u32 = 0x4E45_504F;
pub const RSC7: u32 = 0x3743_5352;
const DIR_IDENT: u32 = 0x7FFF_FF00;
const BLOCK: usize = 512;
const MAX_U24: usize = 0xFF_FFFF;
const MAX_BLOCK_OFFSET: usize = 0x7F_FFFF;

#[derive(Default)]
pub struct Dir {
    pub name: String,
    pub dirs: Vec<Dir>,
    pub files: Vec<File>,
}

pub struct File {
    pub name: String,
    pub content: Content,
}

pub enum Content {
    Data(Vec<u8>),
    Archive(Dir),
}

impl Dir {
    pub fn root() -> Self {
        Self::default()
    }

    pub fn dir(&mut self, name: &str) -> &mut Dir {
        let pos = match self.dirs.iter().position(|d| d.name == name) {
            Some(p) => p,
            None => {
                self.dirs.push(Dir { name: name.to_string(), ..Default::default() });
                self.dirs.len() - 1
            }
        };
        &mut self.dirs[pos]
    }

    pub fn file(&mut self, name: &str, data: Vec<u8>) {
        self.files.push(File { name: name.to_string(), content: Content::Data(data) });
    }

    pub fn archive(&mut self, name: &str) -> &mut Dir {
        self.files.push(File { name: name.to_string(), content: Content::Archive(Dir::root()) });
        match &mut self.files.last_mut().unwrap().content {
            Content::Archive(d) => d,
            Content::Data(_) => unreachable!(),
        }
    }
}

enum Item<'a> {
    Dir(&'a Dir),
    File(&'a File),
}

impl Item<'_> {
    fn name(&self) -> &str {
        match self {
            Item::Dir(d) => &d.name,
            Item::File(f) => &f.name,
        }
    }
}

fn blocks(len: usize) -> usize {
    len.div_ceil(BLOCK)
}

fn pad_to(buf: &mut Vec<u8>, n: usize) {
    let rem = buf.len() % n;
    if rem != 0 {
        buf.resize(buf.len() + n - rem, 0);
    }
}

pub fn is_resource(data: &[u8]) -> bool {
    data.len() >= 16 && data[..4] == RSC7.to_le_bytes()
}

fn deflate(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut enc = DeflateEncoder::new(Vec::new(), Compression::default());
    enc.write_all(data).map_err(|e| e.to_string())?;
    enc.finish().map_err(|e| e.to_string())
}

fn check_names(dir: &Dir) -> Result<(), String> {
    let mut seen = std::collections::HashSet::new();
    let names = dir.dirs.iter().map(|d| &d.name).chain(dir.files.iter().map(|f| &f.name));
    for n in names {
        if n.is_empty() || !n.is_ascii() || n.contains(['\\', '/', '\0']) {
            return Err(format!("Invalid entry name \"{n}\""));
        }
        if !seen.insert(n.to_ascii_lowercase()) {
            return Err(format!("Duplicate entry \"{n}\""));
        }
    }
    Ok(())
}

pub fn build(root: &Dir) -> Result<Vec<u8>, String> {
    let mut all: Vec<Item> = vec![Item::Dir(root)];
    let mut ranges: HashMap<usize, (u32, u32)> = HashMap::new();
    let mut stack = vec![0usize];
    while let Some(i) = stack.pop() {
        let Item::Dir(dir) = all[i] else { unreachable!() };
        check_names(dir)?;
        let mut kids: Vec<Item> = dir.dirs.iter().map(Item::Dir).chain(dir.files.iter().map(Item::File)).collect();
        kids.sort_by(|a, b| a.name().as_bytes().cmp(b.name().as_bytes()));
        ranges.insert(i, (all.len() as u32, kids.len() as u32));
        for k in kids {
            let is_dir = matches!(k, Item::Dir(_));
            all.push(k);
            if is_dir {
                stack.push(all.len() - 1);
            }
        }
    }

    let mut names: Vec<u8> = Vec::new();
    let mut name_offsets: HashMap<&str, u32> = HashMap::new();
    let mut offsets = Vec::with_capacity(all.len());
    for item in &all {
        let n = item.name();
        let off = *name_offsets.entry(n).or_insert_with(|| {
            let o = names.len() as u32;
            names.extend_from_slice(n.as_bytes());
            names.push(0);
            o
        });
        offsets.push(off);
    }
    pad_to(&mut names, 16);

    let header_len = 16 + 16 * all.len() + names.len();
    let mut next_block = blocks(header_len);
    let mut entries: Vec<u8> = Vec::with_capacity(16 * all.len());
    let mut data: Vec<u8> = Vec::new();

    for (i, item) in all.iter().enumerate() {
        match item {
            Item::Dir(_) => {
                let (index, count) = ranges[&i];
                entries.extend_from_slice(&offsets[i].to_le_bytes());
                entries.extend_from_slice(&DIR_IDENT.to_le_bytes());
                entries.extend_from_slice(&index.to_le_bytes());
                entries.extend_from_slice(&count.to_le_bytes());
            }
            Item::File(f) => {
                if let Content::Data(raw) = &f.content {
                    if is_resource(raw) {
                        if offsets[i] > 0xFFFF {
                            return Err("Name table is too large".into());
                        }
                        if raw.len() > u32::MAX as usize {
                            return Err(format!("\"{}\" is too large", f.name));
                        }
                        if next_block > MAX_BLOCK_OFFSET {
                            return Err("Archive is too large".into());
                        }
                        let mut payload = raw.clone();
                        let len = payload.len() as u32;
                        if payload.len() >= MAX_U24 {
                            payload[7] = (len & 0xFF) as u8;
                            payload[14] = ((len >> 8) & 0xFF) as u8;
                            payload[5] = ((len >> 16) & 0xFF) as u8;
                            payload[2] = ((len >> 24) & 0xFF) as u8;
                        }
                        let system = u32::from_le_bytes(raw[8..12].try_into().unwrap());
                        let graphics = u32::from_le_bytes(raw[12..16].try_into().unwrap());
                        let size_field = (payload.len().min(MAX_U24)) as u32;
                        let off = next_block as u32;
                        entries.extend_from_slice(&(offsets[i] as u16).to_le_bytes());
                        entries.extend_from_slice(&size_field.to_le_bytes()[..3]);
                        entries.push((off & 0xFF) as u8);
                        entries.push(((off >> 8) & 0xFF) as u8);
                        entries.push((((off >> 16) & 0xFF) as u8) | 0x80);
                        entries.extend_from_slice(&system.to_le_bytes());
                        entries.extend_from_slice(&graphics.to_le_bytes());
                        next_block += blocks(payload.len());
                        data.extend_from_slice(&payload);
                        pad_to(&mut data, BLOCK);
                        continue;
                    }
                }
                let (payload, stored, size) = match &f.content {
                    Content::Data(raw) => {
                        let packed = deflate(raw)?;
                        let stored = packed.len();
                        (packed, stored, raw.len())
                    }
                    Content::Archive(d) => {
                        let bytes = build(d)?;
                        let size = bytes.len();
                        (bytes, 0, size)
                    }
                };
                if offsets[i] > 0xFFFF {
                    return Err("Name table is too large".into());
                }
                if stored > MAX_U24 || size > u32::MAX as usize {
                    return Err(format!("\"{}\" is too large", f.name));
                }
                if next_block > MAX_BLOCK_OFFSET {
                    return Err("Archive is too large".into());
                }
                entries.extend_from_slice(&(offsets[i] as u16).to_le_bytes());
                entries.extend_from_slice(&(stored as u32).to_le_bytes()[..3]);
                entries.extend_from_slice(&(next_block as u32).to_le_bytes()[..3]);
                entries.extend_from_slice(&(size as u32).to_le_bytes());
                entries.extend_from_slice(&0u32.to_le_bytes());
                next_block += blocks(payload.len());
                data.extend_from_slice(&payload);
                pad_to(&mut data, BLOCK);
            }
        }
    }

    let mut out = Vec::with_capacity(BLOCK + data.len());
    out.extend_from_slice(&MAGIC.to_le_bytes());
    out.extend_from_slice(&(all.len() as u32).to_le_bytes());
    out.extend_from_slice(&(names.len() as u32).to_le_bytes());
    out.extend_from_slice(&ENCRYPTION_OPEN.to_le_bytes());
    out.extend_from_slice(&entries);
    out.extend_from_slice(&names);
    pad_to(&mut out, BLOCK);
    out.extend_from_slice(&data);
    Ok(out)
}
