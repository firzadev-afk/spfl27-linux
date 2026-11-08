#!/usr/bin/env bash
# spfl2027-turf — check, install, and remove the Ultra Graphics Turf mod.
#
# The mod is a third-party Sider + ReShade add-on (Turf Evolution v2.4).
# It adds VAR-style turf wear, static clumps, global grass, and an
# optional halftime PitchCrew repair. The repair needs ReShade 6.x
# with full add-on support; the turf itself works without it.
#
# Usage:
#   spfl2027-turf check
#   spfl2027-turf install
#   spfl2027-turf install --zip "/path/to/Ultra Graphics GfX  Best Turf 2026 FL27.rar"
#   spfl2027-turf remove
#   SPFL2027_PREFIX=/path spfl2027-turf check
#
# "install" without --zip uses the bundled mods/turf-evolution folder.
#
# The script is idempotent: safe to run repeatedly.
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

MOD_LUA="$GAME_DIR/SiderAddons/modules/turf-evolution.lua"
TURF_DIR="$GAME_DIR/SiderAddons/content/turf-evolution"
CREW_DIR="$GAME_DIR/SiderAddons/content/pitch-crew"
LIVECPK_DIR="$GAME_DIR/SiderAddons/livecpk/Ultra Graphics GfX"
ADDON="$GAME_DIR/pitch_crew.addon64"
SIDER_INI="$GAME_DIR/SiderAddons/sider.ini"

check_turf() {
  head "Turf check"

  if [ -f "$MOD_LUA" ]; then ok "turf-evolution.lua present ($(stat -c%s "$MOD_LUA") bytes)"; else bad "turf-evolution.lua MISSING"; fi
  if [ -f "$TURF_DIR/settings.ini" ]; then ok "settings.ini present"; else bad "settings.ini MISSING"; fi
  if [ -d "$TURF_DIR/presets" ]; then ok "presets/ present"; else bad "presets/ MISSING"; fi
  if [ -d "$TURF_DIR/global-grass" ]; then ok "global-grass/ present"; else bad "global-grass/ MISSING"; fi
  if [ -d "$CREW_DIR" ]; then ok "pitch-crew/ present"; else bad "pitch-crew/ MISSING"; fi
  if [ -d "$LIVECPK_DIR" ]; then ok "livecpk/Ultra Graphics GfX/ present"; else bad "livecpk/Ultra Graphics GfX/ MISSING"; fi
  if [ -f "$ADDON" ]; then ok "pitch_crew.addon64 present (halftime repair)"; else bad "pitch_crew.addon64 MISSING (halftime repair)"; fi

  head "Sider config"
  if grep -q "luajit.ext.enabled = 1" "$SIDER_INI" 2>/dev/null; then ok "luajit.ext.enabled = 1"; else bad "luajit.ext.enabled != 1 in sider.ini"; fi
  if grep -q "livecpk.enabled = 1" "$SIDER_INI" 2>/dev/null; then ok "livecpk.enabled = 1"; else bad "livecpk.enabled != 1 in sider.ini"; fi
  if grep -q "match-stats.enabled = 1" "$SIDER_INI" 2>/dev/null; then ok "match-stats.enabled = 1 (PitchCrew)"; else bad "match-stats.enabled != 1 in sider.ini (PitchCrew)"; fi
  cnt=$(grep -c 'lua.module = "turf-evolution.lua"' "$SIDER_INI" 2>/dev/null || true)
  cnt=${cnt//[^0-9]/}
  if [ "$cnt" = "1" ]; then ok 'lua.module = "turf-evolution.lua" (once)';
  elif [ "$cnt" = "0" ]; then bad 'lua.module = "turf-evolution.lua" MISSING in sider.ini';
  else bad "lua.module = turf-evolution.lua loaded $cnt times (must be once)";
  fi

  if [ "$PROBLEMS" -eq 0 ]; then
    echo
    ok "Turf check passed."
    return 0
  else
    echo
    echo "Turf check: $PROBLEMS problem(s)."
    return 1
  fi
}

install_turf() {
  SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
  BUNDLED="$SCRIPT_DIR/../mods/turf-evolution"
  TMP=""

  head "Install Turf"
  if [ -n "$ZIP" ]; then
    [ -f "$ZIP" ] || { echo "Zip not found: $ZIP" >&2; exit 2; }
    TMP=$(mktemp -d)
    trap 'rm -rf "${TMP:-}"' EXIT
    unrar x -o+ -y "$ZIP" "$TMP/" >/dev/null 2>&1 || true
    # RAR has 1_COPY_INTO_SIDER / 2_COPY_INTO_GAME_FOLDER layout
    if [ -d "$TMP/1_COPY_INTO_SIDER" ]; then
      EXTRACTED="$TMP/1_COPY_INTO_SIDER"
    else
      EXTRACTED="$TMP"
    fi
    GAME_EXTRACTED=""
    if [ -d "$TMP/2_COPY_INTO_GAME_FOLDER" ]; then
      GAME_EXTRACTED="$TMP/2_COPY_INTO_GAME_FOLDER"
    fi
  else
    if [ ! -f "$BUNDLED/SiderAddons/modules/turf-evolution.lua" ]; then
      echo "Bundled mod not found: $BUNDLED" >&2
      echo "Expected mods/turf-evolution next to this script, or pass --zip." >&2
      exit 2
    fi
    EXTRACTED="$BUNDLED/SiderAddons"
    GAME_EXTRACTED="$BUNDLED"
  fi

  # Sider files (merge content/, livecpk/, modules/)
  mkdir -p "$GAME_DIR/SiderAddons/modules"
  mkdir -p "$GAME_DIR/SiderAddons/content/pitch-crew"
  mkdir -p "$GAME_DIR/SiderAddons/content/turf-evolution"
  mkdir -p "$GAME_DIR/SiderAddons/livecpk/Ultra Graphics GfX"

  if [ -f "$EXTRACTED/modules/turf-evolution.lua" ]; then
    cp -p "$EXTRACTED/modules/turf-evolution.lua" "$MOD_LUA" && fix "copied turf-evolution.lua"
  else
    bad "turf-evolution.lua not in source"
  fi
  if [ -d "$EXTRACTED/content/turf-evolution" ]; then
    cp -a "$EXTRACTED/content/turf-evolution/." "$TURF_DIR/" && fix "copied content/turf-evolution/"
  fi
  if [ -d "$EXTRACTED/content/pitch-crew" ]; then
    cp -a "$EXTRACTED/content/pitch-crew/." "$CREW_DIR/" && fix "copied content/pitch-crew/"
  fi
  if [ -d "$EXTRACTED/livecpk/Ultra Graphics GfX" ]; then
    cp -a "$EXTRACTED/livecpk/Ultra Graphics GfX/." "$LIVECPK_DIR/" && fix "copied livecpk/Ultra Graphics GfX/"
  fi

  # Add-on (game folder, optional)
  if [ -n "${GAME_EXTRACTED:-}" ] && [ -f "$GAME_EXTRACTED/pitch_crew.addon64" ]; then
    cp -p "$GAME_EXTRACTED/pitch_crew.addon64" "$ADDON" && fix "copied pitch_crew.addon64"
  elif [ -f "$BUNDLED/pitch_crew.addon64" ]; then
    cp -p "$BUNDLED/pitch_crew.addon64" "$ADDON" && fix "copied pitch_crew.addon64"
  else
    warn "pitch_crew.addon64 not in source — repair skipped"
  fi

  # Sider.ini — backup once, then append after the last lua.module line
  if [ ! -f "$GAME_DIR/SiderAddons/sider.ini.bak_turf" ]; then
    cp -p "$SIDER_INI" "$GAME_DIR/SiderAddons/sider.ini.bak_turf" && fix "backed up sider.ini -> sider.ini.bak_turf"
  fi
  if grep -q 'lua.module = "turf-evolution.lua"' "$SIDER_INI"; then
    ok "sider.ini already has turf-evolution.lua"
  else
    python3 - <<PY
import pathlib
p = pathlib.Path("$SIDER_INI")
raw = p.read_bytes()
sep = "\r\n" if b"\r\n" in raw else "\n"
text = raw.decode("utf-8", errors="surrogateescape")
lines = text.split(sep)
# insert after the last lua.module line
idx = max([i for i, l in enumerate(lines) if "lua.module" in l], default=None)
if idx is None:
    raise SystemExit("no lua.module line found in sider.ini")
lines.insert(idx + 1, '\tlua.module = "turf-evolution.lua"')
p.write_bytes(sep.join(lines).encode("utf-8"))
print("FIX  inserted turf-evolution.lua")
PY
  fi

  echo
  echo "Install done. Run '$0 check' to verify, then launch the game."
  return 0
}

remove_turf() {
  head "Remove Turf"
  rm -f "$MOD_LUA" "$ADDON"
  rm -rf "$TURF_DIR" "$CREW_DIR" "$LIVECPK_DIR" 2>/dev/null || true
  fix "removed Turf files"

  if [ -f "$GAME_DIR/SiderAddons/sider.ini.bak_turf" ]; then
    cp -p "$GAME_DIR/SiderAddons/sider.ini.bak_turf" "$SIDER_INI" && fix "restored sider.ini from backup"
  else
    sed -i '/lua.module = "turf-evolution.lua"/d' "$SIDER_INI" && fix "removed turf-evolution.lua line from sider.ini"
  fi

  echo
  ok "Turf removed. ReShade itself left in place."
  return 0
}

case "$MODE" in
  check) check_turf ;;
  install) install_turf ;;
  remove) remove_turf ;;
esac
