# Vivateria — Business Requirements Specification

| | |
|---|---|
| **Project** | Vivateria |
| **Studio** | SP Studios |
| **Engine** | Godot 4.7.2 + Rust server (rustc 1.99) |
| **Document status** | v1.0 — **Final** (confirmed 2026-10-11) |
| **Repository** | https://github.com/JoshBytes/Vivateria |
| **Last updated** | 2026-10-11 |
| **Decision record** | [`grill-log.md`](grill-log.md) (Q1–Q43) |

> Items marked **(proposed)** were derived defaults; accepted with the shared-understanding confirmation.

---

## 1. Vision
Vivateria is an evolution **MMO** set in a persistent cosmos. Every day a **new planet** is
born, and newcomers seed their species there. The player is their species' **leader**:
designing its genome, surviving, and growing it toward civilization. Other players can
drive a species **extinct**, and players can also help it come back. The rest of a species is
driven by built-in AI or by the player's own **AI agents**, and orchestrating those agents is a core
skill. The game runs on a **player-contributed compute network**, and it is meant as an open
platform for AI-agent and distributed-systems research.

**The game is about progression, not winning.**

### Pillars
1. **Creativity**: design a genome and watch a creature grow and come alive.
2. **Survival & adaptation**: evolution has real stakes; extinction is real.
3. **Civilization**: guide a species from first spark toward the stars.
4. **Shared, persistent cosmos**: cooperation, rivalry, and stewardship.
5. **Leader + agents**: immersive direct play combined with agent orchestration; humans stay in the loop for progression.

## 2. Business goals & constraints
- **Release type:** free hobby / passion project
- **Team:** solo
- **Timeline:** none. No deadlines, no planned release date
- **Budget:** ~$0. Free tooling; compute comes from players (§5.4)
- **Platforms:** PC only (Windows first; Linux and macOS exports)
- **Personal goals:** upskilling in Rust and distributed systems; contributing new technology to AI and distributed computing
- **Source:** open source (§9)

## 3. Target audience (proposed)
1. Fans of evolution and creature-creation games who want deeper, persistent stakes.
2. AI/agent builders and researchers who want a fair, open sandbox to compete in.
3. Distributed-systems and optimization tinkerers who want to make the node network faster.

## 4. Relationship to Spore
### 4.1 Kept (similar)
- Progression from cell to space
- An editor whose choices drive gameplay
- Diet shapes progression

### 4.2 Changed (different)
- A shared, persistent MMO cosmos instead of a single-player galaxy
- Leader + followers + AI agents instead of a single avatar
- A **genome-driven** editor where **the body is the stats** (§6.1)
- Symbiosis between players at different stages
- Communication channels determined by body parts
- Real extinction pressure, plus stewardship of abandoned species
- **Stylized realism** instead of a cartoon style

### 4.3 Dropped
- All Spore/EA/Maxis names, lore, art, audio and code
- Launching all five stages at once
- A Sporepedia-style sharing library (for now)

## 5. Scope & game rules
### 5.0 Stages
| Stage | Scope | Core loop | Notes |
|---|---|---|---|
| Cell | ✅ M1 | Eat → avoid predators → collect parts → divide (editor) → grow → crawl to land. Diets: plant / meat / filter / parasite-symbiont | Microscale layer of the same planet |
| Creature | ✅ M1 | Explore as leader → direct followers → grow population at nest → earn evolution points from encounters (hunting, alliances, symbiosis, discovery) → evolve at nest → claim territory | Can host cell-stage players symbiotically |
| Tribe | Later | | |
| Civilization | Later | | |
| Space | Later | | Links planets; cross-planet reintroduction |

### 5.1 World & multiplayer
- **Persistent MMO cosmos.** A new planet spawns daily for new players; older planets persist.
- **Protection:** a new planet is shielded from other planets for **7 days**. On the same planet, players are protected for only **1 hour**; after that extinction is fair game. Being late means negotiating or outsmarting others.
- **Extinction:** extinct players **reseed** as a new cell on the same planet.
- **Communication:** signal channels (**sound / visual / chemical**) come from body parts. Species that share a channel can talk, trade and ally; otherwise they read body language only.
- **Cross-stage play:** cell-stage players live in pools, oceans and soil on the same planet. Creature players affect them through the environment (feeding, pollution, disturbance) and **symbiosis host organs**. Cell players eventually graduate onto land.

### 5.2 Leader & agents
- The player is the species' **leader**; other members follow directives carried out by built-in AI or external agents.
- **Fairness:** every controller gets only what its creature perceives, and acts at a human-equivalent rate.
- **Agent orchestration is an intended skill gap.**
- **Agent API v1:** observations are perception-only (senses, own body state, leader directives). Actions are move / eat / attack / flee / mate / signal / follow directive.
- An agent may control the leader, but an agent-led species **cannot advance stages**.

### 5.3 Human-only actions
Stage advancement · opening the editor to evolve · forming or breaking alliances.

### 5.4 Species health
- If the leader dies, the player takes over another member.
- **Healthy** → **Endangered** (below 30% of peak population: slower mating, visible warning) → **Critical** (5 members or fewer: only the leader can reproduce) → **Extinct** (last member dies; the player reseeds).
- **Abandoned species** become **wildlife**: design frozen, built-in AI only, Agent API locked, huntable, never reclaimable.

### 5.5 Stewardship
- Help abandoned species recover: guard breeding grounds, drive off their predators, share territory, host them symbiotically, and (from the space stage) reintroduce them on other planets.
- Rewards: evolution points, the **Steward** ranking, and "rescued by \<species\>" entries in the tree of life.

### 5.6 Compute contribution (the entry ticket)
- Hosting part of the simulation is **required to play**. It is the same deal for everyone.
- Measured in **verified work units** (results confirmed by redundancy), not CPU time, over a rolling 24h window with no stockpiling.
- **Base ticket** covers your species' share of the simulation; **each extra agent slot** costs additional units.
- Contributing gives no gameplay advantage beyond capacity. Contributing more efficiently is an intended innovation space.
- The **node protocol is open**: any software or hardware is allowed if its results match bit for bit.
- **Anti-cheat:** each region is simulated by **3 or more random uninvolved nodes**; the majority result wins and nodes that disagree are flagged and dropped. Anchor nodes run by the developer bootstrap the network and break ties.

## 6. Core systems
### 6.1 Editor (genome-driven)
- The player edits **genes and growth rules**, and the body grows from them. Offspring vary, so natural selection acts on the population during play.
- **The body is the stats:** abilities come from anatomy through simplified biomechanics (leg length and mass → speed, jaw leverage → bite force, lung size → endurance). Parts carry no flat stat numbers.
- Parts determine **communication channels**, **diet**, and **symbiosis host organs**.

### 6.2 Locomotion & animation
- M1–M2: inverse-kinematics procedural locomotion (standard techniques).
- M3+: **learned locomotion**, where each body plan trains a small neural gait policy on contributed compute. IK remains the fallback.
- Godot physics is visual only; all gameplay outcomes come from the deterministic server simulation.

### 6.3 Progression
- **No winning.**
- Player-named species are recorded in a cosmos-wide public **tree of life** (ancestry, extinctions, symbioses, rescues).
- **Legacy rankings:** longest lineage, most descendant species, largest territory, most stable alliances, best agent orchestration (outcome per compute), Steward.

### 6.4 World / planet (proposed)
- Each planet is generated from a **seed**; the same seed always produces the same planet.
- Two scales on one planet: a **microscale** layer (cell stage) and a **land** layer (creature stage), linked by environmental effects.
- Planets are divided into **regions**, the unit of distributed simulation (§5.6).

### 6.5 Sharing / UGC (proposed)
- The **tree of life** is the public record of every species.
- There is no design-sharing library for now. Designs are earned through play, not downloaded.

## 7. Art & audio
- **Art:** **stylized realism**: believable anatomy, soft realistic lighting, simplified painterly textures. Closer to a nature documentary than a cartoon, but not photoreal. Color and patterns come from genes. The cell stage is viewed from above; the creature stage is third-person.
- **Audio:** creature voices are synthesized from vocal parts (tied to the sound channel). Other sound effects and music are CC0 or original.

## 8. Technical requirements
- **Client:** Godot 4.7.2, GDScript, thin client (rendering, input, prediction of your own leader only).
- **Server:** standalone **Rust** authoritative server; a separate process from M1.
- **Determinism:** fixed-point math, fixed 20 Hz tick, and a **replay test** (identical inputs must give byte-identical state) that runs on every build.
- **Networking:** WebSocket in M1–M2; UDP considered later.
- **Agent API:** WebSocket + published schema; agents run on players' machines, in any language.
- **Repo layout:** `client/` · `server/` · `protocol/` · `docs/`.
- **Target hardware (proposed):** a modest gaming PC (4-core CPU, 8 GB RAM, GTX 1060-class GPU) must be able to play **and** host its base ticket.
- **Persistence (proposed):** server-authoritative world state; M1 uses local snapshot files; distributed persistence is designed in M3–M4.
- **Not doing:** crypto/tokens, a custom programming language, Java.

## 9. Legal, IP & licensing
- No Spore/EA/Maxis names, logos, assets, audio, lore or code.
- Original names for stages, species, factions and items.
- All assets are original or properly licensed (CC0 / purchased), tracked in `assets/LICENSES.md`.
- No GPL code reuse (e.g. Thrive).
- Check creature-animation patents before any commercial use.
- **Licenses:** code is **MIT OR Apache-2.0**; docs and specs are **CC BY 4.0**; assets are licensed individually.
- **Hosting:** git + GitHub.

## 10. Milestones
No dates (hobby project). Each milestone must be playable on its own.

| Milestone | Scope | Exit criteria |
|---|---|---|
| **M1** | Local Rust server + Godot client; cell + creature v0 vs built-in AI | Play from cell to creature solo; replay test passes |
| M2 | One online server with friends; Agent API v1 | Friends + external agents on one planet |
| M3 | Multi-server cosmos; daily planets; learned locomotion begins | Daily planet spawning works across servers |
| M4 | Player-contributed distributed network | Regions simulated on player nodes with redundancy verification |

### M1 checklist
1. Repo layout (`client/`, `server/`, `protocol/`, `docs/`)
2. Deterministic core: fixed-point math, 20 Hz tick, replay test
3. Godot ↔ Rust connection over local WebSocket
4. Genome v0: a few genes → grown body → derived stats
5. Cell stage v0: top-down, eat/avoid, divide → editor
6. Creature stage v0: land, leader + followers with directives, nest, population, endangered tiers
7. Built-in AI rivals and wildlife

## 11. Out of scope (proposed)
- Tribe, civilization and space stages (until after M1–M2)
- Crypto, tokens, NFTs
- Consoles, mobile, VR
- Monetization of any kind
- A design-sharing library
- A custom programming language
