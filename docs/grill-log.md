# Vivateria — Grill Log

Every design decision from `/grill-me` sessions, in order. Settled items are folded into `BRS.md`.

| # | Question | Decision | Status |
|---|---|---|---|
| Q1 | Release type | Free hobby / passion project | ✅ Settled |
| Q2 | Team | Solo | ✅ Settled |
| Q3 | Timeline | No deadlines, no planned release | ✅ Settled |
| Q4 | Core fantasy | Creativity + survival + civilization, plus online competitive play; other players can cause extinction; AI agents can control members of your species while the player leads | ✅ Settled |
| Q5 | Platform | PC only | ✅ Settled |
| Q6 | Language | GDScript (client) + Rust. Java dropped (no official Godot support) | 🟡 Final split decided by Q16 |
| Q7 | Players per world | **True MMO**. Open to distributed servers, Solana/crypto and even a custom language if needed | ✅ Settled (high risk, see note) |
| Q8 | World lifecycle | Persistent **cosmos**. A **new planet spawns each day** for newcomers, with starting protection to encourage cooperation. Body parts decide whether species can communicate | ✅ Settled |
| Q9 | Extinction | Reseed as a new cell elsewhere in the same world | ✅ Settled |
| Q10 | Agents | Full alternative to built-in AI under the fairness rule (perception-limited, human-rate actions). **Agent orchestration is a core skill gap**. Doubles as an AI research platform | ✅ Settled |
| Q11 | Agent leader | Allowed, but an agent leader **cannot advance stages**. Progression needs a human in the loop | ✅ Settled |
| Q12 | First stages | **Cell + Creature** first, sharing one world. Cross-stage interaction (e.g. symbiosis between early creature players and cell players) | ✅ Settled |

| Q13 | Build path | M1 local server+client → M2 small online + Agent API → M3 multi-server cosmos → M4 distributed network. Server is a separate process from day one | ✅ Settled |
| Q14 | Server tech | Standalone **Rust** authoritative server; Godot/GDScript thin client | ✅ Settled |
| Q15 | Crypto | **No crypto/tokens.** Interested in **distributed computing**: every player and agent helps run the global server | ✅ Settled (raises Q22–Q23) |
| Q16 | Custom language | No. Rust + language-neutral agent protocol | ✅ Settled |
| Q17 | Agent API | WebSocket + published schema; agents run on the player's own machine | ✅ Settled |
| Q18 | Human-only actions | Stage advancement, editor/evolution, alliances | ✅ Settled |
| Q19 | Protection | New planets protected from **other planets for 7 days**. On the same planet, **only 1 hour** of protection, then extinction is fair game ("if you're late you're late") | ✅ Settled |
| Q20 | Communication | Signal channels (sound / visual / chemical) set by body parts; shared channel → talk, otherwise body language only | ✅ Settled |
| Q21 | Cell ↔ creature | Cell stage is microscale on the same planet; environmental effects, symbiosis host organs, graduation to land | ✅ Settled |
| Q22 | Anti-cheat on player nodes | Each region simulated by ≥3 random non-involved nodes, majority wins; dev-run anchor nodes; **deterministic fixed-point sim from M1** | ✅ Settled |
| Q23 | Contribution | **Hosting is the entry ticket.** Required compute **scales with what you run**: more agents → more compute owed. Contribution earns no gameplay advantage beyond capacity. Optimizing contribution is an intended innovation space | ✅ Settled (raises Q32–Q33) |
| Q24 | Leader death / extinction | Take over another member. Species has thresholds: **endangered** → reproduction gets harder. If a player abandons their species, it stays as **wildlife** with the Agent API locked ("dumb creatures") | ✅ Settled (raises Q34–Q35) |
| Q25 | Spore keep/change/drop | Accepted, **except** look is **between realistic and cartoony** | ✅ Settled |
| Q26 | Editor body model | Wants something **groundbreaking** rather than plain segments + IK | 🔄 Reopened as Q36 |
| Q27 | Art | Accepted "low-poly flat-shaded", but conflicts with Q25 | 🔄 Reconcile in Q31 |
| Q28 | Audio | Voices synthesized from vocal parts; CC0 + own work for the rest | ✅ Settled |
| Q29 | Cell loop | Eat → avoid → collect parts → divide (editor) → grow → land. Diets: plant / meat / filter / parasite-symbiont | ✅ Settled |
| Q30 | Creature loop | Explore → direct followers → grow population at nest → evolution points from encounters → evolve at nest → claim territory | ✅ Settled |
| Q31 | Art style | **Stylized realism**: believable anatomy, soft lighting, simplified painterly textures, gene-driven color | ✅ Settled (resolves Q27) |
| Q32 | Contribution metric | Verified work units; base ticket + per-agent cost; rolling 24h, no stockpiling | ✅ Settled |
| Q33 | Open node protocol | Yes. Any software is allowed if results match bit for bit | ✅ Settled |
| Q34 | Endangered tiers | Healthy / Endangered (<30% of peak) / Critical (≤5, leader-only reproduction) | ✅ Settled |
| Q35 | Abandoned species | Frozen design, huntable wildlife, unclaimable. **Plus:** progression for helping them recover across the world/galaxy ("you can be good too") | ✅ Settled (raises Q40) |
| Q36 | Editor | **(a) body is the stats + (b) genome-driven growth with variation** as core; **(c) learned locomotion** at M3+, IK fallback | ✅ Settled |
| Q37 | Identity | Player-named species + cosmos-wide public **tree of life** | ✅ Settled |
| Q38 | Winning | **No winning. The game is about progression.** Legacy rankings are progression markers | ✅ Settled |
| Q39 | Agent API v1 | Perception-only observations; move/eat/attack/flee/mate/signal/follow; no editor/stage/alliances | ✅ Settled |
| Q40 | Stewardship | Guard breeding grounds, drive off predators, share territory, symbiosis, cross-planet reintroduction (space stage). Rewards: evolution points, Steward ranking, tree-of-life credit | ✅ Settled |
| Q41 | M1 checklist | Repo layout → deterministic core + replay test → WebSocket link → genome v0 → cell v0 → creature v0 → AI rivals/wildlife | ✅ Settled |
| Q42 | Godot version | Godot **4.7.2** (installed at `D:/Tools/Godot_v4.7.2`) | ✅ Settled |
| Q43 | VCS & license | git + GitHub; open source; code MIT OR Apache-2.0, docs CC BY 4.0, assets individually | ✅ Settled |
| Q44 | Project name | **Vivateria** ("realm of living worlds": *vivus* + *terra* + *-ia*). Seedia dropped (existing Polish clean-tech company); Cladeborn dropped (too close to the evolution game *Clade*). Repo: github.com/JoshBytes/Vivateria | ✅ Settled |

### Risk note (Q7)
A solo hobby MMO is the single biggest risk to the project. Mitigation: treat the MMO as the
**north star architecture** while building in milestones that are each playable on their own (see Q13).

## Round 1 — 2026-10-11
Answered.

## Round 2 — 2026-10-11
Answered. Scope grew to MMO + persistent cosmos. Round 3 covers the build path, server tech,
crypto's role, and the cross-stage / communication mechanics.

## Round 3 — 2026-10-11
Answered. New branch: player-contributed distributed servers (trust, determinism, contribution rules).

## Round 4 — 2026-10-11
Answered. Open threads: art-style conflict, compute accounting, endangered/abandon rules, editor innovation.

## Round 5 — 2026-10-11
Answered. Remaining: stewardship mechanic, M1 build checklist, repo & license.

## Round 6 — 2026-10-11
Answered. Frontier empty except for derived defaults marked **(proposed)** in the BRS (audience, world structure, UGC, target hardware, persistence, out-of-scope). Awaiting shared-understanding confirmation.
