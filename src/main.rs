use custom_game_engine::{App, AppBuilder, InitContext, RunContext};
use winit::keyboard::{Key, NamedKey};

/// Screen dimensions (matches AppBuilder size).
const SCREEN_W: f32 = 800.0;
const SCREEN_H: f32 = 600.0;

/// World-to-clip scale (aspect-corrected so world circles stay circular).
const SCALE_X: f32 = 1.0 / 12.0;
const SCALE_Y: f32 = 1.0 / 9.0;

/// Tuning constants.
const PLAYER_SPEED: f32 = 0.018;
const PLAYER_SIZE: f32 = 0.045;
const PLAYER_MAX_HP: i32 = 100;
const BULLET_SPEED: f32 = 0.05;
const BULLET_SIZE: f32 = 0.010;
const BULLET_TTL: f32 = 1.0;
const ZOMBIE_BASE_SPEED: f32 = 0.006;
const ZOMBIE_SIZE: f32 = 0.040;
const LOOT_SIZE: f32 = 0.030;
const SHOOT_COOLDOWN: f32 = 0.18;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Player,
    Zombie,
    Bullet,
    Loot,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum LootType {
    Health,
}

struct Entity {
    kind: Kind,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    w: f32,
    h: f32,
    color: [f32; 4],
    hp: i32,
    damage: i32,
    alive: bool,
    ttl: Option<f32>,
    loot_type: Option<LootType>,
}

impl Entity {
    fn player(x: f32, y: f32) -> Self {
        Self {
            kind: Kind::Player,
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            w: PLAYER_SIZE,
            h: PLAYER_SIZE,
            color: [0.2, 0.8, 0.3, 1.0],
            hp: PLAYER_MAX_HP,
            damage: 0,
            alive: true,
            ttl: None,
            loot_type: None,
        }
    }

    fn zombie(x: f32, y: f32, hp: i32) -> Self {
        Self {
            kind: Kind::Zombie,
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            w: ZOMBIE_SIZE,
            h: ZOMBIE_SIZE,
            color: [0.8, 0.2, 0.2, 1.0],
            hp,
            damage: 15,
            alive: true,
            ttl: None,
            loot_type: None,
        }
    }

    fn bullet(x: f32, y: f32, vx: f32, vy: f32) -> Self {
        Self {
            kind: Kind::Bullet,
            x,
            y,
            vx,
            vy,
            w: BULLET_SIZE,
            h: BULLET_SIZE,
            color: [0.9, 0.9, 0.2, 1.0],
            hp: 1,
            damage: 25,
            alive: true,
            ttl: Some(BULLET_TTL),
            loot_type: None,
        }
    }

    fn loot(x: f32, y: f32, lt: LootType) -> Self {
        let color = match lt {
            LootType::Health => [0.2, 0.9, 0.4, 1.0],
        };
        Self {
            kind: Kind::Loot,
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            w: LOOT_SIZE,
            h: LOOT_SIZE,
            color,
            hp: 1,
            damage: 0,
            alive: true,
            ttl: None,
            loot_type: Some(lt),
        }
    }
}

struct Game {
    entities: Vec<Entity>,
    player_x: f32,
    player_y: f32,
    dir_x: f32,
    dir_y: f32,
    hp: i32,
    score: u32,
    wave: u32,
    killed: u32,
    shoot_cd: f32,
    spawn_cd: f32,
    loot_cd: f32,
    game_over: bool,
    frame: u32,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            entities: vec![Entity::player(0.0, 0.0)],
            player_x: 0.0,
            player_y: 0.0,
            dir_x: 1.0,
            dir_y: 0.0,
            hp: PLAYER_MAX_HP,
            score: 0,
            wave: 1,
            killed: 0,
            shoot_cd: 0.0,
            spawn_cd: 0.0,
            loot_cd: 0.0,
            game_over: false,
            frame: 0,
        }
    }
}

impl App for Game {
    fn init(&mut self, _ctx: &mut InitContext) {}

    fn update(&mut self, ctx: &mut RunContext) {
        self.frame += 1;
        let dt = ctx.delta;

        // Update cooldowns.
        if self.shoot_cd > 0.0 {
            self.shoot_cd -= dt;
        }
        self.spawn_cd -= dt;
        self.loot_cd -= dt;

        if self.game_over {
            // Restart on R.
            for key in &ctx.input.keys_just_pressed {
                if matches!(key, Key::Character(s) if s == "r") {
                    self.reset();
                }
            }
            return;
        }

        // ---- Player movement (held keys) ----
        let mut dx = 0.0f32;
        let mut dy = 0.0f32;
        for key in &ctx.input.keys_held {
            match key {
                Key::Named(NamedKey::ArrowUp) => dy -= 1.0,
                Key::Character(s) if s == "w" => dy -= 1.0,
                Key::Named(NamedKey::ArrowDown) => dy += 1.0,
                Key::Character(s) if s == "s" => dy += 1.0,
                Key::Named(NamedKey::ArrowLeft) => dx -= 1.0,
                Key::Character(s) if s == "a" => dx -= 1.0,
                Key::Named(NamedKey::ArrowRight) => dx += 1.0,
                Key::Character(s) if s == "d" => dx += 1.0,
                _ => {}
            }
        }
        let len = (dx * dx + dy * dy).sqrt();
        if len > 0.0 {
            dx /= len;
            dy /= len;
            self.player_x += dx * PLAYER_SPEED;
            self.player_y += dy * PLAYER_SPEED;
            self.dir_x = dx;
            self.dir_y = dy;
        }

        // Keep player within world bounds.
        const WORLD_LIMIT: f32 = 200.0;
        self.player_x = self.player_x.clamp(-WORLD_LIMIT, WORLD_LIMIT);
        self.player_y = self.player_y.clamp(-WORLD_LIMIT, WORLD_LIMIT);

        // Update player entity position.
        if let Some(p) = self.entities.iter_mut().find(|e| e.kind == Kind::Player) {
            p.x = self.player_x;
            p.y = self.player_y;
        }

        // ---- Shooting ----
        if self.shoot_cd <= 0.0 {
            for key in &ctx.input.keys_just_pressed {
                if matches!(key, Key::Named(NamedKey::Space)) {
                    self.entities.push(Entity::bullet(
                        self.player_x + self.dir_x * 0.06,
                        self.player_y + self.dir_y * 0.06,
                        self.dir_x * BULLET_SPEED,
                        self.dir_y * BULLET_SPEED,
                    ));
                    self.shoot_cd = SHOOT_COOLDOWN;
                    break;
                }
            }
        }

        // ---- Zombie spawning ----
        let zombies_needed = (self.wave as f32 * 1.5) as usize + 2;
        let zombie_count = self
            .entities
            .iter()
            .filter(|e| e.kind == Kind::Zombie && e.alive)
            .count();
        if self.spawn_cd <= 0.0 && zombie_count < zombies_needed {
            let hp = 30 + (self.wave as i32 * 10);
            // Spawn at a random edge position outside the visible screen.
            let angle = (self.frame as f32 * 0.7 + zombie_count as f32 * 2.3).to_radians();
            let dist = 3.0 + (self.frame as f32 % 5.0);
            let sx = self.player_x + angle.cos() * dist;
            let sy = self.player_y + angle.sin() * dist;
            let mut z = Entity::zombie(sx, sy, hp);
            // Point velocity toward player (chase logic applied in movement step).
            z.vx = self.player_x - sx;
            z.vy = self.player_y - sy;
            let zl = (z.vx * z.vx + z.vy * z.vy).sqrt();
            let zs = ZOMBIE_BASE_SPEED + (self.wave as f32 * 0.0008);
            if zl > 0.0 {
                z.vx = z.vx / zl * zs;
                z.vy = z.vy / zl * zs;
            }
            self.entities.push(z);
            self.spawn_cd = 1.2 - (self.wave as f32 * 0.08).clamp(0.0, 0.9);
        }

        // ---- Entity movement ----
        let player_pos = (self.player_x, self.player_y);
        for e in self.entities.iter_mut() {
            if !e.alive {
                continue;
            }
            match e.kind {
                Kind::Zombie => {
                    // Chase player.
                    let tx = player_pos.0 - e.x;
                    let ty = player_pos.1 - e.y;
                    let l = (tx * tx + ty * ty).sqrt();
                    if l > 0.0 {
                        e.vx = tx / l * ZOMBIE_BASE_SPEED
                            + (self.wave as f32 * 0.0005);
                        e.vy = ty / l * ZOMBIE_BASE_SPEED
                            + (self.wave as f32 * 0.0005);
                    }
                    e.x += e.vx;
                    e.y += e.vy;
                }
                Kind::Bullet => {
                    e.x += e.vx;
                    e.y += e.vy;
                    if let Some(t) = e.ttl.as_mut() {
                        *t -= dt;
                        if *t <= 0.0 {
                            e.alive = false;
                        }
                    }
                }
                Kind::Loot => {
                    // Slow float.
                    e.y += 0.002 * dt * 60.0;
                }
                Kind::Player => {}
            }
        }

        // ---- Collisions ----
        let player_rect = (
            self.player_x - PLAYER_SIZE * 0.5,
            self.player_y - PLAYER_SIZE * 0.5,
            PLAYER_SIZE,
            PLAYER_SIZE,
        );

        // Bullet vs Zombie: collect hits first (avoids double mutable borrow).
        let mut bullet_hits: Vec<(usize, usize, i32)> = Vec::new();
        for (bi, b) in self.entities.iter().enumerate() {
            if b.kind != Kind::Bullet || !b.alive {
                continue;
            }
            for (zi, z) in self.entities.iter().enumerate() {
                if z.kind != Kind::Zombie || !z.alive {
                    continue;
                }
                if rects_overlap(
                    b.x - b.w * 0.5,
                    b.y - b.h * 0.5,
                    b.w,
                    b.h,
                    z.x - z.w * 0.5,
                    z.y - z.h * 0.5,
                    z.w,
                    z.h,
                ) {
                    bullet_hits.push((bi, zi, b.damage));
                    break;
                }
            }
        }

        // Apply bullet hits.
        for (bi, zi, dmg) in bullet_hits {
            self.entities[bi].alive = false;
            let zombie_died = {
                let z = &mut self.entities[zi];
                z.hp -= dmg;
                z.hp <= 0
            };
            if zombie_died {
                let z = &mut self.entities[zi];
                z.alive = false;
                self.killed += 1;
                self.score += 100;
                if fastrand_float() < 0.15 {
                    let z = &self.entities[zi];
                    self.entities.push(Entity::loot(z.x, z.y, LootType::Health));
                }
            }
        }

        // Zombie vs Player.
        for z in self.entities.iter() {
            if z.kind != Kind::Zombie || !z.alive {
                continue;
            }
            if rects_overlap(
                player_rect.0, player_rect.1, player_rect.2, player_rect.3,
                z.x - z.w * 0.5, z.y - z.h * 0.5, z.w, z.h,
            ) {
                self.hp -= z.damage as i32;
                if self.hp <= 0 {
                    self.hp = 0;
                    self.game_over = true;
                }
            }
        }

        // Player vs Loot.
        for l in self.entities.iter_mut() {
            if l.kind != Kind::Loot || !l.alive {
                continue;
            }
            if rects_overlap(
                player_rect.0, player_rect.1, player_rect.2, player_rect.3,
                l.x - l.w * 0.5, l.y - l.h * 0.5, l.w, l.h,
            ) {
                l.alive = false;
                match l.loot_type.unwrap_or(LootType::Health) {
                    LootType::Health => {
                        self.hp = (self.hp + 25).clamp(0, PLAYER_MAX_HP);
                        self.score += 50;
                    }
                }
            }
        }

        // ---- Wave progression ----
        if self.killed >= self.wave * 10 {
            self.wave += 1;
        }

        // ---- Cleanup dead entities ----
        self.entities.retain(|e| e.alive);
    }

    fn render(&mut self, ctx: &mut RunContext) {
        let r = &mut ctx.renderer;

        // World-to-clip transform (camera centered on player).
        let wx = |w: f32| (w - self.player_x) * SCALE_X;
        let wy = |w: f32| (w - self.player_y) * SCALE_Y;
        let ws = |w: f32| w * SCALE_X;
        let hs = |h: f32| h * SCALE_Y;

        if self.game_over {
            // Semi-transparent red tint via full-screen rect.
            r.add_rect(0.0, 0.0, 2.0, 2.0, [0.6, 0.1, 0.1, 0.45]);
            r.add_rect(
                0.0,
                0.0,
                0.5,
                0.12,
                [0.95, 0.2, 0.2, 1.0],
            ); // "GAME OVER" bar
            r.add_rect(0.0, 0.14, 0.42, 0.07, [0.9, 0.9, 0.9, 1.0]); // "PRESS R"
            r.add_rect(0.0, 0.22, 0.34, 0.06, [0.7, 0.7, 0.7, 1.0]); // score hint
            return;
        }

        // ---- Entities ----
        for e in &self.entities {
            if !e.alive {
                continue;
            }
            r.add_rect(
                wx(e.x),
                wy(e.y),
                ws(e.w),
                hs(e.h),
                e.color,
            );
        }

        // ---- Player health bar (top-left corner) ----
        let hp_pct = self.hp as f32 / PLAYER_MAX_HP as f32;
        r.add_rect(-0.92, 0.86, 0.18, 0.035, [0.2, 0.2, 0.25, 1.0]); // background
        r.add_rect(
            -0.92 + 0.18 * (1.0 - hp_pct) * 0.5,
            0.86,
            0.18 * hp_pct,
            0.035,
            [0.2, 0.8, 0.3, 1.0],
        ); // fill

        // ---- Wave / score marker (top-right) ----
        r.add_rect(0.74, 0.86, 0.16, 0.035, [0.2, 0.2, 0.25, 1.0]);
        r.add_rect(
            0.74 + 0.16 * 0.5 - 0.08 * (self.wave as f32 / 10.0).min(1.0) * 0.5,
            0.86,
            0.16 * (self.wave as f32 / 10.0).min(1.0),
            0.035,
            [0.9, 0.7, 0.2, 1.0],
        );
    }
}

impl Game {
    fn reset(&mut self) {
        self.entities = vec![Entity::player(0.0, 0.0)];
        self.player_x = 0.0;
        self.player_y = 0.0;
        self.dir_x = 1.0;
        self.dir_y = 0.0;
        self.hp = PLAYER_MAX_HP;
        self.score = 0;
        self.wave = 1;
        self.killed = 0;
        self.shoot_cd = 0.0;
        self.spawn_cd = 0.0;
        self.loot_cd = 0.0;
        self.game_over = false;
        self.frame = 0;
    }
}

/// Axis-aligned rectangle overlap test.
fn rects_overlap(
    x0: f32,
    y0: f32,
    w0: f32,
    h0: f32,
    x1: f32,
    y1: f32,
    w1: f32,
    h1: f32,
) -> bool {
    x0 < x1 + w1 && x0 + w0 > x1 && y0 < y1 + h1 && y0 + h0 > y1
}

/// Deterministic-ish float in [0, 1) from the frame counter.
fn fastrand_float() -> f32 {
    // Simple LCG-like hash using frame time variance.
    use std::time::SystemTime;
    let n = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64 % 1000)
        .unwrap_or(0) as f32;
    (n / 1000.0).clamp(0.0, 0.999)
}

fn main() {
    env_logger::init();
    AppBuilder::new()
        .title("sombie-survivor")
        .size(800, 600)
        .vsync(false)
        .memory_budget_mb(100)
        .build(Game::default())
        .run();
}
