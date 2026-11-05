//! Pure logic — no UI, no framework. All unit tests live in this file.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

// ── Format constants (from reverse-engineering Settings.exe) ─────────────
pub const SETTINGS_OFFSET: usize = 8;
pub const SCREEN_OPTS: [(&str, &str); 2] = [("Windowed", "6"), ("Full Screen", "7")];
pub const CONTROLLER_OPTS: [(&str, &str); 2] = [("DirectInput", "6"), ("XInput", "7")];
pub const VSYNC_OPTS: [(&str, &str); 3] =
    [("Disabled", "02"), ("Enable1", "04"), ("Enable2", "08")];

/// label -> (source subdir, relative target path, file name)
pub const SWITCH_MAP: [(&str, &str, &str, &str); 3] = [
    ("GP", "GP", "Data/dt18_all.cpk", "dt18_all.cpk"),
    (
        "Controller",
        "Controller",
        "download/SMK_Addcontrol.cpk",
        "SMK_Addcontrol.cpk",
    ),
    (
        "Classics",
        "Classics",
        "download/08_smkdb_ulk.cpk",
        "08_smkdb_ulk.cpk",
    ),
];

pub const GAME_FOLDER_NAME: &str = "SP Football Life 2027";
/// Game launchers in preference order (single source of truth for the exe names).
pub const GAME_EXES: [&str; 2] = ["FL_2027.exe", "FL 2027 start.exe"];

static CONTROLLER_OPTS_REF: &[(&str, &str)] = &CONTROLLER_OPTS;
static SCREEN_OPTS_REF: &[(&str, &str)] = &SCREEN_OPTS;
static VSYNC_OPTS_REF: &[(&str, &str)] = &VSYNC_OPTS;

#[derive(Debug)]
pub enum SpflError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Msg(String),
}

impl std::fmt::Display for SpflError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpflError::Io(e) => write!(f, "I/O error: {e}"),
            SpflError::Json(e) => write!(f, "config corrupt: {e}"),
            SpflError::Msg(m) => write!(f, "{m}"),
        }
    }
}
impl std::error::Error for SpflError {}
impl From<std::io::Error> for SpflError {
    fn from(e: std::io::Error) -> Self {
        SpflError::Io(e)
    }
}
impl From<serde_json::Error> for SpflError {
    fn from(e: serde_json::Error) -> Self {
        SpflError::Json(e)
    }
}
pub type Res<T> = Result<T, SpflError>;

// ── Config ───────────────────────────────────────────────────────────────
pub fn config_path() -> PathBuf {
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".config")
        });
    base.join("spfl27-manager").join("config.json")
}

pub fn load_config() -> BTreeMap<String, String> {
    fs::read_to_string(config_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_game_dir(dir: &Path) -> Res<()> {
    save_kv("game_dir", &dir.to_string_lossy())
}

pub fn save_kv(key: &str, value: &str) -> Res<()> {
    let mut cfg = load_config();
    cfg.insert(key.into(), value.into());
    let p = config_path();
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(p, serde_json::to_string_pretty(&cfg)?)?;
    Ok(())
}

// ── Game detection ───────────────────────────────────────────────────────
/// Which launcher exists in this folder, if any.
pub fn game_exe_name(d: &Path) -> Option<&'static str> {
    GAME_EXES.into_iter().find(|exe| d.join(exe).is_file())
}

/// Does this folder contain the game?
pub fn is_game_dir(d: &Path) -> bool {
    game_exe_name(d).is_some()
}

/// Expand a user-typed path: trim whitespace, strip one layer of surrounding
/// quotes, expand `~` and `$HOME`. Never panics.
pub fn expand_path(input: &str) -> PathBuf {
    let mut s = input.trim().to_string();
    if s.len() >= 2
        && ((s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')))
    {
        s = s[1..s.len() - 1].to_string();
    }
    let s = s.trim();
    let home = std::env::var("HOME").unwrap_or_default();
    if s == "~" {
        return PathBuf::from(&home);
    }
    if let Some(rest) = s.strip_prefix("~/") {
        return PathBuf::from(&home).join(rest);
    }
    for prefix in ["$HOME/", "${HOME}/"] {
        if let Some(rest) = s.strip_prefix(prefix) {
            return PathBuf::from(&home).join(rest);
        }
    }
    if s == "$HOME" || s == "${HOME}" {
        return PathBuf::from(&home);
    }
    PathBuf::from(s)
}

/// One validation row for the Find-game checklist.
#[derive(Debug)]
pub struct DirCheck {
    pub label: &'static str,
    pub ok: bool,
    pub detail: String,
}

/// Validate a folder as the game location. Required: folder exists and holds
/// the game exe. Optional (warn only): Sider, Lutris prefix.
pub fn check_game_dir(dir: &Path) -> Vec<DirCheck> {
    let exists = dir.exists();
    let is_dir = dir.is_dir();
    let (exe_ok, exe_detail) = if !is_dir {
        (false, "needs a folder".to_string())
    } else {
        match game_exe_name(dir) {
            Some(exe) => (true, format!("{exe} found")),
            None => (false, "no FL_2027.exe here".to_string()),
        }
    };
    let (sider_ok, sider_detail) = if is_dir && dir.join("SiderAddons/sider.exe").is_file() {
        (true, "sider.exe found".to_string())
    } else {
        (false, "not installed (optional)".to_string())
    };
    let (prefix_ok, prefix_detail) = match prefix_of(dir) {
        Some(p) => (true, p.to_string_lossy().into_owned()),
        None => (false, "no drive_c above (optional)".to_string()),
    };
    vec![
        DirCheck {
            label: "Folder",
            ok: is_dir,
            detail: if is_dir {
                "exists".to_string()
            } else if !exists {
                "does not exist".to_string()
            } else {
                "not a folder".to_string()
            },
        },
        DirCheck {
            label: "Game file",
            ok: exe_ok,
            detail: exe_detail,
        },
        DirCheck {
            label: "Sider",
            ok: sider_ok,
            detail: sider_detail,
        },
        DirCheck {
            label: "Lutris prefix",
            ok: prefix_ok,
            detail: prefix_detail,
        },
    ]
}

/// Accept or reject a folder with a human-readable reason.
/// Required: existing folder + game exe inside. Everything else is optional.
pub fn accept_game_dir(dir: &Path) -> Res<PathBuf> {
    if !dir.exists() {
        return Err(SpflError::Msg(format!(
            "Folder does not exist: {}",
            dir.display()
        )));
    }
    if !dir.is_dir() {
        return Err(SpflError::Msg(format!("Not a folder: {}", dir.display())));
    }
    if !is_game_dir(dir) {
        return Err(SpflError::Msg(format!(
            "No game file in {} (expected FL_2027.exe). Pick the folder named \"{}\".",
            dir.display(),
            GAME_FOLDER_NAME
        )));
    }
    Ok(dir.to_path_buf())
}

pub fn candidate_dirs() -> Vec<PathBuf> {
    let home = std::env::var("HOME").map(PathBuf::from).unwrap_or_default();
    candidate_dirs_at(&home)
}

pub fn candidate_dirs_at(home: &Path) -> Vec<PathBuf> {
    vec![
        game_in_prefix(&home.join("Games").join("sp-football-life-2027")),
        game_in_prefix(&home.join(".var/app/net.lutris.Lutris/data/Games/sp-football-life-2027")),
    ]
}

/// The game folder inside a Lutris prefix (…/drive_c/Program Files (x86)/…).
fn game_in_prefix(base: &Path) -> PathBuf {
    base.join("drive_c")
        .join("Program Files (x86)")
        .join(GAME_FOLDER_NAME)
}

/// Lutris per-game config dirs (native + Flatpak paths).
pub fn lutris_config_bases_at(home: &Path) -> Vec<PathBuf> {
    vec![
        home.join(".config/lutris/games"),
        home.join(".var/app/net.lutris.Lutris/config/lutris/games"),
    ]
}

/// Read Lutris game configs and return folders that look like this game:
/// the parent of an `exe:` line under `SP Football Life 2027`, or
/// `prefix:` + the standard game sub-path. Only existing game dirs are kept.
pub fn game_dirs_from_lutris_configs(bases: &[PathBuf]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for base in bases {
        let Ok(rd) = fs::read_dir(base) else { continue };
        for entry in rd.flatten() {
            let p = entry.path();
            if !p.is_file() {
                continue;
            }
            let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext != "yml" && ext != "yaml" {
                continue;
            }
            let Ok(text) = fs::read_to_string(&p) else {
                continue;
            };
            let mentions_game =
                text.contains(GAME_FOLDER_NAME) || text.contains("sp-football-life-2027");
            if !mentions_game {
                continue;
            }
            for line in text.lines() {
                let t = line.trim();
                for key in ["exe:", "prefix:"] {
                    let Some(val) = t.strip_prefix(key) else {
                        continue;
                    };
                    let val = val.trim().trim_matches('"').trim_matches('\'').trim();
                    if val.is_empty() || val.contains("$GAMEDIR") {
                        continue;
                    }
                    let cand = if key == "exe:" {
                        // exe points at the launcher inside the game folder
                        let ep = PathBuf::from(val);
                        match ep.parent() {
                            Some(parent)
                                if parent.file_name().is_some_and(|n| n == GAME_FOLDER_NAME) =>
                            {
                                parent.to_path_buf()
                            }
                            _ => continue,
                        }
                    } else {
                        PathBuf::from(val)
                            .join("drive_c")
                            .join("Program Files (x86)")
                            .join(GAME_FOLDER_NAME)
                    };
                    if is_game_dir(&cand) {
                        out.push(cand);
                    }
                }
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

/// Roots for the filesystem scan (only existing dirs are walked).
pub fn search_roots_at(home: &Path) -> Vec<PathBuf> {
    let mut roots = vec![
        home.join("Games"),
        home.join(".local/share/lutris"),
        home.join(".var/app/net.lutris.Lutris/data"),
    ];
    for p in game_dirs_from_lutris_configs(&lutris_config_bases_at(home)) {
        // walk the install root, not the game folder itself
        if let Some(prefix) = prefix_of(&p) {
            if let Some(parent) = prefix.parent() {
                roots.push(parent.to_path_buf());
            }
        }
    }
    roots.into_iter().filter(|r| r.is_dir()).collect()
}

const SCAN_MAX_DEPTH: usize = 7;
const SCAN_MAX_DIRS: usize = 30_000;

/// Walk `roots` (no symlink following, bounded depth + dir budget) and return
/// folders containing the game exe. Stops after `max_results` hits.
pub fn scan_for_game(roots: &[PathBuf], max_results: usize) -> Vec<PathBuf> {
    fn dir_name(p: &Path) -> String {
        p.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase()
    }
    let mut out: Vec<PathBuf> = Vec::new();
    let mut seen: usize = 0;
    let mut stack: Vec<(PathBuf, usize)> = roots.iter().map(|r| (r.clone(), 0)).collect();
    while let Some((dir, depth)) = stack.pop() {
        if out.len() >= max_results || seen >= SCAN_MAX_DIRS {
            break;
        }
        // The game never lives under these Wine-internal trees.
        let n = dir_name(&dir);
        if depth > 0 && (n == "windows" || n == "users" || n == "programdata") {
            continue;
        }
        let Ok(rd) = fs::read_dir(&dir) else { continue };
        seen += 1;
        let mut subdirs: Vec<PathBuf> = Vec::new();
        let mut hit = false;
        for entry in rd.flatten() {
            let p = entry.path();
            if p.is_symlink() {
                continue;
            }
            if p.is_file() {
                if let Some(name) = p.file_name().and_then(|x| x.to_str()) {
                    if GAME_EXES.contains(&name) {
                        hit = true;
                        break;
                    }
                }
            } else if p.is_dir() && depth < SCAN_MAX_DEPTH {
                subdirs.push(p);
            }
        }
        if hit {
            out.push(dir);
        } else {
            for s in subdirs {
                stack.push((s, depth + 1));
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

/// Full automatic search: known candidates + Lutris configs + filesystem scan.
pub fn find_game_installs() -> Vec<PathBuf> {
    let home = std::env::var("HOME").map(PathBuf::from).unwrap_or_default();
    find_game_installs_at(&home)
}

pub fn find_game_installs_at(home: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = candidate_dirs_at(home)
        .into_iter()
        .filter(|p| is_game_dir(p))
        .collect();
    out.extend(game_dirs_from_lutris_configs(&lutris_config_bases_at(home)));
    out.extend(scan_for_game(&search_roots_at(home), 10));
    out.sort();
    out.dedup();
    out
}

pub fn detect_game_dir() -> Option<PathBuf> {
    let cfg = load_config();
    if let Some(saved) = cfg.get("game_dir") {
        let p = PathBuf::from(saved);
        if is_game_dir(&p) {
            return Some(p);
        }
    }
    candidate_dirs().into_iter().find(|p| is_game_dir(p))
}

pub fn prefix_of(game_dir: &Path) -> Option<PathBuf> {
    game_dir
        .ancestors()
        .find(|p| p.join("drive_c").is_dir())
        .map(|p| p.to_path_buf())
}

// ── Game version (PE VS_VERSIONINFO) ─────────────────────────────────────
fn le_u16(b: &[u8], o: usize) -> Option<u16> {
    let s = b.get(o..o + 2)?;
    Some(u16::from_le_bytes([s[0], s[1]]))
}

fn le_u32(b: &[u8], o: usize) -> Option<u32> {
    let s = b.get(o..o + 4)?;
    Some(u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
}

/// Map an RVA to a file offset using the section table. None when unmapped.
fn rva_to_offset(b: &[u8], sect_off: usize, n: usize, rva: u32) -> Option<usize> {
    for i in 0..n {
        let s = sect_off + i * 40;
        let vaddr = le_u32(b, s + 12)? as usize;
        let vsize = le_u32(b, s + 8)? as usize;
        let raw_size = le_u32(b, s + 16)? as usize;
        let raw_ptr = le_u32(b, s + 20)? as usize;
        let span = vsize.max(raw_size);
        let r = rva as usize;
        if r >= vaddr && r < vaddr + span {
            let off = raw_ptr + (r - vaddr);
            if off < b.len() {
                return Some(off);
            }
        }
    }
    None
}

/// One resource-directory entry: (id, offset). Named entries are skipped
/// (high bit of the id field); callers only want numeric ids.
fn res_entry(b: &[u8], rsrc: usize, dir: usize, idx: usize) -> Option<(u32, u32)> {
    let e = rsrc + dir + 16 + idx * 8;
    let id = le_u32(b, e)?;
    if id & 0x8000_0000 != 0 {
        return None;
    }
    Some((id, le_u32(b, e + 4)?))
}

fn res_child_dir(b: &[u8], rsrc: usize, dir: usize) -> Option<usize> {
    let named = le_u16(b, rsrc + dir + 12)? as usize;
    let ids = le_u16(b, rsrc + dir + 14)? as usize;
    for i in 0..named + ids {
        if let Some((_id, off)) = res_entry(b, rsrc, dir, i) {
            if off & 0x8000_0000 != 0 {
                return Some(rsrc + (off & 0x7FFF_FFFF) as usize);
            }
        }
    }
    None
}

/// FileVersion (e.g. "1.4.0.0") from a Windows PE's version resource.
/// None for non-PE files, stripped exes, or anything unparsable — callers
/// must treat that as "unknown", never as an error.
pub fn pe_file_version(path: &Path) -> Option<String> {
    let b = fs::read(path).ok()?;
    pe_file_version_bytes(&b)
}

fn pe_file_version_bytes(b: &[u8]) -> Option<String> {
    if b.len() < 64 || b.get(0..2)? != b"MZ" {
        return None;
    }
    let pe = le_u32(b, 0x3C)? as usize;
    if b.get(pe..pe + 4)? != b"PE\0\0" {
        return None;
    }
    let coff = pe + 4;
    let n_sect = le_u16(b, coff + 2)? as usize;
    let opt_size = le_u16(b, coff + 16)? as usize;
    let opt = coff + 20;
    let dd = match le_u16(b, opt)? {
        0x10b => opt + 96,  // PE32
        0x20b => opt + 112, // PE32+
        _ => return None,
    };
    if le_u32(b, dd - 4)? <= 2 {
        return None; // no resource directory entry
    }
    let rsrc_rva = le_u32(b, dd + 16)?;
    let sect_off = opt + opt_size;
    let rsrc = rva_to_offset(b, sect_off, n_sect, rsrc_rva)?;

    // root -> RT_VERSION (16) -> id -> language -> data
    let named = le_u16(b, rsrc + 12)? as usize;
    let ids = le_u16(b, rsrc + 14)? as usize;
    let mut ver_dir = None;
    for i in 0..named + ids {
        if let Some((id, off)) = res_entry(b, rsrc, 0, i) {
            if id == 16 && off & 0x8000_0000 != 0 {
                ver_dir = Some(rsrc + (off & 0x7FFF_FFFF) as usize);
                break;
            }
        }
    }
    let id_dir = res_child_dir(b, rsrc, ver_dir? - rsrc)?;
    let lang_dir = res_child_dir(b, rsrc, id_dir - rsrc)?;
    // first language entry -> IMAGE_RESOURCE_DATA_ENTRY (16 bytes)
    let ln = le_u16(b, lang_dir + 12)? as usize;
    let li = le_u16(b, lang_dir + 14)? as usize;
    let mut data_rva = None;
    for i in 0..ln + li {
        if let Some((_id, off)) = res_entry(b, rsrc, lang_dir - rsrc, i) {
            if off & 0x8000_0000 == 0 {
                let de = rsrc + off as usize;
                data_rva = Some(le_u32(b, de)?);
                break;
            }
        }
    }
    let voff = rva_to_offset(b, sect_off, n_sect, data_rva?)?;

    // VS_VERSIONINFO: wLength, wValueLength, wType, szKey(UTF-16), pad, fixed
    if le_u16(b, voff + 2)? < 52 {
        return None;
    }
    let mut k = voff + 6;
    loop {
        let c = le_u16(b, k)?;
        k += 2;
        if c == 0 || k > voff + 70 {
            break;
        }
    }
    let fixed = (k + 3) & !3;
    if le_u32(b, fixed)? != 0xFEEF_04BD {
        return None;
    }
    let ms = le_u32(b, fixed + 8)?;
    let ls = le_u32(b, fixed + 12)?;
    Some(format!(
        "{}.{}.{}.{}",
        ms >> 16,
        ms & 0xFFFF,
        ls >> 16,
        ls & 0xFFFF
    ))
}

/// Version string for the Home card, e.g. Some("1.4.0.0"). None means the
/// exe has no readable version — the UI must say "unknown", not guess.
pub fn game_version(game_dir: &Path) -> Option<String> {
    let exe = game_exe_name(game_dir)?;
    pe_file_version(&game_dir.join(exe))
}

// ── Switcher ─────────────────────────────────────────────────────────────
/// CPK source = the game-shipped `switcher/` folder. The repo bundles no .cpk.
pub fn cpk_source(game_dir: &Path) -> Option<PathBuf> {
    let d = game_dir.join("switcher");
    if d.is_dir()
        && fs::read_dir(&d)
            .map(|mut i| i.next().is_some())
            .unwrap_or(false)
    {
        Some(d)
    } else {
        None
    }
}

pub fn switch_def(label: &str) -> Option<(&'static str, &'static str, &'static str)> {
    SWITCH_MAP
        .iter()
        .find(|s| s.0 == label)
        .map(|s| (s.1, s.2, s.3))
}

pub fn list_options(src: &Path, label: &str) -> Vec<String> {
    let Some((sub, _, _)) = switch_def(label) else {
        return Vec::new();
    };
    let d = src.join(sub);
    let Ok(rd) = fs::read_dir(&d) else {
        return Vec::new();
    };
    let mut v: Vec<String> = rd
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    v.sort();
    v
}

pub fn read_current(game_dir: &Path, label: &str) -> String {
    fs::read_to_string(game_dir.join("dataSP").join(format!("{label}.txt")))
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

/// Copy the chosen CPK to its target. `choice` is validated against the option
/// list (anti path-traversal) before use.
pub fn apply_switch(game_dir: &Path, src: &Path, label: &str, choice: &str) -> Res<String> {
    let Some((sub, rel_target, fname)) = switch_def(label) else {
        return Err(SpflError::Msg(format!("unknown label: {label}")));
    };
    if !list_options(src, label).iter().any(|o| o == choice) {
        return Err(SpflError::Msg(format!(
            "Invalid choice for {label}: {choice:?}"
        )));
    }
    let mut origin = src.join(sub).join(choice).join(fname);
    if !origin.is_file() {
        origin = src.join(sub).join(choice);
    }
    if !origin.is_file() {
        return Err(SpflError::Msg(format!(
            "Source not found: {}",
            origin.display()
        )));
    }
    let target = game_dir.join(rel_target);
    if let Some(p) = target.parent() {
        fs::create_dir_all(p)?;
    }
    fs::copy(&origin, &target)?;
    let logdir = game_dir.join("dataSP");
    fs::create_dir_all(&logdir)?;
    fs::write(logdir.join(format!("{label}.txt")), choice)?;
    Ok(format!("{label} -> {choice}  ({})", target.display()))
}

// ── settings.dat ─────────────────────────────────────────────────────────
fn lookup<'a>(map: &'a [(&str, &str)], code: &str) -> Option<&'a str> {
    map.iter().find(|(_, c)| *c == code).map(|(n, _)| *n)
}

pub fn find_settings_dat(game_dir: &Path) -> Option<PathBuf> {
    let mut cands: Vec<PathBuf> = Vec::new();
    if let Some(p) = prefix_of(game_dir) {
        if let Ok(u) = fs::read_dir(p.join("drive_c/users")) {
            for ue in u.flatten() {
                collect_dat(&mut cands, &ue.path().join("Documents/KONAMI"));
            }
        }
    }
    let home = std::env::var("HOME").unwrap_or_default();
    collect_dat(&mut cands, &PathBuf::from(home).join("Documents/KONAMI"));
    cands.into_iter().find(|p| p.is_file())
}

fn collect_dat(out: &mut Vec<PathBuf>, root: &Path) {
    fn walk(out: &mut Vec<PathBuf>, d: &Path, depth: usize) {
        if depth > 6 {
            return;
        }
        let Ok(rd) = fs::read_dir(d) else { return };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(out, &p, depth + 1);
            } else if p.file_name().is_some_and(|n| n == "settings.dat") {
                out.push(p);
            }
        }
    }
    walk(out, root, 0);
}

pub fn find_konami_dir(game_dir: &Path) -> Option<PathBuf> {
    let dat = find_settings_dat(game_dir)?;
    dat.ancestors()
        .find(|p| p.ends_with("KONAMI"))
        .map(|p| p.to_path_buf())
}

pub struct Settings {
    pub controller: String,
    pub screen: String,
    pub vsync: String,
    pub raw: String,
}

pub fn read_settings(dat: &Path) -> Res<Settings> {
    let data = fs::read(dat)?;
    if data.len() < SETTINGS_OFFSET + 2 {
        return Err(SpflError::Msg(format!(
            "settings.dat too short ({} bytes) — corrupt file?",
            data.len()
        )));
    }
    let raw = hex(&data[SETTINGS_OFFSET..SETTINGS_OFFSET + 2]);
    let ctrl = &raw[0..1];
    let scr = &raw[1..2];
    let vs = &raw[2..4];
    Ok(Settings {
        controller: lookup(&CONTROLLER_OPTS, ctrl).unwrap_or("?").to_string(),
        screen: lookup(&SCREEN_OPTS, scr).unwrap_or("?").to_string(),
        vsync: lookup(&VSYNC_OPTS, vs).unwrap_or("?").to_string(),
        raw,
    })
}

pub fn write_settings(dat: &Path, controller: &str, screen: &str, vsync: &str) -> Res<String> {
    fn code_of(m: &[(&str, &str)], want: &str) -> Res<String> {
        m.iter()
            .find(|(n, _)| *n == want)
            .map(|(_, c)| (*c).to_string())
            .ok_or_else(|| SpflError::Msg(format!("unknown value: {want}")))
    }
    let hexstr = format!(
        "{}{}{}",
        code_of(CONTROLLER_OPTS_REF, controller)?,
        code_of(SCREEN_OPTS_REF, screen)?,
        code_of(VSYNC_OPTS_REF, vsync)?
    );
    let new = unhex(&hexstr)?;
    let mut buf = fs::read(dat)?;
    if buf.len() < SETTINGS_OFFSET + 2 {
        return Err(SpflError::Msg(format!(
            "settings.dat too short ({} bytes) — refusing write.",
            buf.len()
        )));
    }
    let old = hex(&buf[SETTINGS_OFFSET..SETTINGS_OFFSET + 2]);
    if old == hexstr {
        return Ok(format!("No change ({old})."));
    }
    let bname = format!("{}.backup", dat.file_name().unwrap().to_string_lossy());
    let bak = dat.with_file_name(bname);
    if !bak.exists() {
        fs::copy(dat, &bak)?;
    }
    buf[SETTINGS_OFFSET..SETTINGS_OFFSET + 2].copy_from_slice(&new);
    // Atomic write: write to a temp file in the same directory, then rename,
    // so an interrupted write cannot corrupt settings.dat.
    let tmp = dat.with_extension("tmp");
    if let Err(e) = fs::write(&tmp, &buf) {
        let _ = fs::remove_file(&tmp);
        return Err(e.into());
    }
    if let Err(e) = fs::rename(&tmp, dat) {
        let _ = fs::remove_file(&tmp);
        return Err(e.into());
    }
    Ok(format!(
        "settings.dat: {old} -> {hexstr} (backup: {})",
        bak.file_name().unwrap().to_string_lossy()
    ))
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}
fn unhex(s: &str) -> Res<Vec<u8>> {
    if s.len() % 2 != 0 {
        return Err(SpflError::Msg("odd-length hex".into()));
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| SpflError::Msg(e.to_string())))
        .collect()
}

// ── Save backup / restore ────────────────────────────────────────────────
pub fn backup_saves(konami: &Path, dest_root: &Path) -> Res<PathBuf> {
    let stamp = stamp_now();
    let dest = dest_root.join(format!("KONAMI-backup-{stamp}"));
    copy_dir(konami, &dest)?;
    Ok(dest)
}

pub fn restore_saves(src: &Path, konami: &Path) -> Res<()> {
    if konami.exists() {
        fs::remove_dir_all(konami)?;
    }
    if let Some(p) = konami.parent() {
        fs::create_dir_all(p)?;
    }
    copy_dir(src, konami)
}

fn copy_dir(src: &Path, dst: &Path) -> Res<()> {
    fs::create_dir_all(dst)?;
    for e in fs::read_dir(src)?.flatten() {
        let p = e.path();
        let t = dst.join(e.file_name());
        if p.is_dir() {
            copy_dir(&p, &t)?;
        } else {
            fs::copy(&p, &t)?;
        }
    }
    Ok(())
}

fn stamp_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // manual format: YYYYMMDD-HHMMSS, no chrono dependency
    let days = secs / 86400;
    let (y, mo, d) = civil_from_days(days as i64);
    let rem = secs % 86400;
    format!(
        "{y:04}{mo:02}{d:02}-{:02}{:02}{:02}",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

// ── Status & launch ──────────────────────────────────────────────────────
#[derive(Default, Clone, Debug)]
pub struct GameStatus {
    pub game: bool,
    pub sider: bool,
    pub aspect: bool,
    pub settings: bool,
    pub lutris: bool,
    pub wine: bool,
}

pub fn game_status(game_dir: Option<&Path>) -> GameStatus {
    let mut st = GameStatus {
        lutris: which("lutris").is_some(),
        wine: which("wine").is_some(),
        ..Default::default()
    };
    if let Some(d) = game_dir {
        if d.is_dir() {
            st.game = is_game_dir(d);
            st.sider = d.join("SiderAddons/sider.exe").is_file();
            st.aspect = d.join("SiderAddons/modules/aspectratio.lua").is_file();
            st.settings = find_settings_dat(d).is_some();
        }
    }
    st
}

pub fn which(cmd: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let p = dir.join(cmd);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

pub fn build_launch_cmd(game_dir: &Path) -> Vec<String> {
    let wine = which("wine")
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| "wine".into());
    let exe = game_exe_name(game_dir).unwrap_or(GAME_EXES[0]);
    vec![wine, game_dir.join(exe).to_string_lossy().into_owned()]
}

/// Mirrors FL_2027.bat: start sider first, wait for sider-app.log, then the game.
pub fn launch_game(game_dir: &Path) -> Res<()> {
    let wine = which("wine").ok_or_else(|| SpflError::Msg("wine not found".into()))?;
    let sider = game_dir.join("SiderAddons/sider.exe");
    let applog = game_dir.join("SiderAddons/sider-app.log");
    let _ = fs::remove_file(&applog);
    if sider.is_file() {
        let w = wine.to_string_lossy().into_owned();
        let sd = sider.to_string_lossy().into_owned();
        spawn_bg(&[&w, &sd], game_dir);
        for _ in 0..30 {
            if applog.exists() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
    let cmd = build_launch_cmd(game_dir);
    let refs: Vec<&str> = cmd.iter().map(|s| s.as_str()).collect();
    spawn_bg(&refs, game_dir);
    Ok(())
}

fn spawn_bg(argv: &[&str], cwd: &Path) {
    if argv.is_empty() {
        return;
    }
    let _ = Command::new(argv[0])
        .args(&argv[1..])
        .current_dir(cwd)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

/// Lutris slug registered by lutris/sp-football-life-2027.yaml.
pub const LUTRIS_SLUG: &str = "sp-football-life-2027";

/// Launch through an existing Lutris install (Deck path: no system wine needed).
pub fn launch_via_lutris() -> Res<()> {
    let lutris = which("lutris").ok_or_else(|| SpflError::Msg("lutris not found".into()))?;
    Command::new(lutris)
        .arg(format!("lutris:rungame/{LUTRIS_SLUG}"))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    Ok(())
}

// ── Lutris cache & Steam shortcut ────────────────────────────────────────
pub fn lutris_cache_dirs() -> Vec<PathBuf> {
    let home = std::env::var("HOME").map(PathBuf::from).unwrap_or_default();
    vec![
        home.join(".var/app/net.lutris.Lutris/cache/lutris"),
        home.join(".cache/lutris"),
    ]
    .into_iter()
    .filter(|p| p.is_dir())
    .collect()
}

pub fn clear_lutris_cache() -> Res<usize> {
    let mut n = 0usize;
    for d in lutris_cache_dirs() {
        for e in fs::read_dir(&d)?.flatten() {
            let p = e.path();
            if p.is_dir() && !p.is_symlink() {
                let _ = fs::remove_dir_all(&p);
            } else {
                let _ = fs::remove_file(&p);
            }
            n += 1;
        }
    }
    Ok(n)
}

pub fn make_game_desktop(game_dir: &Path, dest: &Path) -> Res<()> {
    if let Some(p) = dest.parent() {
        fs::create_dir_all(p)?;
    }
    let exe = std::env::current_exe()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| "spfl27".into());
    let body = format!(
        "[Desktop Entry]\nType=Application\nName=SP Football Life 2027\n\
         Exec=\"{exe}\" --play\nPath={}\nTerminal=false\nCategories=Game;\n",
        game_dir.display()
    );
    fs::write(dest, body)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(dest, fs::Permissions::from_mode(0o755))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static N: AtomicU32 = AtomicU32::new(0);
    fn tmp() -> PathBuf {
        let p = std::env::temp_dir().join(format!(
            "spfl27test-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        p
    }

    fn fake_game(root: &Path) -> PathBuf {
        let g = root
            .join("pfx/drive_c/Program Files (x86)")
            .join(GAME_FOLDER_NAME);
        fs::create_dir_all(g.join("Data")).unwrap();
        fs::create_dir_all(g.join("download")).unwrap();
        fs::create_dir_all(g.join("SiderAddons/modules")).unwrap();
        fs::create_dir_all(g.join("switcher/GP/FL27")).unwrap();
        fs::write(g.join("FL_2027.exe"), b"x").unwrap();
        fs::write(g.join("SiderAddons/sider.exe"), b"x").unwrap();
        fs::write(g.join("SiderAddons/modules/aspectratio.lua"), b"--").unwrap();
        fs::write(g.join("switcher/GP/FL27/dt18_all.cpk"), b"cpk").unwrap();
        g
    }

    /// Minimal valid PE32 with a VS_VERSIONINFO resource. Layout (all LE):
    /// DOS@0, PE@64, COFF@68, optional@88, section@312, .rsrc raw@512,
    /// root@512 -16-> @544 -1-> @576 -lang-> @608 -data-> @640, version@672.
    fn make_test_pe(major: u16, minor: u16, build: u16, rev: u16) -> Vec<u8> {
        let mut b = vec![0u8; 1024];
        let w16 = |b: &mut Vec<u8>, o: usize, v: u16| b[o..o + 2].copy_from_slice(&v.to_le_bytes());
        let w32 = |b: &mut Vec<u8>, o: usize, v: u32| b[o..o + 4].copy_from_slice(&v.to_le_bytes());
        b[0..2].copy_from_slice(b"MZ");
        w32(&mut b, 0x3C, 64);
        b[64..68].copy_from_slice(b"PE\0\0");
        w16(&mut b, 68, 0x14c);
        w16(&mut b, 70, 1);
        w16(&mut b, 84, 224);
        w16(&mut b, 88, 0x10b);
        w32(&mut b, 180, 16);
        w32(&mut b, 200, 0x1000); // resource RVA
        w32(&mut b, 204, 512);
        b[312..317].copy_from_slice(b".rsrc");
        w32(&mut b, 320, 512);
        w32(&mut b, 324, 0x1000);
        w32(&mut b, 328, 512);
        w32(&mut b, 332, 512);
        w16(&mut b, 512 + 14, 1);
        w32(&mut b, 528, 16);
        w32(&mut b, 532, 0x8000_0000 | 32);
        w16(&mut b, 544 + 14, 1);
        w32(&mut b, 560, 1);
        w32(&mut b, 564, 0x8000_0000 | 64);
        w16(&mut b, 576 + 14, 1);
        w32(&mut b, 592, 1033);
        w32(&mut b, 596, 0x8000_0000 | 96);
        w16(&mut b, 608 + 14, 1);
        w32(&mut b, 624, 0);
        w32(&mut b, 628, 128);
        w32(&mut b, 640, 0x10A0); // version data RVA
        w32(&mut b, 644, 92);
        w16(&mut b, 672, 92); // wLength
        w16(&mut b, 674, 52); // wValueLength
        for (i, c) in "VS_VERSION_INFO".encode_utf16().enumerate() {
            w16(&mut b, 678 + i * 2, c);
        }
        w32(&mut b, 712, 0xFEEF_04BD);
        w32(&mut b, 720, ((major as u32) << 16) | minor as u32);
        w32(&mut b, 724, ((build as u32) << 16) | rev as u32);
        b
    }

    #[test]
    fn prefix_detection() {
        let r = tmp();
        let g = fake_game(&r);
        assert_eq!(prefix_of(&g), Some(r.join("pfx")));
        assert_eq!(prefix_of(&r.join("nowhere")), None);
    }

    #[test]
    fn switcher_uses_game_bundle() {
        let r = tmp();
        let g = fake_game(&r);
        assert_eq!(cpk_source(&g), Some(g.join("switcher")));
        assert_eq!(cpk_source(&r.join("empty")), None);
        assert_eq!(list_options(&g.join("switcher"), "GP"), vec!["FL27"]);
    }

    #[test]
    fn apply_switch_writes_target_and_log() {
        let r = tmp();
        let g = fake_game(&r);
        let src = g.join("switcher");
        let msg = apply_switch(&g, &src, "GP", "FL27").unwrap();
        assert!(msg.contains("GP -> FL27"));
        assert_eq!(fs::read(g.join("Data/dt18_all.cpk")).unwrap(), b"cpk");
        assert_eq!(read_current(&g, "GP"), "FL27");
    }

    #[test]
    fn apply_switch_rejects_traversal() {
        let r = tmp();
        let g = fake_game(&r);
        let src = g.join("switcher");
        for bad in ["../../etc", "..", "/etc/passwd"] {
            let e = apply_switch(&g, &src, "GP", bad).unwrap_err();
            assert!(
                e.to_string().contains("Invalid choice"),
                "{bad} slipped through!"
            );
        }
    }

    #[test]
    fn settings_roundtrip_and_backup() {
        let r = tmp();
        let d = r.join("settings.dat");
        fs::write(&d, vec![0u8; 64]).unwrap();
        let msg = write_settings(&d, "XInput", "Full Screen", "Enable2").unwrap();
        assert!(msg.contains("7708"), "{msg}");
        assert!(r.join("settings.dat.backup").exists());
        assert_eq!(read_settings(&d).unwrap().controller, "XInput");
        // idempotent
        let m2 = write_settings(&d, "XInput", "Full Screen", "Enable2").unwrap();
        assert!(m2.contains("No change"), "{m2}");
    }

    #[test]
    fn settings_rejects_truncated_file() {
        let r = tmp();
        let d = r.join("short.dat");
        fs::write(&d, b"\x77").unwrap();
        assert!(read_settings(&d).is_err());
        assert!(write_settings(&d, "XInput", "Full Screen", "Enable1").is_err());
        // file must not change
        assert_eq!(fs::read(&d).unwrap(), b"\x77");
    }

    #[test]
    fn settings_rejects_unknown_value() {
        let r = tmp();
        let d = r.join("s.dat");
        fs::write(&d, vec![0u8; 64]).unwrap();
        assert!(write_settings(&d, "Radar", "Full Screen", "Enable1").is_err());
    }

    #[test]
    fn backup_and_restore_preserve_tree() {
        let r = tmp();
        let k = r.join("pfx/drive_c/users/deck/Documents/KONAMI");
        fs::create_dir_all(k.join("eFootball")).unwrap();
        fs::write(k.join("eFootball/settings.dat"), b"12345").unwrap();
        let dest = backup_saves(&k, &r.join("out")).unwrap();
        assert!(dest.join("eFootball/settings.dat").exists());
        fs::remove_dir_all(&k).unwrap();
        restore_saves(&dest, &k).unwrap();
        assert_eq!(
            fs::read(k.join("eFootball/settings.dat")).unwrap(),
            b"12345"
        );
    }

    #[test]
    fn status_reflects_filesystem() {
        let r = tmp();
        let g = fake_game(&r);
        let st = game_status(Some(&g));
        assert!(st.game && st.sider && st.aspect);
        let empty = game_status(Some(&r.join("nope")));
        assert!(!empty.game && !empty.sider && !empty.aspect);
    }

    #[test]
    fn desktop_entry_points_to_play_flag() {
        let r = tmp();
        let g = fake_game(&r);
        let d = r.join("G.desktop");
        make_game_desktop(&g, &d).unwrap();
        let t = fs::read_to_string(&d).unwrap();
        assert!(t.contains("--play"), "{t}");
        assert!(t.contains(GAME_FOLDER_NAME), "{t}");
    }

    #[test]
    fn desktop_entry_quotes_exe_path() {
        // repo/game paths with spaces (e.g. "Default Project") must still parse
        let r = tmp();
        let g = fake_game(&r);
        let d = r.join("G.desktop");
        make_game_desktop(&g, &d).unwrap();
        let t = fs::read_to_string(&d).unwrap();
        let exec = t.lines().find(|l| l.starts_with("Exec=")).unwrap();
        assert!(exec.contains('"'), "exe path must be quoted: {exec}");
        assert!(exec.ends_with("--play"), "{exec}");
    }

    #[test]
    fn lutris_fallback_errors_clearly_without_lutris() {
        // this machine has no lutris → must be a clean Err, not a panic
        if which("lutris").is_none() {
            assert!(launch_via_lutris().is_err());
        }
    }

    #[test]
    fn hex_roundtrip() {
        assert_eq!(hex(&[0x77, 0x08]), "7708");
        assert_eq!(unhex("7708").unwrap(), vec![0x77, 0x08]);
        assert!(unhex("abc").is_err());
    }

    #[test]
    fn timestamp_format_is_14_digits() {
        let s = stamp_now();
        assert_eq!(s.len(), 15, "{s}");
        assert!(s.chars().all(|c| c.is_ascii_digit() || c == '-'), "{s}");
    }

    #[test]
    fn expand_path_handles_tilde_quotes_and_home_var() {
        let home = std::env::var("HOME").unwrap_or_default();
        assert_eq!(expand_path("~"), PathBuf::from(&home));
        assert_eq!(
            expand_path("~/Games/x"),
            PathBuf::from(&home).join("Games/x")
        );
        assert_eq!(
            expand_path("  \"$HOME/Games/x\"  "),
            PathBuf::from(&home).join("Games/x")
        );
        assert_eq!(
            expand_path("'${HOME}/Games/x'"),
            PathBuf::from(&home).join("Games/x")
        );
        assert_eq!(expand_path("/tmp/abs"), PathBuf::from("/tmp/abs"));
    }

    #[test]
    fn check_and_accept_game_dir() {
        let r = tmp();
        let g = fake_game(&r);
        let checks = check_game_dir(&g);
        assert!(checks.iter().take(2).all(|c| c.ok), "{checks:?}");
        assert!(accept_game_dir(&g).is_ok());

        let empty = r.join("empty");
        fs::create_dir_all(&empty).unwrap();
        assert!(check_game_dir(&empty)[1].ok == false);
        let err = accept_game_dir(&empty).unwrap_err().to_string();
        assert!(err.contains("No game file"), "{err}");

        let missing = r.join("nope-missing");
        assert!(accept_game_dir(&missing)
            .unwrap_err()
            .to_string()
            .contains("does not exist"));
    }

    #[test]
    fn lutris_config_points_to_game() {
        let r = tmp();
        let home = r.join("home");
        let prefix = home.join("fake-pfx");
        let game = prefix
            .join("drive_c")
            .join("Program Files (x86)")
            .join(GAME_FOLDER_NAME);
        fs::create_dir_all(&game).unwrap();
        fs::write(game.join("FL_2027.exe"), b"x").unwrap();
        let cfg = home.join(".config/lutris/games");
        fs::create_dir_all(&cfg).unwrap();
        fs::write(
            cfg.join("sp-football-life-2027.yml"),
            format!(
                "game:\n  exe: {}/FL_2027.exe\n  prefix: {}\n",
                game.display(),
                prefix.display()
            ),
        )
        .unwrap();
        let found = game_dirs_from_lutris_configs(&[cfg]);
        assert_eq!(found, vec![game]);
    }

    #[test]
    fn scan_finds_nested_game_and_skips_unrelated() {
        let r = tmp();
        let home = r.join("home");
        let game = home
            .join("Games/custom/drive_c/Program Files (x86)")
            .join(GAME_FOLDER_NAME);
        fs::create_dir_all(&game).unwrap();
        fs::write(game.join("FL_2027.exe"), b"x").unwrap();
        fs::create_dir_all(home.join("Games/other")).unwrap();
        let hits = scan_for_game(&[home.join("Games")], 10);
        assert_eq!(hits, vec![game]);
        assert!(scan_for_game(&[home.join("Games/other")], 10).is_empty());
    }

    #[test]
    fn find_installs_dedupes_candidates() {
        let r = tmp();
        let home = r.join("home");
        // game exactly at the first known candidate spot
        let mut g = home.join("Games").join("sp-football-life-2027");
        for seg in ["drive_c", "Program Files (x86)", GAME_FOLDER_NAME] {
            g = g.join(seg);
        }
        fs::create_dir_all(&g).unwrap();
        fs::write(g.join("FL_2027.exe"), b"x").unwrap();
        let found = find_game_installs_at(&home);
        assert_eq!(found.iter().filter(|p| *p == &g).count(), 1);
        assert!(found.contains(&g));
    }

    #[test]
    fn pe_version_reads_synthetic_pe() {
        let r = tmp();
        let exe = r.join("game.exe");
        fs::write(&exe, make_test_pe(1, 2, 3, 4)).unwrap();
        assert_eq!(pe_file_version(&exe), Some("1.2.3.4".to_string()));
        let exe2 = r.join("game2.exe");
        fs::write(&exe2, make_test_pe(26, 1, 0, 5)).unwrap();
        assert_eq!(pe_file_version(&exe2), Some("26.1.0.5".to_string()));
    }

    #[test]
    fn pe_version_rejects_garbage() {
        let r = tmp();
        for (name, data) in [
            ("empty.exe", Vec::new()),
            ("text.exe", b"not a pe at all, just text".to_vec()),
            ("mzonly.exe", b"MZ".to_vec()),
        ] {
            let p = r.join(name);
            fs::write(&p, data).unwrap();
            assert_eq!(pe_file_version(&p), None, "{name}");
        }
        assert_eq!(pe_file_version(&r.join("missing.exe")), None);
    }

    #[test]
    fn pe_version_none_without_resources() {
        let r = tmp();
        let mut pe = make_test_pe(1, 0, 0, 0);
        // zero the resource directory entry -> no .rsrc to walk
        for o in 200..208 {
            pe[o] = 0;
        }
        let p = r.join("norsrc.exe");
        fs::write(&p, pe).unwrap();
        assert_eq!(pe_file_version(&p), None);
    }

    #[test]
    fn pe_version_never_panics_on_truncation() {
        let full = make_test_pe(9, 9, 9, 9);
        let r = tmp();
        let mut off = 0;
        while off <= full.len() {
            let p = r.join(format!("t{off}.exe"));
            fs::write(&p, &full[..off]).unwrap();
            let _ = pe_file_version(&p); // must not panic; result irrelevant
            off += 16;
        }
    }

    #[test]
    fn game_version_uses_game_exe() {
        let r = tmp();
        let g = fake_game(&r); // FL_2027.exe is 1 byte of junk here
        assert_eq!(game_version(&g), None);
        fs::write(g.join("FL_2027.exe"), make_test_pe(2, 0, 1, 0)).unwrap();
        assert_eq!(game_version(&g), Some("2.0.1.0".to_string()));
        assert_eq!(game_version(&r.join("nope")), None);
    }
}
