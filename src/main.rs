//! Sombie Survivor — a 2D top-down zombie survival game built on
//! `custom-game-engine` staying strictly within a <= 100 MB RAM budget.

use custom_game_engine::{wgpu, App, AppBuilder, Camera, InitContext, RunContext};
use winit::keyboard::{Key, NamedKey};

pub mod art;
pub mod entity;
pub mod world;

use art::id;
use entity::{rects_overlap, Entity, Kind, ZombieKind};
use world::{Biome, LootKind, Rng, World, CHUNK_SIZE};

const DAY_LENGTH: f32 = 90.0;
const NIGHT_START: f32 = 50.0;
const NIGHT_END: f32 = 80.0;

const PLAYER_BASE_SPEED: f32 = 5.2;
const PLAYER_SPRINT_MULT: f32 = 1.7;
const PLAYER_SIZE: f32 = 0.95;
const MAX_HP: i32 = 100;
const MAX_STAMINA: f32 = 100.0;
const MAX_HUNGER: f32 = 100.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Weapon {
    Pistol,
    Shotgun,
    Rifle,
}

impl Weapon {
    pub fn name(self) -> &'static str {
        match self {
            Weapon::Pistol => "Pistol 9mm",
            Weapon::Shotgun => "Shotgun 12G",
            Weapon::Rifle => "Hunting Rifle",
        }
    }

    pub fn cooldown(self) -> f32 {
        match self {
            Weapon::Pistol => 0.22,
            Weapon::Shotgun => 0.75,
            Weapon::Rifle => 0.40,
        }
    }

    pub fn damage(self) -> i32 {
        match self {
            Weapon::Pistol => 34,
            Weapon::Shotgun => 28,
            Weapon::Rifle => 75,
        }
    }

    pub fn bullet_speed(self) -> f32 {
        match self {
            Weapon::Pistol => 24.0,
            Weapon::Shotgun => 20.0,
            Weapon::Rifle => 34.0,
        }
    }

    pub fn pellets(self) -> usize {
        match self {
            Weapon::Shotgun => 5,
            _ => 1,
        }
    }

    pub fn spread(self) -> f32 {
        match self {
            Weapon::Pistol => 0.05,
            Weapon::Shotgun => 0.24,
            Weapon::Rifle => 0.02,
        }
    }
}

#[derive(Clone, Debug)]
pub struct BaseStructure {
    pub x: f32,
    pub y: f32,
    pub tile: u32,
    pub hp: i32,
    pub max_hp: i32,
    pub is_campfire: bool,
}

pub struct Game {
    pub atlas_texture: u32,
    pub world: World,
    pub entities: Vec<Entity>,
    pub structures: Vec<BaseStructure>,
    pub player_x: f32,
    pub player_y: f32,
    pub player_aim_x: f32,
    pub player_aim_y: f32,
    pub hp: i32,
    pub stamina: f32,
    pub hunger: f32,
    pub wood: u32,
    pub ammo: u32,
    pub weapon: Weapon,
    pub driving_car_index: Option<usize>,
    pub shoot_cooldown: f32,
    pub day_timer: f32,
    pub days_survived: u32,
    pub wave: u32,
    pub score: u32,
    pub kills: u32,
    pub horde_spawn_timer: f32,
    pub message: String,
    pub message_timer: f32,
    pub game_over: bool,
    pub frame: u64,
    pub rng: Rng,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            atlas_texture: 0,
            world: World::new(1337),
            entities: Vec::with_capacity(256),
            structures: Vec::with_capacity(64),
            player_x: 20.0,
            player_y: 20.0,
            player_aim_x: 1.0,
            player_aim_y: 0.0,
            hp: MAX_HP,
            stamina: MAX_STAMINA,
            hunger: MAX_HUNGER,
            wood: 12,
            ammo: 45,
            weapon: Weapon::Pistol,
            driving_car_index: None,
            shoot_cooldown: 0.0,
            day_timer: 15.0,
            days_survived: 1,
            wave: 1,
            score: 0,
            kills: 0,
            horde_spawn_timer: 5.0,
            message: "Survive the infection. WASD move, LMB aim & shoot, E interact/chop, F fence, C campfire.".into(),
            message_timer: 8.0,
            game_over: false,
            frame: 0,
            rng: Rng::new(54321),
        }
    }
}

impl Game {
    pub fn restart(&mut self) {
        let seed = self.rng.next_u64();
        *self = Self {
            atlas_texture: self.atlas_texture,
            world: World::new(seed),
            rng: Rng::new(seed ^ 0xABCD_EF01),
            ..Self::default()
        };
    }

    pub fn set_message(&mut self, text: impl Into<String>, dur: f32) {
        self.message = text.into();
        self.message_timer = dur;
    }

    pub fn is_night(&self) -> bool {
        self.day_timer >= NIGHT_START && self.day_timer <= NIGHT_END
    }

    pub fn night_factor(&self) -> f32 {
        if self.day_timer < NIGHT_START - 8.0 || self.day_timer > NIGHT_END + 8.0 {
            0.0
        } else if self.day_timer < NIGHT_START {
            (self.day_timer - (NIGHT_START - 8.0)) / 8.0
        } else if self.day_timer <= NIGHT_END {
            1.0
        } else {
            1.0 - (self.day_timer - NIGHT_END) / 8.0
        }
    }

    fn spawn_zombie_at(&mut self, x: f32, y: f32, kind: ZombieKind) {
        let mut z = Entity::zombie(x, y, kind);
        z.aggro = true;
        self.entities.push(z);
    }

    fn populate_new_chunk_loot(&mut self) {
        let (pcx, pcy) = World::chunk_of(self.player_x, self.player_y);
        for dy in -1..=1 {
            for dx in -1..=1 {
                let key = (pcx + dx, pcy + dy);
                if self.world.looted.insert(key) {
                    if let Some(chunk) = self.world.chunk(key.0, key.1) {
                        for prop in &chunk.props {
                            if let Some(loots) = &prop.loot {
                                for item in loots {
                                    self.entities.push(Entity::loot(item.x, item.y, item.kind));
                                }
                            }
                            if prop.tile == id::CAR {
                                self.entities.push(Entity::car(prop.x, prop.y, 0.0));
                            }
                        }
                    }
                }
            }
        }
    }

    fn check_solid_prop_collision(&self, x: f32, y: f32, radius: f32) -> bool {
        let (pcx, pcy) = World::chunk_of(x, y);
        for dy in -1..=1 {
            for dx in -1..=1 {
                if let Some(chunk) = self.world.chunk(pcx + dx, pcy + dy) {
                    for prop in &chunk.props {
                        if !prop.solid {
                            continue;
                        }
                        let pw = if prop.tile == id::WALL { 1.0 } else { 1.2 };
                        let ph = if prop.tile == id::WALL { 1.0 } else { 1.2 };
                        if rects_overlap(
                            x - radius,
                            y - radius,
                            radius * 2.0,
                            radius * 2.0,
                            prop.x - pw * 0.5,
                            prop.y - ph * 0.5,
                            pw,
                            ph,
                        ) {
                            return true;
                        }
                    }
                }
            }
        }
        for st in &self.structures {
            if !st.is_campfire
                && rects_overlap(
                    x - radius,
                    y - radius,
                    radius * 2.0,
                    radius * 2.0,
                    st.x - 0.5,
                    st.y - 0.5,
                    1.0,
                    1.0,
                )
            {
                return true;
            }
        }
        false
    }

    fn shoot_weapon(&mut self) {
        if self.ammo == 0 {
            self.set_message("Out of ammo! Search houses or loot zombies.", 2.5);
            return;
        }
        self.ammo -= 1;
        self.shoot_cooldown = self.weapon.cooldown();

        let base_angle = self.player_aim_y.atan2(self.player_aim_x);
        let pellets = self.weapon.pellets();
        let spread = self.weapon.spread();
        let speed = self.weapon.bullet_speed();
        let dmg = self.weapon.damage();

        for i in 0..pellets {
            let offset = if pellets == 1 {
                (self.rng.next_f32() - 0.5) * spread
            } else {
                -spread * 0.5 + (i as f32 / (pellets - 1) as f32) * spread
                    + (self.rng.next_f32() - 0.5) * 0.05
            };
            let angle = base_angle + offset;
            let vx = angle.cos() * speed;
            let vy = angle.sin() * speed;
            self.entities.push(Entity::bullet(
                self.player_x + self.player_aim_x * 0.6,
                self.player_y + self.player_aim_y * 0.6,
                vx,
                vy,
                dmg,
            ));
        }
    }
}

impl App for Game {
    fn init(&mut self, ctx: &mut InitContext) {
        let atlas = art::build_atlas();
        self.atlas_texture = ctx.renderer.create_texture(ctx.device, ctx.queue, &atlas);
        self.world.stream_around(self.player_x, self.player_y);
        self.populate_new_chunk_loot();
    }

    fn update(&mut self, ctx: &mut RunContext) {
        self.frame += 1;
        let dt = ctx.delta.min(0.05);

        if self.message_timer > 0.0 {
            self.message_timer -= dt;
        }

        if self.game_over {
            for key in &ctx.input.keys_just_pressed {
                if matches!(key, Key::Character(s) if s.eq_ignore_ascii_case("r")) {
                    self.restart();
                }
            }
            return;
        }

        // 1. Time & day cycle
        self.day_timer += dt;
        if self.day_timer >= DAY_LENGTH {
            self.day_timer = 0.0;
            self.days_survived += 1;
            self.wave += 1;
            self.set_message(
                format!("Day {} begins! Wave {} incoming.", self.days_survived, self.wave),
                4.0,
            );
        }

        // 2. Stream world chunks
        self.world.stream_around(self.player_x, self.player_y);
        self.populate_new_chunk_loot();

        // 3. Aim direction from mouse or keyboard fallback
        let (screen_w, screen_h) = ctx.renderer.size();
        let (cursor_px, cursor_py) = ctx.input.mouse_position;
        if cursor_px >= 0.0 && cursor_px <= screen_w as f32 && cursor_py >= 0.0 && cursor_py <= screen_h as f32 {
            let (target_wx, target_wy) = ctx.renderer.screen_to_world(cursor_px, cursor_py);
            let dx = target_wx - self.player_x;
            let dy = target_wy - self.player_y;
            let len = (dx * dx + dy * dy).sqrt();
            if len > 0.001 {
                self.player_aim_x = dx / len;
                self.player_aim_y = dy / len;
            }
        }

        // 4. Weapon hotkeys
        for key in &ctx.input.keys_just_pressed {
            match key {
                Key::Character(s) if s == "1" => self.weapon = Weapon::Pistol,
                Key::Character(s) if s == "2" => self.weapon = Weapon::Shotgun,
                Key::Character(s) if s == "3" => self.weapon = Weapon::Rifle,
                _ => {}
            }
        }

        // 5. Player movement & actions
        let mut move_x = 0.0f32;
        let mut move_y = 0.0f32;
        let mut sprinting = false;
        for key in &ctx.input.keys_held {
            match key {
                Key::Named(NamedKey::ArrowUp) => move_y += 1.0,
                Key::Character(s) if s.eq_ignore_ascii_case("w") => move_y += 1.0,
                Key::Named(NamedKey::ArrowDown) => move_y -= 1.0,
                Key::Character(s) if s.eq_ignore_ascii_case("s") => move_y -= 1.0,
                Key::Named(NamedKey::ArrowLeft) => move_x -= 1.0,
                Key::Character(s) if s.eq_ignore_ascii_case("a") => move_x -= 1.0,
                Key::Named(NamedKey::ArrowRight) => move_x += 1.0,
                Key::Character(s) if s.eq_ignore_ascii_case("d") => move_x += 1.0,
                Key::Named(NamedKey::Shift) => sprinting = true,
                _ => {}
            }
        }

        // Car driving or on-foot movement
        if let Some(car_idx) = self.driving_car_index {
            if car_idx < self.entities.len() && self.entities[car_idx].alive {
                let steer_speed = 3.2 * dt;
                let (mut angle, mut vx, mut vy, size, x, y) = {
                    let car = &self.entities[car_idx];
                    (car.angle, car.vx, car.vy, car.size, car.x, car.y)
                };
                if move_x < 0.0 {
                    angle += steer_speed;
                }
                if move_x > 0.0 {
                    angle -= steer_speed;
                }
                let accel = if move_y > 0.0 { 16.0 } else if move_y < 0.0 { -8.0 } else { 0.0 };
                let forward_x = angle.cos();
                let forward_y = angle.sin();
                vx += forward_x * accel * dt;
                vy += forward_y * accel * dt;
                vx *= 0.94;
                vy *= 0.94;

                let next_x = x + vx * dt;
                let next_y = y + vy * dt;
                let collides = self.check_solid_prop_collision(next_x, next_y, size * 0.45);

                let car = &mut self.entities[car_idx];
                car.angle = angle;
                if !collides {
                    car.vx = vx;
                    car.vy = vy;
                    car.x = next_x;
                    car.y = next_y;
                } else {
                    car.vx = -vx * 0.3;
                    car.vy = -vy * 0.3;
                    car.integrity -= 2.0;
                }
                self.player_x = car.x;
                self.player_y = car.y;

                // Exit car on E
                for key in &ctx.input.keys_just_pressed {
                    if matches!(key, Key::Character(s) if s.eq_ignore_ascii_case("e")) {
                        self.driving_car_index = None;
                        self.player_x += 1.5;
                        self.set_message("Exited vehicle.", 1.5);
                        break;
                    }
                }
            } else {
                self.driving_car_index = None;
            }
        } else {
            let move_len = (move_x * move_x + move_y * move_y).sqrt();
            if move_len > 0.0 {
                move_x /= move_len;
                move_y /= move_len;

                let can_sprint = sprinting && self.stamina > 5.0;
                let speed = if can_sprint {
                    self.stamina = (self.stamina - 18.0 * dt).max(0.0);
                    PLAYER_BASE_SPEED * PLAYER_SPRINT_MULT
                } else {
                    self.stamina = (self.stamina + 12.0 * dt).min(MAX_STAMINA);
                    PLAYER_BASE_SPEED
                };

                let step_x = move_x * speed * dt;
                let step_y = move_y * speed * dt;

                let try_x = self.player_x + step_x;
                if !self.check_solid_prop_collision(try_x, self.player_y, PLAYER_SIZE * 0.45) {
                    self.player_x = try_x;
                }
                let try_y = self.player_y + step_y;
                if !self.check_solid_prop_collision(self.player_x, try_y, PLAYER_SIZE * 0.45) {
                    self.player_y = try_y;
                }
            } else {
                self.stamina = (self.stamina + 20.0 * dt).min(MAX_STAMINA);
            }

            // Hunger decay
            self.hunger = (self.hunger - 0.45 * dt).max(0.0);
            if self.hunger <= 0.0 {
                self.hp -= (5.0 * dt) as i32;
                if self.hp <= 0 {
                    self.game_over = true;
                    self.set_message("Starvation claimed you! Press R to restart.", 10.0);
                }
            }

            // Shooting: LMB or Space
            if self.shoot_cooldown > 0.0 {
                self.shoot_cooldown -= dt;
            }
            let is_shooting = ctx.input.mouse_held.contains(&winit::event::MouseButton::Left)
                || ctx.input.keys_held.iter().any(|k| matches!(k, Key::Named(NamedKey::Space)));
            if is_shooting && self.shoot_cooldown <= 0.0 {
                self.shoot_weapon();
            }

            // Interact / Action keys
            for key in &ctx.input.keys_just_pressed {
                match key {
                    // E: Interact (chop tree, enter car, fish)
                    Key::Character(s) if s.eq_ignore_ascii_case("e") => {
                        let mut did_interact = false;
                        // 1. Enter car
                        for (idx, e) in self.entities.iter().enumerate() {
                            if e.kind == Kind::Car && e.alive {
                                let dist = ((e.x - self.player_x).powi(2) + (e.y - self.player_y).powi(2)).sqrt();
                                if dist < 2.5 {
                                    self.driving_car_index = Some(idx);
                                    self.set_message("Entered vehicle. WASD steer/drive, E exit.", 2.5);
                                    did_interact = true;
                                    break;
                                }
                            }
                        }
                        // 2. Chop tree
                        if !did_interact {
                            let (pcx, pcy) = World::chunk_of(self.player_x, self.player_y);
                            for dy in -1..=1 {
                                for dx in -1..=1 {
                                    if let Some(chunk) = self.world.chunk_mut(pcx + dx, pcy + dy) {
                                        for prop in &mut chunk.props {
                                            if prop.tree && prop.solid {
                                                let dist = ((prop.x - self.player_x).powi(2) + (prop.y - self.player_y).powi(2)).sqrt();
                                                if dist < 2.2 {
                                                    prop.solid = false;
                                                    prop.tree = false;
                                                    prop.tile = id::STUMP;
                                                    self.wood += 4;
                                                    self.score += 25;
                                                    self.set_message("+4 Wood chopped! Use F for Fence, C for Campfire.", 2.0);
                                                    did_interact = true;
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    if did_interact {
                                        break;
                                    }
                                }
                                if did_interact {
                                    break;
                                }
                            }
                        }
                        // 3. Fish at fishing spot or lake
                        if !did_interact {
                            if let Some(_) = self.world.lake_at(self.player_x, self.player_y) {
                                self.hunger = (self.hunger + 35.0).min(MAX_HUNGER);
                                self.hp = (self.hp + 15).min(MAX_HP);
                                self.set_message("Caught fresh fish! +35 Hunger, +15 Health.", 2.5);
                                did_interact = true;
                            }
                        }
                        if !did_interact {
                            self.set_message("Nothing nearby to interact with.", 1.2);
                        }
                    }
                    // F: Place wooden barricade / fence (costs 3 wood)
                    Key::Character(s) if s.eq_ignore_ascii_case("f") => {
                        if self.wood >= 3 {
                            self.wood -= 3;
                            let fx = (self.player_x + self.player_aim_x * 1.5).round();
                            let fy = (self.player_y + self.player_aim_y * 1.5).round();
                            self.structures.push(BaseStructure {
                                x: fx,
                                y: fy,
                                tile: id::FENCE,
                                hp: 120,
                                max_hp: 120,
                                is_campfire: false,
                            });
                            self.set_message("Wooden barricade built! Holds off zombie attacks.", 2.0);
                        } else {
                            self.set_message("Need 3 Wood to build a barricade. Chop trees with E.", 2.0);
                        }
                    }
                    // C: Place campfire (costs 5 wood, provides heat & light)
                    Key::Character(s) if s.eq_ignore_ascii_case("c") => {
                        if self.wood >= 5 {
                            self.wood -= 5;
                            let cx = (self.player_x + self.player_aim_x * 1.5).round();
                            let cy = (self.player_y + self.player_aim_y * 1.5).round();
                            self.structures.push(BaseStructure {
                                x: cx,
                                y: cy,
                                tile: id::CAMPFIRE,
                                hp: 80,
                                max_hp: 80,
                                is_campfire: true,
                            });
                            self.set_message("Campfire lit! Warms and heals you at night.", 2.5);
                        } else {
                            self.set_message("Need 5 Wood to build a campfire. Chop trees with E.", 2.0);
                        }
                    }
                    _ => {}
                }
            }
        }

        // 6. Horde spawning
        self.horde_spawn_timer -= dt;
        if self.horde_spawn_timer <= 0.0 {
            let night = self.is_night();
            let base_interval = if night { 4.0 } else { 9.0 };
            self.horde_spawn_timer = (base_interval / (1.0 + self.wave as f32 * 0.12)).max(1.5);

            let count = (1 + self.wave / 2 + if night { 3 } else { 0 }) as usize;
            for _ in 0..count {
                let angle = self.rng.next_f32() * std::f32::consts::TAU;
                let dist = 14.0 + self.rng.next_f32() * 8.0;
                let zx = self.player_x + angle.cos() * dist;
                let zy = self.player_y + angle.sin() * dist;

                let roll = self.rng.next_f32();
                let kind = if self.wave >= 3 && roll < 0.20 {
                    ZombieKind::Brute
                } else if roll < 0.45 {
                    ZombieKind::Runner
                } else {
                    ZombieKind::Walker
                };
                self.spawn_zombie_at(zx, zy, kind);
            }
        }

        // 7. Campfire healing
        for st in &self.structures {
            if st.is_campfire {
                let dist = ((st.x - self.player_x).powi(2) + (st.y - self.player_y).powi(2)).sqrt();
                if dist < 4.0 && self.hp < MAX_HP {
                    self.hp = (self.hp + (6.0 * dt) as i32).min(MAX_HP);
                }
            }
        }

        // 8. Update entities (bullets, zombies, loot, car physics)
        let player_x = self.player_x;
        let player_y = self.player_y;
        let night_speed_mult = if self.is_night() { 1.35 } else { 1.0 };
        let mut new_msg: Option<(String, f32)> = None;
        let mut damage_taken = 0.0f32;
        let driving_idx = self.driving_car_index;

        for (e_idx, e) in self.entities.iter_mut().enumerate() {
            if !e.alive {
                continue;
            }
            match e.kind {
                Kind::Bullet => {
                    e.x += e.vx * dt;
                    e.y += e.vy * dt;
                    e.ttl -= dt;
                    if e.ttl <= 0.0 {
                        e.alive = false;
                    }
                }
                Kind::Zombie => {
                    let dx = player_x - e.x;
                    let dy = player_y - e.y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    let zk = e.zombie_kind.unwrap_or(ZombieKind::Walker);

                    if dist < zk.aggro_range() * (if night_speed_mult > 1.0 { 1.5 } else { 1.0 }) {
                        e.aggro = true;
                    }

                    if e.aggro && dist > 0.4 {
                        let speed = zk.speed() * night_speed_mult;
                        let z_vx = (dx / dist) * speed;
                        let z_vy = (dy / dist) * speed;

                        e.x += z_vx * dt;
                        e.y += z_vy * dt;
                    }

                    // Attack barricades or player
                    if dist <= PLAYER_SIZE * 0.75 {
                        damage_taken += zk.damage() as f32 * dt * 2.0;
                    }
                }
                Kind::Car => {
                    // Car coasting when driver is out
                    if driving_idx != Some(e_idx) {
                        e.vx *= 0.90;
                        e.vy *= 0.90;
                        e.x += e.vx * dt;
                        e.y += e.vy * dt;
                    }
                }
                Kind::Loot | Kind::DroppedWood => {
                    let dist = ((e.x - player_x).powi(2) + (e.y - player_y).powi(2)).sqrt();
                    if dist < 1.4 {
                        e.alive = false;
                        match e.loot {
                            Some(LootKind::Food) => {
                                self.hunger = (self.hunger + 30.0).min(MAX_HUNGER);
                                self.hp = (self.hp + 20).min(MAX_HP);
                                new_msg = Some(("Picked up Food rations. +30 Hunger, +20 HP.".into(), 1.5));
                            }
                            Some(LootKind::Ammo) => {
                                self.ammo += 25;
                                new_msg = Some(("Found Ammo cache! +25 Rounds.".into(), 1.5));
                            }
                            Some(LootKind::Weapon) => {
                                self.weapon = if self.weapon == Weapon::Pistol {
                                    Weapon::Shotgun
                                } else {
                                    Weapon::Rifle
                                };
                                self.ammo += 15;
                                new_msg = Some((format!("Found {}! Press 1/2/3 to switch.", self.weapon.name()), 2.5));
                            }
                            Some(LootKind::Material) | None => {
                                self.wood += 3;
                                new_msg = Some(("Salvaged building materials (+3 Wood).".into(), 1.5));
                            }
                        }
                        self.score += 50;
                    }
                }
            }
        }

        if damage_taken > 0.0 {
            self.hp -= damage_taken as i32;
            if self.hp <= 0 {
                self.game_over = true;
                self.set_message("The horde overwhelmed you! Press R to restart.", 10.0);
            }
        }
        if let Some((msg, dur)) = new_msg {
            self.set_message(msg, dur);
        }

        // 9. Bullet vs Zombie collisions & Bullet vs Structures
        let mut new_loots = Vec::new();
        for b_idx in 0..self.entities.len() {
            if self.entities[b_idx].kind != Kind::Bullet || !self.entities[b_idx].alive {
                continue;
            }
            let bx = self.entities[b_idx].x;
            let by = self.entities[b_idx].y;
            let bdmg = self.entities[b_idx].damage;

            // Check vs zombies
            for z_idx in 0..self.entities.len() {
                if self.entities[z_idx].kind != Kind::Zombie || !self.entities[z_idx].alive {
                    continue;
                }
                let zx = self.entities[z_idx].x;
                let zy = self.entities[z_idx].y;
                let zsize = self.entities[z_idx].size;

                if rects_overlap(bx - 0.2, by - 0.2, 0.4, 0.4, zx - zsize * 0.5, zy - zsize * 0.5, zsize, zsize) {
                    self.entities[b_idx].alive = false;
                    self.entities[z_idx].hp -= bdmg;
                    self.entities[z_idx].aggro = true;

                    if self.entities[z_idx].hp <= 0 {
                        self.entities[z_idx].alive = false;
                        self.kills += 1;
                        self.score += 100;

                        // Zombie drops loot
                        if self.rng.next_f32() < 0.40 {
                            let roll = self.rng.next_f32();
                            let lk = if roll < 0.45 {
                                LootKind::Ammo
                            } else if roll < 0.75 {
                                LootKind::Food
                            } else {
                                LootKind::Material
                            };
                            new_loots.push(Entity::loot(zx, zy, lk));
                        }
                    }
                    break;
                }
            }
        }
        self.entities.extend(new_loots);

        // 10. Zombie vs Base Structures (Zombies attack barricades)
        for z in self.entities.iter_mut() {
            if z.kind != Kind::Zombie || !z.alive {
                continue;
            }
            for st in self.structures.iter_mut() {
                if st.hp <= 0 {
                    continue;
                }
                if rects_overlap(z.x - 0.5, z.y - 0.5, 1.0, 1.0, st.x - 0.5, st.y - 0.5, 1.0, 1.0) {
                    st.hp -= (z.damage as f32 * dt * 2.0) as i32;
                    z.x -= z.vx * dt * 1.5;
                    z.y -= z.vy * dt * 1.5;
                }
            }
        }
        self.structures.retain(|st| st.hp > 0);

        // 11. Car vs Zombie ramming damage
        for c_idx in 0..self.entities.len() {
            if self.entities[c_idx].kind != Kind::Car || !self.entities[c_idx].alive {
                continue;
            }
            let cx = self.entities[c_idx].x;
            let cy = self.entities[c_idx].y;
            let speed_sq = self.entities[c_idx].vx.powi(2) + self.entities[c_idx].vy.powi(2);
            if speed_sq < 16.0 {
                continue;
            }
            for z_idx in 0..self.entities.len() {
                if self.entities[z_idx].kind != Kind::Zombie || !self.entities[z_idx].alive {
                    continue;
                }
                let zx = self.entities[z_idx].x;
                let zy = self.entities[z_idx].y;
                let dist = ((cx - zx).powi(2) + (cy - zy).powi(2)).sqrt();
                if dist < 1.8 {
                    self.entities[z_idx].alive = false;
                    self.kills += 1;
                    self.score += 150;
                    self.entities[c_idx].integrity -= 4.0;
                }
            }
        }

        // Cleanup dead entities (cap buffer length to prevent memory leakage)
        self.entities.retain(|e| e.alive);
        if self.entities.len() > 300 {
            // Prune distant entities
            let px = self.player_x;
            let py = self.player_y;
            self.entities.retain(|e| {
                e.kind == Kind::Car || ((e.x - px).powi(2) + (e.y - py).powi(2)) < 3600.0
            });
        }
    }

    fn render(&mut self, ctx: &mut RunContext) {
        let (screen_w, screen_h) = ctx.renderer.size();
        let night = self.night_factor();

        // 1. Camera centered on player
        ctx.renderer.set_camera(Camera {
            x: self.player_x,
            y: self.player_y,
            view_height: 24.0,
        });

        // Clear color adjusted by day/night cycle
        let bg_r = 0.08 * (1.0 - night * 0.7);
        let bg_g = 0.12 * (1.0 - night * 0.7);
        let bg_b = 0.08 * (1.0 - night * 0.5);
        ctx.renderer.set_clear_color(wgpu::Color {
            r: bg_r as f64,
            g: bg_g as f64,
            b: bg_b as f64,
            a: 1.0,
        });

        let tex = self.atlas_texture;

        // 2. Draw ground chunks & props
        let (pcx, pcy) = World::chunk_of(self.player_x, self.player_y);
        for dy in -world::VIEW_RADIUS..=world::VIEW_RADIUS {
            for dx in -world::VIEW_RADIUS..=world::VIEW_RADIUS {
                let cx = pcx + dx;
                let cy = pcy + dy;
                if let Some(chunk) = self.world.chunk(cx, cy) {
                    let (ox, oy) = World::chunk_origin(cx, cy);
                    // Ground tile based on biome
                    let ground_tile = match chunk.biome {
                        Biome::Town => id::ROAD,
                        Biome::Forest => id::DIRT,
                        Biome::Lake => id::WATER,
                        Biome::Field => id::GRASS,
                    };
                    let g_uv = art::tile_uv(ground_tile);
                    let ground_tint = [1.0 - night * 0.45, 1.0 - night * 0.45, 1.0 - night * 0.35, 1.0];

                    // Draw chunk ground quad
                    ctx.renderer.add_sprite(
                        tex,
                        g_uv,
                        ox + CHUNK_SIZE * 0.5,
                        oy + CHUNK_SIZE * 0.5,
                        CHUNK_SIZE,
                        CHUNK_SIZE,
                        ground_tint,
                    );

                    // Lake surface
                    if let Some((lx, ly, lr)) = chunk.lake_center {
                        ctx.renderer.add_sprite(
                            tex,
                            art::tile_uv(id::WATER),
                            lx,
                            ly,
                            lr * 2.0,
                            lr * 2.0,
                            [0.4, 0.7, 0.9, 0.85],
                        );
                    }

                    // Props (walls, doors, trees, furniture, etc.)
                    for prop in &chunk.props {
                        let prop_uv = art::tile_uv(prop.tile);
                        let pw = if prop.tile == id::WALL || prop.tile == id::DOOR { 1.0 } else { 1.3 };
                        let ph = if prop.tile == id::WALL || prop.tile == id::DOOR { 1.0 } else { 1.3 };
                        let tint = if prop.solid {
                            [1.0 - night * 0.4, 1.0 - night * 0.4, 1.0 - night * 0.3, 1.0]
                        } else {
                            [1.0, 1.0, 1.0, 1.0]
                        };
                        ctx.renderer.add_sprite(tex, prop_uv, prop.x, prop.y, pw, ph, tint);
                    }
                }
            }
        }

        // 3. Draw crafted base structures (barricades & campfires)
        for st in &self.structures {
            let uv = art::tile_uv(st.tile);
            let tint = if st.is_campfire {
                [1.0, 0.8 + (self.frame % 10) as f32 * 0.02, 0.4, 1.0]
            } else {
                let hp_factor = st.hp as f32 / st.max_hp as f32;
                [hp_factor, hp_factor, hp_factor, 1.0]
            };
            ctx.renderer.add_sprite(tex, uv, st.x, st.y, 1.4, 1.4, tint);
        }

        // 4. Draw entities (loot, bullets, cars, zombies)
        for e in &self.entities {
            if !e.alive {
                continue;
            }
            let uv = art::tile_uv(e.tile());
            let (w, h) = (e.size, e.size);
            let tint = match e.kind {
                Kind::Zombie => {
                    let zk = e.zombie_kind.unwrap_or(ZombieKind::Walker);
                    match zk {
                        ZombieKind::Walker => [0.8, 0.9, 0.8, 1.0],
                        ZombieKind::Runner => [1.0, 0.6, 0.6, 1.0],
                        ZombieKind::Brute => [0.9, 0.3, 0.3, 1.0],
                    }
                }
                Kind::Bullet => [1.0, 0.9, 0.3, 1.0],
                Kind::Loot | Kind::DroppedWood => [1.0, 1.0, 1.0, 1.0],
                Kind::Car => [0.9, 0.9, 0.9, 1.0],
            };
            ctx.renderer.add_sprite(tex, uv, e.x, e.y, w, h, tint);
        }

        // 5. Draw Player (if not inside car)
        if self.driving_car_index.is_none() {
            let p_uv = art::tile_uv(id::PLAYER);
            ctx.renderer.add_sprite(
                tex,
                p_uv,
                self.player_x,
                self.player_y,
                PLAYER_SIZE,
                PLAYER_SIZE,
                [1.0, 1.0, 1.0, 1.0],
            );

            // Aim cursor in world space
            ctx.renderer.add_rect(
                self.player_x + self.player_aim_x * 1.8,
                self.player_y + self.player_aim_y * 1.8,
                0.22,
                0.22,
                [1.0, 0.2, 0.2, 0.8],
            );
        }

        // 6. Night darkness ambient overlay
        if night > 0.01 {
            ctx.renderer.add_rect_screen(
                0.0,
                0.0,
                screen_w as f32,
                screen_h as f32,
                [0.02, 0.02, 0.06, night * 0.62],
            );
        }

        // 7. HUD Bars & Status in Screen Space
        // Top-left status background panel
        ctx.renderer.add_rect_screen(12.0, 12.0, 240.0, 118.0, [0.08, 0.08, 0.10, 0.85]);

        // Health bar
        let hp_pct = (self.hp as f32 / MAX_HP as f32).clamp(0.0, 1.0);
        ctx.renderer.draw_text(&format!("HP: {}/{}", self.hp.max(0), MAX_HP), 18.0, 18.0, 1.8, [1.0, 1.0, 1.0, 1.0]);
        ctx.renderer.add_rect_screen(18.0, 34.0, 220.0, 10.0, [0.3, 0.1, 0.1, 1.0]);
        ctx.renderer.add_rect_screen(18.0, 34.0, 220.0 * hp_pct, 10.0, [0.9, 0.2, 0.2, 1.0]);

        // Stamina bar
        let sta_pct = (self.stamina / MAX_STAMINA).clamp(0.0, 1.0);
        ctx.renderer.draw_text(&format!("STA: {:.0}%", self.stamina), 18.0, 48.0, 1.8, [1.0, 1.0, 1.0, 1.0]);
        ctx.renderer.add_rect_screen(18.0, 64.0, 220.0, 8.0, [0.1, 0.2, 0.3, 1.0]);
        ctx.renderer.add_rect_screen(18.0, 64.0, 220.0 * sta_pct, 8.0, [0.2, 0.6, 0.9, 1.0]);

        // Hunger bar
        let hng_pct = (self.hunger / MAX_HUNGER).clamp(0.0, 1.0);
        ctx.renderer.draw_text(&format!("FOOD: {:.0}%", self.hunger), 18.0, 76.0, 1.8, [1.0, 1.0, 1.0, 1.0]);
        ctx.renderer.add_rect_screen(18.0, 92.0, 220.0, 8.0, [0.25, 0.2, 0.1, 1.0]);
        ctx.renderer.add_rect_screen(18.0, 92.0, 220.0 * hng_pct, 8.0, [0.9, 0.7, 0.2, 1.0]);

        ctx.renderer.draw_text(
            &format!("Wood: {}   Ammo: {}", self.wood, self.ammo),
            18.0,
            106.0,
            1.8,
            [0.8, 0.9, 0.8, 1.0],
        );

        // Top-right Info panel (Day/Night, Wave, Kills, RAM RSS)
        let right_x = screen_w as f32 - 250.0;
        ctx.renderer.add_rect_screen(right_x, 12.0, 238.0, 118.0, [0.08, 0.08, 0.10, 0.85]);
        let time_str = if self.is_night() { "NIGHT (SWARM!)" } else { "DAY" };
        let time_color = if self.is_night() { [1.0, 0.3, 0.3, 1.0] } else { [1.0, 0.9, 0.4, 1.0] };
        ctx.renderer.draw_text(&format!("Day {}: {}", self.days_survived, time_str), right_x + 8.0, 18.0, 2.0, time_color);
        ctx.renderer.draw_text(&format!("Wave: {}  Score: {}", self.wave, self.score), right_x + 8.0, 42.0, 1.8, [1.0, 1.0, 1.0, 1.0]);
        ctx.renderer.draw_text(&format!("Zombies Killed: {}", self.kills), right_x + 8.0, 64.0, 1.8, [0.9, 0.7, 0.7, 1.0]);
        ctx.renderer.draw_text(&format!("Equipped: {}", self.weapon.name()), right_x + 8.0, 86.0, 1.8, [0.4, 0.9, 0.5, 1.0]);
        ctx.renderer.draw_text(&format!("RSS RAM: {:.1} MB / 100 MB", ctx.rss_mb), right_x + 8.0, 106.0, 1.8, [0.6, 0.8, 1.0, 1.0]);

        // Bottom banner: instructions / active event message
        if !self.message.is_empty() && self.message_timer > 0.0 {
            let msg_w = ctx.renderer.text_width(&self.message, 2.0) + 24.0;
            let msg_x = (screen_w as f32 - msg_w) * 0.5;
            let msg_y = screen_h as f32 - 50.0;
            ctx.renderer.add_rect_screen(msg_x, msg_y, msg_w, 32.0, [0.05, 0.05, 0.07, 0.92]);
            ctx.renderer.draw_text(&self.message, msg_x + 12.0, msg_y + 8.0, 2.0, [1.0, 0.95, 0.5, 1.0]);
        }

        // Game Over overlay
        if self.game_over {
            ctx.renderer.add_rect_screen(0.0, 0.0, screen_w as f32, screen_h as f32, [0.4, 0.05, 0.05, 0.75]);
            let go_txt = "YOU DIED";
            let go_w = ctx.renderer.text_width(go_txt, 5.0);
            ctx.renderer.draw_text(go_txt, (screen_w as f32 - go_w) * 0.5, screen_h as f32 * 0.35, 5.0, [1.0, 0.2, 0.2, 1.0]);

            let stat_txt = format!("Survived {} Days | Wave {} | Score {} | Kills {}", self.days_survived, self.wave, self.score, self.kills);
            let stat_w = ctx.renderer.text_width(&stat_txt, 2.4);
            ctx.renderer.draw_text(&stat_txt, (screen_w as f32 - stat_w) * 0.5, screen_h as f32 * 0.48, 2.4, [1.0, 1.0, 1.0, 1.0]);

            let r_txt = "PRESS 'R' TO RESTART";
            let r_w = ctx.renderer.text_width(r_txt, 2.6);
            ctx.renderer.draw_text(r_txt, (screen_w as f32 - r_w) * 0.5, screen_h as f32 * 0.58, 2.6, [1.0, 0.85, 0.2, 1.0]);
        }
    }
}

fn main() {
    env_logger::init();
    AppBuilder::new()
        .title("Sombie Survivor (custom-game)")
        .size(960, 640)
        .vsync(true)
        .memory_budget_mb(100)
        .build(Game::default())
        .run();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_initializes_with_correct_budget_and_stats() {
        let g = Game::default();
        assert_eq!(g.hp, MAX_HP);
        assert_eq!(g.wood, 12);
        assert_eq!(g.weapon, Weapon::Pistol);
        assert!(!g.game_over);
    }

    #[test]
    fn day_night_cycle_transitions() {
        let mut g = Game::default();
        g.day_timer = 20.0;
        assert!(!g.is_night());
        assert_eq!(g.night_factor(), 0.0);

        g.day_timer = 65.0;
        assert!(g.is_night());
        assert_eq!(g.night_factor(), 1.0);
    }

    #[test]
    fn weapon_switching_and_stats() {
        let pistol = Weapon::Pistol;
        let shotgun = Weapon::Shotgun;
        let rifle = Weapon::Rifle;

        assert!(shotgun.pellets() > 1);
        assert_eq!(pistol.pellets(), 1);
        assert!(rifle.damage() > pistol.damage());
    }

    #[test]
    fn solid_prop_collision_detection() {
        let mut g = Game::default();
        g.world.stream_around(0.0, 0.0);
        g.structures.push(BaseStructure {
            x: 5.0,
            y: 5.0,
            tile: id::FENCE,
            hp: 100,
            max_hp: 100,
            is_campfire: false,
        });
        assert!(g.check_solid_prop_collision(5.0, 5.0, 0.4));
        assert!(!g.check_solid_prop_collision(50.0, 50.0, 0.4));
    }
}
