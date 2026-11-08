#!/usr/bin/env bash
# spfl2027-var — check, install, and remove the Offside VAR mod in a Wine prefix.
#
# The VAR mod is a third-party Sider + ReShade add-on (Kris1982, FL27).
# It adds a VAR-style offside wall, camera sequence, and an "OFFSIDE" banner.
# The banner needs ReShade 6.x with full add-on support.
#
# Usage:
#   spfl2027-var check
#   spfl2027-var install
#   spfl2027-var install --zip /path/to/FL27_Pes21_Offside_VAR_1.0-rc.zip
#   spfl2027-var remove
#   SPFL2027_PREFIX=/path spfl2027-var check
#
# "install" without --zip uses the bundled mods/offside-var folder.
#
# The script is idempotent: safe to run repeatedly.
# It never commits or downloads game binaries; the VAR zip is supplied by the user.
set -uo pipefail

MODE="${1:-check}"
shift 2>/dev/null || true

ZIP=""
while [ $# -gt 0 ]; do
  case "$1" in
    --zip) ZIP="${2:-}"; shift 2 ;;
    *) echo "Unknown option: $1" >&2; exit 2 ;;
  esac
done

case "$MODE" in
  check|install|remove) ;;
  -h|--help|help)
    sed -n '2,16p' "$0" | sed 's/^# \{0,1\}//'
    exit 0
    ;;
  *)
    echo "Unknown mode: $MODE (use: check | install | remove)" >&2
    exit 2
    ;;
esac

GAME_SUBPATH="drive_c/Program Files (x86)/SP Football Life 2027"
CHANGED=0
PROBLEMS=0

ok()   { printf '  \033[32mOK\033[0m   %s\n' "$*"; }
fix()  { printf '  \033[33mFIX\033[0m  %s\n' "$*"; CHANGED=$((CHANGED + 1)); }
warn() { printf '  \033[33mWARN\033[0m %s\n' "$*"; PROBLEMS=$((PROBLEMS + 1)); }
bad()  { printf '  \033[31mFAIL\033[0m %s\n' "$*"; PROBLEMS=$((PROBLEMS + 1)); }
head() { printf '\n\033[1m%s\033[0m\n' "$*"; }

valid_prefix() {
  [ -n "${1:-}" ] && [ -d "$1/drive_c" ] && [ -d "$1/$GAME_SUBPATH" ]
}

PREFIX=""
for cand in \
  "${SPFL2027_PREFIX:-}" \
  "${WINEPREFIX:-}" \
  "$HOME/Games/sp-football-life-2027" \
  "$HOME/.var/app/net.lutris.Lutris/data/Games/sp-football-life-2027" \
  /run/media/"$USER"/*/Games/sp-football-life-2027 \
  /media/"$USER"/*/Games/sp-football-life-2027 \
  /mnt/*/Games/sp-football-life-2027; do
  if valid_prefix "$cand"; then PREFIX="$cand"; break; fi
done

if [ -z "$PREFIX" ]; then
  for y in "$HOME/.config/lutris/games/sp-football-life-2027.yml" \
           "$HOME/.local/share/lutris/games/sp-football-life-2027.yml"; do
    [ -f "$y" ] || continue
    cand=$(sed -n 's/^prefix: *//p' "$y" | head -n1)
    cand="${cand/#\~/$HOME}"
    if valid_prefix "$cand"; then PREFIX="$cand"; break; fi
  done
fi

GAME_DIR="${PREFIX:+$PREFIX/$GAME_SUBPATH}"

if [ -z "$PREFIX" ]; then
  echo "SP Football Life 2027 prefix not found." >&2
  echo "Set it manually: SPFL2027_PREFIX=/path/to/prefix $0 $MODE" >&2
  exit 2
fi

echo "Prefix : $PREFIX"
echo "Game   : $GAME_DIR"
echo "Mode   : $MODE"

MOD_LUA="$GAME_DIR/SiderAddons/modules/offside.lua"
MOD_INI="$GAME_DIR/SiderAddons/modules/offside.ini"
FMDL="$GAME_DIR/SiderAddons/livecpk/root/Asset/model/game2d/d2_offside_line/scenes/#Win/d2_offside_line.fmdl"
ADDON="$GAME_DIR/offside_banner.addon64"
FX="$GAME_DIR/reshade-shaders/Shaders/OffsideBanner.fx"
SIDER_INI="$GAME_DIR/SiderAddons/sider.ini"

check_var() {
  head "VAR check"

  if [ -f "$MOD_LUA" ]; then ok "offside.lua present ($(stat -c%s "$MOD_LUA") bytes)"; else bad "offside.lua MISSING"; fi
  if [ -f "$MOD_INI" ]; then ok "offside.ini present ($(stat -c%s "$MOD_INI") bytes)"; else bad "offside.ini MISSING"; fi
  if [ -f "$FMDL" ]; then ok "d2_offside_line.fmdl present"; else bad "d2_offside_line.fmdl MISSING"; fi
  if [ -f "$ADDON" ]; then ok "offside_banner.addon64 present (banner)"; else bad "offside_banner.addon64 MISSING (banner)"; fi
  if [ -f "$FX" ]; then ok "OffsideBanner.fx present (banner)"; else bad "OffsideBanner.fx MISSING (banner)"; fi

  for tex in offside_banner.png offside_logo_map.png offside_logos.png; do
    if [ -f "$GAME_DIR/reshade-shaders/Textures/$tex" ]; then
      ok "Textures/$tex present"
    else
      bad "Textures/$tex MISSING (banner)"
    fi
  done

  head "ReShade check"
  if [ -f "$GAME_DIR/dxgi.dll" ]; then ok "dxgi.dll present (ReShade D3D11/DXGI)"; else bad "dxgi.dll MISSING (install ReShade 6.x Addon build)"; fi
  if [ -f "$GAME_DIR/reshade-shaders/Shaders/ReShade.fxh" ]; then ok "ReShade.fxh present"; else bad "ReShade.fxh MISSING (download standard shaders)"; fi
  if [ -f "$GAME_DIR/ReShade.ini" ]; then ok "ReShade.ini present"; else warn "ReShade.ini missing (created on first game run)"; fi

  head "Sider config"
  if grep -q "luajit.ext.enabled = 1" "$SIDER_INI" 2>/dev/null; then ok "luajit.ext.enabled = 1"; else bad "luajit.ext.enabled != 1 in sider.ini"; fi
  if grep -qF 'cpk.root = ".\livecpk\root"' "$SIDER_INI" 2>/dev/null; then ok "cpk.root = .\\livecpk\\root"; else bad "cpk.root not set in sider.ini"; fi
  if grep -q 'lua.module = "lib\\nesalib.lua"' "$SIDER_INI" 2>/dev/null; then ok 'lib\nesalib.lua loaded'; else bad 'lib\nesalib.lua MISSING in sider.ini'; fi
  cnt=$(grep -c 'lua.module = "offside.lua"' "$SIDER_INI" 2>/dev/null || true)
  cnt=${cnt//[^0-9]/}
  if [ "$cnt" = "1" ]; then ok 'lua.module = "offside.lua" (once, after nesalib)';
  elif [ "$cnt" = "0" ]; then bad 'lua.module = "offside.lua" MISSING in sider.ini';
  else bad "lua.module = offside.lua loaded $cnt times (must be once)";
  fi

  if [ "$PROBLEMS" -eq 0 ]; then
    echo
    ok "VAR check passed — banner ready."
    return 0
  else
    echo
    echo "VAR check: $PROBLEMS problem(s)."
    return 1
  fi
}

install_var() {
  # Default: bundled mods/offside-var next to this script; --zip overrides it.
  SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
  BUNDLED="$SCRIPT_DIR/../mods/offside-var"
  TMP=""
  trap 'rm -rf "${TMP:-}"' EXIT

  head "Install VAR"
  if [ -n "$ZIP" ]; then
    [ -f "$ZIP" ] || { echo "Zip not found: $ZIP" >&2; exit 2; }
    TMP=$(mktemp -d)
    unzip -q -o "$ZIP" -d "$TMP"
    EXTRACTED="$TMP"
  else
    if [ ! -f "$BUNDLED/SiderAddons/modules/offside.lua" ]; then
      echo "Bundled mod not found: $BUNDLED" >&2
      echo "Expected mods/offside-var next to this script, or pass --zip." >&2
      exit 2
    fi
    EXTRACTED="$BUNDLED"
  fi

  # Core files
  mkdir -p "$GAME_DIR/SiderAddons/modules"
  mkdir -p "$GAME_DIR/SiderAddons/livecpk/root/Asset/model/game2d/d2_offside_line/scenes/#Win"
  mkdir -p "$GAME_DIR/reshade-shaders/Shaders"
  mkdir -p "$GAME_DIR/reshade-shaders/Textures"

  cp -p "$EXTRACTED/SiderAddons/modules/offside.lua" "$MOD_LUA" && fix "copied offside.lua"
  cp -p "$EXTRACTED/SiderAddons/modules/offside.ini" "$MOD_INI" && fix "copied offside.ini"
  cp -p "$EXTRACTED/SiderAddons/livecpk/root/Asset/model/game2d/d2_offside_line/scenes/#Win/d2_offside_line.fmdl" "$FMDL" && fix "copied d2_offside_line.fmdl"

  # Banner files
  if [ -f "$EXTRACTED/offside_banner.addon64" ]; then
    cp -p "$EXTRACTED/offside_banner.addon64" "$ADDON" && fix "copied offside_banner.addon64"
  else
    warn "offside_banner.addon64 not in zip — banner skipped"
  fi
  if [ -f "$EXTRACTED/reshade-shaders/Shaders/OffsideBanner.fx" ]; then
    cp -p "$EXTRACTED/reshade-shaders/Shaders/OffsideBanner.fx" "$FX" && fix "copied OffsideBanner.fx"
  fi
  cp -p "$EXTRACTED/reshade-shaders/Textures/"*.png "$GAME_DIR/reshade-shaders/Textures/" && fix "copied banner textures"
  cp -p "$EXTRACTED/README_Offside_VAR.txt" "$GAME_DIR/README_Offside_VAR.txt" 2>/dev/null && fix "copied README_Offside_VAR.txt"

  # Sider.ini — backup once, then insert after nesalib (idempotent)
  if [ ! -f "$GAME_DIR/SiderAddons/sider.ini.bak_var" ]; then
    cp -p "$SIDER_INI" "$GAME_DIR/SiderAddons/sider.ini.bak_var" && fix "backed up sider.ini -> sider.ini.bak_var"
  fi
  if grep -q 'lua.module = "offside.lua"' "$SIDER_INI"; then
    ok "sider.ini already has offside.lua"
  else
    # insert after the nesalib line, preserving CRLF and leading tab
    python3 - <<PY
import pathlib
p = pathlib.Path("$SIDER_INI")
raw = p.read_bytes()
sep = "\r\n" if b"\r\n" in raw else "\n"
text = raw.decode("utf-8", errors="surrogateescape")
lines = text.split(sep)
idx = None
for i, l in enumerate(lines):
    if "lua.module" in l and "nesalib.lua" in l:
        idx = i
        break
if idx is not None:
    lines.insert(idx + 1, '\tlua.module = "offside.lua"')
    p.write_bytes(sep.join(lines).encode("utf-8"))
    print("FIX  inserted offside.lua after nesalib")
else:
    raise SystemExit("nesalib line not found in sider.ini")
PY
  fi

  echo
  echo "Install done. Run '$0 check' to verify, then launch the game."
  return 0
}

remove_var() {
  head "Remove VAR"
  rm -f "$MOD_LUA" "$MOD_INI" "$ADDON" "$FX"
  rm -f "$GAME_DIR/reshade-shaders/Textures/offside_banner.png" \
        "$GAME_DIR/reshade-shaders/Textures/offside_logo_map.png" \
        "$GAME_DIR/reshade-shaders/Textures/offside_logos.png"
  rm -rf "$GAME_DIR/SiderAddons/livecpk/root/Asset/model/game2d/d2_offside_line" 2>/dev/null || true
  fix "removed VAR files"

  if [ -f "$GAME_DIR/SiderAddons/sider.ini.bak_var" ]; then
    cp -p "$GAME_DIR/SiderAddons/sider.ini.bak_var" "$SIDER_INI" && fix "restored sider.ini from backup"
  else
    sed -i '/lua.module = "offside.lua"/d' "$SIDER_INI" && fix "removed offside.lua line from sider.ini"
  fi

  echo
  ok "VAR removed. ReShade itself (dxgi.dll, shaders) left in place."
  return 0
}

case "$MODE" in
  check) check_var ;;
  install) install_var ;;
  remove) remove_var ;;
esac
