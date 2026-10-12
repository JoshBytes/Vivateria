//! Deterministic Vivateria simulation core.
//!
//! Fixed 20 Hz tick, fixed-point math only, no floats, no wall-clock time.

pub mod hash;
pub mod input;
pub mod vec2;
pub mod world;

pub use input::{DecodeError, InputLog, PlayerId, PlayerInput};
pub use vec2::FxVec2;
pub use world::{Entity, EntityId, World, DT};

/// Fixed simulation tick rate in Hz.
pub const TICK_HZ: u32 = 20;

/// Replay `ticks` ticks from a fresh world, feeding `log.inputs_for(t)` at tick `t`
/// (0-based), and return the state hash after every tick.
pub fn run_replay(seed: u64, log: &InputLog, ticks: u64) -> Vec<u64> {
    let mut world = World::new(seed);
    let mut hashes = Vec::new();
    for t in 0..ticks {
        world.step(&log.inputs_for(t));
        hashes.push(world.state_hash());
    }
    hashes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_rate_is_20hz() {
        assert_eq!(TICK_HZ, 20);
    }
}
