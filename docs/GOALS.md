# Vivateria — /goal conditions

Paste one of these into a fresh Claude Code session started in the repo root. For unattended
turns, run it in **auto mode**. The evaluator only sees the conversation, so every condition
asks the orchestrator to paste its proof.

---

## G1 — M1 foundation (M1-01 → M1-03) · attended

```
/goal Acting as orchestrator per CLAUDE.md (delegate all code to subagents in worktrees, review every branch with determinism-reviewer), complete backlog tasks M1-01, M1-02 and M1-03. Done when, in the latest turn, the conversation shows: (1) `cargo test --workspace` in server/ exiting 0 with a replay test that runs the same input log twice and asserts identical state hashes; (2) `cargo clippy --workspace -- -D warnings` exiting 0; (3) the Godot 4.7.2 console executable run with `--headless --path client --quit` exiting 0 with no errors; (4) docs/BACKLOG.md with M1-01..M1-03 checked; (5) `git log --oneline main` showing each task merged from its own branch. Do not push to the remote and do not edit docs/BRS.md. Stop and ask me if a design decision is needed, or after 40 turns.
```

## G2 — M1 networking (M1-04 → M1-05)

```
/goal Acting as orchestrator per CLAUDE.md, complete M1-04 and M1-05. Done when the latest turn shows: `cargo test --workspace` exits 0 including a protocol round-trip test; the server and the headless Godot client both start, the client connects, and the server log shows input received and state sent; and BACKLOG.md has both tasks checked. Delegate M1-04 to codex-delegate if Codex is available, otherwise to rust-dev. No pushing, no BRS edits. Stop and ask me after 30 turns or on any design decision.
```

## G3 — M1 gameplay (M1-06 → M1-09)

```
/goal Acting as orchestrator per CLAUDE.md, complete M1-06 through M1-09. Done when the latest turn shows: all workspace tests pass (genome determinism, endangered tiers, extinction → reseed); the replay test still passes; and BACKLOG.md has M1-06..M1-09 checked. M1-06 is attended: present the genome design to me and get approval before dispatching it. No pushing, no BRS edits. Stop after 60 turns.
```

---

### Tips
- `/goal` shows progress; `/goal clear` stops it.
- Resuming the session restores an unfinished goal.
- Background subagents pause evaluation until they finish, so parallel workers are fine.
