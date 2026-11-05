# Agent instructions — sp-football-life-2027-linux

Installer and Playtools project for SP Football Life 2027 on Linux and Steam Deck.
Native Rust launcher (`native/`), Lutris installer (`lutris/`), Sider Lua module
(`sider/modules/`), and CPK content packs (`data/cpk/`).

```text
native/src/{main,lib,core,deps,ui,theme}.rs + tests/e2e.rs + assets/ + run.sh
lutris/sp-football-life-2027.yaml + spfl2027-fix.sh + spfl2027-stop.sh
sider/modules/aspectratio.lua
data/cpk/{Classics,Controller/{ps3,ps4,ps5,xbox},GP/}  (empty placeholders;
  real CPKs downloaded separately — not committed)
README.md  INSTALL.md  AGENTS.md  TEST_NOTES.md
```

## Build / test / lint (exact)

- `cd native && cargo build --release` (or `./native/run.sh` from the repo root)
- `cd native && cargo test` (e2e suite in `native/tests/e2e.rs`: 31 + 19 + 5 tests)
- `cd native && cargo clippy --all-targets` (6 pre-existing style warnings)
- `lutris -i lutris/sp-football-life-2027.yaml` (needs Lutris + Wine)
- `./lutris/spfl2027-fix.sh check` (health of the Settings/Switcher tools in a prefix)
- `./lutris/spfl2027-fix.sh` (repair)
- `spfl2027-stop` (stop a stuck game)

## Coding conventions

- Rust package `spfl27` (edition 2021, needs Rust >= 1.88, iced 0.14):
  logic in `core.rs`/`deps.rs`, presentation in `ui.rs`/`theme.rs`.
- `run.sh` is the single launcher entry (rebuild if stale, then exec);
  do not duplicate its build logic elsewhere.
- One Sider module = one self-contained `sider/modules/<name>.lua`; one content
  pack = one folder under `data/cpk/<Category>/<name>/`.

## Placement rules for new files

- New feature = extend the existing slice in its layer (Rust slice, Lua module,
  CPK folder, or Lutris yaml); never generic dumping folders (`utils/`,
  `helpers/`, `common/`, `misc/`, `stuff/`).
- Never put Rust code outside `native/`, CPK binaries inside `native/`, or
  installer logic outside `lutris/`.
- Launcher features must extend `native/tests/e2e.rs`.
- Real CPK binaries are not committed; keep `data/cpk/` placeholders only.

## Verification before claiming done

- `cargo test` (in `native/`) and a release build must be green
  (`verification-gate`); Lutris/Sider changes need a real install or game-side
  smoke check where possible, otherwise say so honestly.
- Run `./lutris/spfl2027-fix.sh check` for prefix health.
- Confirm no Indonesian (or any non-English) text in the project.

## Language rule

All project content must be in English. Do not introduce any other language into
the repository (source, comments, docs, scripts, YAML descriptions). The
`intros_are_english_only` test enforces this for UI guides.

## Safety

- All scripts are plain bash; read them before running.
- All Rust code compiles from source; no binary blobs are executed except the
  game's own installers from SmokePatch.
- Backups are kept: `xinput1_3.dll.bak`, `download-backup/`.

## Agent entry points

- Health: `./lutris/spfl2027-fix.sh check` (exit 0 = healthy).
- Repair: `./lutris/spfl2027-fix.sh` (idempotent).
- Stop: `spfl2027-stop`.
- Install: `lutris/sp-football-life-2027.yaml`.
- Overview: `README.md`.
- Full install guide: `INSTALL.md`.
- This file: `AGENTS.md`.