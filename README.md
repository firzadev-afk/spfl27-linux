# SP Football Life 2027 for Linux

Installer and Playtools package for **SP Football Life 2027** (SmokePatch) on Linux and
Steam Deck. It ships a native Rust launcher (iced), a Lutris installer script, one
Sider Lua module, and CPK content packs (classic teams, controllers, gameplay).

This project is **safe to use and fully auditable**: all scripts are plain text, all
Rust code is open and compiles from source, and every step is verifiable. If anything
is unclear, ask an AI agent (e.g. OpenCode) — it can read this repository and run
the checks for you.

## What you need (prerequisites)

| Component | Minimum | Check command |
|-----------|---------|---------------|
| Rust | 1.88 | `rustc --version` |
| Lutris | 0.5.20 | `lutris --version` |
| Wine | 10.0 | `wine --version` |
| Winetricks | any | `winetricks --version` |
| Vulkan drivers | any | `vulkaninfo --summary` (optional) |

Arch/CachyOS one-liner:

```bash
sudo pacman -S rust lutris wine winetricks vulkan-tools
```

## Pre-install checklist

Before installing, verify:

1. `rustc --version` prints 1.88 or newer.
2. `lutris --version` prints 0.5.20 or newer.
3. `wine --version` prints 10.0 or newer.
4. `winetricks --version` works.
5. You have the two SmokePatch files: `SPFL27_setup.exe` and the latest update.
6. At least 80 GB of free space on the target disk (game ~23 GB + facepacks ~42 GB).
7. The target disk is mounted and writable.

## Layout

- `native/` — Rust app `spfl27` (`src/`, `tests/e2e.rs`, `assets/`, `run.sh`)
- `lutris/` — `sp-football-life-2027.yaml` installer + `spfl2027-fix.sh` + `spfl2027-stop.sh`
- `sider/modules/` — `aspectratio.lua`
- `data/cpk/` — `Classics/`, `Controller/{ps3,ps4,ps5,xbox}/`, `GP/`
  (empty placeholders — real CPKs are downloaded separately and not committed)

## Build and test (Playtools app)

```bash
cd native && cargo build --release     # build the spfl27 app
./native/run.sh                        # build if stale, then run
cd native && cargo test                 # run the e2e suite (31 + 19 + 5 tests)
cd native && cargo clippy --all-targets # lint (6 pre-existing style warnings)
```

## Install the game

```bash
lutris -i lutris/sp-football-life-2027.yaml
```

The installer does, in order:

1. Create the Wine prefix.
2. Install .NET Framework 4.8.
3. Install .NET Framework 8.0.
4. Install `vcrun2022`, `unifont`, `win10`.
5. Run the game installer (you point Lutris at `SPFL27_setup.exe`).
6. Rename `xinput1_3.dll` to `ddraw.dll` (backup kept as `xinput1_3.dll.bak`).
7. Map `Segoe UI` / `Segoe UI Semibold` fonts to a system font (for the WPF Switcher).
8. Run the gameplay Switcher once.
9. Apply the aspect-ratio fix (`aspectratio.lua`).
10. Write `FL_2027.bat`.

## How to use

- **Play:** menu entry `SP Football Life 2027` (or `~/.local/bin/spfl2027`).
  Sider starts first, then the game. Only the game window opens.
- **Settings:** menu entry `FL27 Settings` — display, controller, audio, etc.
  Set VSync to `Enable1` to cap 60 fps and reduce heat.
- **Switcher:** menu entry `FL27 Switcher` — gameplay, controller buttons, classic
  teams, lockdown, replay logo.
- **Stop a stuck game:** `spfl2027-stop` (or Ctrl+C in the launcher terminal).
- **Health check:** `./lutris/spfl2027-fix.sh check` (exit 0 = healthy).
- **Test notes:** `TEST_NOTES.md` (OS, device, settings, FPS, temperature).

## Tool repair (Settings / Switcher)

If Settings or Switcher fail to open:

```bash
./lutris/spfl2027-fix.sh check    # check only, change nothing
./lutris/spfl2027-fix.sh         # check and repair what is missing
```

Checks: tool files present, Windows 10 prefix, .NET 4.8 (registry `Release` >= 528040),
`Segoe UI` font mapping, `ddraw.dll`, DXVK. Idempotent and safe to run repeatedly.

Locate a non-standard prefix with `SPFL2027_PREFIX=/path/to/prefix`.

## Safety and verification

- All scripts are plain bash; read them before running.
- All Rust code compiles from source; run `cargo test` to verify (31 + 19 + 5 tests).
- The installer YAML is plain text; inspect `lutris/sp-football-life-2027.yaml`.
- No binary blobs are executed except the game's own installers from SmokePatch.
- Backups are kept: `xinput1_3.dll.bak`, `download-backup/`.
- If something breaks, restore from the backups or re-run `spfl2027-fix.sh`.

## If you are confused

Ask an AI agent (OpenCode, Claude, etc.). It can read this repository, run
`cargo test`, run `./lutris/spfl2027-fix.sh check`, and explain any step. Point it
at `README.md`, `INSTALL.md`, and `AGENTS.md`.

## Agent entry points

- Health: `./lutris/spfl2027-fix.sh check` (exit 0 = healthy).
- Repair: `./lutris/spfl2027-fix.sh`.
- Stop: `spfl2027-stop`.
- Installer: `lutris/sp-football-life-2027.yaml`.
- Details: `INSTALL.md`, `AGENTS.md`.