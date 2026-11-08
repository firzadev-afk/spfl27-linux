# Turf Evolution — mods/turf-evolution

Bundled Ultra Graphics GfX Best Turf 2026 for SP Football Life 2027
(Sider + optional ReShade PitchCrew repair).

This folder mirrors the upstream RAR layout: its root is a game folder.
The installer script copies it into
`drive_c/Program Files (x86)/SP Football Life 2027`.

## Quick install

Requires Sider 7 with `luajit.ext.enabled = 1`,
`livecpk.enabled = 1`, and `match-stats.enabled = 1` (PitchCrew wants
the latter). The banner repair also needs ReShade 6.x with full
add-on support; the turf itself works without it.

```bash
./lutris/spfl2027-turf.sh check
./lutris/spfl2027-turf.sh install   # uses this bundled folder
# or: ./lutris/spfl2027-turf.sh install --zip "/path/to/Ultra Graphics GfX  Best Turf 2026 FL27.rar"
```

Then launch the game as usual. Check `sider.log` for lines starting
with `turf-evolution:` and `pitch_crew:`.

To remove it:

```bash
./lutris/spfl2027-turf.sh remove
```

## What is in this folder

- `SiderAddons/modules/turf-evolution.lua` — Sider module (wear, marks, clumps)
- `SiderAddons/content/turf-evolution/` — presets, marks, visibility,
  global grass, clumps, `settings.ini` (280 files)
- `SiderAddons/content/pitch-crew/README.txt` — trigger folder note
- `SiderAddons/livecpk/Ultra Graphics GfX/` — LUT and light package (59 files)
- `pitch_crew.addon64` — ReShade add-on bridge (optional halftime repair)
- `SOURCE.md` — provenance and checksums

## Requirements

- Sider 7, `luajit.ext.enabled = 1`, `livecpk.enabled = 1`,
  `match-stats.enabled = 1`
- ReShade 6.x with full add-on support for the PitchCrew repair only

## Settings

`SiderAddons/content/turf-evolution/settings.ini` — apply after
restarting both the game and Sider. Default keys: `C` (clumps),
`T` (global grass), `MINUS` / `PLUS` (brown), `LBRACKET` / `RBRACKET`
(marks), `F8` / `F9` (visibility), `H` (repair toggle), `G` (repair now).

## Uninstall

```bash
./lutris/spfl2027-turf.sh remove
```
