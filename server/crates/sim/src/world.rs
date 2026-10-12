//! World state and the fixed-tick step function.

use std::collections::BTreeMap;

use vivateria_fixed::{Fx, Pcg32};

use crate::hash::Fnv1a64;
use crate::input::{PlayerId, PlayerInput};
use crate::vec2::FxVec2;

pub type EntityId = u32;

/// Seconds per tick.
pub const DT: Fx = Fx::from_ratio(1, 20);
/// Half-width of the square world; positions are clamped to `[-BOUND, BOUND]`.
pub const WORLD_BOUND: Fx = Fx::from_int(100);
const PLAYER_ACCEL: Fx = Fx::from_int(40);
const NPC_ACCEL: Fx = Fx::from_int(15);
const MAX_SPEED: Fx = Fx::from_int(12);
const FRICTION: Fx = Fx::from_ratio(15, 16);
const ENERGY_DRAIN: Fx = Fx::from_ratio(1, 100);
const NUM_PLAYERS: u32 = 4;
const NUM_NPCS: u32 = 12;
const RNG_STREAM: u64 = 0x5649_5641; // "VIVA"

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Entity {
    pub id: EntityId,
    pub owner: Option<PlayerId>,
    pub pos: FxVec2,
    pub vel: FxVec2,
    pub energy: Fx,
}

#[derive(Clone, Debug)]
pub struct World {
    pub tick: u64,
    rng: Pcg32,
    entities: BTreeMap<EntityId, Entity>,
}

impl World {
    /// Spawns `NUM_PLAYERS` player entities (owner = player ids 0..) and `NUM_NPCS` NPCs.
    pub fn new(seed: u64) -> World {
        let mut rng = Pcg32::new(seed, RNG_STREAM);
        let mut entities = BTreeMap::new();
        for id in 0..(NUM_PLAYERS + NUM_NPCS) {
            let pos = FxVec2::new(
                rng.fx_range(-WORLD_BOUND, WORLD_BOUND),
                rng.fx_range(-WORLD_BOUND, WORLD_BOUND),
            );
            let owner = if id < NUM_PLAYERS { Some(id) } else { None };
            entities.insert(
                id,
                Entity {
                    id,
                    owner,
                    pos,
                    vel: FxVec2::ZERO,
                    energy: Fx::from_int(100),
                },
            );
        }
        World {
            tick: 0,
            rng,
            entities,
        }
    }

    pub fn entities(&self) -> impl Iterator<Item = &Entity> {
        self.entities.values()
    }

    pub fn entity(&self, id: EntityId) -> Option<&Entity> {
        self.entities.get(&id)
    }

    /// Advance one fixed tick. Inputs are applied sorted by player id (stable, so
    /// log order breaks ties), then NPCs wander in id order, then everything integrates.
    pub fn step(&mut self, inputs: &[PlayerInput]) {
        let mut sorted: Vec<PlayerInput> = inputs.to_vec();
        sorted.sort_by_key(|i| i.player);
        for inp in &sorted {
            let dx = Fx::from_int(i32::from(inp.move_x.clamp(-1, 1)));
            let dy = Fx::from_int(i32::from(inp.move_y.clamp(-1, 1)));
            let accel = FxVec2::new(dx, dy).scale(PLAYER_ACCEL * DT);
            // BTreeMap iterates in id order.
            for e in self.entities.values_mut() {
                if e.owner == Some(inp.player) {
                    e.vel = e.vel + accel;
                }
            }
        }

        for e in self.entities.values_mut() {
            if e.owner.is_none() && self.rng.range_u32(8) == 0 {
                let heading = self.rng.fx_range(Fx::ZERO, Fx::TAU);
                let accel = FxVec2::new(heading.cos(), heading.sin()).scale(NPC_ACCEL);
                e.vel = e.vel + accel;
            }
        }

        for e in self.entities.values_mut() {
            let speed = e.vel.length();
            if speed > MAX_SPEED {
                e.vel = e.vel.scale(MAX_SPEED / speed);
            }
            e.pos = e.pos + e.vel.scale(DT);
            e.vel = e.vel.scale(FRICTION);
            if e.pos.x.abs() > WORLD_BOUND {
                e.pos.x = e.pos.x.clamp(-WORLD_BOUND, WORLD_BOUND);
                e.vel.x = Fx::ZERO;
            }
            if e.pos.y.abs() > WORLD_BOUND {
                e.pos.y = e.pos.y.clamp(-WORLD_BOUND, WORLD_BOUND);
                e.vel.y = Fx::ZERO;
            }
            e.energy = e.energy.saturating_sub(ENERGY_DRAIN).max(Fx::ZERO);
        }
        self.tick += 1;
    }

    /// FNV-1a 64 over the canonical little-endian encoding: tick, RNG state words,
    /// entity count, then each entity (id order): id, owner tag+id, pos, vel, energy.
    pub fn state_hash(&self) -> u64 {
        let mut h = Fnv1a64::new();
        h.write_u64(self.tick);
        let (s, inc) = self.rng.state();
        h.write_u64(s);
        h.write_u64(inc);
        h.write_u64(self.entities.len() as u64);
        for e in self.entities.values() {
            h.write_u32(e.id);
            match e.owner {
                None => h.write_u8(0),
                Some(p) => {
                    h.write_u8(1);
                    h.write_u32(p);
                }
            }
            h.write_i64(e.pos.x.to_raw());
            h.write_i64(e.pos.y.to_raw());
            h.write_i64(e.vel.x.to_raw());
            h.write_i64(e.vel.y.to_raw());
            h.write_i64(e.energy.to_raw());
        }
        h.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_counts_and_stepping() {
        let mut w = World::new(1);
        assert_eq!(w.entities().count(), (NUM_PLAYERS + NUM_NPCS) as usize);
        let h0 = w.state_hash();
        w.step(&[]);
        assert_eq!(w.tick, 1);
        assert_ne!(h0, w.state_hash());
    }

    #[test]
    fn input_moves_owned_entity() {
        let mut w = World::new(5);
        let before = w.entity(0).unwrap().pos;
        w.step(&[PlayerInput {
            player: 0,
            move_x: 100,
            move_y: 0,
        }]);
        assert!(w.entity(0).unwrap().pos.x > before.x || before.x >= WORLD_BOUND);
    }
}
