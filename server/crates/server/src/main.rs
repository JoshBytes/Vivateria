//! Vivateria authoritative server entry point.

use vivateria_sim::{InputLog, World};

fn main() {
    println!(
        "vivateria-server starting (sim tick rate: {} Hz)",
        vivateria_sim::TICK_HZ
    );
    // No real-time loop yet (networking is M1-05): step a few ticks and print the hash.
    let mut world = World::new(1);
    let log = InputLog::new();
    for t in 0..5 {
        world.step(&log.inputs_for(t));
    }
    println!("tick {} state hash {:016x}", world.tick, world.state_hash());
}
