//! Binary entry point: wires [`custom_game::Game`] into the engine.
//!
//! Keep this file thin. Logic belongs in `src/lib.rs` so it can be built and
//! tested without pulling in wgpu/winit/rodio.

use custom_game_engine::{App, AppBuilder, InitContext, RunContext, wgpu};

use custom_game::Game;

struct GameApp {
    game: Game,
}

impl App for GameApp {
    fn init(&mut self, _ctx: &mut InitContext) {
        log::info!("game initialised");
    }

    fn update(&mut self, ctx: &mut RunContext) {
        self.game.tick(ctx.delta);
    }

    fn render(&mut self, ctx: &mut RunContext) {
        ctx.renderer.set_clear_color(wgpu::Color {
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
        .build(GameApp { game: Game::new() })
        .run();
}
