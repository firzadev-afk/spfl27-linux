# Offside VAR — mods/offside-var

Bundled copy of the VAR-style offside wall, camera sequence, and banner
for SP Football Life 2027 (Sider 7 + ReShade).

This folder mirrors the upstream zip layout exactly: its root is a game
folder. The installer script copies it into
`drive_c/Program Files (x86)/SP Football Life 2027`.

## Quick install

Requires Sider 7 with `luajit.ext.enabled = 1`,
`nesalib.lua` + `CommonLib.lua` in `SiderAddons/modules/lib/` (already
shipped by the base game), and ReShade 6.x with full add-on support
(banner only; the wall and camera work without it).

```bash
./lutris/spfl2027-var.sh check
./lutris/spfl2027-var.sh install   # uses this bundled folder
# or: ./lutris/spfl2027-var.sh install --zip /path/to/FL27_Pes21_Offside_VAR_1.0-rc.zip
```

Then launch the game as usual. Press the ReShade overlay key (default
Insert) to verify, and check `sider.log` for lines starting with
`offside:`.

To remove it:

```bash
./lutris/spfl2027-var.sh remove
```

## What is in this folder

- `SiderAddons/modules/offside.lua` — Sider module (wall, camera, free camera)
- `SiderAddons/modules/offside.ini` — per-user settings (author defaults)
- `SiderAddons/livecpk/root/Asset/model/game2d/d2_offside_line/scenes/#Win/d2_offside_line.fmdl` — offside line model
- `offside_banner.addon64` — ReShade add-on bridge (Windows DLL, banner)
- `reshade-shaders/Shaders/OffsideBanner.fx` — banner effect (needs `ReShade.fxh`)
- `reshade-shaders/Textures/` — banner mask, logo atlas, and logo map
- `README_Offside_VAR.txt` — upstream README from the mod author
- `SOURCE.md` — provenance and checksums

## Requirements

- Sider 7, `luajit.ext.enabled = 1`, `cpk.root = ".\livecpk\root"`
- ReShade 6.x with full add-on support for the banner (from reshade.me)
- Standard ReShade shaders (`ReShade.fxh`) for the banner effect

## Uninstall

```bash
./lutris/spfl2027-var.sh remove
```

or manually: comment out `lua.module = "offside.lua"` in
`SiderAddons/sider.ini` and rename
`SiderAddons/livecpk/root/Asset/model/game2d/d2_offside_line`.
