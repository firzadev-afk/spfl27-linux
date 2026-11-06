# SP Football Life 2027 on Linux — Installation Guide

This guide shows you how to install SP Football Life 2027 (SmokePatch) on Linux
and Steam Deck, how to start the game, and how to fix the two Windows tools
(FL27 Settings and FL27 Switcher) if they break. It uses plain language and
exact commands. Every command in this guide comes from this repository's own
documents.

A few terms you will see:

- **Wine** — a free layer that lets Windows programs run on Linux.
- **Wine prefix** — a folder that acts as a private fake Windows installation.
  The game and its tools live inside it.
- **Lutris** — a free app that installs and launches Windows games on Linux.
- **DXVK** — a translator that turns the game's DirectX graphics calls into
  Vulkan (the graphics system Linux uses). Required for good speed.

## 1. Prerequisites (check these first)

Tick every box before you start:

- [ ] Lutris 0.5.20 or newer
- [ ] Wine 10.0 or newer
- [ ] Winetricks (any recent version)
- [ ] Vulkan drivers (required by DXVK)
- [ ] Rust 1.88 or newer — **only if you want to build the optional Playtools app**
- [ ] Two files from SmokePatch (pessmokepatch.com): the base installer
      `SPFL27_setup.exe` and the latest update file
- [ ] At least 80 GB of free disk space (game ~23 GB + facepacks ~42 GB)
- [ ] The target disk is mounted and writable

Verify what you have:

```bash
rustc --version          # only needed for the optional Playtools app
lutris --version
wine --version
winetricks --version
vulkaninfo --summary     # optional but recommended
```

Arch/CachyOS one-liner to install everything:

```bash
sudo pacman -S rust lutris wine winetricks vulkan-tools
```

On Steam Deck, Lutris comes via Flatpak and Wine rides along with it. The
same steps below apply; see section 9 for Deck specifics.

## 2. Get the project

```bash
git clone <repo-url> sp-football-life-2027-linux
cd sp-football-life-2027-linux
```

## 3. Build the Playtools app (optional)

The Playtools app (`spfl27`) is a small native helper for setup, saves and
settings. You do **not** need it to play the game. Rust is required only for
this step.

```bash
cd native && cargo build --release
./native/run.sh            # builds if stale, then runs
cd native && cargo test    # run the e2e suite (31 + 19 + 5 tests)
cd native && cargo clippy --all-targets
```

## 4. Install the game with Lutris

1. Make sure you have the two SmokePatch files: `SPFL27_setup.exe` and the
   latest update file.
2. From the repository folder, run:

   ```bash
   lutris -i lutris/sp-football-life-2027.yaml
   ```

3. When Lutris asks for the installer, point it at `SPFL27_setup.exe`.
4. Wait. The installer does the following, in order:
   1. Create the Wine prefix.
   2. Install .NET Framework 4.8.
   3. Install .NET Framework 8.0.
   4. Install `vcrun2022`, `unifont`, `win10`.
   5. Run the game installer (you point Lutris at `SPFL27_setup.exe`).
   6. Rename `xinput1_3.dll` to `ddraw.dll` (backup kept as `xinput1_3.dll.bak`).
   7. Map the `Segoe UI` and `Segoe UI Semibold` fonts to a system font (for
      the WPF Switcher — a Windows tool that needs that font).
   8. Run the gameplay Switcher once.
   9. Apply the aspect-ratio fix (copies `aspectratio.lua` when needed).
   10. Write `FL_2027.bat`.

After the install finishes, the prefix lives at the path chosen by Lutris
(default `~/Games/sp-football-life-2027`).

## 5. Start the game

- Menu entry **SP Football Life 2027**, or run `~/.local/bin/spfl2027` if the
  launcher installed it.
- Or launch from Lutris, or run `FL 2027 start.exe` / `FL_2027.bat` from the
  game folder inside the prefix.

Sider starts first, then the game. Only the game window opens. The game does
not exit when its window is closed: Sider exits, but `FL_2027.exe` keeps
running in the background. The desktop launcher waits for that (Sider gone,
game still alive), gives the game a few seconds to quit on its own, then
force-closes the whole prefix — nothing is left behind.

If anything remains (for example after launching from a terminal), press
Ctrl+C in the launcher terminal, or run:

```bash
spfl2027-stop
```

(`spfl2027-stop` is the installed name of `lutris/spfl2027-stop.sh`. If it is
not on your PATH, run `./lutris/spfl2027-stop.sh` from the repository folder.)

## 6. Keep the tools working (FL27 Settings and FL27 Switcher)

Two Windows tools ship with the game:

- **FL27 Settings** — display, keyboard, controller, audio, online. Set VSync
  to `Enable1` to cap 60 fps and reduce heat.
- **FL27 Switcher** — theme color, render lighting, gameplay, controller
  buttons, classic teams, lockdown, replay logo.

Neither tool starts the game. They break on a plain Wine prefix because of
missing .NET and fonts. If one fails to open, run the repair script:

```bash
./lutris/spfl2027-fix.sh check    # check only, change nothing
./lutris/spfl2027-fix.sh         # check and repair what is missing
```

What the script checks:

1. Tool files present: `Settings.exe`, `FL27 Switcher.exe`, `FL_2027.exe`.
2. Prefix Windows version is Windows 10 (build >= 19041).
3. .NET Framework 4.8 installed (registry `Release` >= 528040).
4. `Segoe UI` and `Segoe UI Semibold` mapped in
   `HKCU\Software\Wine\Fonts\Replacements`.
5. `ddraw.dll` present (created from `xinput1_3.dll` if missing).
6. DXVK present (large `d3d11.dll` + `dxgi.dll`).

The script is idempotent — safe to run repeatedly. It changes nothing in
`check` mode.

Exit code 0 means healthy.

If your prefix is in a non-standard place:

```bash
SPFL2027_PREFIX=/path/to/prefix ./lutris/spfl2027-fix.sh check
```

The script also reads the prefix from Lutris config files and scans
`~/Games`, `/run/media/*/Games` and `~/.var/app/net.lutris.Lutris/data/Games`.

## 7. Where your saves live — back them up

Your career and match progress lives under **Football Life** inside the Wine
prefix, usually at:

```text
~/Games/sp-football-life-2027/drive_c/users/<your-name>/Documents/SPGAME/Football Life
```

(The exact user folder name depends on the prefix. For older saves, the
Playtools app also checks the **KONAMI** folder: `Documents/KONAMI` and
`~/Documents/KONAMI`.)

**Warning: uninstalling the game deletes this folder.** Back it up before you
uninstall, reinstall, or repair anything. The Playtools app can make
timestamped backups under `~/spfl27-backups` and restore them (restoring
overwrites, so it asks twice). You can also copy the save folder anywhere
safe yourself.

## 8. Troubleshooting

| Symptom | Fix |
|---|---|
| Settings.exe crashes with `Unexpected number of bits: 0` | Wine Mono is in use. Install real .NET 4.8: `winetricks -q dotnet48` (or run `./lutris/spfl2027-fix.sh`). |
| FL27 Switcher.exe crashes with `Unrecoverable system error` | The `Segoe UI` font is not mapped. Run `./lutris/spfl2027-fix.sh` (see section 6). |
| Game runs slowly | DXVK is not installed. Lutris installs it automatically; for manual Wine runs, install DXVK into the prefix. |
| Controller not detected | Make sure `ddraw.dll` exists (renamed from `xinput1_3.dll`). Install the latest update; play via Steam (XInput). DirectInput is for PlayStation pads. |
| Game does not exit when the window is closed | Normal. The desktop launcher cleans up automatically a few seconds after the window is gone. From a terminal, press Ctrl+C or run `spfl2027-stop`. |
| High CPU / heat | Set VSync to `Enable1`, lower in-game graphics (crowd, shadows, anti-aliasing), ensure good cooling. |
| Black screen in Game Mode (Steam Deck) | Set `ENABLE_GAMESCOPE_WSI=0` (the installer already does this). |
| Black bars in menus | Normal — the aspect-ratio fix only applies during matches. |
| Flickering / missing textures | Clear the Lutris cache (Install → Maintenance in the Playtools app) and reinstall. |

## 9. Steam Deck notes

- Lutris comes via Flatpak, and Wine comes with its runners.
- The default Flatpak prefix is
  `~/.var/app/net.lutris.Lutris/data/Games/sp-football-life-2027`. The repair
  script checks this location automatically.
- For a non-standard Deck prefix, use
  `SPFL2027_PREFIX=/path/to/prefix ./lutris/spfl2027-fix.sh check`.
- The Playtools app can create a Steam shortcut that launches the game
  directly (`--play`, no window).
- VSync `Enable1` caps the game at 60 fps, which suits the Deck screen and
  keeps heat down.
- Reference machine in TEST_NOTES.md: 1600 x 900 full screen, DirectInput
  controller (settings.dat = `67 04`), smooth and playable.

## 10. Before you change anything — safety

- All scripts are plain bash; read them before running.
- All Rust code compiles from source; run `cargo test` to verify.
- The installer YAML is plain text; inspect `lutris/sp-football-life-2027.yaml`.
- No binary blobs are executed except the game's own installers from SmokePatch.
- Backups are kept: `xinput1_3.dll.bak`, `download-backup/`.
- If something breaks, restore from the backups or re-run `spfl2027-fix.sh`.

## 11. License and content ownership

- This project is free for the community, non-commercial only (CC BY-NC 4.0).
  You may use, copy, modify and share it, but you may not sell it.
- The game itself, the SmokePatch installers/CPKs and all game content belong
  to SmokePatch (pessmokepatch.com). Get them from the official site and
  follow their terms.
