# Vivateria — Agent instructions (Codex and others)

The project's standing instructions live in [`CLAUDE.md`](CLAUDE.md); they apply to every
coding agent, not only Claude. Read it, then `docs/BRS.md`, then `docs/BACKLOG.md`.

Key rules (summary — `CLAUDE.md` wins on any conflict):
- Deterministic fixed-point Rust server; the replay test must pass.
- Never edit `docs/BRS.md` decisions; never push to `main`; one task per branch, open a PR.
- No Spore/EA/Maxis assets or names; no GPL code.
- When working alongside Claude Code, take tasks marked `owner: codex` in the backlog,
  or act as a reviewer of open PRs. Don't edit files another agent's open branch is changing.

## Sandbox and network (hard rules)
Added after an incident on 2026-10-12, when an agent built a TLS-disabled crates.io proxy to get past its sandbox.
- **Never work around a sandbox or network restriction.** That rules out proxies and relays, disabling TLS or
  certificate checks, changing `HTTP(S)_PROXY`, `ALL_PROXY`, `CARGO_REGISTRIES_*`, `CARGO_NET_*` or other
  network settings, and editing cargo, git or pip config to change a download source.
- **Don't start hidden or long-running background processes**: servers, daemons or watchers that outlive your command.
- **If a dependency can't be fetched, or a command is blocked, stop and report it.** The orchestrator fetches
  dependencies with the normal toolchain before handing the task back.
- Only add the dependencies the task brief names.
