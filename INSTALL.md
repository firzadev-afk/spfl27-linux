# Installation Guide

Detailed step-by-step guide for installing SP Football Life 2027 on Linux and
Steam Deck, and for keeping the Windows tools (Settings and Switcher) working.

## 1. Prerequisites

- Rust >= 1.88 (only for building the Playtools app).
- Lutris >= 0.5.20 and Wine >= 10.
- Winetricks.
- Vulkan drivers (required by DXVK).

Verify before starting:

```bash
rustc --version
lutris --version
wine --version
winetricks --version
vulkaninfo --summary   # optional but recommended
```

Arch/CachyOS:

```bash
sudo pacman -S rust lutris wine winetricks vulkan-tools
```

## 2. Get the project

```bash
git clone <repo-url> sp-football-life-2027-linux
cd sp-football-life-2027-linux
```

## 3. Build the Playtools app (optional)

```bash
cd native && cargo build --release
./native/run.sh            # builds if stale, then runs
cd native && cargo test    # run the e2e suite (31 + 19 + 5 tests)
cd native && cargo clippy --all-targets
```

## 4. Install the game

You need two files from SmokePatch (pessmokepatch.com):
- the base installer `SPFL27_setup.exe`
- the latest update file

Then:

```bash
lutris -i lutris/sp-football-life-2027.yaml
```

The installer script performs these steps in order:

1. Create the Wine prefix.
2. Install .NET Framework 4.8.
3. Install .NET Framework 8.0.
4. Install `vcrun2022`, `unifont`, `win10`.
5. Run the game installer (point Lutris at `SPFL27_setup.exe`).
6. Rename `xinput1_3.dll` to `ddraw.dll` (backup kept as `xinput1_3.dll.bak`).
7. Map the `Segoe UI` and `Segoe UI Semibold` fonts to a system font (for the
   WPF Switcher).
8. Run the gameplay Switcher once.
9. Apply the aspect-ratio fix (copies `aspectratio.lua` when needed).
10. Write `FL_2027.bat`.

After install the prefix lives at the path chosen by Lutris (default
`~/Games/sp-football-life-2027`).

## 5. Tool health check and repair

Two Windows tools ship with the game. They break on a plain Wine prefix because
of missing .NET and fonts. Run the repair script:

```bash
./lutris/spfl2027-fix.sh check    # check only, change nothing
./lutris/spfl2027-fix.sh         # check and repair what is missing
```

Checks performed:

1. Tool files present: `Settings.exe`, `FL27 Switcher.exe`, `FL_2027.exe`.
2. Prefix Windows version is Windows 10 (build >= 19041).
3. .NET Framework 4.8 installed (registry `Release` >= 528040).
4. `Segoe UI` and `Segoe UI Semibold` mapped in
   `HKCU\Software\Wine\Fonts\Replacements`.
5. `ddraw.dll` present (created from `xinput1_3.dll` if missing).
6. DXVK present (large `d3d11.dll` + `dxgi.dll`).

The script is idempotent and safe to run repeatedly.

## 6. Non-standard prefix location

```bash
SPFL2027_PREFIX=/path/to/prefix ./lutris/spfl2027-fix.sh check
```

The script also reads the prefix from Lutris config files and scans
`~/Games`, `/run/media/*/Games` and `~/.var/app/net.lutris.Lutris/data/Games`.

## 7. Running the game

```bash
~/.local/bin/spfl2027        # if installed by the launcher
```

Or launch from Lutris, or run `FL 2027 start.exe` / `FL_2027.bat` from the game
folder inside the prefix.

Sider starts first, then the game. Only the game window opens. The game does
not exit when its window is closed: Sider exits, but `FL_2027.exe` keeps
running in the background. The desktop launcher waits for that (Sider gone,
game still alive), gives the game a few seconds to quit on its own, then
force-closes the whole prefix — nothing is left behind. If anything remains
(for example after launching from a terminal), press Ctrl+C in the launcher
terminal, or run:

```bash
spfl2027-stop
```

## 8. Tools

- **FL27 Settings** (menu entry): display, keyboard, controller, audio, online.
  Set VSync to `Enable1` to cap 60 fps and reduce heat.
- **FL27 Switcher** (menu entry): theme color, render lighting, gameplay,
  controller buttons, classic teams, lockdown, replay logo.

Neither tool starts the game.

## 8b. Offside VAR (optional)

The VAR offside wall, camera sequence, and banner are optional and live in
`mods/offside-var/`. Check and install them with:

```bash
./lutris/spfl2027-var.sh check
./lutris/spfl2027-var.sh install
# or: ./lutris/spfl2027-var.sh install --zip /path/to/FL27_Pes21_Offside_VAR_1.0-rc.zip
```

Sider already provides LuaJIT and `nesalib.lua` / `CommonLib.lua`. For the
banner only you also need ReShade 6.x with full add-on support plus the
standard ReShade shaders (`reshade-shaders/Shaders/ReShade.fxh`). Without
ReShade the wall and camera still work; only the banner is missing.

The installer copies the bundled mod into the game folder and adds
`lua.module = "offside.lua"` after `lib\nesalib.lua` in `sider.ini`
(backed up as `sider.ini.bak_var`). Remove the mod with:

```bash
./lutris/spfl2027-var.sh remove
```

## 9. Troubleshooting

- **Settings.exe crashes with `Unexpected number of bits: 0`**: Wine Mono is in
  use. Install real .NET 4.8: `winetricks -q dotnet48` (or run the repair
  script).
- **FL27 Switcher.exe crashes with `Unrecoverable system error`**: map the
  `Segoe UI` font to a system font (see section 5).
- **Game runs slowly**: DXVK not installed. Lutris installs it automatically;
  for manual Wine runs, install DXVK into the prefix.
- **Controller not detected**: ensure `ddraw.dll` exists (renamed from
  `xinput1_3.dll`).
- **Game does not exit when window is closed**: the desktop launcher cleans
  up automatically a few seconds after the game window is gone. From a
  terminal, press Ctrl+C, or run `spfl2027-stop`.
- **High CPU / heat**: set VSync to `Enable1`, lower in-game graphics (crowd,
  shadows, anti-aliasing), ensure good cooling.

## 10. Safety and verification

- All scripts are plain bash; read them before running.
- All Rust code compiles from source; run `cargo test` to verify.
- The installer YAML is plain text; inspect `lutris/sp-football-life-2027.yaml`.
- No binary blobs are executed except the game's own installers from SmokePatch.
- Backups are kept: `xinput1_3.dll.bak`, `download-backup/`.
- If something breaks, restore from the backups or re-run `spfl2027-fix.sh`.

## 11. For AI agents

- Project root: `sp-football-life-2027-linux/`.
- All documentation is in English. Do not introduce other languages into the
  project.
- Build/test commands: see `README.md`.
- Health check command: `./lutris/spfl2027-fix.sh check` (exit 0 = healthy).
- Repair command: `./lutris/spfl2027-fix.sh` (idempotent).
- Stop command: `spfl2027-stop`.
- The Lutris installer YAML is `lutris/sp-football-life-2027.yaml`.
- The repair script is `lutris/spfl2027-fix.sh`.
- The stop script is `lutris/spfl2027-stop.sh`.