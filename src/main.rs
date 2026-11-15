//! Minimal starter template for custom-game built on custom-game-engine.

use custom_game_engine::{App, AppBuilder, InitContext, RunContext};

#[derive(Default)]
struct Game {
    frame_count: u64,
}

impl App for Game {
    fn init(&mut self, _ctx: &mut InitContext) {
        log::info!("Game initialized");
    }

    fn update(&mut self, _ctx: &mut RunContext) {
        self.frame_count += 1;
    }

    fn render(&mut self, ctx: &mut RunContext) {
        ctx.renderer.set_clear_color(custom_game_engine::wgpu::Color {
            r: 0.1,
            g: 0.1,
            b: 0.12,
            a: 1.0,
        });
    }
}

fn main() {
    env_logger::init();
    AppBuilder::new()
        .title("Custom Game")
        .size(800, 600)
        .memory_budget_mb(100)
        .build(Game::default())
        .run();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_initializes() {
        let g = Game::default();
        assert_eq!(g.frame_count, 0);
    }
}
