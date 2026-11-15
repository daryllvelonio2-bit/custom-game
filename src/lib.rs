//! Game logic.
//!
//! This crate deliberately has **no GPU, windowing or audio dependency** so
//! that `cargo t` / `cargo l` (see `.cargo/config.toml`) stay in the seconds
//! range. Everything that touches `wgpu`, `winit` or `rodio` lives behind the
//! `app` feature, in the binary target.

/// Simulation step the game runs at, in seconds (60 Hz).
pub const FIXED_DT: f32 = 1.0 / 60.0;

/// Top-level game state. Add fields and systems here.
#[derive(Debug, Default)]
pub struct Game {
    frame_count: u64,
    elapsed: f32,
}

impl Game {
    pub fn new() -> Self {
        Self::default()
    }

    /// Advance the simulation by `dt` seconds.
    pub fn tick(&mut self, dt: f32) {
        if !dt.is_finite() || dt <= 0.0 {
            return;
        }
        self.frame_count += 1;
        self.elapsed += dt;
    }

    /// Number of simulated frames so far.
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Total simulated time in seconds.
    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_empty() {
        let game = Game::new();
        assert_eq!(game.frame_count(), 0);
        assert_eq!(game.elapsed(), 0.0);
    }

    #[test]
    fn tick_accumulates_time() {
        let mut game = Game::new();
        for _ in 0..60 {
            game.tick(FIXED_DT);
        }
        assert_eq!(game.frame_count(), 60);
        assert!((game.elapsed() - 1.0).abs() < 1e-4);
    }

    #[test]
    fn invalid_delta_is_ignored() {
        let mut game = Game::new();
        game.tick(0.0);
        game.tick(-1.0);
        game.tick(f32::NAN);
        assert_eq!(game.frame_count(), 0);
        assert_eq!(game.elapsed(), 0.0);
    }
}
