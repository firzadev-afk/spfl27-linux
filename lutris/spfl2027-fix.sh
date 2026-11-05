#!/usr/bin/env bash
# spfl2027-fix — check and repair the dependencies of the SP Football Life 2027 tools
# inside a Wine prefix.
#
# Why this is needed:
#   - Settings.exe is a .NET Framework 4.5 application. Without the real .NET,
#     Wine Mono crashes while loading the window icon ("Unexpected number of bits: 0").
#   - FL27 Switcher.exe is a WPF application. Without the "Segoe UI" font it crashes
#     with "Unrecoverable system error".
#
# Usage:
#   spfl2027-fix            # check and repair what is missing (default)
#   spfl2027-fix check      # check only, do not change anything
#   SPFL2027_PREFIX=/path spfl2027-fix
#
# The script is idempotent: safe to run repeatedly.
set -uo pipefail

MODE="${1:-repair}"
case "$MODE" in
  check|repair) ;;
  -h|--help|help)
    sed -n '2,18p' "$0" | sed 's/^# \{0,1\}//'
    exit 0
    ;;
  *)
    echo "Unknown mode: $MODE (use: check | repair)" >&2
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

# ── Find the prefix ────────────────────────────────────────────────────────
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

export WINEPREFIX="$PREFIX"
export WINEDEBUG="${WINEDEBUG:--all}"
WINE_BIN=$(command -v wine || true)
[ -n "$WINE_BIN" ] || { echo "wine not found in PATH." >&2; exit 2; }

echo "Prefix : $PREFIX"
echo "Game   : $GAME_DIR"
echo "Mode   : $MODE"

reg_get() { # $1=key $2=value
  "$WINE_BIN" reg query "$1" /v "$2" 2>/dev/null | tr -d '\r' \
    | awk -v k="$2" '
        {
          line = $0
          sub(/^[ \t]+/, "", line)
          if (index(line, k) == 1) {
            rest = substr(line, length(k) + 1)
            sub(/^[ \t]+/, "", rest)
            sub(/^[A-Za-z_]+[ \t]+/, "", rest)
            print rest
            exit
          }
        }'
}

reg_set() { # $1=key $2=value $3=data
  "$WINE_BIN" reg add "$1" /v "$2" /d "$3" /f >/dev/null 2>&1
}

# ── 1. Tool files ──────────────────────────────────────────────────────────
head "1. Tool files"
for exe in "Settings.exe" "FL27 Switcher.exe" "FL_2027.exe"; do
  if [ -f "$GAME_DIR/$exe" ]; then ok "$exe present"; else bad "$exe MISSING"; fi
done

# ── 2. Windows version of the prefix ───────────────────────────────────────
head "2. Windows version of the prefix"
CV=$(reg_get 'HKLM\Software\Microsoft\Windows NT\CurrentVersion' CurrentVersion)
CB=$(reg_get 'HKLM\Software\Microsoft\Windows NT\CurrentVersion' CurrentBuild)
if [ -n "$CB" ] && [ "$CB" -ge 19041 ] 2>/dev/null; then
  ok "Windows 10 (build $CB)"
else
  if [ "$MODE" = repair ]; then
    "$WINE_BIN" winecfg -v win10 >/dev/null 2>&1 && fix "set to Windows 10 (was build ${CB:-?})"
  else
    warn "not Windows 10 (build ${CB:-?}, version ${CV:-?}) — run without 'check' to repair"
  fi
fi

# ── 3. .NET Framework 4.8 (needed by Settings.exe) ────────────────────────
head "3. .NET Framework 4.8 (needed by Settings.exe)"
REL=$(reg_get 'HKLM\Software\Microsoft\NET Framework Setup\NDP\v4\Full' Release)
REL_DEC=""
[ -n "$REL" ] && REL_DEC=$((REL))
if [ -n "$REL_DEC" ] && [ "$REL_DEC" -ge 528040 ] 2>/dev/null; then
  VER=$(reg_get 'HKLM\Software\Microsoft\NET Framework Setup\NDP\v4\Full' Version)
  ok ".NET ${VER:-4.8} installed (Release $REL)"
elif [ "$MODE" = repair ]; then
  if command -v winetricks >/dev/null 2>&1; then
    echo "     Installing .NET 4.8 via winetricks (may take 5-15 minutes)..."
    if timeout 900 winetricks -q dotnet48 >/tmp/spfl2027-dotnet48.log 2>&1; then
      fix ".NET 4.8 installed (log: /tmp/spfl2027-dotnet48.log)"
      # The dotnet48 installer switches the prefix to Windows 7; restore it.
      "$WINE_BIN" winecfg -v win10 >/dev/null 2>&1 \
        && fix "prefix restored to Windows 10"
    else
      bad "winetricks dotnet48 failed or timed out (15 min) — see /tmp/spfl2027-dotnet48.log"
    fi
  else
    bad "winetricks not found; cannot install .NET 4.8"
  fi
else
  warn ".NET 4.8 not installed (Release ${REL:-none}) — will be installed on repair"
fi

# ── 4. Segoe UI font for the Switcher (WPF) ───────────────────────────────
head "4. Segoe UI font (needed by FL27 Switcher)"
pick_font() {
  local got
  for f in "Noto Sans" "DejaVu Sans" "Liberation Sans" "Arial"; do
    got=$(fc-match -f '%{family}' "$f" 2>/dev/null || true)
    case "$got" in
      *"$f"*) printf '%s' "$f"; return 0 ;;
    esac
  done
  printf 'DejaVu Sans'
}
FONT=$(pick_font)
for v in "Segoe UI" "Segoe UI Semibold"; do
  cur=$(reg_get 'HKCU\Software\Wine\Fonts\Replacements' "$v")
  if [ -n "$cur" ]; then
    ok "$v -> $cur"
  elif [ "$MODE" = repair ]; then
    if reg_set 'HKCU\Software\Wine\Fonts\Replacements' "$v" "$FONT"; then
      fix "$v -> $FONT"
    else
      bad "failed to set $v"
    fi
  else
    warn "$v not mapped (will be set to $FONT on repair)"
  fi
done

# ── 5. ddraw.dll (xinput1_3 rename) ───────────────────────────────────────
head "5. ddraw.dll"
if [ -f "$GAME_DIR/ddraw.dll" ]; then
  ok "ddraw.dll present"
elif [ -f "$GAME_DIR/xinput1_3.dll" ]; then
  if [ "$MODE" = repair ]; then
    cp -f "$GAME_DIR/xinput1_3.dll" "$GAME_DIR/xinput1_3.dll.bak" 2>/dev/null || true
    if cp -f "$GAME_DIR/xinput1_3.dll" "$GAME_DIR/ddraw.dll"; then
      fix "ddraw.dll created from xinput1_3.dll (backup .bak)"
    else
      bad "failed to create ddraw.dll"
    fi
  else
    warn "ddraw.dll missing — will be created from xinput1_3.dll on repair"
  fi
else
  bad "neither ddraw.dll nor xinput1_3.dll found"
fi

# ── 6. DXVK ────────────────────────────────────────────────────────────────
head "6. DXVK"
D3D11="$PREFIX/drive_c/windows/system32/d3d11.dll"
DXGI="$PREFIX/drive_c/windows/system32/dxgi.dll"
SZ=0
[ -f "$D3D11" ] && SZ=$(stat -c%s "$D3D11" 2>/dev/null || echo 0)
if [ "$SZ" -gt 3000000 ] && [ -f "$DXGI" ]; then
  ok "DXVK installed (d3d11.dll $((SZ / 1024 / 1024)) MB + dxgi.dll)"
elif [ -f "$D3D11" ]; then
  warn "d3d11.dll present but does not look like DXVK (size $SZ bytes)"
else
  warn "DXVK not installed (Lutris usually installs it automatically)"
fi

# ── Summary ────────────────────────────────────────────────────────────────
head "Summary"
if [ "$PROBLEMS" -eq 0 ] && [ "$CHANGED" -eq 0 ]; then
  echo "  All checks passed. Settings and Switcher are ready to use."
  exit 0
elif [ "$PROBLEMS" -eq 0 ]; then
  echo "  $CHANGED repair(s) applied. Please test Settings and Switcher."
  exit 0
else
  echo "  $PROBLEMS issue(s) remain, $CHANGED repair(s) applied."
  exit 1
fi
