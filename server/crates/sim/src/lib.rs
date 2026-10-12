//! Deterministic Vivateria simulation core.
//!
//! The fixed 20 Hz tick loop will live here. Fixed-point math only; no floats.

/// Fixed simulation tick rate in Hz.
pub const TICK_HZ: u32 = 20;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_rate_is_20hz() {
        assert_eq!(TICK_HZ, 20);
    }
}
