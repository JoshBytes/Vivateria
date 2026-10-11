# Vivateria

> *Realm of living worlds.* An open-source evolution MMO where a new planet is born
> every day, species are led by humans and driven by AI agents, and the world runs
> on compute contributed by its players.

**Status:** Pre-production. Requirements are final ([BRS](docs/BRS.md)); starting milestone M1.

*Vivateria*: Latin-inspired, from *vivus* (living) + *terra* (land, world) + *-ia* (realm).

## Project layout

```
Vivateria/
├── README.md
├── CLAUDE.md              ← standing instructions for Claude Code sessions
├── AGENTS.md              ← standing instructions for Codex / other agents
├── docs/
│   ├── BRS.md             ← Business Requirements Specification (source of truth)
│   ├── BACKLOG.md         ← task queue for humans, sessions and routines
│   ├── GOALS.md           ← ready-made /goal conditions for orchestrator sessions
│   └── grill-log.md       ← every design decision and why
├── client/                ← Godot 4.7.2 client (GDScript)            [M1]
├── server/                ← Rust authoritative, deterministic server [M1]
├── protocol/              ← shared message schemas                   [M1]
└── .claude/
    ├── agents/            ← worker subagents (rust-dev, godot-dev, codex-delegate, determinism-reviewer)
    └── skills/            ← grill-me / grilling skills (MIT, Matt Pocock)
```

## Toolchain
- Godot **4.7.2**
- Rust **1.99** (stable, MSVC)

## Design process
Run `/grill-me` in Claude Code to interview through new design questions. Decisions are
logged in `docs/grill-log.md` and folded into `docs/BRS.md`.

## IP ground rules
Vivateria borrows **ideas and mechanics** from the evolution-game genre. It never uses Spore's
(EA/Maxis) assets, names, lore, audio, code, or exact look-and-feel. See BRS §9.

## License
- Code: **MIT OR Apache-2.0**, at your option ([LICENSE-MIT](LICENSE-MIT), [LICENSE-APACHE](LICENSE-APACHE))
- Docs & specs (`docs/`, `protocol/` specs): **CC BY 4.0**
- Assets: licensed individually, tracked in `assets/LICENSES.md`
- `.claude/skills/grill-me` and `grilling`: MIT © Matt Pocock (see `.claude/skills/LICENSE-mattpocock-skills`)
