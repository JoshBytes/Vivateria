# Vivateria — Agent instructions (Codex and others)

The project's standing instructions live in [`CLAUDE.md`](CLAUDE.md); they apply to every
coding agent, not only Claude. Read it, then `docs/BRS.md`, then `docs/BACKLOG.md`.

Key rules (summary — `CLAUDE.md` wins on any conflict):
- Deterministic fixed-point Rust server; the replay test must pass.
- Never edit `docs/BRS.md` decisions; never push to `main`; one task per branch, open a PR.
- No Spore/EA/Maxis assets or names; no GPL code.
- When working alongside Claude Code, take tasks marked `owner: codex` in the backlog,
  or act as a reviewer of open PRs. Don't edit files another agent's open branch is changing.
