//! Modern native UI (iced 0.14): sidebar nav, setup wizard, light/dark themes,
//! Lucide SVG icons. All strings in English.

use crate::theme::{self, icon as isize, space, text as tsize, Mode};
use iced::keyboard::Key;
use iced::widget::{
    button, column, container, pick_list, progress_bar, row, rule, scrollable, stack, svg, text,
    text_input, Column, Row, Space,
};
use iced::{Alignment, Color, Length, Task};
use spfl27::core::*;
use spfl27::deps;

// ── Lucide icons (ISC, bundled) ──────────────────────────────────────────
fn icon(name: &str, size: f32) -> iced::Element<'static, Message> {
    let bytes: &'static [u8] = match name {
        "play" => include_bytes!("../assets/icons/play.svg"),
        "settings" => include_bytes!("../assets/icons/settings.svg"),
        "gamepad-2" => include_bytes!("../assets/icons/gamepad-2.svg"),
        "save" => include_bytes!("../assets/icons/save.svg"),
        "download" => include_bytes!("../assets/icons/download.svg"),
        "wrench" => include_bytes!("../assets/icons/wrench.svg"),
        "folder-open" => include_bytes!("../assets/icons/folder-open.svg"),
        "sun" => include_bytes!("../assets/icons/sun.svg"),
        "moon" => include_bytes!("../assets/icons/moon.svg"),
        "check" => include_bytes!("../assets/icons/check.svg"),
        "x" => include_bytes!("../assets/icons/x.svg"),
        "alert-triangle" => include_bytes!("../assets/icons/alert-triangle.svg"),
        "chevron-right" => include_bytes!("../assets/icons/chevron-right.svg"),
        "arrow-left" => include_bytes!("../assets/icons/arrow-left.svg"),
        "arrow-right" => include_bytes!("../assets/icons/arrow-right.svg"),
        "sparkles" => include_bytes!("../assets/icons/sparkles.svg"),
        "hard-drive" => include_bytes!("../assets/icons/hard-drive.svg"),
        "trash-2" => include_bytes!("../assets/icons/trash-2.svg"),
        "copy" => include_bytes!("../assets/icons/copy.svg"),
        "refresh-cw" => include_bytes!("../assets/icons/refresh-cw.svg"),
        "package" => include_bytes!("../assets/icons/package.svg"),
        "package-open" => include_bytes!("../assets/icons/package-open.svg"),
        "monitor" => include_bytes!("../assets/icons/monitor.svg"),
        "info" => include_bytes!("../assets/icons/info.svg"),
        "file-text" => include_bytes!("../assets/icons/file-text.svg"),
        "zap" => include_bytes!("../assets/icons/zap.svg"),
        "search" => include_bytes!("../assets/icons/search.svg"),
        "shield-check" => include_bytes!("../assets/icons/shield-check.svg"),
        "layers" => include_bytes!("../assets/icons/layers.svg"),
        "book-open" => include_bytes!("../assets/icons/book-open.svg"),
        "gauge" => include_bytes!("../assets/icons/gauge.svg"),
        "cpu" => include_bytes!("../assets/icons/cpu.svg"),
        "external-link" => include_bytes!("../assets/icons/external-link.svg"),
        "check-circle" => include_bytes!("../assets/icons/check-circle.svg"),
        "x-circle" => include_bytes!("../assets/icons/x-circle.svg"),
        "spfl-logo" => include_bytes!("../assets/icons/spfl-logo.svg"),
        _ => include_bytes!("../assets/icons/info.svg"),
    };
    svg(svg::Handle::from_memory(bytes))
        .width(size)
        .height(size)
        .into()
}

use std::path::PathBuf;

/// Product name shown in the sidebar, window title and Help.
pub const BRAND: &str = "SPFL27 Playtools";

/// One guide per page: title + 2-3 plain sentences (what / what to do / where).
/// Every page MUST have an entry — enforced by `ui_tests::all_pages_have_intros`.
pub fn page_intro(page: Page) -> (&'static str, &'static str) {
    match page {
        Page::Home => (
            "Home — ready to play?",
            "This page shows whether the game and its tools are found. Press PLAY to launch \
             everything — Sider, the small game helper, starts before the game.",
        ),
        Page::Install => (
            "Install — 3 easy steps",
            "You need 2 files from SmokePatch (pessmokepatch.com): the installer and the \
             latest update. The three steps below walk you through the rest.",
        ),
        Page::System => (
            "System — what Linux needs",
            "This page checks the Linux programs the game needs. Present items are \
             skipped; missing ones install with one click, and you will be asked for your password.",
        ),
        Page::Switcher => (
            "Switcher — change how it plays",
            "Use the game's own Switcher for gameplay, button icons and team modes. The default options are safe for new players.",
        ),
        Page::Settings => (
            "Settings — display & controller",
            "Use the game's own Settings tool for display, audio and controller settings. It is safer than editing files directly.",
        ),
        Page::Saves => (
            "Saves — protect your progress",
            "Your career and match progress lives in the game's save folder — inside its \
             Wine installation (uninstalling the game deletes it). Back up here in one click.",
        ),
        Page::Help => (
            "Help — fixes that work",
            "The most common problems and their fixes, in the order you should try them. \
             Keyboard shortcuts are listed at the bottom.",
        ),
    }
}

/// One line per dependency: what it is FOR. Shown under each row.
pub fn dep_purpose(key: &str) -> &'static str {
    match key {
        "lutris" => "Installs the game and provides its Wine runners.",
        "wine" => "Runs Sider and the game directly from this app.",
        "winetricks" => "Installs .NET/VCRedist inside the prefix during setup.",
        "vulkan" => "Graphics driver layer that DXVK needs to draw the match.",
        "gamemode" => "Optional: squeezes extra FPS out of your CPU/GPU.",
        _ => "",
    }
}

/// Setup wizard step guides: title + body.
pub fn setup_intro(step: usize) -> (&'static str, &'static str) {
    match step {
        0 => (
            "Welcome to SPFL27 Playtools",
            "This app gets the game ready on Linux and Steam Deck in three short steps: \
             find your game folder, install only what is missing, then press PLAY. You need \
             two files from pessmokepatch.com: the installer and the latest update.",
        ),
        1 => (
            "Step 1 of 3 — Find your game",
            "Press Search, or Browse to pick the folder yourself. A green check means the \
             game was found. No game yet? Finish this setup, then use the Install page.",
        ),
        2 => (
            "Step 2 of 3 — Fix the system",
            "Check the Linux programs the game needs. Only the missing ones are installed, \
             and you will be asked for your password.",
        ),
        _ => (
            "Step 3 of 3 — Ready to play",
            "Everything is in place. Go to Home and press the big PLAY button — it starts Sider first, then the game. Have fun!",
        ),
    }
}

// ── Pages ────────────────────────────────────────────────────────────────
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Page {
    Home,
    Install,
    System,
    Switcher,
    Settings,
    Saves,
    Help,
}

impl Page {
    fn title(self) -> &'static str {
        match self {
            Page::Home => "Home",
            Page::Install => "Install",
            Page::System => "System",
            Page::Switcher => "Switcher",
            Page::Settings => "Settings",
            Page::Saves => "Saves",
            Page::Help => "Help",
        }
    }
    fn icon(self) -> &'static str {
        match self {
            Page::Home => "play",
            Page::Install => "package",
            Page::System => "cpu",
            Page::Switcher => "refresh-cw",
            Page::Settings => "settings",
            Page::Saves => "save",
            Page::Help => "book-open",
        }
    }
    const ALL: [Page; 7] = [
        Page::Home,
        Page::Install,
        Page::System,
        Page::Switcher,
        Page::Settings,
        Page::Saves,
        Page::Help,
    ];
}

/// How long a toast stays on screen, in seconds.
const TOAST_SECS: u64 = 5;

/// Toast tone — maps to semantic colours in `theme.rs`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ToastKind {
    Success,
    Error,
    Info,
}

/// One modal sheet. Grows as the app needs more confirmations.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Modal {
    /// Confirm overwriting the current save (destructive).
    Restore,
    /// Change the game folder from anywhere (same card as wizard step 1).
    FindGame,
}

// ── Messages ─────────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub enum Message {
    Nav(Page),
    ToggleTheme,
    // toast
    ShowToast(ToastKind, String),
    DismissToast,
    Tick,
    // modal
    OpenRestoreModal,
    OpenFindGame,
    OpenDocs,
    CloseModal,
    RevealPath(PathBuf),
    PickRestore(String),
    // keyboard
    KeyPressed(iced::Event),
    ManualPathChanged(String),
    ApplyManualPath,
    BrowseFolder,
    BrowsePicked(Option<PathBuf>),
    BrowseSetupExe,
    BrowseSetupPicked(Option<PathBuf>),
    BrowseUpdate,
    BrowseUpdatePicked(Option<PathBuf>),
    PickCandidate(PathBuf),
    // setup wizard
    SetupNext,
    SetupBack,
    AutoDetect,
    ScanDone(Vec<PathBuf>),
    // install page
    SetupExeChanged(String),
    ApplySetupExe,
    OpenLutris,
    CopyYaml,
    UpdateFileChanged(String),
    RunUpdate,
    MakeShortcut,
    ClearCache,
    // system page
    DepsRefresh,
    DepsInstall,
    CopyInstallCmd,
    // switcher
    SwitcherPick(&'static str, String),
    SwitcherApply,
    OpenSwitcher,
    ToggleSwitcherAdvanced,
    OpenSettings,
    ToggleSettingsAdvanced,
    // settings
    SettingPick(&'static str, String),
    SettingsSave,
    // saves
    BackupNow,
    RestoreConfirm,
    // launch
    LaunchGame,
    // log panel
    ToggleLog,
    // background workers
    Log(String),
    WorkerDone(String),
}

// ── State ────────────────────────────────────────────────────────────────
pub struct App {
    mode: Mode,
    page: Page,
    game_dir: Option<PathBuf>,
    status: GameStatus,
    deps: Vec<deps::DepState>,
    dist: String,
    pm: String,
    deck: bool,
    logs: Vec<String>,
    log_open: bool,
    // toast + modal
    toast: Option<(ToastKind, String)>,
    toast_born: Option<std::time::Instant>,
    modal: Option<Modal>,
    /// Indeterminate sweep used while a worker is running.
    sweep: iced::animation::Animation<f32>,
    busy: bool,
    // setup wizard
    setup: bool,
    setup_step: usize,
    candidates: Vec<PathBuf>,
    scanning: bool,
    // install
    setup_exe: String,
    update_file: String,
    manual_path: String,
    // switcher
    sw_opts: Vec<(&'static str, Vec<String>)>,
    sw_cur: Vec<(&'static str, String)>,
    // settings
    set_screen: String,
    set_vsync: String,
    set_ctrl: String,
    dat_path: String,
    // saves
    backups: Vec<String>,
    restore_path: String,
    restore_armed: bool,
    switcher_advanced_open: bool,
    settings_advanced_open: bool,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let cfg = load_config();
        let mode = match cfg.get("theme").map(|s| s.as_str()) {
            Some("light") => Mode::Light,
            _ => Mode::Dark,
        };
        let mut app = App {
            mode,
            page: Page::Home,
            game_dir: detect_game_dir(),
            status: GameStatus::default(),
            deps: Vec::new(),
            dist: deps::distro_name(),
            pm: deps::pkg_manager(),
            deck: deps::is_steamos(),
            logs: Vec::new(),
            log_open: false,
            toast: None,
            toast_born: None,
            modal: None,
            sweep: iced::animation::Animation::new(0.25)
                .repeat_forever()
                .auto_reverse()
                .duration(std::time::Duration::from_millis(1100))
                .easing(iced::animation::Easing::EaseInOut),
            busy: false,
            setup: false,
            setup_step: 0,
            candidates: Vec::new(),
            scanning: false,
            setup_exe: String::new(),
            update_file: String::new(),
            manual_path: String::new(),
            sw_opts: Vec::new(),
            sw_cur: Vec::new(),
            set_screen: "Full Screen".into(),
            set_vsync: "Enable2".into(),
            set_ctrl: "XInput".into(),
            dat_path: String::new(),
            backups: Vec::new(),
            restore_path: String::new(),
            restore_armed: false,
            switcher_advanced_open: false,
            settings_advanced_open: false,
        };
        app.setup = app.game_dir.is_none();
        app.candidates = initial_candidates(app.game_dir.as_deref());
        app.refresh_all();
        app.push_log(format!("Ready. System: {}", app.dist));
        if app.setup {
            app.push_log("First run — opening Setup.".into());
        }
        (app, Task::none())
    }

    /// Fire a one-shot toast (the newest toast replaces the previous one).
    fn toast(&mut self, kind: ToastKind, msg: impl Into<String>) {
        self.toast = Some((kind, msg.into()));
        self.toast_born = Some(std::time::Instant::now());
    }

    fn push_log(&mut self, m: String) {
        for line in m.lines() {
            if self.logs.len() > 400 {
                self.logs.remove(0);
            }
            self.logs.push(line.to_string());
        }
    }

    /// Toast timeout tick (1s is enough granularity for a 5s toast).
    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::batch([
            iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::Tick),
            iced::event::listen().map(Message::KeyPressed),
        ])
    }

    pub fn theme(&self) -> iced::Theme {
        theme::theme_of(self.mode)
    }

    fn refresh_status(&mut self) {
        self.status = game_status(self.game_dir.as_deref());
    }

    fn refresh_switcher(&mut self) {
        self.sw_opts.clear();
        self.sw_cur.clear();
        if let Some(g) = &self.game_dir {
            if let Some(src) = cpk_source(g) {
                for sm in SWITCH_MAP.iter() {
                    let opts = list_options(&src, sm.0);
                    let mut cur = read_current(g, sm.0);
                    if !opts.contains(&cur) {
                        cur = opts.first().cloned().unwrap_or_default();
                    }
                    self.sw_opts.push((sm.0, opts));
                    self.sw_cur.push((sm.0, cur));
                }
            }
        }
    }

    fn refresh_settings(&mut self) {
        match self.game_dir.as_ref().and_then(|g| find_settings_dat(g)) {
            None => {
                self.dat_path =
                    "the game settings file not found yet — launch the game once to create it."
                        .into();
            }
            Some(d) => {
                self.dat_path = d.display().to_string();
                if let Ok(s) = read_settings(&d) {
                    self.set_ctrl = s.controller;
                    self.set_screen = s.screen;
                    self.set_vsync = s.vsync;
                }
            }
        }
    }

    fn refresh_backups(&mut self) {
        self.backups.clear();
        let home = std::env::var("HOME").unwrap_or_else(|_| "/".into());
        let root = std::path::PathBuf::from(home).join("spfl27-backups");
        if let Ok(rd) = std::fs::read_dir(&root) {
            let mut v: Vec<String> = rd
                .flatten()
                .filter(|e| e.path().is_dir())
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect();
            v.sort();
            v.reverse();
            self.backups = v;
        }
    }

    fn refresh_deps(&mut self) {
        self.deps = deps::check_all();
        self.pm = deps::pkg_manager();
    }

    fn refresh_all(&mut self) {
        self.refresh_status();
        self.refresh_switcher();
        self.refresh_settings();
        self.refresh_backups();
        self.refresh_deps();
    }

    fn set_game_dir(&mut self, d: PathBuf) {
        let _ = save_game_dir(&d);
        self.game_dir = Some(d.clone());
        self.push_log(format!("Game folder: {}", d.display()));
        self.refresh_all();
    }

    /// Validate a raw (typed, pasted, browsed, or found) path and adopt it.
    /// Shared by Apply / Browse / Search-result / candidate picks.
    fn apply_game_path(&mut self, raw: &str) -> Task<Message> {
        let expanded = expand_path(raw);
        match accept_game_dir(&expanded) {
            Ok(dir) => {
                let extras: Vec<String> = check_game_dir(&dir)
                    .into_iter()
                    .skip(1)
                    .filter(|c| c.ok)
                    .map(|c| format!("{}: {}", c.label, c.detail))
                    .collect();
                self.set_game_dir(dir.clone());
                if extras.is_empty() {
                    self.push_log(format!("Game folder accepted: {}", dir.display()));
                } else {
                    self.push_log(format!(
                        "Game folder accepted: {} ({})",
                        dir.display(),
                        extras.join(", ")
                    ));
                }
                if self.setup {
                    self.setup_step = 2;
                } else if self.modal == Some(Modal::FindGame) {
                    self.modal = None;
                }
            }
            Err(e) => self.push_log(format!("Not accepted: {e}")),
        }
        Task::none()
    }

    fn konami(&self) -> Option<PathBuf> {
        self.game_dir.as_ref().and_then(|g| find_konami_dir(g))
    }

    fn spawn<F>(&mut self, f: F) -> Task<Message>
    where
        F: FnOnce(futures::channel::mpsc::UnboundedSender<Message>) + Send + 'static,
    {
        self.busy = true;
        let (tx, rx) = futures::channel::mpsc::unbounded::<Message>();
        std::thread::spawn(move || f(tx));
        Task::stream(rx)
    }
}

// ── update ───────────────────────────────────────────────────────────────
impl App {
    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::Nav(p) => {
                self.page = p;
                self.refresh_all();
            }
            Message::ToggleTheme => {
                self.mode = match self.mode {
                    Mode::Dark => Mode::Light,
                    Mode::Light => Mode::Dark,
                };
                let _ = save_kv(
                    "theme",
                    match self.mode {
                        Mode::Dark => "dark",
                        Mode::Light => "light",
                    },
                );
            }
            Message::ManualPathChanged(s) => self.manual_path = s,
            Message::ApplyManualPath => {
                let raw = self.manual_path.clone();
                return self.apply_game_path(&raw);
            }
            Message::BrowseFolder => {
                return self.spawn(move |tx| {
                    let picked = rfd::FileDialog::new()
                        .set_title("Pick the SP Football Life 2027 folder")
                        .pick_folder();
                    let _ = tx.unbounded_send(Message::BrowsePicked(picked));
                    let _ = tx.unbounded_send(Message::WorkerDone("browse".into()));
                });
            }
            Message::BrowsePicked(opt) => match opt {
                None => self.push_log("Folder picker closed, nothing chosen.".into()),
                Some(p) => {
                    self.manual_path = p.to_string_lossy().into_owned();
                    let raw = self.manual_path.clone();
                    return self.apply_game_path(&raw);
                }
            },
            Message::BrowseSetupExe => {
                return self.spawn(move |tx| {
                    let picked = rfd::FileDialog::new()
                        .set_title("Pick SPFL27_setup.exe")
                        .add_filter("Windows installers", &["exe"])
                        .pick_file();
                    let _ = tx.unbounded_send(Message::BrowseSetupPicked(picked));
                    let _ = tx.unbounded_send(Message::WorkerDone("browse-setup".into()));
                });
            }
            Message::BrowseSetupPicked(opt) => match opt {
                None => {}
                Some(p) => {
                    self.setup_exe = p.to_string_lossy().into_owned();
                    let raw = self.setup_exe.trim().to_string();
                    self.push_log(format!("Installer set: {raw}"));
                    if std::path::Path::new(&raw).is_file() {
                        self.toast(
                            ToastKind::Success,
                            "Installer path set — now press Open Lutris.",
                        );
                    } else {
                        self.toast(ToastKind::Error, "That file was not found. Check the path.");
                    }
                }
            },
            Message::BrowseUpdate => {
                return self.spawn(move |tx| {
                    let picked = rfd::FileDialog::new()
                        .set_title("Pick the SPFL27 update file")
                        .add_filter("Windows installers", &["exe"])
                        .pick_file();
                    let _ = tx.unbounded_send(Message::BrowseUpdatePicked(picked));
                    let _ = tx.unbounded_send(Message::WorkerDone("browse-update".into()));
                });
            }
            Message::BrowseUpdatePicked(opt) => match opt {
                None => {}
                Some(p) => {
                    self.update_file = p.to_string_lossy().into_owned();
                    self.push_log(format!("Update file set: {}", self.update_file));
                    self.toast(
                        ToastKind::Info,
                        "Update file selected — press Run update to install it.",
                    );
                }
            },
            Message::PickCandidate(p) => {
                let raw = p.to_string_lossy().into_owned();
                return self.apply_game_path(&raw);
            }
            Message::AutoDetect => {
                if self.scanning {
                    return Task::none();
                }
                self.scanning = true;
                self.push_log("Searching the usual Lutris places for the game...".into());
                return self.spawn(move |tx| {
                    let found = find_game_installs();
                    let _ = tx.unbounded_send(Message::ScanDone(found));
                    let _ = tx.unbounded_send(Message::WorkerDone("scan".into()));
                });
            }
            Message::ScanDone(found) => {
                self.scanning = false;
                self.candidates = found;
                if self.candidates.is_empty() {
                    self.push_log(
                        "Search finished: no game found. Use Browse, paste the path, \
                         or install the game via the Install page."
                            .into(),
                    );
                } else if self.candidates.len() == 1 && self.game_dir.is_none() {
                    let only = self.candidates[0].to_string_lossy().into_owned();
                    self.push_log("Search finished: 1 game found, selected automatically.".into());
                    return self.apply_game_path(&only);
                } else {
                    self.push_log(format!(
                        "Search finished: {} game(s) found. Pick one below.",
                        self.candidates.len()
                    ));
                }
            }
            Message::SetupNext => {
                if self.setup_step == 1 && self.game_dir.is_none() {
                    self.push_log(
                        "Pick your game folder first (or install the game, then come back).".into(),
                    );
                    return Task::none();
                }
                if self.setup_step < 3 {
                    self.setup_step += 1;
                }
                if self.setup_step == 3 {
                    self.setup = false;
                    self.page = Page::Home;
                    self.refresh_all();
                }
            }
            Message::SetupBack => {
                if self.setup_step > 0 {
                    self.setup_step -= 1;
                }
            }

            Message::SetupExeChanged(s) => self.setup_exe = s,
            // Enter in the installer path field: remember it, then open Lutris.
            Message::ApplySetupExe => {
                let raw = self.setup_exe.trim().to_string();
                if raw.is_empty() {
                    self.toast(ToastKind::Error, "Paste the installer path first");
                    return Task::none();
                }
                if !std::path::Path::new(&raw).is_file() {
                    self.toast(ToastKind::Error, "That file was not found. Check the path.");
                    return Task::none();
                }
                self.push_log(format!("Installer path set: {raw}"));
                self.toast(ToastKind::Success, "Installer path set");
                return self.update(Message::OpenLutris);
            }
            Message::OpenLutris => {
                let y = spfl27::yaml_path().display().to_string();
                self.push_log(format!("Installer script: {y} (copy it into Lutris)"));
                match which("lutris") {
                    Some(p) => {
                        let _ = std::process::Command::new(p).spawn();
                        self.push_log(
                            "Lutris opened: + → Install from a local file → paste the path.".into(),
                        );
                    }
                    None => {
                        self.push_log("Lutris is not installed — see the System page.".into());
                        self.toast(
                            ToastKind::Error,
                            "Lutris is not installed — see the System page.",
                        );
                    }
                }
            }
            Message::CopyYaml => {
                let y = spfl27::yaml_path().display().to_string();
                if copy_text(&y) {
                    self.push_log(format!("Copied to clipboard: {y}"));
                } else {
                    self.push_log(format!("Clipboard unavailable — copy manually: {y}"));
                }
            }
            Message::UpdateFileChanged(s) => self.update_file = s,
            Message::RunUpdate => {
                let (Some(g), Some(pfx)) = (
                    self.game_dir.clone(),
                    self.game_dir.as_ref().and_then(|x| prefix_of(x)),
                ) else {
                    self.push_log("Select the game folder first.".into());
                    return Task::none();
                };
                let exe = self.update_file.clone();
                if exe.is_empty() {
                    self.push_log("Pick an update file first (SPFL27_*.exe).".into());
                    return Task::none();
                }
                let Some(wine) = which("wine") else {
                    self.push_log("Wine is missing — install it on the System page.".into());
                    self.toast(
                        ToastKind::Error,
                        "Wine is missing — install it on the System page.",
                    );
                    return Task::none();
                };
                let wine = wine.to_string_lossy().into_owned();
                self.push_log(format!("Running update (close the game first): {exe}"));
                return self.spawn(move |tx| {
                    let out = std::process::Command::new(&wine)
                        .arg(&exe)
                        .current_dir(&g)
                        .env("WINEPREFIX", &pfx)
                        .output();
                    match out {
                        Ok(o) => {
                            let _ = tx.unbounded_send(Message::Log(format!(
                                "Update finished (exit {:?}). Check the version on the main menu.",
                                o.status.code()
                            )));
                            let ok = o.status.success();
                            let msg = if ok {
                                "Update finished. Check the version on the main menu"
                            } else {
                                "Update reported an error. Open Activity for details."
                            };
                            let _ = tx.unbounded_send(Message::ShowToast(
                                if ok {
                                    ToastKind::Success
                                } else {
                                    ToastKind::Error
                                },
                                msg.to_string(),
                            ));
                        }
                        Err(e) => {
                            let _ = tx.unbounded_send(Message::Log(format!("Update failed: {e}")));
                            let _ = tx.unbounded_send(Message::ShowToast(
                                ToastKind::Error,
                                format!("Update could not start: {e}"),
                            ));
                        }
                    }
                    let _ = tx.unbounded_send(Message::WorkerDone("update".into()));
                });
            }
            Message::MakeShortcut => {
                let Some(g) = self.game_dir.clone() else {
                    self.push_log("Select the game folder first.".into());
                    return Task::none();
                };
                let home = std::env::var("HOME").unwrap_or_else(|_| "/".into());
                let dest = PathBuf::from(home).join("Desktop/SP Football Life 2027.desktop");
                match make_game_desktop(&g, &dest) {
                    Ok(()) => {
                        self.push_log(format!(
                            "Shortcut written: {}\nIn Steam: Add a Game → Add a Non-Steam Game → pick that file.",
                            dest.display()
                        ));
                        self.toast(ToastKind::Success, "Steam shortcut created");
                    }
                    Err(e) => self.push_log(format!("Failed: {e}")),
                }
            }
            Message::ClearCache => match clear_lutris_cache() {
                Ok(n) => self.push_log(format!(
                    "Lutris cache cleared ({n} items). Reopen Lutris to re-download."
                )),
                Err(e) => self.push_log(format!("Failed: {e}")),
            },
            Message::DepsRefresh => {
                self.refresh_deps();
                self.refresh_status();
            }
            Message::DepsInstall => {
                let skip: &[&str] = if self.deck {
                    &["wine", "gamemode"]
                } else {
                    &[]
                }; // Deck: Lutris runners cover wine
                let missing: Vec<&str> = self
                    .deps
                    .iter()
                    .filter(|d| !d.ok && !skip.contains(&d.key))
                    .map(|d| d.key)
                    .collect();
                let already: Vec<&str> = self.deps.iter().filter(|d| d.ok).map(|d| d.key).collect();
                let cmds = deps::install_plan(&missing, &self.pm, self.deck);
                if cmds.is_empty() {
                    self.push_log("Everything needed is already installed.".into());
                    self.toast(ToastKind::Info, "Everything needed is already installed");
                    return Task::none();
                }
                self.push_log(format!(
                    "Installing: {} | skipping (present): {}",
                    missing.join(", "),
                    if already.is_empty() {
                        "-".into()
                    } else {
                        already.join(", ")
                    }
                ));
                return self.spawn(move |tx| {
                    let send = |m: String| {
                        tx.unbounded_send(Message::Log(m)).ok();
                    };
                    let mut send = send;
                    let _ = deps::run_install(&cmds, &mut send);
                    let _ = tx.unbounded_send(Message::WorkerDone("install".into()));
                });
            }
            Message::CopyInstallCmd => {
                let skip: &[&str] = if self.deck {
                    &["wine", "gamemode"]
                } else {
                    &[]
                }; // Deck: Lutris runners cover wine
                let missing: Vec<&str> = self
                    .deps
                    .iter()
                    .filter(|d| !d.ok && !skip.contains(&d.key))
                    .map(|d| d.key)
                    .collect();
                let cmds = deps::install_plan(&missing, &self.pm, self.deck);
                let txt = if cmds.is_empty() {
                    "# everything needed is already installed".to_string()
                } else {
                    cmds.join("\n")
                };
                if copy_text(&txt) {
                    self.push_log("Commands copied to clipboard.".into());
                }
                self.push_log(format!("Copy this into a terminal:\n{txt}"));
            }
            Message::OpenSwitcher => {
                let Some(g) = self.game_dir.clone() else {
                    self.toast(ToastKind::Error, "Select the game folder first (Home page).");
                    return Task::none();
                };
                let switcher = g.join(SWITCHER_EXE);
                if !switcher.is_file() {
                    self.push_log(format!("{SWITCHER_EXE} is not installed."));
                    self.toast(
                        ToastKind::Error,
                        "FL27 Switcher is not installed. Reinstall the full game.",
                    );
                    return Task::none();
                }
                return self.spawn(move |tx| {
                    match launch_switcher(&g) {
                        Ok(()) => {
                            let _ = tx.unbounded_send(Message::Log("Opened FL27 Switcher.".into()));
                            let _ = tx.unbounded_send(Message::ShowToast(
                                ToastKind::Success,
                                "FL27 Switcher opened. Change settings, then press Apply and Close.".into(),
                            ));
                        }
                        Err(e) => {
                            let _ = tx.unbounded_send(Message::Log(format!("Could not open FL27 Switcher: {e}")));
                            let _ = tx.unbounded_send(Message::ShowToast(
                                ToastKind::Error,
                                format!("Could not open FL27 Switcher: {e}"),
                            ));
                        }
                    }
                    let _ = tx.unbounded_send(Message::WorkerDone("open-switcher".into()));
                });
            }
            Message::ToggleSwitcherAdvanced => {
                self.switcher_advanced_open = !self.switcher_advanced_open;
            }
            Message::OpenSettings => {
                let Some(g) = self.game_dir.clone() else {
                    self.toast(ToastKind::Error, "Select the game folder first (Home page).");
                    return Task::none();
                };
                let settings = g.join(SETTINGS_EXE);
                if !settings.is_file() {
                    self.push_log(format!("{SETTINGS_EXE} is not installed."));
                    self.toast(
                        ToastKind::Error,
                        "FL27 Settings is not installed. Reinstall the full game.",
                    );
                    return Task::none();
                }
                return self.spawn(move |tx| {
                    match launch_settings(&g) {
                        Ok(()) => {
                            let _ = tx.unbounded_send(Message::Log("Opened FL27 Settings.".into()));
                            let _ = tx.unbounded_send(Message::ShowToast(
                                ToastKind::Success,
                                "FL27 Settings opened. Change settings, then save and close.".into(),
                            ));
                        }
                        Err(e) => {
                            let _ = tx.unbounded_send(Message::Log(format!("Could not open FL27 Settings: {e}")));
                            let _ = tx.unbounded_send(Message::ShowToast(
                                ToastKind::Error,
                                format!("Could not open FL27 Settings: {e}"),
                            ));
                        }
                    }
                    let _ = tx.unbounded_send(Message::WorkerDone("open-settings".into()));
                });
            }
            Message::ToggleSettingsAdvanced => {
                self.settings_advanced_open = !self.settings_advanced_open;
            }
            Message::SwitcherPick(label, choice) => {
                if let Some(slot) = self.sw_cur.iter_mut().find(|(l, _)| *l == label) {
                    slot.1 = choice;
                }
            }
            Message::SwitcherApply => {
                let (Some(g), Some(src)) = (
                    self.game_dir.clone(),
                    self.game_dir.as_ref().and_then(|x| cpk_source(x)),
                ) else {
                    self.push_log(
                        "The game's 'switcher' folder was not found — install the full game."
                            .into(),
                    );
                    return Task::none();
                };
                let mut applied = false;
                for (label, choice) in self.sw_cur.clone() {
                    if choice.is_empty() {
                        continue;
                    }
                    match apply_switch(&g, &src, label, &choice) {
                        Ok(m) => {
                            self.push_log(m);
                            applied = true;
                        }
                        Err(e) => self.push_log(format!("{label}: {e}")),
                    }
                }
                if applied {
                    self.push_log("Switcher choices applied.".into());
                    self.refresh_switcher();
                }
            }
            Message::SettingPick(kind, v) => match kind {
                "screen" => self.set_screen = v,
                "vsync" => self.set_vsync = v,
                _ => self.set_ctrl = v,
            },
            Message::SettingsSave => {
                let Some(g) = self.game_dir.clone() else {
                    self.push_log("Select the game folder first.".into());
                    return Task::none();
                };
                match find_settings_dat(&g) {
                    None => self.push_log("Game settings file not found.".into()),
                    Some(d) => {
                        let (c, s, v) = (
                            self.set_ctrl.clone(),
                            self.set_screen.clone(),
                            self.set_vsync.clone(),
                        );
                        match write_settings(&d, &c, &s, &v) {
                            Ok(m) => self.push_log(m),
                            Err(e) => self.push_log(format!("Failed: {e}")),
                        }
                        self.refresh_settings();
                    }
                }
            }
            Message::BackupNow => {
                let Some(k) = self.konami() else {
                    self.push_log("Save folder not found.".into());
                    return Task::none();
                };
                let home = std::env::var("HOME").unwrap_or_else(|_| "/".into());
                let dest = PathBuf::from(home).join("spfl27-backups");
                if let Err(e) = std::fs::create_dir_all(&dest) {
                    self.push_log(format!("Cannot create backup folder: {e}"));
                    return Task::none();
                }
                match backup_saves(&k, &dest) {
                    Ok(d) => {
                        self.push_log(format!("Save backed up → {}", d.display()));
                        self.toast(ToastKind::Success, "Save backed up");
                        self.refresh_backups();
                    }
                    Err(e) => {
                        self.push_log(format!("Backup failed: {e}"));
                        self.toast(ToastKind::Error, format!("Backup failed: {e}"));
                    }
                }
            }
            Message::PickRestore(p) => {
                self.restore_path = p;
                self.restore_armed = false;
                self.modal = Some(Modal::Restore);
            }
            Message::RestoreConfirm => {
                let Some(k) = self.konami() else {
                    self.push_log("Save folder not found.".into());
                    self.toast(ToastKind::Error, "Save folder not found");
                    return Task::none();
                };
                let src = PathBuf::from(self.restore_path.trim());
                if !src.is_dir() {
                    self.push_log(format!("Not a valid backup folder: {}", src.display()));
                    self.toast(
                        ToastKind::Error,
                        "That is not a backup folder. Pick one from the list.",
                    );
                    return Task::none();
                }
                let looks_ok = std::fs::read_dir(&src)
                    .map(|mut it| it.any(|_| true))
                    .unwrap_or(false);
                if !looks_ok {
                    self.push_log("Backup folder is empty — cancelled.".into());
                    self.toast(ToastKind::Error, "That backup folder is empty");
                    return Task::none();
                }
                // inside the modal the sheet is the first confirmation, so a
                // second press does the work. Outside it, arm then confirm.
                if !self.restore_armed && self.modal.is_none() {
                    self.restore_armed = true;
                    self.push_log(format!(
                        "CONFIRM: {} will be OVERWRITTEN by {}. Press Restore again.",
                        k.display(),
                        src.display()
                    ));
                    return Task::none();
                }
                self.restore_armed = false;
                match restore_saves(&src, &k) {
                    Ok(()) => {
                        self.modal = None;
                        self.refresh_settings();
                        self.refresh_backups();
                        self.push_log(format!("Save restored ← {}", src.display()));
                        self.toast(ToastKind::Success, "Save restored");
                    }
                    Err(e) => {
                        self.push_log(format!("Restore failed: {e}"));
                        self.toast(ToastKind::Error, format!("Restore failed: {e}"));
                    }
                }
            }
            Message::LaunchGame => {
                let Some(g) = self.game_dir.clone() else {
                    self.push_log(
                        "Select the game folder first (Home page, Change button).".into(),
                    );
                    self.toast(
                        ToastKind::Error,
                        "Select the game folder first (Home page).",
                    );
                    return Task::none();
                };
                let st = game_status(Some(&g));
                if !st.game {
                    self.push_log(format!("FL_2027.exe not found in {}", g.display()));
                    self.toast(
                        ToastKind::Error,
                        "Game not found there — pick the folder ending in 'SP Football Life 2027'.",
                    );
                    return Task::none();
                }
                if !st.wine {
                    if st.lutris {
                        self.push_log("No system wine — launching through Lutris instead.".into());
                        self.toast(
                            ToastKind::Info,
                            "No system Wine — launching through Lutris.",
                        );
                        return self.spawn(move |tx| {
                            match launch_via_lutris() {
                                Ok(()) => {
                                    let _ = tx.unbounded_send(Message::Log(
                                        "Launched via Lutris.".into(),
                                    ));
                                    let _ = tx.unbounded_send(Message::ShowToast(
                                        ToastKind::Success,
                                        "Launching through Lutris…".into(),
                                    ));
                                }
                                Err(e) => {
                                    let _ = tx.unbounded_send(Message::Log(format!("ERROR: {e}")));
                                    let _ = tx.unbounded_send(Message::ShowToast(
                                        ToastKind::Error,
                                        format!("Could not launch via Lutris: {e}"),
                                    ));
                                }
                            }
                            let _ = tx.unbounded_send(Message::WorkerDone("launch".into()));
                        });
                    }
                    self.push_log("Wine is missing — install it on the System page.".into());
                    self.toast(
                        ToastKind::Error,
                        "Wine is missing — install it on the System page.",
                    );
                    return Task::none();
                }
                self.push_log(format!("Launching: {}", g.display()));
                return self.spawn(move |tx| {
                    if let Err(e) = launch_game(&g) {
                        let _ = tx.unbounded_send(Message::Log(format!("ERROR: {e}")));
                        let _ = tx.unbounded_send(Message::ShowToast(
                            ToastKind::Error,
                            format!("Could not launch: {e}"),
                        ));
                    } else {
                        let _ = tx.unbounded_send(Message::Log("Sider + game launched.".into()));
                        let _ = tx.unbounded_send(Message::ShowToast(
                            ToastKind::Success,
                            "Launching — Sider first, then the game.".into(),
                        ));
                    }
                    let _ = tx.unbounded_send(Message::WorkerDone("launch".into()));
                });
            }
            Message::ToggleLog => {
                self.log_open = !self.log_open;
            }
            Message::Log(line) => self.push_log(line),
            Message::ShowToast(kind, msg) => {
                self.toast = Some((kind, msg));
                self.toast_born = Some(std::time::Instant::now());
            }
            Message::DismissToast => {
                self.toast = None;
                self.toast_born = None;
            }
            Message::Tick => {
                // auto-dismiss after 5s
                let expired = self
                    .toast_born
                    .is_some_and(|t| t.elapsed().as_secs() >= TOAST_SECS);
                if expired {
                    self.toast = None;
                    self.toast_born = None;
                }
            }
            Message::OpenRestoreModal => {
                self.modal = Some(Modal::Restore);
            }
            Message::OpenFindGame => {
                // pre-fill with the current folder so its checklist shows
                self.manual_path = self
                    .game_dir
                    .as_ref()
                    .map(|g| g.to_string_lossy().into_owned())
                    .unwrap_or_default();
                self.candidates = initial_candidates(self.game_dir.as_deref());
                self.modal = Some(Modal::FindGame);
            }
            Message::CloseModal => {
                self.modal = None;
                self.restore_armed = false;
            }
            Message::KeyPressed(ev) => {
                if let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key,
                    modifiers,
                    ..
                }) = ev
                {
                    use iced::keyboard::key::Named;
                    match (key.as_ref(), modifiers.control()) {
                        (Key::Character("l"), true) => self.log_open = !self.log_open,
                        (Key::Named(Named::Escape), false) => {
                            if self.modal.is_some() {
                                self.modal = None;
                                self.restore_armed = false;
                            }
                        }
                        (Key::Named(Named::F5), false) => {
                            self.refresh_deps();
                            self.refresh_status();
                            self.push_log("Dependencies rechecked.".into());
                        }
                        _ => {}
                    }
                }
            }
            Message::OpenDocs => match spfl27::docs_dir() {
                Some(d) => {
                    let shown = d.display().to_string();
                    match std::process::Command::new("xdg-open").arg(&d).spawn() {
                        Ok(_) => self.push_log(format!("Opened guides: {shown}")),
                        Err(e) => {
                            self.push_log(format!("Could not open guides: {e}"));
                            self.toast(ToastKind::Error, "Could not open the guides folder");
                        }
                    }
                }
                None => {
                    self.push_log("Guides folder not found next to the app.".into());
                    self.toast(ToastKind::Error, "Guides folder not found");
                }
            },
            Message::RevealPath(p) => {
                let dir = p.parent().map(|d| d.to_path_buf()).unwrap_or(p);
                match which("xdg-open")
                    .or_else(|| which("gio"))
                    .or_else(|| which("nautilus"))
                {
                    Some(opener) => {
                        let shown = opener.display().to_string();
                        match std::process::Command::new(&opener).arg(&dir).spawn() {
                            Ok(_) => self.push_log(format!("Opened: {}", dir.display())),
                            Err(e) => {
                                let _ = self.update(Message::ShowToast(
                                    ToastKind::Error,
                                    format!("Could not open {shown}: {e}"),
                                ));
                            }
                        }
                    }
                    None => {
                        let _ = self.update(Message::ShowToast(
                            ToastKind::Error,
                            "No file manager found (xdg-open/gio missing).".into(),
                        ));
                    }
                }
            }

            Message::WorkerDone(kind) => {
                self.busy = false;
                // scan/browse already report their own outcome — skip the routine line.
                if kind != "scan" && kind != "browse" {
                    self.push_log(format!("Worker finished: {kind}."));
                }
                if kind == "install" {
                    self.refresh_deps();
                    self.refresh_status();
                }
            }
        }
        Task::none()
    }
}

fn copy_text(s: &str) -> bool {
    arboard::Clipboard::new()
        .and_then(|mut c| c.set_text(s.to_string()))
        .is_ok()
}

// ── text + chip helpers (all sizes from theme::text / theme::space) ───────
/// Page title.
fn h1(s: impl Into<String>) -> iced::Element<'static, Message> {
    text(s.into())
        .size(tsize::H2)
        .line_height(tsize::LH_TIGHT)
        .into()
}

/// Secondary text: muted, comfortable leading, themed colour.
fn sub(s: impl Into<String>, dark: bool) -> iced::Element<'static, Message> {
    text(s.into())
        .size(tsize::SMALL)
        .line_height(tsize::LH_NORMAL)
        .color(theme::tok(dark).text_muted)
        .into()
}

/// Wizard intro paragraph: readable measure, relaxed leading, muted colour.
fn prose(s: &'static str, dark: bool) -> iced::Element<'static, Message> {
    let c = theme::tok(dark);
    text(s)
        .size(tsize::BODY)
        .line_height(tsize::LH_RELAXED)
        .color(c.text_muted)
        .width(Length::Fixed(tsize::MEASURE))
        .into()
}

/// Truncate a log line so the collapsed drawer stays one row.
fn ellipsis(s: &str, max: usize) -> String {
    let one = s.lines().next().unwrap_or("");
    if one.chars().count() <= max {
        return one.to_string();
    }
    let head: String = one.chars().take(max.saturating_sub(1)).collect();
    format!("{head}\u{2026}")
}

/// Small pill label (welcome step previews).
fn chip(label: impl Into<String>, dark: bool) -> iced::Element<'static, Message> {
    let c = theme::tok(dark);
    container(text(label.into()).size(tsize::SMALL).color(c.text_muted))
        .padding([space::XS, space::MD])
        .style(theme::pill_style(c.surface_2, c.border))
        .into()
}

/// One-line display for a long path: keeps both ends, drops the middle.
/// Non-path text (sentences) is returned unchanged.
fn short_path(p: &str, max: usize) -> String {
    if !p.contains('/') || p.chars().count() <= max {
        return p.to_string();
    }
    let keep = (max - 1) / 2;
    let head: String = p.chars().take(keep).collect();
    let tail: String = p
        .chars()
        .rev()
        .take(keep)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    format!("{head}…{tail}")
}

/// Monospace chip for a file path.
fn path_chip(path: impl Into<String>, dark: bool) -> iced::Element<'static, Message> {
    container(
        text(path.into())
            .size(tsize::SMALL)
            .font(iced::Font::MONOSPACE)
            .line_height(tsize::LH_NORMAL),
    )
    .padding([space::SM, space::MD])
    .style(theme::chip_style(dark))
    .width(Length::Fill)
    .into()
}

/// One status line: icon (state) + label + the actual detail on the right.
fn status_row(
    icon_name: &'static str,
    label: impl Into<String>,
    detail: impl Into<String>,
    dark: bool,
) -> iced::Element<'static, Message> {
    container(
        row![
            icon(icon_name, isize::MD),
            text(label.into()).size(tsize::BODY),
            iced::widget::Space::new().width(Length::Fill),
            // Always print the detail, so a good row says "Wine 11.0", not "OK".
            text(detail.into()).size(tsize::SMALL),
        ]
        .spacing(space::MD)
        .align_y(Alignment::Center),
    )
    .padding(space::MD)
    .style(theme::card_style(dark))
    .width(Length::Fill)
    .into()
}

/// Validation checklist for a game folder, one row per check.
/// `skip` drops leading rows (the success card skips "Folder", already proven).
fn dir_check_list(dir: &std::path::Path, skip: usize, dark: bool) -> Column<'static, Message> {
    let mut col = column![].spacing(space::SM);
    for c in check_game_dir(dir).into_iter().skip(skip) {
        col = col.push(status_row(
            if c.ok { "check-circle" } else { "x-circle" },
            c.label,
            c.detail,
            dark,
        ));
    }
    col
}

/// One search-result row: status icon + path + Use/In-use button.
fn candidate_row(c: PathBuf, in_use: bool, dark: bool) -> iced::Element<'static, Message> {
    container(
        row![
            icon(if in_use { "check-circle" } else { "hard-drive" }, 15.0),
            text(c.to_string_lossy().into_owned()).size(tsize::SMALL),
            iced::widget::Space::new().width(Length::Fill),
            if in_use {
                button(text("In use").size(tsize::MICRO)).style(theme::btn_ghost(dark))
            } else {
                button(text("Use").size(tsize::MICRO))
                    .on_press(Message::PickCandidate(c))
                    .style(theme::btn_ghost(dark))
            },
        ]
        .spacing(space::SM)
        .align_y(Alignment::Center),
    )
    .padding(space::SM)
    .style(theme::card_style(dark))
    .width(Length::Fill)
    .into()
}

// ── view ─────────────────────────────────────────────────────────────────
impl App {
    pub fn view(&self) -> iced::Element<'_, Message> {
        if self.setup {
            return self.view_setup();
        }
        let dark = self.mode == Mode::Dark;
        let sidebar = self.view_sidebar(dark);
        let content: iced::Element<'_, Message> = match self.page {
            Page::Home => self.view_home(dark),
            Page::Install => self.view_install(dark),
            Page::System => self.view_system(dark),
            Page::Switcher => self.view_switcher(dark),
            Page::Settings => self.view_settings(dark),
            Page::Saves => self.view_saves(dark),
            Page::Help => self.view_help(dark),
        };
        let log_panel = self.view_log(dark);
        let main: Column<Message> = column![
            scrollable(self.page_shell(content)).height(Length::Fill),
            log_panel
        ]
        .width(Length::Fill)
        .spacing(0);
        // overlay the sheet / toast on top of the page
        // the modal takes priority over a toast; only one overlay at a time
        let overlay = if self.modal.is_some() {
            self.modal_layer(dark)
        } else {
            None
        };
        match overlay {
            Some(m) => stack![self.shell_wrap(sidebar, main), m].into(),
            None => match self.toast_layer(dark) {
                Some(t) => stack![self.shell_wrap(sidebar, main), t].into(),
                None => self.shell_wrap(sidebar, main).into(),
            },
        }
    }

    /// Sidebar + main column, as one row (extracted so the overlay path reuses it).
    fn shell_wrap<'a>(
        &'a self,
        sidebar: iced::Element<'a, Message>,
        main: Column<'a, Message>,
    ) -> iced::Element<'a, Message> {
        row![sidebar, main].into()
    }

    /// Centre the page and cap its width so text never runs edge to edge.
    /// Centre the page, and cap its width when there is room for it so text
    /// never runs edge to edge on a wide screen. Below the breakpoint the page
    /// goes fluid instead of overflowing, which is what a Steam Deck gets.
    fn page_shell<'a>(&self, content: iced::Element<'a, Message>) -> iced::Element<'a, Message> {
        // Centred and capped: `max_width` (not a fixed width) lets the page
        // shrink on a small screen instead of overflowing it.
        row![
            Space::new().width(Length::Fill),
            container(content)
                .width(Length::Fill)
                .max_width(tsize::PAGE),
            Space::new().width(Length::Fill),
        ]
        .into()
    }

    /// Sidebar: brand block, nav, theme toggle. The active page is tinted with
    /// the accent colour and its text stays legible in both modes.
    fn view_sidebar(&self, dark: bool) -> iced::Element<'_, Message> {
        let c = theme::tok(dark);
        let ready = self.game_dir.is_some() && self.status.game;
        let (dot, label) = if ready {
            (c.success, "Game ready")
        } else if self.game_dir.is_some() {
            (c.warning, "Game folder set")
        } else {
            (c.text_faint, "No game yet")
        };
        let mut nav: Column<Message> = column![
            // brand: logo + wordmark
            row![
                icon("spfl-logo", isize::XL),
                column![
                    text("SPFL27").size(tsize::H3),
                    text("Playtools").size(tsize::SMALL).color(c.text_muted),
                ]
                .spacing(space::XXS),
            ]
            .spacing(space::MD)
            .align_y(Alignment::Center),
            // one-glance readiness, so Home is not the only place that knows
            row![
                text("●").size(tsize::SMALL).color(dot),
                text(label).size(tsize::SMALL).color(c.text_muted),
            ]
            .spacing(space::SM)
            .align_y(Alignment::Center),
            rule::horizontal(1),
        ]
        .spacing(space::LG)
        .padding([space::LG, space::MD]);

        for p in Page::ALL {
            let active = self.page == p;
            let item = button(
                row![icon(p.icon(), isize::MD), text(p.title()).size(tsize::BODY)]
                    .spacing(space::MD)
                    .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .padding([space::SM, space::MD])
            .style(theme::nav_item(dark, active));
            nav = nav.push(if active {
                item
            } else {
                item.on_press(Message::Nav(p))
            });
        }

        nav = nav.push(Space::new().height(Length::Fill));
        let theme_btn = button(
            row![
                icon(if dark { "moon" } else { "sun" }, isize::MD),
                text(if dark { "Dark theme" } else { "Light theme" }).size(tsize::SMALL),
            ]
            .spacing(space::MD)
            .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .padding([space::SM, space::MD])
        .on_press(Message::ToggleTheme)
        .style(theme::btn_ghost(dark));
        nav = nav.push(theme_btn);
        nav = nav.push(
            text(format!("v{}", env!("CARGO_PKG_VERSION")))
                .size(tsize::MICRO)
                .color(c.text_faint),
        );

        container(nav)
            .width(space::SIDEBAR)
            .height(Length::Fill)
            .style(move |_t: &iced::Theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(theme::tok(dark).bg)),
                border: iced::Border {
                    radius: 0.0.into(),
                    width: 0.0,
                    color: iced::Color::TRANSPARENT,
                },
                text_color: None,
                shadow: iced::Shadow::default(),
                snap: true,
            })
            .into()
    }

    /// Plain-language guide shown at the top of every page.
    /// Centralized here so completeness is unit-tested (see ui_tests).
    fn intro_card(&self, page: Page, dark: bool) -> iced::Element<'_, Message> {
        let (title, body) = page_intro(page);
        container(
            column![
                row![icon(page.icon(), isize::MD), h1(title),]
                    .spacing(space::MD)
                    .align_y(Alignment::Center),
                // prose is capped to ~72 characters so it stays readable
                text(body)
                    .size(tsize::BODY)
                    .line_height(tsize::LH_RELAXED)
                    .width(Length::Fixed(tsize::MEASURE)),
            ]
            .spacing(space::MD),
        )
        .padding(space::LG)
        .style(theme::card_style(dark))
        .width(Length::Fill)
        .into()
    }

    fn view_home(&self, dark: bool) -> iced::Element<'_, Message> {
        let st = &self.status;
        let game_name = self
            .game_dir
            .as_ref()
            .map(|p| short_path(&p.to_string_lossy(), 88))
            .unwrap_or_else(|| "No game folder selected".into());
        let version = self
            .game_dir
            .as_ref()
            .and_then(|g| game_version(g))
            .unwrap_or_else(|| "unknown".to_string());
        let mut col: Column<Message> = column![
            self.intro_card(Page::Home, dark),
            container(
                column![
                    row![
                        icon("gamepad-2", isize::XL),
                        column![
                            row![
                                text("Football Life 2027").size(tsize::H2),
                                text(format!("v{version}")).size(tsize::SMALL),
                            ]
                            .spacing(space::SM)
                            .align_y(Alignment::Center),
                            sub(game_name.clone(), dark),
                        ]
                        .spacing(space::XS),
                        Space::new().width(Length::Fill),
                        button(
                            row![
                                icon("folder-open", isize::SM),
                                text("Change").size(tsize::BODY)
                            ]
                            .spacing(space::SM)
                            .align_y(Alignment::Center),
                        )
                        .on_press(Message::OpenFindGame)
                        .style(theme::btn_ghost(dark)),
                    ]
                    .spacing(space::MD)
                    .align_y(Alignment::Center),
                    button(
                        row![
                            icon("play", isize::LG),
                            text("PLAY  —  Launch Game").size(tsize::BODY_LG)
                        ]
                        .spacing(space::MD)
                        .align_y(Alignment::Center),
                    )
                    .width(Length::Fill)
                    .padding([space::MD, space::MD])
                    .on_press(Message::LaunchGame)
                    .style(theme::btn_primary(dark)),
                ]
                .spacing(space::XS),
            )
            .padding(space::LG)
            .style(theme::card_style(dark))
            .width(Length::Fill),
        ]
        .spacing(space::LG)
        .padding(space::GUTTER);

        // Each row states the good AND the bad case in plain language.
        let cards = [
            ("zap", "Game", st.game, "found", "not found"),
            ("layers", "Sider", st.sider, "running", "not installed"),
            (
                "save",
                "Save",
                st.settings,
                "save file found",
                "not found yet",
            ),
        ];
        // 2x2 grid: readable at 720p, breathable at 1440p.
        let mut grid: Column<Message> = column![].spacing(space::MD);
        for pair in cards.chunks(2) {
            let mut r = row![].spacing(space::MD);
            for (ic, label, ok, good, bad) in
                pair.iter().map(|(a, b, c, d, e)| (*a, *b, *c, *d, *e))
            {
                r = r.push(status_row(ic, label, if ok { good } else { bad }, dark));
            }
            grid = grid.push(r);
        }
        col = col.push(grid);

        if self.game_dir.is_none() {
            col = col.push(
                container(
                    column![
                        row![icon("info", isize::MD), text("No game yet? Start here.").size(tsize::BODY)]
                            .spacing(space::SM)
                            .align_y(Alignment::Center),
                        sub("1. Get SPFL27_setup.exe + update from SmokePatch. 2. Open the Install page and follow steps 1-2-3.".to_string(), dark),
                        button(text("Go to Install").size(tsize::BODY))
                            .on_press(Message::Nav(Page::Install))
                            .style(theme::btn_primary(dark)),
                    ]
                    .spacing(space::SM),
                )
                .padding(space::MD)
                .style(theme::card_style(dark))
                .width(Length::Fill),
            );
        }

        let missing: Vec<&str> = self
            .deps
            .iter()
            .filter(|d| !d.ok && d.level == "required")
            .map(|d| d.key)
            .collect();
        if !missing.is_empty() {
            col = col.push(
                container(
                    row![
                        icon("alert-triangle", isize::MD),
                        text(format!("{} required component(s) missing", missing.len()))
                            .size(tsize::BODY),
                        iced::widget::Space::new().width(Length::Fill),
                        button(text("Open System").size(tsize::SMALL))
                            .on_press(Message::Nav(Page::System))
                            .style(theme::btn_ghost(dark)),
                    ]
                    .spacing(space::MD)
                    .align_y(Alignment::Center),
                )
                .padding(space::MD)
                .style(theme::card_style(dark))
                .width(Length::Fill),
            );
        }
        col.into()
    }

    fn view_install(&self, _dark: bool) -> iced::Element<'_, Message> {
        let dark = self.mode == Mode::Dark;
        let mut col: Column<Message> = column![self.intro_card(Page::Install, dark),]
            .spacing(space::LG)
            .padding(space::GUTTER);

        col = col.push(step_card(
            dark,
            "1",
            "Get the installer",
            "Download SPFL27_setup.exe and the latest update from pessmokepatch.com, then paste the installer path here.",
            column![
                row![
                    text_input("Path to SPFL27_setup.exe …", &self.setup_exe)
                        .on_input(Message::SetupExeChanged)
                        .on_submit(Message::ApplySetupExe)
                        .padding(space::MD)
                        .style(theme::input_style(dark))
                        .width(Length::Fill),
                    button(row![icon("folder-open", isize::SM), text("Browse").size(tsize::BODY)].spacing(space::SM))
                        .on_press(Message::BrowseSetupExe)
                        .style(theme::btn_ghost(dark)),
                ]
                .spacing(space::SM)
                .align_y(Alignment::Center),
            ]
            .spacing(space::SM)
        .into()
        ));

        col = col.push(step_card(
            dark,
            "2",
            "Install through Lutris",
            "Opens Lutris. Then: + → Install from a local file → paste the YAML path (copied automatically).",
            row![
                button(row![icon("copy", isize::SM), text("Copy YAML path").size(tsize::BODY)].spacing(space::SM))
                    .on_press(Message::CopyYaml)
                    .style(theme::btn_ghost(dark)),
                button(row![icon("external-link", isize::SM), text("Open Lutris").size(tsize::BODY)].spacing(space::SM))
                    .on_press(Message::OpenLutris)
                    .style(theme::btn_primary(dark)),
            ]
            .spacing(space::MD)
        .into()
        ));

        col = col.push(step_card(
            dark,
            "3",
            "Apply the update",
            "Close the game first, paste the update file (SPFL27_*.exe), then run it. Check the version on the main menu afterwards.",
            column![
                row![
                    text_input("Path to SPFL27_*.exe (update) …", &self.update_file)
                        .on_input(Message::UpdateFileChanged)
                        .on_submit(Message::RunUpdate)
                        .padding(space::MD)
                        .style(theme::input_style(dark))
                        .width(Length::Fill),
                    button(row![icon("folder-open", isize::SM), text("Browse").size(tsize::BODY)].spacing(space::SM))
                        .on_press(Message::BrowseUpdate)
                        .style(theme::btn_ghost(dark)),
                ]
                .spacing(space::SM)
                .align_y(Alignment::Center),
                button(row![icon("download", isize::SM), text("Run update").size(tsize::BODY)].spacing(space::SM))
                    .on_press(Message::RunUpdate)
                    .style(theme::btn_primary(dark)),
            ]
            .spacing(space::SM)
        .into()
        ));

        col = col.push(
            container(
                column![
                    text("Maintenance").size(tsize::H3),
                    row![
                        button(
                            row![
                                icon("gamepad-2", isize::SM),
                                text("Steam shortcut").size(tsize::BODY)
                            ]
                            .spacing(space::SM)
                        )
                        .on_press(Message::MakeShortcut)
                        .style(theme::btn_ghost(dark)),
                        button(
                            row![
                                icon("trash-2", isize::SM),
                                text("Clear Lutris cache").size(tsize::BODY)
                            ]
                            .spacing(space::SM)
                        )
                        .on_press(Message::ClearCache)
                        .style(theme::btn_ghost(dark)),
                    ]
                    .spacing(space::MD),
                    sub(
                        "The Steam shortcut launches the game directly (--play, no window).",
                        dark
                    ),
                ]
                .spacing(space::SM),
            )
            .padding(space::LG)
            .style(theme::card_style(dark))
            .width(Length::Fill),
        );
        col.into()
    }

    fn view_system(&self, _dark: bool) -> iced::Element<'_, Message> {
        let dark = self.mode == Mode::Dark;
        let missing: Vec<&str> = self.deps.iter().filter(|d| !d.ok).map(|d| d.key).collect();
        let sys_hint = format!(
            "{}  •  package manager: {}  •  missing: {}",
            self.dist,
            if self.pm.is_empty() { "-" } else { &self.pm },
            missing.len()
        );
        let mut col: Column<Message> = column![
            self.intro_card(Page::System, dark),
            path_chip(sys_hint, dark),
        ]
        .spacing(space::LG)
        .padding(space::GUTTER);

        if self.deck {
            col = col.push(note_row(
                dark,
                "Steam Deck detected: Lutris installs via Flatpak; Wine comes with its runners.",
            ));
        }

        let mut list: Column<Message> = column![].spacing(space::SM);
        for d in &self.deps {
            list = list.push(status_row(
                if d.ok { "check-circle" } else { "x-circle" },
                d.label,
                d.detail.clone(),
                dark,
            ));
            let purpose = dep_purpose(d.key);
            if !purpose.is_empty() {
                // indent the purpose line to sit under its row label
                list = list.push(sub(format!("{}    {purpose}", "\u{2002}".repeat(3)), dark));
            }
        }
        col = col.push(
            container(list)
                .padding(space::LG)
                .style(theme::card_style(dark))
                .width(Length::Fill),
        );

        col = col.push(
            container(
                row![
                    button(
                        row![
                            icon("refresh-cw", isize::SM),
                            text("Recheck").size(tsize::BODY)
                        ]
                        .spacing(space::SM)
                    )
                    .on_press(Message::DepsRefresh)
                    .style(theme::btn_ghost(dark)),
                    install_button(dark, missing.len(), self.busy),
                    button(
                        row![
                            icon("copy", isize::SM),
                            text("Copy commands").size(tsize::BODY)
                        ]
                        .spacing(space::SM)
                    )
                    .on_press(Message::CopyInstallCmd)
                    .style(theme::btn_ghost(dark)),
                ]
                .spacing(space::MD),
            )
            .padding(space::MD)
            .style(theme::card_style(dark))
            .width(Length::Fill),
        );
        if self.busy {
            col = col.push(busy_bar(dark, &self.sweep));
        }
        col = col.push(sub(
            "Checked items are skipped automatically. Installing asks for your password.",
            dark,
        ));
        col.into()
    }

    fn view_switcher(&self, dark: bool) -> iced::Element<'_, Message> {
        let game_dir = self.game_dir.as_ref();
        let switcher = game_dir.map(|g| g.join(SWITCHER_EXE));
        let installed = switcher.as_ref().is_some_and(|path| path.is_file());
        let wine = which("wine").is_some();
        let status = if !installed {
            "FL27 Switcher is not installed. Reinstall the full game from SmokePatch."
        } else if wine {
            "FL27 Switcher is installed — change settings, then press Apply and Close."
        } else {
            "FL27 Switcher is installed, but system Wine is missing. On Steam Deck, start SP Football Life 2027 from Lutris and choose FL27 Switcher from the launch options."
        };

        let mut col: Column<Message> = column![
            self.intro_card(Page::Switcher, dark),
            container(
                column![
                    row![
                        icon("check-circle", isize::MD),
                        text("The safest way to change the gameplay settings").size(tsize::H3),
                    ]
                    .spacing(space::MD)
                    .align_y(Alignment::Center),
                    sub(status, dark),
                    button(
                        row![
                            icon("settings", isize::MD),
                            text("Open FL27 Switcher").size(tsize::BODY_LG),
                        ]
                        .spacing(space::SM)
                        .align_y(Alignment::Center),
                    )
                    .on_press(Message::OpenSwitcher)
                    .style(theme::btn_primary(dark)),
                    sub(
                        "Choose Gameplay, Controller Buttons, Classic Teams, Theme or Render Lighting, press Apply, then Close. Restart the game to see the changes.",
                        dark,
                    ),
                ]
                .spacing(space::MD),
            )
            .padding(space::LG)
            .style(theme::card_style(dark))
            .width(Length::Fill),
        ]
        .spacing(space::LG)
        .padding(space::GUTTER);

        if self.switcher_advanced_open {
            if let Some(src) = self.game_dir.as_ref().and_then(|g| cpk_source(g)) {
                let mut card = column![
                    row![
                        text("Advanced: replace game data packs").size(tsize::H3),
                        Space::new().width(Length::Fill),
                        button(text("Close").size(tsize::SMALL))
                            .on_press(Message::ToggleSwitcherAdvanced)
                            .style(theme::btn_ghost(dark)),
                    ]
                    .align_y(Alignment::Center),
                    sub(
                        "Only use this if you downloaded the optional content packs yourself. It replaces the game files for gameplay, controller icons and classic-team modes.",
                        dark,
                    ),
                    sub(
                        "These files are included with the game installation — they are not part of this repository.",
                        dark,
                    ),
                ]
                .spacing(space::MD);

                for (key, label, target) in [
                    ("GP", "Gameplay pack", "Replaces the gameplay data file."),
                    ("Controller", "Controller icons", "Replaces the controller button-icon file."),
                    ("Classics", "Classic career modes", "Replaces the classic-team data file."),
                ] {
                    let opts: Vec<String> = self
                        .sw_opts
                        .iter()
                        .find(|(l, _)| *l == key)
                        .map(|(_, o)| o.clone())
                        .unwrap_or_default();
                    let cur: Option<String> = self
                        .sw_cur
                        .iter()
                        .find(|(l, _)| *l == key)
                        .map(|(_, c)| c.clone())
                        .filter(|c| !c.is_empty());

                    card = card.push(column![
                        text(label).size(tsize::BODY_LG),
                        sub(target, dark),
                        if opts.is_empty() {
                            sub("No options found in the game folder.", dark)
                        } else {
                            container(
                                pick_list(opts, cur, move |c: String| Message::SwitcherPick(key, c))
                                    .placeholder("Choose an option")
                                    .width(Length::Fill),
                            )
                            .width(Length::Fill)
                            .into()
                        },
                    ]
                    .spacing(space::XS));
                }

                card = card.push(
                    button(
                        row![
                            icon("check", isize::SM),
                            text("Apply selected packs").size(tsize::BODY),
                        ]
                        .spacing(space::SM)
                        .align_y(Alignment::Center),
                    )
                    .on_press(Message::SwitcherApply)
                    .style(theme::btn_primary(dark)),
                );
                card = card.push(sub(
                    "Make a backup of your saves first if you changed them since the last match.",
                    dark,
                ));
                card = card.push(sub(
                    format!("Content-pack folder: {}", short_path(&src.to_string_lossy(), 72)),
                    dark,
                ));

                col = col.push(
                    container(card)
                        .padding(space::LG)
                        .style(theme::card_style(dark))
                        .width(Length::Fill),
                );
            } else {
                col = col.push(note_row(
                    dark,
                    "Advanced content packs are not installed. You can use Open FL27 Switcher above for theme, controller icons, gameplay and classic-team modes without them.",
                ));
                col = col.push(
                    button(text("Close advanced section").size(tsize::SMALL))
                        .on_press(Message::ToggleSwitcherAdvanced)
                        .style(theme::btn_ghost(dark)),
                );
            }
        } else {
            col = col.push(
                button(text("Advanced: replace game data packs").size(tsize::SMALL))
                    .on_press(Message::ToggleSwitcherAdvanced)
                    .style(theme::btn_ghost(dark)),
            );
        }

        col.into()
    }

    fn view_settings(&self, dark: bool) -> iced::Element<'_, Message> {
        let game_dir = self.game_dir.as_ref();
        let settings = game_dir.map(|g| g.join(SETTINGS_EXE));
        let installed = settings.as_ref().is_some_and(|p| p.is_file());
        let wine = which("wine").is_some();
        let status = if !installed {
            "FL27 Settings is not installed. Reinstall the full game from SmokePatch."
        } else if wine {
            "FL27 Settings is installed — change the settings you need, then save and close."
        } else {
            "FL27 Settings is installed, but system Wine is missing. On Steam Deck, start SP Football Life 2027 from Lutris and choose FL27 Settings from the launch options."
        };

        let mut col: Column<Message> = column![
            self.intro_card(Page::Settings, dark),
            container(
                column![
                    row![
                        icon("settings", isize::MD),
                        text("The safest way to change game settings").size(tsize::H3),
                    ]
                    .spacing(space::MD)
                    .align_y(Alignment::Center),
                    sub(status, dark),
                    button(
                        row![
                            icon("settings", isize::MD),
                            text("Open FL27 Settings").size(tsize::BODY_LG),
                        ]
                        .spacing(space::SM)
                        .align_y(Alignment::Center),
                    )
                    .on_press(Message::OpenSettings)
                    .style(theme::btn_primary(dark)),
                    sub(
                        "Choose Display, Keyboard, Controller, Audio or Online, save the changes, then close FL27 Settings. Restart the game to see the changes.",
                        dark,
                    ),
                ]
                .spacing(space::MD),
            )
            .padding(space::LG)
            .style(theme::card_style(dark))
            .width(Length::Fill),
        ]
        .spacing(space::LG)
        .padding(space::GUTTER);

        if self.settings_advanced_open {
            let mut card_col: Column<Message> = column![
                row![
                    text("Advanced: edit the game settings file directly").size(tsize::H3),
                    Space::new().width(Length::Fill),
                    button(text("Close").size(tsize::SMALL))
                        .on_press(Message::ToggleSettingsAdvanced)
                        .style(theme::btn_ghost(dark)),
                ]
                .align_y(Alignment::Center),
                path_chip(
                    format!("File: {}", short_path(&self.dat_path, 84)),
                    dark,
                ),
                sub(
                    "Only use this if FL27 Settings cannot fix the problem. A backup is made automatically before the first change.",
                    dark,
                ),
            ]
            .spacing(space::MD);

            let combos = [
                (
                    "controller",
                    "Controller",
                    "Steam and Xbox pads: XInput. PlayStation pads: DirectInput.",
                    vec!["DirectInput".to_string(), "XInput".to_string()],
                    self.set_ctrl.clone(),
                ),
                (
                    "screen",
                    "Screen mode",
                    "Full Screen for matches; Windowed if you multitask.",
                    vec!["Windowed".to_string(), "Full Screen".to_string()],
                    self.set_screen.clone(),
                ),
                (
                    "vsync",
                    "V-Sync",
                    "Enable1 caps 60 fps and reduces heat (recommended).",
                    vec![
                        "Disabled".to_string(),
                        "Enable1".to_string(),
                        "Enable2".to_string(),
                    ],
                    self.set_vsync.clone(),
                ),
            ];
            for (kind, label, caption, opts, cur) in combos {
                let sel = if cur.is_empty() { None } else { Some(cur) };
                card_col = card_col.push(
                    column![
                        text(label).size(tsize::BODY_LG),
                        sub(caption.to_string(), dark),
                        pick_list(opts, sel, move |c: String| Message::SettingPick(kind, c))
                            .placeholder("—")
                            .width(Length::Fill),
                    ]
                    .spacing(space::XS),
                );
            }
            card_col = card_col.push(
                button(
                    row![
                        icon("save", isize::SM),
                        text("Save settings").size(tsize::BODY),
                    ]
                    .spacing(space::SM),
                )
                .on_press(Message::SettingsSave)
                .style(theme::btn_primary(dark)),
            );
            col = col.push(
                container(card_col)
                    .padding(space::LG)
                    .style(theme::card_style(dark))
                    .width(Length::Fill),
            );
        } else {
            col = col.push(
                button(text("Advanced: edit the game settings file directly").size(tsize::SMALL))
                    .on_press(Message::ToggleSettingsAdvanced)
                    .style(theme::btn_ghost(dark)),
            );
        }

        col.into()
    }

    fn view_saves(&self, _dark: bool) -> iced::Element<'_, Message> {
        let dark = self.mode == Mode::Dark;
        let mut col: Column<Message> = column![self.intro_card(Page::Saves, dark),]
            .spacing(space::LG)
            .padding(space::GUTTER);

        let kon = self
            .konami()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| {
                "the game save folder not found yet — launch the game once to create it.".into()
            });
        col = col.push(path_chip(
            format!("Location: {}", short_path(&kon, 84)),
            dark,
        ));

        col = col.push(
            container(
                row![
                    button(
                        row![
                            icon("save", isize::SM),
                            text("Back up now").size(tsize::BODY)
                        ]
                        .spacing(space::SM)
                    )
                    .on_press(Message::BackupNow)
                    .style(theme::btn_primary(dark)),
                    button(
                        row![
                            icon("folder-open", isize::SM),
                            text("Restore…").size(tsize::BODY)
                        ]
                        .spacing(space::SM)
                    )
                    .on_press(Message::OpenRestoreModal)
                    .style(theme::btn_ghost(dark)),
                ]
                .spacing(space::MD),
            )
            .padding(space::MD)
            .style(theme::card_style(dark))
            .width(Length::Fill),
        );

        if !self.backups.is_empty() {
            let mut list: Column<Message> =
                column![text("Available backups").size(tsize::BODY_LG)].spacing(space::SM);
            let home = std::env::var("HOME").unwrap_or_default();
            for b in self.backups.iter().take(8) {
                let full = PathBuf::from(&home)
                    .join("spfl27-backups")
                    .join(b)
                    .to_string_lossy()
                    .into_owned();
                let selected = self.restore_path.trim() == full;
                let reveal_dir = PathBuf::from(&home).join("spfl27-backups").join(b);
                list = list.push(
                    container(
                        row![
                            icon(
                                if selected {
                                    "check-circle"
                                } else {
                                    "hard-drive"
                                },
                                isize::SM,
                            ),
                            text(b).size(tsize::SMALL),
                            iced::widget::Space::new().width(Length::Fill),
                            // reveal the backup in the file manager
                            button(
                                row![
                                    icon("folder-open", isize::SM),
                                    text("Show files").size(tsize::MICRO)
                                ]
                                .spacing(space::XS),
                            )
                            .on_press(Message::RevealPath(reveal_dir))
                            .style(theme::btn_ghost(dark)),
                            button(text("Restore").size(tsize::MICRO))
                                .on_press(Message::PickRestore(full))
                                .style(theme::btn_selectable(dark, selected)),
                        ]
                        .spacing(space::SM)
                        .align_y(Alignment::Center),
                    )
                    .padding(space::SM)
                    .style(theme::card_style(dark))
                    .width(Length::Fill),
                );
            }
            col = col.push(list);
        }

        // Confirmation moved to a modal sheet (see `modal_layer`): the list
        // above now only selects which backup to restore.
        col.into()
    }

    fn view_help(&self, _dark: bool) -> iced::Element<'_, Message> {
        let dark = self.mode == Mode::Dark;
        let mut col: Column<Message> = column![self.intro_card(Page::Help, dark)]
            .spacing(space::LG)
            .padding(space::GUTTER);
        let rows = [
            ("Black screen in Game Mode", "Set ENABLE_GAMESCOPE_WSI=0 (the installer already does this)."),
            ("Controller not detected", "Install the latest update; play via Steam."),
            ("Flickering / missing textures", "Clear the Lutris cache (Install → Maintenance) and reinstall."),
            ("Black bars in menus", "Normal — the aspect fix only applies during matches."),
            ("Where is my data?", "Game folder (Home page, Change button) · switcher sources (Switcher page) · game settings file (Settings page) · saves (Saves page)."),
        ];
        for (k, v) in rows {
            col = col.push(
                container(column![text(k).size(tsize::BODY_LG), sub(v, dark)].spacing(space::XS))
                    .padding(space::MD)
                    .style(theme::card_style(dark))
                    .width(Length::Fill),
            );
        }
        col = col.push(
            container(
                column![
                    text("Keyboard shortcuts").size(tsize::BODY_LG),
                    row![
                        chip("F5", dark),
                        sub("Recheck dependencies on the System page", dark)
                    ]
                    .spacing(space::SM)
                    .align_y(Alignment::Center),
                    row![
                        chip("Ctrl+L", dark),
                        sub("Show or hide the Activity log", dark)
                    ]
                    .spacing(space::SM)
                    .align_y(Alignment::Center),
                    row![chip("Esc", dark), sub("Close any open dialog", dark)]
                        .spacing(space::SM)
                        .align_y(Alignment::Center),
                ]
                .spacing(space::MD),
            )
            .padding(space::MD)
            .style(theme::card_style(dark))
            .width(Length::Fill),
        );
        col = col.push(sub(
            format!(
                "{} v{} — built with Rust and iced. The core logic is covered by automated tests.",
                BRAND,
                env!("CARGO_PKG_VERSION")
            ),
            dark,
        ));
        col = col.push(
            container(
                row![
                    icon("book-open", isize::SM),
                    text("Full guides live in the docs folder.").size(tsize::BODY),
                    Space::new().width(Length::Fill),
                    button(
                        row![
                            icon("folder-open", isize::SM),
                            text("Open guides").size(tsize::BODY)
                        ]
                        .spacing(space::SM)
                        .align_y(Alignment::Center),
                    )
                    .on_press(Message::OpenDocs)
                    .style(theme::btn_ghost(dark)),
                ]
                .spacing(space::MD)
                .align_y(Alignment::Center),
            )
            .padding(space::MD)
            .style(theme::card_style(dark))
            .width(Length::Fill),
        );
        col.into()
    }

    /// Toast rendered over the page content (bottom-centre), via `stack`.
    /// Dismissed after `TOAST_SECS` by the 1s `Tick` subscription.
    fn toast_layer(&self, dark: bool) -> Option<iced::Element<'_, Message>> {
        let (kind, msg) = self.toast.as_ref()?;
        let c = theme::tok(dark);
        let (color, name) = match kind {
            ToastKind::Success => (c.success, "Success"),
            ToastKind::Error => (c.danger, "Error"),
            ToastKind::Info => (c.text_muted, "Info"),
        };
        let card = container(
            row![
                text(name).size(tsize::SMALL).color(color),
                text(msg).size(tsize::SMALL),
                Space::new().width(Length::Fill),
                button(text("\u{2715}").size(tsize::SMALL))
                    .on_press(Message::DismissToast)
                    .style(theme::btn_quiet(dark)),
            ]
            .spacing(space::MD)
            .align_y(Alignment::Center),
        )
        .padding([space::SM, space::MD])
        .max_width(space::SIDEBAR * 4.0)
        .style(move |_t: &iced::Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(c.surface_3)),
            border: iced::Border {
                radius: theme::radius::MD.into(),
                width: 1.0,
                color: color,
            },
            text_color: None,
            shadow: iced::Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.35),
                offset: iced::Vector::new(0.0, 4.0),
                blur_radius: 12.0,
            },
            snap: true,
        });

        Some(
            container(card)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::End)
                .padding(space::XXL)
                .into(),
        )
    }

    /// Modal sheet + scrim, rendered above the page content.
    fn modal_layer(&self, dark: bool) -> Option<iced::Element<'_, Message>> {
        let sheet = match self.modal.as_ref()? {
            Modal::Restore => {
                let src = self.restore_path.trim().to_string();
                let dest = self
                    .konami()
                    .map(|k| k.display().to_string())
                    .unwrap_or_else(|| "Save folder not found".to_string());
                column![
                    row![
                        icon("save", isize::LG),
                        text("Restore save").size(tsize::H2)
                    ]
                    .spacing(space::MD)
                    .align_y(Alignment::Center),
                    sub(
                        "This overwrites your current career and match progress.",
                        dark
                    ),
                    column![
                        row![
                            text("From").size(tsize::SMALL).width(Length::Fixed(64.0)),
                            text(src.clone()).size(tsize::SMALL)
                        ]
                        .spacing(space::MD),
                        row![
                            text("To").size(tsize::SMALL).width(Length::Fixed(64.0)),
                            text(dest).size(tsize::SMALL)
                        ]
                        .spacing(space::MD),
                    ]
                    .spacing(space::XXS),
                    if self.restore_armed {
                        sub("Last chance: this cannot be undone.", dark)
                    } else {
                        Space::new().height(space::XS).into()
                    },
                    row![
                        button(text("Cancel").size(tsize::BODY))
                            .on_press(Message::CloseModal)
                            .style(theme::btn_ghost(dark)),
                        Space::new().width(Length::Fill),
                        button(
                            text(if self.restore_armed {
                                "Overwrite now"
                            } else {
                                "Restore"
                            })
                            .size(tsize::BODY)
                        )
                        .on_press(Message::RestoreConfirm)
                        .style(theme::btn_danger(dark)),
                    ]
                    .spacing(space::MD),
                ]
                .spacing(space::LG)
                .max_width(space::SIDEBAR * 4.0)
            }
            Modal::FindGame => {
                // Same card as wizard step 1, scrollable: search results and the
                // checklist can grow taller than the sheet.
                let inner: iced::Element<'_, Message> = self.find_game_card(dark).into();
                column![
                    row![
                        icon("search", isize::LG),
                        text("Find game").size(tsize::H2),
                        Space::new().width(Length::Fill),
                        button(text("Done").size(tsize::BODY))
                            .on_press(Message::CloseModal)
                            .style(theme::btn_ghost(dark)),
                    ]
                    .spacing(space::MD)
                    .align_y(Alignment::Center),
                    scrollable(inner).height(Length::Fixed(472.0)), // sheet must fit a 620px window
                ]
                .spacing(space::LG)
                .max_width(space::SIDEBAR * 4.0)
            }
        };
        Some(
            container(
                container(sheet)
                    .padding(space::XXL)
                    .style(theme::card_style_with(dark, None, 2))
                    .max_width(space::SIDEBAR * 4.0),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(move |_t: &iced::Theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(Color::from_rgba(
                    0.0, 0.0, 0.0, 0.55,
                ))),
                ..Default::default()
            })
            .into(),
        )
    }

    /// Activity log — a drawer, collapsed by default so it never competes
    /// with the page content. Shows the newest line even when collapsed.
    fn view_log(&self, dark: bool) -> iced::Element<'_, Message> {
        let c = theme::tok(dark);
        let newest = self.logs.last().cloned().unwrap_or_default();
        let mut body: Column<Message> =
            column![].spacing(space::SM).padding([space::SM, space::MD]);
        let mut head: Row<Message> = row![
            icon(
                if self.log_open {
                    "chevron-right"
                } else {
                    "layers"
                },
                isize::SM
            ),
            text("Activity").size(tsize::SMALL),
            Space::new().width(Length::Fill),
        ]
        .spacing(space::SM)
        .align_y(Alignment::Center);
        if !self.log_open && !newest.is_empty() {
            head = head.push(
                text(ellipsis(&newest, 72))
                    .size(tsize::MICRO)
                    .font(iced::Font::MONOSPACE)
                    .color(c.text_faint),
            );
        }
        head = head.push(
            button(text(if self.log_open { "Hide" } else { "Show" }).size(tsize::MICRO))
                .on_press(Message::ToggleLog)
                .style(theme::btn_quiet(dark)),
        );
        body = body.push(head);
        if self.log_open {
            let mut lines: Column<Message> = column![].spacing(space::XS);
            for l in self.logs.iter().rev().take(80).rev() {
                lines = lines.push(text(l).size(tsize::MICRO).font(iced::Font::MONOSPACE));
            }
            body = body.push(
                scrollable(lines)
                    .height(Length::Fixed(88.0))
                    .width(Length::Fill),
            );
        }
        container(body)
            .style(move |_t: &iced::Theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(theme::tok(dark).surface)),
                border: iced::Border {
                    radius: 0.0.into(),
                    width: 1.0,
                    color: theme::tok(dark).border,
                },
                text_color: None,
                shadow: iced::Shadow::default(),
                snap: true,
            })
            .width(Length::Fill)
            .into()
    }

    /// The whole Find-game step as a reusable card: current folder, search,
    /// results, manual pick + live checklist. Used by the wizard AND the
    /// FindGame modal, so both stay identical by construction.
    fn find_game_card<'a>(&self, dark: bool) -> Column<'a, Message> {
        let mut col: Column<Message> = column![].spacing(space::LG);

        // Already-selected game: success card with component checklist.
        if let Some(g) = self.game_dir.clone() {
            let success: Column<Message> = column![
                row![
                    icon("check-circle", isize::MD),
                    text("Game folder set").size(tsize::BODY_LG)
                ]
                .spacing(space::SM)
                .align_y(Alignment::Center),
                path_chip(g.display().to_string(), dark),
                dir_check_list(&g, 1, dark),
            ]
            .spacing(space::SM);
            col = col.push(
                container(success)
                    .padding(space::MD)
                    .style(theme::card_style(dark))
                    .width(Length::Fill),
            );
        }

        // Automatic search.
        let scan_status = if self.scanning {
            "Looking in the usual Lutris places...".to_string()
        } else if self.candidates.is_empty() {
            "No game found yet.".to_string()
        } else {
            format!("{} game(s) listed below.", self.candidates.len())
        };
        col = col.push(
            row![
                button(
                    row![
                        icon("search", isize::SM),
                        text(if self.scanning {
                            "Searching..."
                        } else {
                            "Search automatically"
                        })
                        .size(tsize::BODY)
                    ]
                    .spacing(space::SM)
                )
                .on_press(Message::AutoDetect)
                .style(theme::btn_primary(dark)),
                text(scan_status).size(tsize::SMALL),
            ]
            .spacing(space::MD)
            .align_y(Alignment::Center),
        );

        // Search results: pick one.
        if !self.candidates.is_empty() {
            let mut list: Column<Message> =
                column![text("Found").size(tsize::BODY_LG)].spacing(space::SM);
            for c in self.candidates.clone() {
                let in_use = self.game_dir.as_ref().is_some_and(|g| g == &c);
                list = list.push(candidate_row(c, in_use, dark));
            }
            col = col.push(list);
        }

        // Manual pick: browse the disk or paste a path.
        col = col.push(text("Or choose the folder yourself:").size(tsize::BODY));
        col = col.push(
            row![
                text_input("Paste game folder path ...", &self.manual_path)
                    .on_input(Message::ManualPathChanged)
                    .padding(space::MD)
                    .width(Length::Fill)
                    .style(theme::input_style(dark)),
                button(
                    row![
                        icon("folder-open", isize::SM),
                        text("Browse...").size(tsize::BODY)
                    ]
                    .spacing(space::SM)
                )
                .on_press(Message::BrowseFolder)
                .style(theme::btn_ghost(dark)),
            ]
            .spacing(space::MD)
            .align_y(Alignment::Center),
        );

        // Live validation checklist for the typed path.
        if !self.manual_path.trim().is_empty() {
            let expanded = expand_path(&self.manual_path);
            col = col.push(dir_check_list(&expanded, 0, dark));
        }

        col = col.push(
            button(text("Use this folder").size(tsize::BODY))
                .on_press(Message::ApplyManualPath)
                .style(theme::btn_primary(dark)),
        );
        col = col.push(sub(
            "Tip: the folder name ends with \"SP Football Life 2027\".",
            dark,
        ));
        col
    }

    // ── setup wizard ──
    fn view_setup(&self) -> iced::Element<'_, Message> {
        let dark = self.mode == Mode::Dark;
        let steps = ["Welcome", "Find game", "System", "Ready"];
        let c = theme::tok(dark);
        let mut rail: Row<Message> = row![].spacing(space::SM).align_y(Alignment::Center);
        for (i, s) in steps.iter().enumerate() {
            let done = i < self.setup_step;
            let cur = i == self.setup_step;
            // active step gets an accent pill; done steps a tinted one.
            let (bg, bd, fg) = if cur {
                (c.accent, c.accent, c.on_accent)
            } else if done {
                (c.surface_3, c.accent, c.text)
            } else {
                (c.surface_2, c.border, c.text_faint)
            };
            rail = rail.push(
                container(
                    row![
                        text(if done {
                            "\u{2713}".to_string()
                        } else {
                            format!("{}", i + 1)
                        })
                        .size(tsize::SMALL)
                        .color(fg),
                        text(*s).size(tsize::SMALL).color(fg),
                    ]
                    .spacing(space::SM)
                    .align_y(Alignment::Center),
                )
                .padding([space::SM, space::MD])
                .style(theme::pill_style(bg, bd)),
            );
            if i + 1 < steps.len() {
                rail = rail.push(Space::new().width(space::XXS));
            }
        }

        let body: iced::Element<'_, Message> = match self.setup_step {
            0 => {
                let (t, b) = setup_intro(0);
                column![
                    icon("spfl-logo", isize::HERO),
                    text(t).size(tsize::DISPLAY).line_height(tsize::LH_TIGHT),
                    prose(b, dark),
                    row![
                        chip("1  Find game", dark),
                        chip("2  Fix system", dark),
                        chip("3  Press PLAY", dark),
                    ]
                    .spacing(space::SM),
                    path_chip(
                        "Needed from pessmokepatch.com: SPFL27_setup.exe + latest update",
                        dark
                    ),
                ]
                .spacing(space::HUGE)
                .align_x(Alignment::Center)
                .into()
            }
            1 => {
                let (t, b) = setup_intro(1);
                let col: Column<Message> = column![
                    text(t).size(tsize::H1).line_height(tsize::LH_TIGHT),
                    prose(b, dark),
                    self.find_game_card(dark),
                ]
                .spacing(space::LG);
                return col.into();
            }
            2 => {
                let missing: Vec<&str> =
                    self.deps.iter().filter(|d| !d.ok).map(|d| d.key).collect();
                let (t, b) = setup_intro(2);
                column![
                    text(t).size(tsize::H1).line_height(tsize::LH_TIGHT),
                    prose(b, dark),
                    sub(
                        format!(
                            "{}  •  missing: {} (only these will be installed)",
                            self.dist,
                            missing.len()
                        ),
                        dark,
                    ),
                    install_button(dark, missing.len(), self.busy),
                    if self.busy {
                        busy_bar(dark, &self.sweep)
                    } else {
                        Space::new().height(Length::Shrink).into()
                    },
                ]
                .spacing(space::LG)
                .into()
            }
            _ => {
                let (t, b) = setup_intro(3);
                column![
                    icon("check-circle", isize::HERO),
                    text(t).size(tsize::DISPLAY).line_height(tsize::LH_TIGHT),
                    prose(b, dark),
                ]
                .spacing(space::XXXL)
                .align_x(Alignment::Center)
                .into()
            }
        };

        let back: iced::Element<'_, Message> = if self.setup_step > 0 {
            button(
                row![
                    icon("arrow-left", isize::SM),
                    text("Back").size(tsize::BODY)
                ]
                .spacing(space::SM),
            )
            .on_press(Message::SetupBack)
            .style(theme::btn_ghost(dark))
            .into()
        } else {
            Space::new().width(Length::Shrink).into()
        };
        let nav = row![
            back,
            iced::widget::Space::new().width(Length::Fill),
            button(
                row![
                    text(if self.setup_step == 3 {
                        "Finish"
                    } else {
                        "Continue"
                    })
                    .size(tsize::BODY),
                    icon("chevron-right", isize::SM)
                ]
                .spacing(space::SM)
            )
            .on_press(Message::SetupNext)
            .style(theme::btn_primary(dark)),
        ]
        .align_y(Alignment::Center);

        // Progress bar: thin, rounded. Colour comes from the theme accent.
        let progress = progress_bar(0.0..=1.0, self.setup_step as f32 / 3.0).girth(4.0);

        container(
            column![
                // top bar
                container(
                    row![
                        icon("spfl-logo", isize::LG),
                        column![
                            text("SP Football Life 2027").size(tsize::H3),
                            text("Setup").size(tsize::MICRO).color(c.text_muted),
                        ]
                        .spacing(space::XXS),
                        Space::new().width(Length::Fill),
                        button(icon(if dark { "moon" } else { "sun" }, isize::MD))
                            .on_press(Message::ToggleTheme)
                            .style(theme::btn_ghost(dark)),
                    ]
                    .spacing(space::MD)
                    .align_y(Alignment::Center),
                )
                .height(Length::Fixed(isize::BAR))
                .padding([0.0, space::GUTTER])
                .style(move |_t: &iced::Theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(theme::tok(dark).surface)),
                    border: iced::Border {
                        radius: 0.0.into(),
                        width: 1.0,
                        color: theme::tok(dark).border,
                    },
                    text_color: None,
                    shadow: iced::Shadow::default(),
                    snap: true,
                }),
                rail,
                progress,
                container(body)
                    .padding(space::XXL)
                    .style(theme::card_style(dark))
                    .width(Length::Fill),
                nav,
                self.wizard_log(dark),
            ]
            .spacing(space::LG)
            .padding([space::LG, space::GUTTER]),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    /// Collapsed-by-default activity card inside the wizard.
    fn wizard_log(&self, dark: bool) -> iced::Element<'_, Message> {
        let c = theme::tok(dark);
        let newest = self.logs.last().cloned().unwrap_or_default();
        let mut head: Row<Message> = row![
            icon(
                if self.log_open {
                    "chevron-right"
                } else {
                    "layers"
                },
                isize::SM
            ),
            text("Activity").size(tsize::SMALL),
            Space::new().width(Length::Fill),
        ]
        .spacing(space::SM)
        .align_y(Alignment::Center);
        if !self.log_open && !newest.is_empty() {
            head = head.push(
                text(ellipsis(&newest, 72))
                    .size(tsize::MICRO)
                    .font(iced::Font::MONOSPACE)
                    .color(c.text_faint),
            );
        }
        head = head.push(
            button(text(if self.log_open { "Hide" } else { "Show" }).size(tsize::MICRO))
                .on_press(Message::ToggleLog)
                .style(theme::btn_quiet(dark)),
        );
        let mut body: Column<Message> = column![head].spacing(space::SM);
        if self.log_open {
            let mut lines: Column<Message> = column![].spacing(space::XS);
            for l in self.logs.iter().rev().take(30).rev() {
                lines = lines.push(text(l).size(tsize::MICRO).font(iced::Font::MONOSPACE));
            }
            body = body.push(
                scrollable(lines)
                    .height(Length::Fixed(88.0))
                    .width(Length::Fill),
            );
        }
        container(body)
            .padding(space::MD)
            .style(theme::card_style(dark))
            .width(Length::Fill)
            .into()
    }
}

/// Fast first paint for the Find-game step: known spots only (no disk scan).
fn initial_candidates(current: Option<&std::path::Path>) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = candidate_dirs()
        .into_iter()
        .filter(|p| is_game_dir(p))
        .collect();
    if let Some(g) = current {
        if is_game_dir(g) && !out.contains(&g.to_path_buf()) {
            out.push(g.to_path_buf());
        }
    }
    out.sort();
    out.dedup();
    out
}

/// Indeterminate progress bar. The sweep is animated by the 1s tick; a fixed
/// value would look frozen, so the bar reads as "working" rather than stuck.
fn busy_bar(
    dark: bool,
    sweep: &iced::animation::Animation<f32>,
) -> iced::Element<'static, Message> {
    let now = std::time::Instant::now();
    let v = sweep
        .interpolate_with(|t| t * 0.70 + 0.15, now)
        .clamp(0.0, 1.0);
    container(progress_bar(0.0..=1.0, v).girth(4.0).length(Length::Fill))
        .padding([space::XS, 0.0])
        .style(move |_t: &iced::Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(theme::tok(dark).surface)),
            ..Default::default()
        })
        .into()
}

fn install_button(dark: bool, missing: usize, busy: bool) -> iced::Element<'static, Message> {
    let label = if busy {
        "Installing…".to_string()
    } else {
        format!("Install missing ({missing})")
    };
    let btn =
        button(row![icon("download", isize::SM), text(label).size(tsize::BODY)].spacing(space::SM))
            .style(theme::btn_primary(dark));
    if busy {
        btn.into()
    } else {
        btn.on_press(Message::DepsInstall).into()
    }
}

fn note_row(dark: bool, msg: impl Into<String>) -> iced::Element<'static, Message> {
    let msg = msg.into();
    container(
        row![icon("info", isize::MD), text(msg).size(tsize::SMALL)]
            .spacing(space::SM)
            .align_y(Alignment::Center),
    )
    .padding(space::MD)
    .style(theme::card_style(dark))
    .width(Length::Fill)
    .into()
}

fn step_card<'a>(
    dark: bool,
    n: &'a str,
    title: &'a str,
    hint: &'a str,
    body: iced::Element<'a, Message>,
) -> iced::Element<'a, Message> {
    container(
        column![
            row![
                container(text(n).size(tsize::BODY))
                    .padding([space::XXS, space::SM])
                    .style(theme::pill_style(
                        theme::tok(dark).surface_2,
                        theme::tok(dark).border
                    )),
                text(title).size(tsize::H3),
            ]
            .spacing(space::MD)
            .align_y(Alignment::Center),
            sub(hint, dark),
            body,
        ]
        .spacing(space::SM),
    )
    .padding(space::LG)
    .style(theme::card_style(dark))
    .width(Length::Fill)
    .into()
}

#[cfg(test)]
mod ui_tests {
    use super::*;

    /// Tests share one real config file — serialize the ones that write it.
    static CONFIG_GUARD: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn lock_config() -> std::sync::MutexGuard<'static, ()> {
        CONFIG_GUARD.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Backs up the real config file and restores it on drop — even when the
    /// test panics. Hold the return value for the whole test.
    struct ConfigBackup {
        _lock: std::sync::MutexGuard<'static, ()>,
        path: std::path::PathBuf,
        before: Option<Vec<u8>>,
    }

    impl ConfigBackup {
        fn take() -> Self {
            let lock = lock_config();
            let path = config_path();
            let before = std::fs::read(&path).ok();
            Self {
                _lock: lock,
                path,
                before,
            }
        }
    }

    impl Drop for ConfigBackup {
        fn drop(&mut self) {
            if let Some(b) = &self.before {
                if let Some(parent) = self.path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let _ = std::fs::write(&self.path, b);
            } else {
                let _ = std::fs::remove_file(&self.path);
            }
        }
    }

    #[test]
    fn theme_toggle_persists() {
        let _guard = lock_config();
        let (mut app, _) = App::new();
        let start = app.mode;
        let _ = app.update(Message::ToggleTheme);
        assert_ne!(app.mode, start);
        let mid = match app.mode {
            Mode::Dark => "dark",
            Mode::Light => "light",
        };
        let cfg = load_config();
        assert_eq!(cfg.get("theme").map(|s| s.as_str()), Some(mid));
        let _ = app.update(Message::ToggleTheme);
        assert_eq!(app.mode, start);
    }

    #[test]
    fn setup_wizard_appears_without_game() {
        let _guard = lock_config();
        // Isolate the "first run" scenario from the developer's real HOME:
        // no saved config and no game under the standard default paths.
        let home = std::env::temp_dir().join(format!("spfl27-no-game-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&home);
        let old_home = std::env::var_os("HOME");
        let old_xdg = std::env::var_os("XDG_CONFIG_HOME");
        std::env::set_var("HOME", &home);
        std::env::set_var("XDG_CONFIG_HOME", home.join(".config"));
        let built = std::panic::catch_unwind(std::panic::AssertUnwindSafe(App::new));
        match old_home {
            Some(v) => std::env::set_var("HOME", v),
            None => std::env::remove_var("HOME"),
        }
        match old_xdg {
            Some(v) => std::env::set_var("XDG_CONFIG_HOME", v),
            None => std::env::remove_var("XDG_CONFIG_HOME"),
        }
        let (app, _) = built.expect("App::new must not panic without game");
        assert!(app.setup, "wizard must show on first run without game");
        assert_eq!(app.setup_step, 0);
        let _ = std::fs::remove_dir_all(home);
    }

    #[test]
    fn wizard_advances_and_finishes() {
        let (mut app, _) = App::new();
        app.setup = true;
        app.setup_step = 0;
        let _ = app.update(Message::SetupNext);
        assert_eq!(app.setup_step, 1);
        // gate: step 1 needs a game folder — fake one for the test
        app.game_dir = Some(std::path::PathBuf::from("/tmp/spfl27-test-game"));
        let _ = app.update(Message::SetupNext);
        assert_eq!(app.setup_step, 2);
        let _ = app.update(Message::SetupNext);
        assert!(!app.setup, "wizard must close after last step");
        assert_eq!(app.page, Page::Home);
    }

    #[test]
    fn wizard_step1_blocked_without_game() {
        let (mut app, _) = App::new();
        app.setup = true;
        app.setup_step = 1;
        app.game_dir = None;
        let _ = app.update(Message::SetupNext);
        assert_eq!(
            app.setup_step, 1,
            "must not advance past step 1 without a game"
        );
    }

    #[test]
    fn wizard_back_never_negative() {
        let (mut app, _) = App::new();
        app.setup = true;
        app.setup_step = 0;
        let _ = app.update(Message::SetupBack);
        assert_eq!(app.setup_step, 0);
    }

    #[test]
    fn nav_switches_pages() {
        let (mut app, _) = App::new();
        for p in Page::ALL {
            let _ = app.update(Message::Nav(p));
            assert_eq!(app.page, p);
        }
    }

    #[test]
    fn restore_requires_double_confirm() {
        let (mut app, _) = App::new();
        app.restore_armed = false;
        // without a game folder, restore stops before arming
        let _ = app.update(Message::RestoreConfirm);
        assert!(!app.restore_armed);
    }

    #[test]
    fn all_pages_have_intros() {
        for p in Page::ALL {
            let (t, b) = page_intro(p);
            assert!(!t.is_empty(), "{p:?} missing title");
            assert!(b.len() > 80, "{p:?} guide too short to be useful");
            assert!(!b.contains("TODO"), "{p:?} has placeholder");
        }
    }

    #[test]
    fn all_setup_steps_have_intros() {
        for step in 0..=3 {
            let (t, b) = setup_intro(step);
            assert!(!t.is_empty(), "step {step} missing title");
            assert!(b.len() > 80, "step {step} guide too short");
        }
    }

    #[test]
    fn short_path_keeps_short_strings_and_truncates_long_paths() {
        // short strings and non-path prose pass through unchanged
        assert_eq!(short_path("/games", 88), "/games");
        assert_eq!(
            short_path("Choose the game folder.", 88),
            "Choose the game folder."
        );
        let long = "/run/media/alfarez/DataLinux/Games/sp-football-life-2027/drive_c/Program Files (x86)/SP Football Life 2027";
        let t = short_path(long, 88);
        assert!(t.chars().count() <= 88);
        assert!(t.starts_with("/run/media/alfarez/DataLinux"));
        assert!(t.ends_with("SP Football Life 2027"));
        assert!(t.contains('…'));
    }

    /// Token discipline: view code must use theme tokens, not magic numbers.
    /// Guards the redesign from regressing back to hand-picked sizes.
    #[test]
    fn ui_uses_design_tokens_not_magic_numbers() {
        const METHODS: [&str; 3] = [".size(", ".spacing(", ".padding("];
        let mut offenders: Vec<String> = Vec::new();
        for (n, line) in include_str!("ui.rs").lines().enumerate() {
            let line = line.trim();
            if line.starts_with("//") || line.contains("Length::") {
                continue;
            }
            for m in METHODS {
                let Some(rest) = line.split(m).nth(1) else {
                    continue;
                };
                let arg = rest
                    .trim_start()
                    .trim_start_matches('(')
                    .trim_start_matches('[')
                    .trim_start();
                // a bare 0 is not a magic number (flush edge-to-edge)
                let nonzero = arg
                    .split(|c: char| !(c.is_ascii_digit() || c == '.'))
                    .next()
                    .and_then(|t| t.parse::<f32>().ok())
                    .is_some_and(|v| v != 0.0);
                if nonzero {
                    offenders.push(format!("line {}: {m}{rest}", n + 1));
                }
            }
        }
        assert!(
            offenders.is_empty(),
            "raw magic numbers left in UI:\n{}",
            offenders.join("\n")
        );
    }

    #[test]
    fn intros_are_english_only() {
        // spot-check: common Indonesian words must not appear in UI guides
        let id_words = [
            " yang ", " dan ", " untuk ", " dengan ", " belum ", " sudah ",
        ];
        for p in Page::ALL {
            let (_, b) = page_intro(p);
            let low = format!(" {b} ").to_lowercase();
            for w in id_words {
                assert!(!low.contains(w), "{p:?} contains Indonesian: {w}");
            }
        }
        for step in 0..=3 {
            let (_, b) = setup_intro(step);
            let low = format!(" {b} ").to_lowercase();
            for w in id_words {
                assert!(
                    !low.contains(w),
                    "setup step {step} contains Indonesian: {w}"
                );
            }
        }
    }

    /// Build a synthetic KeyPressed event for shortcut tests.
    fn key_press(key: iced::keyboard::Key, ctrl: bool) -> Message {
        let mut modifiers = iced::keyboard::Modifiers::default();
        if ctrl {
            modifiers = modifiers | iced::keyboard::Modifiers::CTRL;
        }
        Message::KeyPressed(iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key,
            physical_key: iced::keyboard::key::Physical::Unidentified(
                iced::keyboard::key::NativeCode::Unidentified,
            ),
            location: iced::keyboard::Location::Standard,
            modifiers,
            text: None,
            repeat: false,
        }))
    }

    #[test]
    fn keyboard_shortcuts_toggle_log_and_recheck() {
        let (mut app, _) = App::new();
        let start = app.log_open;
        let _ = app.update(key_press(iced::keyboard::Key::Character("l".into()), true));
        assert_eq!(app.log_open, !start, "Ctrl+L must toggle the log");
        let _ = app.update(key_press(iced::keyboard::Key::Character("l".into()), true));
        assert_eq!(app.log_open, start, "Ctrl+L twice must restore it");

        // plain "l" without Ctrl must do nothing
        let _ = app.update(key_press(iced::keyboard::Key::Character("l".into()), false));
        assert_eq!(app.log_open, start);

        // F5 rechecks without crashing and leaves a log trail
        let n = app.logs.len();
        let _ = app.update(key_press(
            iced::keyboard::Key::Named(iced::keyboard::key::Named::F5),
            false,
        ));
        assert!(app.logs.len() > n, "F5 must log the recheck");
    }

    #[test]
    fn open_find_game_prefills_and_modal_closes_on_pick() {
        let (mut app, _) = App::new();
        app.setup = false;
        let _ = app.update(Message::OpenFindGame);
        assert_eq!(app.modal, Some(Modal::FindGame));
        // invalid pick keeps the modal open so the error stays visible
        app.manual_path = "/tmp/spfl27-ui-definitely-missing-xyz".into();
        let _ = app.update(Message::ApplyManualPath);
        assert_eq!(app.modal, Some(Modal::FindGame));
    }

    #[test]
    fn pick_restore_opens_modal_for_that_backup() {
        let (mut app, _) = App::new();
        app.backups = vec!["KONAMI-backup-20270901-120000".to_string()];
        let home = std::env::var("HOME").unwrap_or_default();
        let want = format!("{}/spfl27-backups/KONAMI-backup-20270901-120000", home);
        let _ = app.update(Message::PickRestore(want.clone()));
        assert_eq!(app.restore_path, want);
        assert_eq!(app.modal, Some(Modal::Restore));
        assert!(!app.restore_armed, "a fresh pick must not be pre-armed");
    }

    #[test]
    fn toast_and_modal_lifecycle() {
        let (mut app, _) = App::new();

        // toast shows and can be dismissed
        let _ = app.update(Message::ShowToast(ToastKind::Success, "Done".into()));
        assert!(app.toast.is_some());
        assert!(matches!(app.toast, Some((ToastKind::Success, _))));
        let _ = app.update(Message::DismissToast);
        assert!(app.toast.is_none());

        // the modal opens, closes on CloseModal, and restores nothing on its own
        let _ = app.update(Message::OpenRestoreModal);
        assert_eq!(app.modal, Some(Modal::Restore));
        let _ = app.update(Message::CloseModal);
        assert!(app.modal.is_none());
        assert!(!app.restore_armed);

        // Escape while the modal is up must close it without restoring
        let _ = app.update(Message::OpenRestoreModal);
        let _ = app.update(Message::KeyPressed(iced::Event::Keyboard(
            iced::keyboard::Event::KeyPressed {
                key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
                modified_key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
                physical_key: iced::keyboard::key::Physical::Unidentified(
                    iced::keyboard::key::NativeCode::Unidentified,
                ),
                location: iced::keyboard::Location::Standard,
                modifiers: iced::keyboard::Modifiers::default(),
                text: None,
                repeat: false,
            },
        )));
        assert!(app.modal.is_none(), "Escape must close the modal");
    }

    #[test]
    fn find_game_flow_accept_reject_browse_scan() {
        // apply paths persist game_dir into the real config file — back it
        // up with RAII so a panic mid-test cannot dirty the user config.
        let _cfg = ConfigBackup::take();

        let root = std::env::temp_dir().join(format!("spfl27-ui-find-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let game = root.join("SP Football Life 2027");
        std::fs::create_dir_all(&game).unwrap();
        std::fs::write(game.join("FL_2027.exe"), b"x").unwrap();
        let empty = root.join("empty");
        std::fs::create_dir_all(&empty).unwrap();

        // 1. valid pasted path is accepted and advances the wizard
        let (mut app, _) = App::new();
        app.setup = true;
        app.setup_step = 1;
        app.game_dir = None;
        app.manual_path = game.to_string_lossy().into_owned();
        let _ = app.update(Message::ApplyManualPath);
        assert_eq!(app.game_dir, Some(game.clone()));
        assert_eq!(app.setup_step, 2);
        assert!(app.logs.last().is_some_and(|l| l.contains("accepted")));

        // 2. folder without the game exe is rejected with a reason
        let (mut app, _) = App::new();
        app.setup = true;
        app.setup_step = 1;
        app.game_dir = None;
        app.manual_path = empty.to_string_lossy().into_owned();
        let _ = app.update(Message::ApplyManualPath);
        assert!(app.game_dir.is_none());
        assert_eq!(app.setup_step, 1);
        assert!(app.logs.last().is_some_and(|l| l.contains("No game file")));

        // 3. missing folder is rejected with a reason
        app.manual_path = root.join("does-not-exist").to_string_lossy().into_owned();
        let _ = app.update(Message::ApplyManualPath);
        assert!(app.game_dir.is_none());
        assert!(app
            .logs
            .last()
            .is_some_and(|l| l.contains("does not exist")));

        // 4. surrounding quotes are stripped before validation
        app.manual_path = format!("\"{}\" ", empty.display());
        let _ = app.update(Message::ApplyManualPath);
        assert!(app.game_dir.is_none());
        assert!(app.logs.last().is_some_and(|l| l.contains("No game file")));

        // 5. folder-picker result is applied the same way
        let _ = app.update(Message::BrowsePicked(Some(game.clone())));
        assert_eq!(app.game_dir, Some(game.clone()));
        assert_eq!(app.manual_path, game.to_string_lossy());

        // 6. picker cancel changes nothing
        let _ = app.update(Message::BrowsePicked(None));
        assert_eq!(app.game_dir, Some(game.clone()));

        // 6b. valid pick from the FindGame modal closes the modal
        // (modal only opens outside the wizard, so leave setup mode first)
        app.setup = false;
        let _ = app.update(Message::OpenFindGame);
        assert_eq!(app.modal, Some(Modal::FindGame));
        app.manual_path = game.to_string_lossy().into_owned();
        let _ = app.update(Message::ApplyManualPath);
        assert_eq!(app.game_dir, Some(game.clone()));
        assert!(app.modal.is_none(), "modal must close on success");

        // 7. a single search result is auto-selected
        let (mut app2, _) = App::new();
        app2.setup = true;
        app2.setup_step = 1;
        app2.game_dir = None;
        app2.scanning = true;
        let _ = app2.update(Message::ScanDone(vec![game.clone()]));
        assert!(!app2.scanning);
        assert_eq!(app2.game_dir, Some(game.clone()));
        assert_eq!(app2.setup_step, 2);

        // 8. empty search keeps the step and explains what to do
        let _ = app2.update(Message::ScanDone(vec![]));
        assert!(app2.candidates.is_empty());
        assert!(app2
            .logs
            .last()
            .is_some_and(|l| l.contains("no game found")));

        // 9. candidate pick adopts a valid folder
        let (mut app3, _) = App::new();
        app3.setup = true;
        app3.setup_step = 1;
        app3.game_dir = None;
        let _ = app3.update(Message::PickCandidate(game.clone()));
        assert_eq!(app3.game_dir, Some(game.clone()));

        let _ = std::fs::remove_dir_all(&root);
    }
}
