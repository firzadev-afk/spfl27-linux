#!/usr/bin/env bash
# spfl2027-stop — forcefully stop a running SP Football Life 2027 game.
#
# The game (FL_2027.exe) sometimes does not exit when its window is closed,
# and Sider keeps running in the background. Run this (or bind it to a key)
# to clean everything up.
set -uo pipefail

GAME_SUBPATH="drive_c/Program Files (x86)/SP Football Life 2027"

find_prefix() {
  for cand in \
    "${SPFL2027_PREFIX:-}" \
    "${WINEPREFIX:-}" \
    "$HOME/Games/sp-football-life-2027" \
    "$HOME/.var/app/net.lutris.Lutris/data/Games/sp-football-life-2027" \
    /run/media/"$USER"/*/Games/sp-football-life-2027 \
    /media/"$USER"/*/Games/sp-football-life-2027 \
    /mnt/*/Games/sp-football-life-2027; do
    if [ -d "$cand/drive_c" ]; then printf '%s' "$cand"; return 0; fi
  done
  for y in "$HOME/.config/lutris/games/sp-football-life-2027.yml" \
           "$HOME/.local/share/lutris/games/sp-football-life-2027.yml"; do
    [ -f "$y" ] || continue
    cand=$(sed -n 's/^prefix: *//p' "$y" | head -n1)
    cand="${cand/#\~/$HOME}"
    if [ -d "$cand/drive_c" ]; then printf '%s' "$cand"; return 0; fi
  done
  return 1
}

if ! PREFIX=$(find_prefix); then
  echo "SP Football Life 2027 prefix not found." >&2
  echo "Set it manually: SPFL2027_PREFIX=/path/to/prefix spfl2027-stop" >&2
  exit 2
fi
export WINEPREFIX="$PREFIX"
export WINEDEBUG=-all

echo "Stopping SP Football Life 2027..."
pkill -x "FL_2027.exe" 2>/dev/null || true
pkill -x "sider.exe" 2>/dev/null || true
pkill -f "FL_2027.bat" 2>/dev/null || true
sleep 1
wineserver -k 2>/dev/null || true
sleep 1

if pgrep -x "FL_2027.exe" >/dev/null || pgrep -x "sider.exe" >/dev/null; then
  echo "Some processes are still alive; retry or kill manually." >&2
  exit 1
fi
echo "Stopped."