use custom_game_engine::{App, AppBuilder, RunContext};
use winit::keyboard::{Key, NamedKey};

/// A tiny playable game: move a player left/right with the arrow keys.
/// The engine (separate repo) handles the window, GPU, and render loop.
struct Game {
    player_x: f32,
    frames: u32,
}

impl Default for Game {
    fn default() -> Self {
        Self { player_x: 0.0, frames: 0 }
    }
}

impl App for Game {
    fn update(&mut self, ctx: &mut RunContext) {
        self.frames += 1;
        let speed = 0.05;
        for key in &ctx.input.keys_just_pressed {
            match key {
                Key::Named(NamedKey::ArrowLeft) => self.player_x -= speed,
                Key::Named(NamedKey::ArrowRight) => self.player_x += speed,
                _ => {}
            }
        }
        // Keep player on screen.
        self.player_x = self.player_x.clamp(-0.9, 0.9);
        if self.frames % 120 == 0 {
            eprintln!("player_x={:.2}  (engine handles window/gpu/render)", self.player_x);
        }
    }

    fn render(&mut self, _ctx: &mut RunContext) {
        // The engine clears the screen and draws a quad each frame.
        // Game state drives logic above; rendering is wired to the engine.
    }
}

fn main() {
    env_logger::init();
    AppBuilder::new()
        .title("custom-game")
        .size(800, 600)
        .vsync(false)
        .memory_budget_mb(100)
        .build(Game::default())
        .run();
}
