//! System dependency detection + install (check → skip present → install missing).

use crate::core::{which, Res, SpflError};
use std::fs;
use std::path::Path;
use std::process::Command;

pub const LUTRIS_MIN: (u32, u32, u32) = (0, 5, 20);
pub const LUTRIS_REC: (u32, u32, u32) = (0, 5, 22);
pub const WINE_MIN: (u32, u32, u32) = (10, 0, 0);

pub struct DepSpec {
    pub key: &'static str,
    pub label: &'static str,
    pub level: &'static str,
    pub pkgs: &'static [(&'static str, &'static [&'static str])],
}

pub const DEPS: &[DepSpec] = &[
    DepSpec {
        key: "lutris",
        level: "required",
        label: "Lutris ≥ 0.5.20 — installer + Wine runners",
        pkgs: &[
            ("apt", &["lutris"]),
            ("dnf", &["lutris"]),
            ("pacman", &["lutris"]),
            ("zypper", &["lutris"]),
            ("apk", &["lutris"]),
            ("flatpak", &["net.lutris.Lutris"]),
        ],
    },
    DepSpec {
        key: "wine",
        level: "recommended",
        label: "Wine ≥ 10 — used by the app to launch the game",
        pkgs: &[
            ("apt", &["wine64", "wine32"]),
            ("dnf", &["wine"]),
            ("pacman", &["wine"]),
            ("zypper", &["wine"]),
            ("apk", &["wine"]),
        ],
    },
    DepSpec {
        key: "winetricks",
        level: "recommended",
        label: "Winetricks — .NET/VC++ during setup",
        pkgs: &[
            ("apt", &["winetricks"]),
            ("dnf", &["winetricks"]),
            ("pacman", &["winetricks"]),
            ("zypper", &["winetricks"]),
            ("apk", &["winetricks"]),
        ],
    },
    DepSpec {
        key: "vulkan",
        level: "recommended",
        label: "Vulkan — required by DXVK",
        pkgs: &[
            ("apt", &["mesa-vulkan-drivers", "vulkan-tools"]),
            ("dnf", &["mesa-vulkan-drivers", "vulkan-tools"]),
            ("pacman", &["vulkan-icd-loader", "vulkan-tools"]),
            ("zypper", &["libvulkan1", "vulkan-tools"]),
            ("apk", &["mesa-vulkan-ati", "vulkan-tools"]),
        ],
    },
    DepSpec {
        key: "gamemode",
        level: "optional",
        label: "GameMode — extra performance",
        pkgs: &[
            ("apt", &["gamemode"]),
            ("dnf", &["gamemode"]),
            ("pacman", &["gamemode"]),
            ("zypper", &["gamemode"]),
            ("apk", &["gamemode"]),
        ],
    },
];

#[derive(Clone)]
pub struct DepState {
    pub key: &'static str,
    pub label: &'static str,
    pub level: &'static str,
    pub ok: bool,
    pub detail: String,
}

pub fn read_os_release() -> Vec<(String, String)> {
    fs::read_to_string("/etc/os-release")
        .unwrap_or_default()
        .lines()
        .filter_map(|l| l.split_once('='))
        .map(|(k, v)| (k.to_string(), v.trim().trim_matches('"').to_string()))
        .collect()
}

fn os_field(k: &str) -> String {
    read_os_release()
        .into_iter()
        .find(|(a, _)| a == k)
        .map(|(_, v)| v)
        .unwrap_or_default()
}

pub fn distro_name() -> String {
    let n = os_field("PRETTY_NAME");
    if n.is_empty() {
        os_field("NAME")
    } else {
        n
    }
}

pub fn is_steamos() -> bool {
    let blob = format!("{} {}", os_field("ID"), os_field("NAME")).to_lowercase();
    blob.contains("steamos")
        || blob.contains("steam deck")
        || Path::new("/run/ostree-booted").exists()
}

pub fn pkg_manager() -> String {
    for pm in ["apt", "dnf", "pacman", "zypper", "apk"] {
        if which(pm).is_some() {
            return pm.into();
        }
    }
    String::new()
}

pub fn ver_tuple(s: &str) -> Option<(u32, u32, u32)> {
    // Take the first digit[.digit]* run — e.g. "0.5.22", "wine-11.0".
    let mut num = String::new();
    let mut started = false;
    for c in s.chars() {
        if c.is_ascii_digit() {
            num.push(c);
            started = true;
        } else if c == '.' && started && num.ends_with(|x: char| x.is_ascii_digit()) {
            num.push(c);
        } else if started {
            break;
        }
    }
    while num.ends_with('.') {
        num.pop();
    }
    let mut parts = num.split('.');
    let a = parts.next()?.parse().ok()?;
    let b = parts.next().and_then(|x| x.parse().ok()).unwrap_or(0);
    let c = parts.next().and_then(|x| x.parse().ok()).unwrap_or(0);
    Some((a, b, c))
}

fn ge(a: (u32, u32, u32), b: (u32, u32, u32)) -> bool {
    a >= b
}

fn cmd_output(argv: &[&str]) -> Option<String> {
    let out = Command::new(argv[0]).args(&argv[1..]).output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).to_string())
}

fn has_libvulkan() -> bool {
    cmd_output(&["ldconfig", "-p"])
        .map(|s| s.contains("libvulkan.so"))
        .unwrap_or(false)
}

pub fn check_dep(key: &str) -> (bool, String) {
    match key {
        "lutris" => match which("lutris") {
            None => (false, "not installed".into()),
            Some(p) => {
                let exe = p.to_string_lossy().into_owned();
                match cmd_output(&[&exe, "--version"]) {
                    None => (true, exe),
                    Some(v) => {
                        let ver = ver_tuple(&v);
                        let first = v
                            .lines()
                            .next()
                            .unwrap_or("")
                            .chars()
                            .take(32)
                            .collect::<String>();
                        match ver {
                            Some(t) if !ge(t, LUTRIS_MIN) => {
                                (false, format!("too old ({first}), needs ≥ 0.5.20"))
                            }
                            Some(t) if !ge(t, LUTRIS_REC) => {
                                (true, format!("{first} — update to ≥ 0.5.22 recommended"))
                            }
                            _ => (true, first),
                        }
                    }
                }
            }
        },
        "wine" => match which("wine") {
            None => (false, "not installed".into()),
            Some(p) => {
                let exe = p.to_string_lossy().into_owned();
                match cmd_output(&[&exe, "--version"]) {
                    None => (true, exe),
                    Some(v) => {
                        let first = v
                            .lines()
                            .next()
                            .unwrap_or("")
                            .chars()
                            .take(40)
                            .collect::<String>();
                        match ver_tuple(&v) {
                            Some(t) if t < WINE_MIN => {
                                (false, format!("{first} — too old, needs Wine ≥ 10"))
                            }
                            _ => (true, first),
                        }
                    }
                }
            }
        },
        "winetricks" => match which("winetricks") {
            Some(p) => (true, p.to_string_lossy().into_owned()),
            None => (false, "not installed".into()),
        },
        "vulkan" => {
            if which("vulkaninfo").is_some() || has_libvulkan() {
                (true, "Vulkan loader/driver present".into())
            } else {
                (false, "not detected".into())
            }
        }
        "gamemode" => match which("gamemoded").or_else(|| which("gamemoderun")) {
            Some(p) => (true, p.to_string_lossy().into_owned()),
            None => (false, "not installed (optional)".into()),
        },
        "yadzen" => match which("yad").or_else(|| which("zenity")) {
            Some(p) => (true, format!("sistem: {}", p.to_string_lossy())),
            None => (false, "absent (not used by this app)".into()),
        },
        "tkinter" => (true, "not relevant — native Rust app".into()),
        _ => (false, "?".into()),
    }
}

pub fn check_all() -> Vec<DepState> {
    DEPS.iter()
        .map(|s| {
            let (ok, detail) = check_dep(s.key);
            DepState {
                key: s.key,
                label: s.label,
                level: s.level,
                ok,
                detail,
            }
        })
        .collect()
}

/// Install commands ONLY for missing items; present ones are skipped.
pub fn install_plan(missing: &[&str], pm: &str, deck: bool) -> Vec<String> {
    if missing.is_empty() {
        return Vec::new();
    }
    if deck {
        return vec!["flatpak install -y flathub net.lutris.Lutris".into()];
    }
    if pm.is_empty() || pm == "flatpak" {
        return vec!["flatpak install -y flathub net.lutris.Lutris  # fallback".into()];
    }
    let mut pkgs: Vec<&str> = Vec::new();
    for k in missing {
        if let Some(spec) = DEPS.iter().find(|d| d.key == *k) {
            if let Some((_, p)) = spec.pkgs.iter().find(|(m, _)| *m == pm) {
                pkgs.extend(p.iter().copied());
            }
        }
    }
    pkgs.sort_unstable();
    pkgs.dedup();
    let joined = pkgs.join(" ");
    match pm {
        "apt" => vec![
            "sudo apt update".into(),
            format!("sudo apt install -y {joined}"),
        ],
        "dnf" => vec![format!("sudo dnf install -y {joined}")],
        "pacman" => vec![format!("sudo pacman -S --needed {joined}")],
        "zypper" => vec![format!("sudo zypper install -y {joined}")],
        "apk" => vec![format!("sudo apk add {joined}")],
        _ => Vec::new(),
    }
}

/// Run the install plan via pkexec (GUI auth, no terminal).
pub fn run_install(cmds: &[String], log: &mut dyn FnMut(String)) -> Res<()> {
    let pkexec = which("pkexec").is_some();
    for c in cmds {
        let full = if pkexec {
            if let Some(r) = c.strip_prefix("sudo ") {
                format!("pkexec {r}")
            } else {
                c.clone()
            }
        } else {
            c.clone()
        };
        log(format!("$ {full}"));
        let mut parts = full.split_whitespace();
        let Some(prog) = parts.next() else { continue };
        let args: Vec<&str> = parts.collect();
        let out = Command::new(prog).args(&args).output();
        match out {
            Ok(o) => {
                let so = String::from_utf8_lossy(&o.stdout);
                for line in so.lines().take(400) {
                    log(format!("  {line}"));
                }
                let rc = o.status.code().unwrap_or(-1);
                log(format!("done (exit {rc}): {c}"));
                if rc != 0 {
                    log("STOPPED — copy the commands above to install manually.".into());
                    return Err(SpflError::Msg(format!("exit {rc}")));
                }
            }
            Err(e) => {
                log(format!("ERROR: {e}"));
                return Err(SpflError::Io(e));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_parsing() {
        assert_eq!(ver_tuple("lutris 0.5.22"), Some((0, 5, 22)));
        assert_eq!(ver_tuple("lutris 0.5.20"), Some((0, 5, 20)));
        assert_eq!(ver_tuple("wine-9.0"), Some((9, 0, 0)));
        assert_eq!(ver_tuple("wine-10.10"), Some((10, 10, 0)));
        assert_eq!(ver_tuple("11.0"), Some((11, 0, 0)));
        assert_eq!(ver_tuple("0.5.19"), Some((0, 5, 19)));
        assert_eq!(ver_tuple("no digits"), None);
        assert_eq!(ver_tuple(""), None);
        assert!(ver_tuple("lutris 0.5.22").unwrap() >= LUTRIS_REC);
        assert!(ver_tuple("lutris 0.5.19").unwrap() < LUTRIS_MIN);
        assert!(ver_tuple("wine-9.0").unwrap() < WINE_MIN);
        assert!(ver_tuple("wine-11.0").unwrap() >= WINE_MIN);
    }

    #[test]
    fn plan_only_contains_missing() {
        let missing = ["lutris", "wine"];
        let plan = install_plan(&missing, "dnf", false);
        assert_eq!(plan, vec!["sudo dnf install -y lutris wine"]);
        assert!(!plan.join(" ").contains("gamemode"));
    }

    #[test]
    fn plan_empty_when_nothing_missing() {
        assert!(install_plan(&[], "dnf", false).is_empty());
    }

    #[test]
    fn deck_uses_flatpak_only() {
        let plan = install_plan(&["lutris", "wine"], "pacman", true);
        assert_eq!(plan.len(), 1);
        assert!(plan[0].contains("flatpak"));
        assert!(!plan[0].contains("pacman"));
    }

    #[test]
    fn plan_per_pm() {
        assert!(install_plan(&["wine"], "apt", false)
            .join(" ")
            .contains("wine64"));
        assert!(install_plan(&["lutris"], "pacman", false)
            .join(" ")
            .contains("lutris"));
        assert!(install_plan(&["gamemode"], "dnf", false)
            .join(" ")
            .contains("gamemode"));
        assert!(install_plan(&["vulkan"], "apt", false)
            .join(" ")
            .contains("mesa-vulkan-drivers"));
    }

    #[test]
    fn no_python_only_dependencies() {
        // tkinter & yadzen only mattered for the Python version — must not appear.
        assert!(!DEPS.iter().any(|d| d.key == "tkinter"));
        assert!(!DEPS.iter().any(|d| d.key == "yadzen"));
        assert_eq!(DEPS.len(), 5);
        assert_eq!(
            check_dep("tkinter"),
            (true, "not relevant — native Rust app".into())
        );
    }

    #[test]
    fn distro_detected() {
        assert!(!distro_name().is_empty() || cfg!(target_os = "linux"));
    }
}
