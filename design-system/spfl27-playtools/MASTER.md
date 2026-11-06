# MASTER — SPFL27 Playtools

Design memory for the Rust app in this repository (`native/`, iced desktop GUI).
Lives next to the project so future polish passes can stay consistent.

## Identity

- Product: SP Football Life 2027 for Linux and Steam Deck — Playtools (setup + launcher + tools).
- Surface mode: Operate (a tool people use, not a showcase).
- Audience: Windows/PC football-game players on Linux; many are beginners; must work on Steam Deck.
- Style lock: **glassmorphism** (Operate mode). Translucent panels (0.62–0.96
  opacity) over a soft gradient canvas; light 1px borders; subtle outer shadow.
  iced 0.14 has no backdrop blur, so blur is approximated by gradients and
  translucency; text contrast is unchanged (tokens measured by the WCAG test).
- Type/palette: keep the token roles in `native/src/theme.rs`; no invented colours per page.

## Dials

- density: 7 (was effectively ~9 before the 2026-11-06 pass; lowered)
- motion: 2 (functional only)
- variance: 3

## 2026-11-06 glass pass

- Canvas shell paints a subtle gradient (dark + light).
- Cards are translucent gradient panels (surface-2 → surface).
- Sidebar and setup top bar use translucent surfaces over the gradient.
- Borders switch to translucent white (dark) / white + dark ring (light).
- All tokens pass the WCAG AA contrast test; 31 + 20 + 8 tests green;
  clippy shows only pre-existing warnings; release build verified.

## Critique history

| Tanggal | Permukaan | Skor /40 | P0 | P1 |
|---|---|---|---|---|
| 2026-11-06 | Playtools (Home, wizard, all pages) | 26/40 | 0 | 7 |

## 2026-11-06 correction pass (chosen scope: P1 + P2)

- Install page: native file pickers ("Browse") for installer and update files.
- PLAY: every failure path now shows a clear toast; success also toasts.
- Every page intro and the welcome screen rewritten to one or two plain sentences;
  the empty and misleading welcome control removed.
- V-Sync caption aligned with project docs (Enable1, 60 fps, reduced heat).
- Guides: `docs/01-installation.md` (+ `docs/02-tools.md`) created; OpenDocs works.
- Path chips truncate the middle; keyboard shortcuts documented in Help;
  small-text scale raised one step; user-facing naming unified to "Setup".
- Verification: 31 + 20 + 5 tests green; clippy shows only pre-existing warnings;
  release build verified at commit time (cargo build --release green). Live screenshot check pending
  (no screenshot tool available in the build session).

## Open items

- Live visual pass (light and dark) on Home, Setup and each tool page — pending a
  session with a display or the operator's confirmation.
- Lutris launch path and `--play` shortcut share the same "window gone but process
  stays" behaviour; desktop entry already cleans up automatically.
