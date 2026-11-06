# FL27 Tools Quick Reference

A short reference for the two Windows tools that ship with the game, and the
Linux launcher pieces around them. For installation and repair, see
`01-installation.md`.

Neither tool starts the game.

## FL27 Settings (menu entry: FL27 Settings)

Same options as the original Settings.exe:

- **Display** — screen mode, V-Sync, frame rate. Set VSync to `Enable1` to cap
  60 fps and reduce heat. In-game reference: 1600 x 900, Full Screen.
- **Keyboard** — key bindings.
- **Controller** — XInput for Steam; DirectInput for PlayStation pads. Saved
  into the game's settings file (`settings.dat`; DirectInput reference value: `67 04`).
- **Audio** — sound settings.
- **Online** — online options.

If the file is missing, launch the game once to create a save.

## FL27 Switcher (menu entry: FL27 Switcher)

- **Theme Color** — menu theme.
- **Render Lighting** — in-game lighting option.
- **Gameplay** — swaps physics/AI variants without reinstalling. Your choices
  are copied into `Data/download` and remembered.
- **Controller Buttons** — swaps the button icons (PlayStation/Xbox).
- **Classic Teams** — legends in career modes. Source files ship with the
  game itself (its Switcher folder).
- **Lockdown** — lockdown option.
- **Replay Logo** — replay logo option.

## SPFL27 Playtools app (optional, Linux)

Built with Rust and iced. Pages:

- **Home** — shows whether the game, Sider and your save are detected. The
  PLAY button launches everything (Sider first, then the game). If no game
  folder is selected, press Change next to the game title, or run the
  first-time setup screen, which appears on first run.
- **Install** — 3 steps: point the app to your game folder; check/install the
  Linux programs the game needs (Lutris, Wine, graphics drivers); go to Home.
- **System** — dependency check. Green OK means present. Missing items install
  with one click through your distro's package manager (admin rights
  required). On Steam Deck, Lutris comes via Flatpak.
- **Switcher** — the easiest and safest way to open the game's FL27
  Switcher, where you can change gameplay, controller icons and team modes.
- **Settings** — the easiest and safest way to open the game's FL27 Settings
  tool, where you can change display, audio and controller settings.
- **Saves** — one-click timestamped backups of the game's save folder
  (KONAMI, inside its Windows-on-Linux installation) under `~/spfl27-backups`.
  Restoring overwrites, so it asks twice. Uninstalling the game deletes the
  save folder — back it up.
- **Help** — the most common problems and their fixes. The log panel at the
  bottom shows exactly what the app did.

Build and run:

```bash
cd native && cargo build --release
./native/run.sh            # builds if stale, then runs
```

## Stop a stuck game

The game does not exit when its window is closed: Sider exits, but
`FL_2027.exe` keeps running in the background. The desktop launcher waits a
few seconds after the window is gone, then force-closes the whole prefix. If
anything remains:

```bash
spfl2027-stop              # or Ctrl+C in the launcher terminal
```

## Health check and repair

```bash
./lutris/spfl2027-fix.sh check    # check only, change nothing
./lutris/spfl2027-fix.sh         # check and repair what is missing
```

Exit code 0 = healthy. Safe to run repeatedly. For a non-standard prefix:

```bash
SPFL2027_PREFIX=/path/to/prefix ./lutris/spfl2027-fix.sh check
```
