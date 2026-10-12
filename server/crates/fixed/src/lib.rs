//! Deterministic fixed-point math (Q32.32) and PCG32 RNG for the Vivateria simulation.
//!
//! No floating point appears in any public signature or in simulation code paths.

mod fx;
mod rng;
mod tables;

pub use fx::Fx;
pub use rng::Pcg32;

#[cfg(test)]
#[allow(clippy::disallowed_types, clippy::float_arithmetic)]
mod table_gen {
    /// Regenerates `src/tables.rs` contents (run with `--ignored --nocapture`).
    #[test]
    #[ignore]
    fn print_tables() {
        let scale = 4294967296.0f64;
        println!("pub const SIN_QUARTER: [i64; 1025] = [");
        for i in 0..=1024 {
            let t = f64::from(i) / 1024.0;
            println!(
                "    {},",
                ((t * core::f64::consts::FRAC_PI_2).sin() * scale).round() as i64
            );
        }
        println!("];\npub const ATAN_UNIT: [i64; 1025] = [");
        for i in 0..=1024 {
            let t = f64::from(i) / 1024.0;
            println!("    {},", (t.atan() * scale).round() as i64);
        }
        println!("];");
    }
}
