# Test Notes

Recorded on the reference machine used to validate this project. All values are
measured, not estimated.

## Machine

- **OS:** CachyOS (rolling), kernel 7.3.1-1-cachyos
- **Desktop:** COSMIC
- **CPU:** AMD Ryzen 5 5500GT with Radeon Graphics (6 cores / 12 threads)
- **GPU:** AMD Cezanne (Radeon Vega / Renoir), integrated, Mesa 26.2.5 (radv)
- **RAM:** 14 GiB (+ 14 GiB zram swap)
- **Storage:** 2x 238 GB SSD; game on a secondary SSD
- **Wine:** 11.21
- **Lutris:** 0.5.22
- **DXVK:** 3.1.1 (RADV RENOIR, feature level 11_0)

## Game

- **Title:** SP Football Life 2027 (SmokePatch)
- **Install path:** standard Lutris prefix (`~/Games/sp-football-life-2027` or a mounted SSD)
- **Game folder size:** ~75 GB
  - `Data/` 23 GB (47 base CPKs)
  - `download/` 42 GB (facepacks fa1-fa7)
  - `SiderAddons/` 7.8 GB (content mods)
  - Wine prefix 1.9 GB
- **Resolution:** 1600 x 900, Full Screen
- **VSync:** Enable1 (60 fps cap)
- **Controller:** DirectInput (settings.dat = `67 04`)
- **Facepacks:** fa1-fa7 installed; fa8 and faA remain placeholders

## Performance (in-game)

- **FPS:** capped at 60 (VSync Enable1)
- **CPU usage:** ~30% of total (6 cores / 12 threads); FL_2027.exe uses ~2.4 cores
- **Temperature (Tctl):** ~56 C under sustained load (heavy CPU usage, performance governor)
- **iGPU edge:** ~60 C
- **Package power (PPT):** 52-60 W
- **Subjective:** smooth, playable, not hot

## Tools

- **FL27 Settings:** opens without launching the game; shows Display (1920x1080
  16:9, 60 fps, HDR, Vsync), Keyboard, Controller, Audio, Online, etc.
- **FL27 Switcher:** opens without launching the game; shows Theme Color, Render
  Lighting, Gameplay, Controller Buttons, Classic Teams, Lockdown, Replay Logo.
- **Root cause of earlier crashes:** Wine Mono (Settings) and missing Segoe UI
  font (Switcher); fixed with real .NET 4.8 and font mapping.

## Verification commands

```bash
./lutris/spfl2027-fix.sh check        # exit 0 = healthy
cd native && cargo test               # 31 + 19 + 5 tests
cd native && cargo build --release    # release build
```

## Notes

- CPU governor set to `performance` (high power draw, not power saving) during
  play. For lower power/heat, use `schedutil` or `powersave`
  (`sudo cpupower frequency-set -g schedutil`), at the cost of performance.
- Close the game with Ctrl+C in the launcher terminal, or run `spfl2027-stop`.
- If temperature exceeds 80 C sustained, lower in-game graphics (crowd, shadows,
  anti-aliasing) and check cooling.