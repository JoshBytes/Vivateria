# Vivateria — Claude Code instructions

Solo hobby project by Josh (SP Studios). Open-source evolution MMO.
**Read `docs/BRS.md` before any design or architecture work.** `docs/BACKLOG.md` is the task queue.

## Toolchain
- Godot 4.7.2 at `D:/Tools/Godot_v4.7.2` (client, GDScript)
- Rust 1.99 stable MSVC (server, `server/` Cargo workspace)
- Shell on this machine: Windows (PowerShell / Git Bash)

## Architecture in one breath
Thin Godot client ↔ (WebSocket) ↔ authoritative **deterministic** Rust server.
Shared message schemas live in `protocol/`. Godot physics is visual only and never decides
gameplay. External AI agents connect through the Agent API (WebSocket) and run on players' machines.

## Non-negotiable rules
1. **Determinism.** The server simulation uses fixed-point math only (no `f32`/`f64` in sim
   state), a fixed 20 Hz tick, no `HashMap` iteration order, no wall-clock time, no
   unseeded RNG. The **replay test must pass** before any commit to the server.
2. **The BRS is human-owned.** Never change `docs/BRS.md` decisions without Josh's explicit
   approval. Propose new questions via `/grill-me` and log them in `docs/grill-log.md`.
3. **IP.** No Spore/EA/Maxis names, assets, lore, audio or code. No GPL code (e.g. Thrive).
   New assets must be original or CC0/purchased, recorded in `assets/LICENSES.md`.
4. **Human-only game actions** (stage advancement, editor/evolution, alliances) must never
   be exposed to the Agent API.
5. **Unattended work** (routines, background agents): work on a branch, open a PR, never
   push to `main`, and stop and report if the replay test fails.

## Workflow
- Pick the top unchecked task in `docs/BACKLOG.md`; each task has a "Done when" line.
- Small commits, one task per branch (`m1/<short-name>`).
- Update `docs/BACKLOG.md` checkboxes as part of the PR.
