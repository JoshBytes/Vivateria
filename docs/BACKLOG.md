# Vivateria — Backlog

Task queue for Josh, Claude Code sessions, Codex and scheduled routines.
Take the **top unchecked task whose owner fits you**. Owners: `josh` (human-only), `attended`
(do with Josh present), `any` (safe for routines/agents).

## M1 — Local playable core
Goal: play from cell to creature solo against AI; replay test passes. (BRS §10)

- [ ] **M1-01 Repo scaffold** · owner: attended
  `client/` Godot 4.7.2 project, `server/` Cargo workspace, `protocol/` folder, `.gitignore`, CI stub.
  *Done when:* Godot opens `client/project.godot`; `cargo build` and `cargo test` pass in `server/`.
- [ ] **M1-02 Fixed-point math crate** · owner: attended
  `server/crates/fixed`: Q-format type, + − × ÷, sqrt, trig via lookup tables, deterministic RNG.
  *Done when:* unit tests pass; no floats in the public API.
- [ ] **M1-03 Tick loop + replay test** · owner: attended
  20 Hz simulation step over world state; input log; state hashing.
  *Done when:* replaying the same inputs twice gives byte-identical state hashes; this test runs in CI.
- [ ] **M1-04 Protocol v0** · owner: any
  Message schemas for connect, input, and state snapshot/delta in `protocol/`.
  *Done when:* schema doc + Rust types + GDScript decoder, round-trip test passes.
- [ ] **M1-05 WebSocket link** · owner: any
  Rust server accepts a local client; Godot client connects and renders server entities as placeholders.
  *Done when:* a moving dot driven by keyboard input in Godot is simulated by the server.
- [ ] **M1-06 Genome v0** · owner: attended
  A handful of genes → grown body → derived stats ("body is the stats", BRS §6.1).
  *Done when:* the same genome always grows the same body and stats (test); offspring vary with a seeded RNG.
- [ ] **M1-07 Cell stage v0** · owner: any
  Top-down plane; eat food, avoid predators, divide → opens genome editor.
  *Done when:* a full cell loop is playable end to end.
- [ ] **M1-08 Creature stage v0** · owner: any
  Crawl to land; leader + followers with directives; nest; population; endangered tiers.
  *Done when:* species can grow, decline to Critical, and go extinct → reseed.
- [ ] **M1-09 Built-in AI rivals & wildlife** · owner: any
  Utility-AI rival species and wildlife on the same planet.
  *Done when:* a solo session feels populated; AI uses only perception-limited observations.

## Human-only
- [ ] Extract Godot 4.7.2 (`D:/Tools/Godot_v4.7.2/*.zip`) · owner: josh
- [ ] Trademark check for "Vivateria" (USPTO, EUIPO) · owner: josh
- [ ] Optional: register vivateria.com · owner: josh
- [ ] Finish Codex CLI setup · owner: josh
