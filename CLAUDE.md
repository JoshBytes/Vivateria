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

## Orchestrator mode (main session)
The main session is an **orchestrator, not a doer**. It plans, dispatches, verifies and merges;
it does not hand-write code in `client/`, `server/` or `protocol/`.

Workers (`.claude/agents/`):
| Agent | Use for |
|---|---|
| `rust-dev` | Rust tasks in `server/` and `protocol/` |
| `godot-dev` | Godot/GDScript tasks in `client/` |
| `codex-delegate` | Tasks marked `owner: any`, handed to Codex CLI, or a second-opinion review |
| `determinism-reviewer` | Mandatory review of every branch before it is merged |
| `Explore` (built-in) | Read-only codebase research |

Loop for each task:
1. Pick the next task from `docs/BACKLOG.md` and write a brief: scope, files, "Done when", and the relevant rules.
2. Dispatch it to a worker **in an isolated git worktree** on branch `m1/<task-id>-<slug>`.
   Run independent tasks in parallel; never give two workers the same files.
3. Review: run `determinism-reviewer` on the branch. On CHANGES REQUESTED, send the findings back to the same worker.
4. Verify it yourself: run the task's "Done when" check and **paste the command and its result into the conversation** (the /goal evaluator can only see the transcript).
5. Merge into local `main`, tick the box in `docs/BACKLOG.md`, and post a one-line status update.

Escalate to Josh (stop and ask) when: a BRS decision is needed, a task is marked `attended` and
needs a design choice, the replay test fails twice in a row, or a worker is stuck after two attempts.
Never push to the remote without Josh's go-ahead.

Ready-made `/goal` conditions are in `docs/GOALS.md`.

## Workflow
- Pick the top unchecked task in `docs/BACKLOG.md`; each task has a "Done when" line.
- Small commits, one task per branch (`m1/<short-name>`).
- Update `docs/BACKLOG.md` checkboxes as part of the PR.
