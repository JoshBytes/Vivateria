use vivateria_fixed::Pcg32;
use vivateria_sim::{run_replay, InputLog, PlayerInput, World};

const TICKS: u64 = 1200;
const SEED: u64 = 0xDEC0DE;

fn build_log() -> InputLog {
    let mut rng = Pcg32::new(777, 1);
    let mut log = InputLog::new();
    for t in 0..TICKS {
        // Dense in 200..400, sparse elsewhere.
        let dense = (200..400).contains(&t);
        for player in 0..4u32 {
            let fire = dense || rng.range_u32(10) == 0;
            if fire {
                log.push(
                    t,
                    PlayerInput {
                        player,
                        move_x: rng.range_i32(-1, 2) as i8,
                        move_y: rng.range_i32(-1, 2) as i8,
                    },
                );
            }
        }
    }
    log
}

#[test]
fn replay_same_inputs_gives_identical_state_hashes() {
    let log = build_log();
    assert!(log.len() > 500);
    let decoded = InputLog::decode(&log.encode()).expect("decode");
    assert_eq!(log, decoded);

    let a = run_replay(SEED, &log, TICKS);
    let b = run_replay(SEED, &decoded, TICKS);
    assert_eq!(a.len(), TICKS as usize);
    assert_eq!(a, b);
    assert_eq!(a.last(), b.last());
}

#[test]
fn changed_input_changes_hash_from_that_tick() {
    let log = build_log();
    let mut entries = log.entries().to_vec();
    let t = entries[10].0 as usize;
    entries[10].1.move_x = if entries[10].1.move_x == 1 { -1 } else { 1 };
    let other = InputLog::from_entries(entries);
    let a = run_replay(SEED, &log, TICKS);
    let b = run_replay(SEED, &other, TICKS);
    assert_eq!(a[..t], b[..t]);
    assert_ne!(a[t], b[t]);
}

/// Pin once tests can link: run `cargo test replay_golden_hash -- --nocapture`.
const GOLDEN_FINAL_HASH: Option<u64> = None;

#[test]
fn replay_golden_hash() {
    let hashes = run_replay(SEED, &build_log(), TICKS);
    let last = *hashes.last().unwrap();
    match GOLDEN_FINAL_HASH {
        Some(g) => assert_eq!(last, g),
        None => println!("GOLDEN_FINAL_HASH not pinned; computed {last:#018x}"),
    }
}

#[test]
fn different_seed_changes_hash() {
    let log = build_log();
    let a = run_replay(SEED, &log, TICKS);
    let b = run_replay(SEED + 1, &log, TICKS);
    assert_ne!(a.last(), b.last());
}

#[test]
fn mid_run_snapshot_matches_full_run() {
    let log = build_log();
    let full = run_replay(SEED, &log, TICKS);
    let prefix = run_replay(SEED, &log, 500);
    assert_eq!(prefix.as_slice(), &full[..500]);

    // Clone the world mid-run and continue: must follow the same trajectory.
    let mut w = World::new(SEED);
    for t in 0..500 {
        w.step(&log.inputs_for(t));
    }
    assert_eq!(w.state_hash(), full[499]);
    let mut snap = w.clone();
    for t in 500..TICKS {
        snap.step(&log.inputs_for(t));
    }
    assert_eq!(snap.state_hash(), *full.last().unwrap());
}
