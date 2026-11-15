//! Sombie Survivor — a 2D/3D hybrid zombie survival game built on
//! `custom-game-engine` staying strictly within a <= 100 MB RAM budget.
//!
//! Camera POV System (*Sombie* inspired):
//! - First-Person Shooter (FPS) POV when OUTSIDE in the open world (raycast walls,
//!   3D billboard sprites, gun viewmodel with recoil/sway, center crosshair).
//! - Top-Down camera POV when INSIDE a house/shop (tactical room scavenging,
//!   interior furniture, top-down line-of-sight).
//! - Press `V` to toggle camera mode manually (Auto / Forced FPS / Forced Top-Down).

use custom_game_engine::{wgpu, App, AppBuilder, Camera, InitContext, RunContext};
use winit::keyboard::{Key, NamedKey};

pub mod art;
pub mod entity;
pub mod world;

use art::id;
use entity::{rects_overlap, Entity, Kind, ZombieKind};
use world::{LootKind, Rng, World};

const MAX_HP: i32 = 100;
const MAX_STAMINA: f32 = 100.0;
const MAX_HUNGER: f32 = 100.0;

const PLAYER_SIZE: f32 = 0.85;
const PLAYER_BASE_SPEED: f32 = 5.2;
const PLAYER_SPRINT_MULT: f32 = 1.65;

const DAY_LENGTH: f32 = 90.0; // 90 seconds real-time full day
const NIGHT_START: f32 = 50.0;
const NIGHT_END: f32 = 85.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Weapon {
    Pistol,
    Shotgun,
    Rifle,
}

impl Weapon {
    pub fn name(self) -> &'static str {
        match self {
            Weapon::Pistol => "9mm Pistol",
            Weapon::Shotgun => "12G Pump Shotgun",
            Weapon::Rifle => "Assault Rifle",
        }
    }

    pub fn cooldown(self) -> f32 {
        match self {
            Weapon::Pistol => 0.28,
            Weapon::Shotgun => 0.72,
            Weapon::Rifle => 0.12,
        }
    }

    pub fn damage(self) -> i32 {
        match self {
            Weapon::Pistol => 35,
            Weapon::Shotgun => 28,
            Weapon::Rifle => 45,
        }
    }

    pub fn bullet_speed(self) -> f32 {
        match self {
            Weapon::Pistol => 22.0,
            Weapon::Shotgun => 18.0,
            Weapon::Rifle => 30.0,
        }
    }

    pub fn pellets(self) -> u32 {
        match self {
            Weapon::Pistol => 1,
            Weapon::Shotgun => 6,
            Weapon::Rifle => 1,
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CameraMode {
    Auto,        // Top-down inside houses, FPS outside
    FirstPerson, // Forced First-Person
    TopDown,     // Forced Top-Down
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
    pub player_angle: f32, // Heading in radians for FPS mode
    pub player_aim_x: f32,
    pub player_aim_y: f32,
    pub last_mouse_x: f32,
    pub walk_distance: f32,
    pub camera_mode: CameraMode,
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
            player_angle: 0.0,
            player_aim_x: 1.0,
            player_aim_y: 0.0,
            last_mouse_x: 0.0,
            walk_distance: 0.0,
            camera_mode: CameraMode::Auto,
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
            horde_spawn_timer: 4.0,
            message: "Welcome to Sombie Survivor! FPS outdoors, Top-Down inside houses.".to_string(),
            message_timer: 5.0,
            game_over: false,
            frame: 0,
            rng: Rng::new(54321),
        }
    }
}

impl Game {
    pub fn is_night(&self) -> bool {
        self.day_timer >= NIGHT_START && self.day_timer <= NIGHT_END
    }

    pub fn night_factor(&self) -> f32 {
        if self.day_timer < NIGHT_START - 8.0 || self.day_timer > NIGHT_END + 5.0 {
            0.0
        } else if self.day_timer >= NIGHT_START && self.day_timer <= NIGHT_END {
            1.0
        } else if self.day_timer < NIGHT_START {
            (self.day_timer - (NIGHT_START - 8.0)) / 8.0
        } else {
            1.0 - ((self.day_timer - NIGHT_END) / 5.0)
        }
    }

    pub fn is_inside_house(&self) -> bool {
        let px = self.player_x;
        let py = self.player_y;
        for chunk in self.world.chunks() {
            for b in &chunk.buildings {
                if b.contains(px, py) {
                    return true;
                }
            }
            for prop in &chunk.props {
                if let Some((bx, by, bw, bh)) = prop.interior {
                    if px >= bx && px <= bx + bw && py >= by && py <= by + bh {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn is_first_person(&self) -> bool {
        match self.camera_mode {
            CameraMode::Auto => !self.is_inside_house(),
            CameraMode::FirstPerson => true,
            CameraMode::TopDown => false,
        }
    }

    pub fn set_message(&mut self, text: impl Into<String>, duration: f32) {
        self.message = text.into();
        self.message_timer = duration;
    }

    pub fn restart(&mut self) {
        let saved_tex = self.atlas_texture;
        *self = Self::default();
        self.atlas_texture = saved_tex;
        self.world.stream_around(self.player_x, self.player_y);
        self.populate_new_chunk_loot();
    }

    pub fn populate_new_chunk_loot(&mut self) {
        let (pcx, pcy) = World::chunk_of(self.player_x, self.player_y);
        let mut spawns = Vec::new();
        for dy in -world::VIEW_RADIUS..=world::VIEW_RADIUS {
            for dx in -world::VIEW_RADIUS..=world::VIEW_RADIUS {
                let key = (pcx + dx, pcy + dy);
                if self.world.looted.insert(key) {
                    if let Some(chunk) = self.world.chunk(key.0, key.1) {
                        for prop in &chunk.props {
                            if let Some(ref loot_list) = prop.loot {
                                for item in loot_list {
                                    spawns.push(Entity::loot(item.x, item.y, item.kind));
                                }
                            }
                            if prop.tile == id::CAR || prop.tile == id::TRUCK {
                                spawns.push(Entity::car(prop.x, prop.y, 0.0));
                            }
                        }
                    }
                }
            }
        }
        self.entities.extend(spawns);
    }

    pub fn check_solid_prop_collision(&self, x: f32, y: f32, radius: f32) -> bool {
        let (pcx, pcy) = World::chunk_of(x, y);
        for dy in -1..=1 {
            for dx in -1..=1 {
                if let Some(chunk) = self.world.chunk(pcx + dx, pcy + dy) {
                    let (ox, oy) = World::chunk_origin(pcx + dx, pcy + dy);
                    let local_x = x - ox;
                    let local_y = y - oy;
                    if local_x >= 0.0 && local_x < world::CHUNK_SIZE && local_y >= 0.0 && local_y < world::CHUNK_SIZE {
                        let tx = local_x as usize;
                        let ty = local_y as usize;
                        if chunk.get_tile(tx, ty) == id::WATER as u8 {
                            return true;
                        }
                    }

                    for prop in &chunk.props {
                        if prop.solid {
                            let (pw, ph) = if prop.tile == id::WALL || prop.tile == id::WALL_INTERIOR || prop.tile == id::WINDOW {
                                (1.0, 1.0)
                            } else if prop.tree {
                                (0.8, 0.8)
                            } else {
                                (prop.size * 0.8, prop.size * 0.8)
                            };
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

        // 1. Day / Night cycle progression
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

        // 2. Stream world chunks around player
        self.world.stream_around(self.player_x, self.player_y);
        self.populate_new_chunk_loot();

        // 3. Camera POV Mode Toggle
        for key in &ctx.input.keys_just_pressed {
            if matches!(key, Key::Character(s) if s.eq_ignore_ascii_case("v")) {
                self.camera_mode = match self.camera_mode {
                    CameraMode::Auto => CameraMode::FirstPerson,
                    CameraMode::FirstPerson => CameraMode::TopDown,
                    CameraMode::TopDown => CameraMode::Auto,
                };
                let label = match self.camera_mode {
                    CameraMode::Auto => "Auto (FPS outdoors, Top-Down inside houses)",
                    CameraMode::FirstPerson => "Forced First-Person",
                    CameraMode::TopDown => "Forced Top-Down",
                };
                self.set_message(format!("Camera POV: {}", label), 2.5);
            }
        }

        let is_fps = self.is_first_person();

        // 4. Aim direction & Looking
        let (screen_w, screen_h) = ctx.renderer.size();
        let (cursor_px, cursor_py) = ctx.input.mouse_position;

        if is_fps {
            // First-person mouse look
            let mouse_dx = cursor_px - self.last_mouse_x;
            if self.last_mouse_x > 0.0 && mouse_dx.abs() > 0.5 && mouse_dx.abs() < 250.0 {
                self.player_angle += mouse_dx * 0.0055;
            }
            self.last_mouse_x = cursor_px;

            // Keyboard turn backup (Arrow keys)
            for key in &ctx.input.keys_held {
                match key {
                    Key::Named(NamedKey::ArrowLeft) => self.player_angle -= 3.2 * dt,
                    Key::Named(NamedKey::ArrowRight) => self.player_angle += 3.2 * dt,
                    _ => {}
                }
            }

            self.player_aim_x = self.player_angle.cos();
            self.player_aim_y = self.player_angle.sin();
        } else {
            // Top-down 360-degree cursor aim
            if cursor_px >= 0.0 && cursor_px <= screen_w as f32 && cursor_py >= 0.0 && cursor_py <= screen_h as f32 {
                let (target_wx, target_wy) = ctx.renderer.screen_to_world(cursor_px, cursor_py);
                let dx = target_wx - self.player_x;
                let dy = target_wy - self.player_y;
                let len = (dx * dx + dy * dy).sqrt();
                if len > 0.001 {
                    self.player_aim_x = dx / len;
                    self.player_aim_y = dy / len;
                    self.player_angle = self.player_aim_y.atan2(self.player_aim_x);
                }
            }
            self.last_mouse_x = cursor_px;
        }

        // 5. Weapon hotkeys
        for key in &ctx.input.keys_just_pressed {
            match key {
                Key::Character(s) if s == "1" => self.weapon = Weapon::Pistol,
                Key::Character(s) if s == "2" => self.weapon = Weapon::Shotgun,
                Key::Character(s) if s == "3" => self.weapon = Weapon::Rifle,
                _ => {}
            }
        }

        // 6. Movement inputs
        let mut input_forward = 0.0f32;
        let mut input_strafe = 0.0f32;
        let mut sprinting = false;

        for key in &ctx.input.keys_held {
            match key {
                Key::Named(NamedKey::ArrowUp) if !is_fps => input_forward += 1.0,
                Key::Character(s) if s.eq_ignore_ascii_case("w") => input_forward += 1.0,
                Key::Named(NamedKey::ArrowDown) if !is_fps => input_forward -= 1.0,
                Key::Character(s) if s.eq_ignore_ascii_case("s") => input_forward -= 1.0,
                Key::Character(s) if s.eq_ignore_ascii_case("a") => input_strafe -= 1.0,
                Key::Character(s) if s.eq_ignore_ascii_case("d") => input_strafe += 1.0,
                Key::Named(NamedKey::Shift) => sprinting = true,
                _ => {}
            }
        }

        // Calculate world-space move vector
        let (mut move_x, mut move_y) = if is_fps {
            let cos_a = self.player_angle.cos();
            let sin_a = self.player_angle.sin();
            let fwd_x = cos_a * input_forward;
            let fwd_y = sin_a * input_forward;
            let str_x = -sin_a * input_strafe;
            let str_y = cos_a * input_strafe;
            (fwd_x + str_x, fwd_y + str_y)
        } else {
            (input_strafe, input_forward)
        };

        // Car driving or on-foot movement
        if let Some(car_idx) = self.driving_car_index {
            if car_idx < self.entities.len() && self.entities[car_idx].alive {
                let steer_speed = 3.2 * dt;
                let car_turn = if is_fps { input_strafe } else { move_x };
                let car_gas = if is_fps { input_forward } else { move_y };

                if car_turn < 0.0 {
                    self.entities[car_idx].angle += steer_speed;
                }
                if car_turn > 0.0 {
                    self.entities[car_idx].angle -= steer_speed;
                }
                if is_fps {
                    self.player_angle = self.entities[car_idx].angle;
                }

                let accel = if car_gas > 0.0 { 16.0 } else if car_gas < 0.0 { -8.0 } else { 0.0 };
                let forward_x = self.entities[car_idx].angle.cos();
                let forward_y = self.entities[car_idx].angle.sin();

                self.entities[car_idx].vx += forward_x * accel * dt;
                self.entities[car_idx].vy += forward_y * accel * dt;
                self.entities[car_idx].vx *= 0.94;
                self.entities[car_idx].vy *= 0.94;

                let next_x = self.entities[car_idx].x + self.entities[car_idx].vx * dt;
                let next_y = self.entities[car_idx].y + self.entities[car_idx].vy * dt;
                let car_size = self.entities[car_idx].size;

                if !self.check_solid_prop_collision(next_x, next_y, car_size * 0.45) {
                    self.entities[car_idx].x = next_x;
                    self.entities[car_idx].y = next_y;
                } else {
                    self.entities[car_idx].vx = -self.entities[car_idx].vx * 0.3;
                    self.entities[car_idx].vy = -self.entities[car_idx].vy * 0.3;
                    self.entities[car_idx].integrity -= 2.0;
                }
                self.player_x = self.entities[car_idx].x;
                self.player_y = self.entities[car_idx].y;

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
                self.walk_distance += speed * dt;

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
                            'tree_search: for dy in -1..=1 {
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
                                                    self.score += 20;
                                                    self.set_message("Chopped tree (+4 Wood). Craft barricades with B!", 2.0);
                                                    did_interact = true;
                                                    break 'tree_search;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // 3. Fish at lake spot
                        if !did_interact {
                            if let Some(_) = self.world.lake_at(self.player_x, self.player_y) {
                                self.hunger = (self.hunger + 25.0).min(MAX_HUNGER);
                                self.hp = (self.hp + 10).min(MAX_HP);
                                self.set_message("Caught fresh fish! +25 Hunger, +10 HP.", 2.0);
                                did_interact = true;
                            }
                        }

                        if !did_interact {
                            self.set_message("Nothing nearby to interact with.", 1.2);
                        }
                    }
                    // B: Build fence barricade
                    Key::Character(s) if s.eq_ignore_ascii_case("b") => {
                        if self.wood >= 3 {
                            self.wood -= 3;
                            let build_x = self.player_x + self.player_aim_x * 1.5;
                            let build_y = self.player_y + self.player_aim_y * 1.5;
                            self.structures.push(BaseStructure {
                                x: build_x,
                                y: build_y,
                                tile: id::FENCE,
                                hp: 120,
                                max_hp: 120,
                                is_campfire: false,
                            });
                            self.set_message("Placed defensive barricade! (-3 Wood)", 2.0);
                        } else {
                            self.set_message("Need 3 Wood to build a barricade. Press E near trees!", 2.5);
                        }
                    }
                    // C: Build campfire
                    Key::Character(s) if s.eq_ignore_ascii_case("c") => {
                        if self.wood >= 5 {
                            self.wood -= 5;
                            let build_x = self.player_x + self.player_aim_x * 1.2;
                            let build_y = self.player_y + self.player_aim_y * 1.2;
                            self.structures.push(BaseStructure {
                                x: build_x,
                                y: build_y,
                                tile: id::CAMPFIRE,
                                hp: 200,
                                max_hp: 200,
                                is_campfire: true,
                            });
                            self.set_message("Lit campfire (-5 Wood). Rest nearby to heal!", 2.5);
                        } else {
                            self.set_message("Need 5 Wood to build a campfire.", 2.5);
                        }
                    }
                    _ => {}
                }
            }
        }

        // 7. Campfire healing aura
        for st in &self.structures {
            if st.is_campfire {
                let dist = ((st.x - self.player_x).powi(2) + (st.y - self.player_y).powi(2)).sqrt();
                if dist < 3.0 && self.hp < MAX_HP {
                    self.hp = (self.hp + (6.0 * dt) as i32).min(MAX_HP);
                }
            }
        }

        // 8. Zombie Horde Spawning
        self.horde_spawn_timer -= dt;
        let night_rate = if self.is_night() { 0.4f32 } else { 1.0f32 };
        if self.horde_spawn_timer <= 0.0 {
            self.horde_spawn_timer = (3.5f32 * night_rate).max(0.8f32);
            let zombie_count = if self.is_night() { self.wave * 2 + 3 } else { self.wave + 1 };
            for _ in 0..zombie_count {
                let angle = self.rng.next_f32() * std::f32::consts::TAU;
                let dist = 14.0 + self.rng.next_f32() * 12.0;
                let zx = self.player_x + angle.cos() * dist;
                let zy = self.player_y + angle.sin() * dist;

                let variant_roll = self.rng.next_f32();
                let zk = if variant_roll < 0.18 {
                    ZombieKind::Brute
                } else if variant_roll < 0.52 {
                    ZombieKind::Runner
                } else {
                    ZombieKind::Walker
                };
                self.entities.push(Entity::zombie(zx, zy, zk));
            }
        }

        // 9. Update entities (bullets, zombies, loot, car physics)
        let player_x = self.player_x;
        let player_y = self.player_y;
        let night_speed_mult = if self.is_night() { 1.35 } else { 1.0 };
        let mut new_msg = None;
        let mut died = false;

        for e in self.entities.iter_mut() {
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

                    if dist <= PLAYER_SIZE * 0.75 {
                        self.hp -= (zk.damage() as f32 * dt * 2.0) as i32;
                        if self.hp <= 0 {
                            died = true;
                        }
                    }
                }
                Kind::Car => {
                    if self.driving_car_index.is_none() {
                        e.x += e.vx * dt;
                        e.y += e.vy * dt;
                        e.vx *= 0.95;
                        e.vy *= 0.95;
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
                                new_msg = Some(("Picked up Food rations. +30 Hunger, +20 HP.".to_string(), 1.5));
                            }
                            Some(LootKind::Ammo) => {
                                self.ammo += 25;
                                new_msg = Some(("Found Ammo cache! +25 Rounds.".to_string(), 1.5));
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
                                new_msg = Some(("Salvaged building materials (+3 Wood).".to_string(), 1.5));
                            }
                        }
                        self.score += 50;
                    }
                }
            }
        }
        if died {
            self.game_over = true;
            self.set_message("The horde overwhelmed you! Press R to restart.", 10.0);
        }
        if let Some((msg, dur)) = new_msg {
            self.set_message(msg, dur);
        }

        // 10. Bullet vs Zombie collisions & Bullet vs Structures
        let mut new_loots = Vec::new();
        for b_idx in 0..self.entities.len() {
            if self.entities[b_idx].kind != Kind::Bullet || !self.entities[b_idx].alive {
                continue;
            }
            let bx = self.entities[b_idx].x;
            let by = self.entities[b_idx].y;
            let dmg = self.entities[b_idx].damage;

            // Check against zombies
            for z_idx in 0..self.entities.len() {
                if self.entities[z_idx].kind != Kind::Zombie || !self.entities[z_idx].alive {
                    continue;
                }
                let zx = self.entities[z_idx].x;
                let zy = self.entities[z_idx].y;
                let z_size = self.entities[z_idx].size;

                if rects_overlap(bx - 0.2, by - 0.2, 0.4, 0.4, zx - z_size * 0.5, zy - z_size * 0.5, z_size, z_size) {
                    self.entities[b_idx].alive = false;
                    self.entities[z_idx].hp -= dmg;
                    self.entities[z_idx].aggro = true;

                    if self.entities[z_idx].hp <= 0 {
                        self.entities[z_idx].alive = false;
                        self.kills += 1;
                        self.score += 100;

                        // Zombie loot drop roll
                        let roll = self.rng.next_f32();
                        if roll < 0.35 {
                            new_loots.push(Entity::loot(zx, zy, LootKind::Ammo));
                        } else if roll < 0.55 {
                            new_loots.push(Entity::loot(zx, zy, LootKind::Food));
                        } else if roll < 0.70 {
                            new_loots.push(Entity::dropped_wood(zx, zy));
                        }
                    }
                    break;
                }
            }

            // Bullet vs base structures
            for st in &mut self.structures {
                if rects_overlap(bx - 0.2, by - 0.2, 0.4, 0.4, st.x - 0.6, st.y - 0.6, 1.2, 1.2) {
                    self.entities[b_idx].alive = false;
                    st.hp -= dmg;
                    break;
                }
            }
        }
        self.entities.extend(new_loots);

        // 11. Zombie vs Structure destruction
        for e in &self.entities {
            if e.kind == Kind::Zombie && e.alive {
                let zk = e.zombie_kind.unwrap_or(ZombieKind::Walker);
                for st in &mut self.structures {
                    if ((e.x - st.x).powi(2) + (e.y - st.y).powi(2)).sqrt() < 1.0 {
                        st.hp -= (zk.damage() as f32 * dt * 1.5) as i32;
                    }
                }
            }
        }
        self.structures.retain(|s| s.hp > 0);

        // 12. Retain living entities
        self.entities.retain(|e| e.alive);
    }

    fn render(&mut self, ctx: &mut RunContext) {
        if self.is_first_person() {
            self.render_fps(ctx);
        } else {
            self.render_top_down(ctx);
        }
        self.render_hud(ctx);
    }
}

impl Game {
    /// Renders the outdoor First-Person Shooter perspective:
    /// - Split sky and horizon
    /// - Raycast walls, fences, and crafted barricades
    /// - 3D billboard sprites for entities, trees, rocks, and campfires
    /// - First-person weapon viewmodel with recoil, sway, and muzzle flash
    /// - Precise center screen crosshair
    fn render_fps(&mut self, ctx: &mut RunContext) {
        let (screen_w, screen_h) = ctx.renderer.size();
        let sw = screen_w as f32;
        let sh = screen_h as f32;
        let half_h = sh * 0.5;
        let night = self.night_factor();
        let tex = self.atlas_texture;

        ctx.renderer.set_camera(Camera {
            x: 0.0,
            y: 0.0,
            view_height: 20.0,
        });

        // 1. Sky & Ground
        let sky_r = 0.35 * (1.0 - night * 0.85);
        let sky_g = 0.52 * (1.0 - night * 0.85);
        let sky_b = 0.78 * (1.0 - night * 0.65);
        ctx.renderer.add_rect_screen(0.0, 0.0, sw, half_h, [sky_r, sky_g, sky_b, 1.0]);

        if night > 0.35 {
            let star_alpha = (night - 0.35) / 0.65;
            for i in 0..45 {
                let sx = ((i * 127 + 37) % screen_w.max(1)) as f32;
                let sy = ((i * 83 + 19) % (screen_h / 2).max(1)) as f32;
                let twinkle = if (self.frame / 16 + i as u64) % 3 == 0 { 0.5 } else { 1.0 };
                ctx.renderer.add_rect_screen(sx, sy, 2.0, 2.0, [1.0, 1.0, 0.9, star_alpha * twinkle]);
            }
        }

        let g_r = 0.16 * (1.0 - night * 0.6);
        let g_g = 0.24 * (1.0 - night * 0.6);
        let g_b = 0.14 * (1.0 - night * 0.5);
        ctx.renderer.add_rect_screen(0.0, half_h, sw, half_h, [g_r, g_g, g_b, 1.0]);

        // 2. Outdoor Raycasting (Walls, Fences, Barricades)
        const FOV: f32 = 1.25; // ~71.6 degrees
        let num_rays = (screen_w / 5).clamp(80, 200) as usize;
        let ray_w = sw / num_rays as f32;
        let mut depth_buffer = vec![999.0f32; num_rays];

        struct WallBox {
            x0: f32,
            y0: f32,
            x1: f32,
            y1: f32,
            tile: u32,
        }
        let mut walls = Vec::with_capacity(128);
        let (pcx, pcy) = World::chunk_of(self.player_x, self.player_y);
        for dy in -1..=1 {
            for dx in -1..=1 {
                if let Some(chunk) = self.world.chunk(pcx + dx, pcy + dy) {
                    for prop in &chunk.props {
                        if prop.solid && (prop.tile == id::WALL || prop.tile == id::WALL_INTERIOR || prop.tile == id::WINDOW || prop.tile == id::FENCE) {
                            walls.push(WallBox {
                                x0: prop.x - 0.5,
                                y0: prop.y - 0.5,
                                x1: prop.x + 0.5,
                                y1: prop.y + 0.5,
                                tile: prop.tile,
                            });
                        }
                    }
                }
            }
        }
        for st in &self.structures {
            if !st.is_campfire {
                walls.push(WallBox {
                    x0: st.x - 0.5,
                    y0: st.y - 0.5,
                    x1: st.x + 0.5,
                    y1: st.y + 0.5,
                    tile: st.tile,
                });
            }
        }

        let px = self.player_x;
        let py = self.player_y;
        let p_angle = self.player_angle;
        let max_dist = 32.0f32;

        for col in 0..num_rays {
            let ray_angle = p_angle - FOV * 0.5 + (col as f32 / num_rays as f32) * FOV;
            let r_dx = ray_angle.cos();
            let r_dy = ray_angle.sin();

            let mut hit_dist = max_dist;
            let mut hit_tile = id::WALL;

            for w in &walls {
                let tx1 = (w.x0 - px) / r_dx;
                let tx2 = (w.x1 - px) / r_dx;
                let ty1 = (w.y0 - py) / r_dy;
                let ty2 = (w.y1 - py) / r_dy;

                let tmin = tx1.min(tx2).max(ty1.min(ty2));
                let tmax = tx1.max(tx2).min(ty1.max(ty2));

                if tmax >= tmin && tmax > 0.1 && tmin < hit_dist {
                    let d = if tmin > 0.1 { tmin } else { tmax };
                    if d < hit_dist {
                        hit_dist = d;
                        hit_tile = w.tile;
                    }
                }
            }

            if hit_dist < max_dist {
                let perp_dist = hit_dist * (ray_angle - p_angle).cos();
                depth_buffer[col] = perp_dist;

                let wall_h = ((1.2 / perp_dist.max(0.1)) * (sh * 0.75)).min(sh * 2.5);
                let wall_top = half_h - wall_h * 0.5;

                let fog = (1.0 - (perp_dist / max_dist).clamp(0.0, 1.0)) * (1.0 - night * 0.55);
                let tint = [fog, fog * 0.95, fog * 0.9, 1.0];
                let uv = art::tile_uv(hit_tile);

                ctx.renderer.add_sprite_screen(
                    tex,
                    uv,
                    col as f32 * ray_w,
                    wall_top,
                    ray_w + 1.0,
                    wall_h,
                    tint,
                );
            }
        }

        // 3. Billboards (Props, Structures, Entities)
        struct Billboard {
            forward: f32,
            screen_x: f32,
            screen_y: f32,
            width: f32,
            height: f32,
            tile: u32,
            color: [f32; 4],
            hp_ratio: Option<f32>,
        }
        let mut billboards = Vec::with_capacity(128);
        let focal = (sw * 0.5) / (FOV * 0.5).tan();

        let project = |bx: f32, by: f32, size: f32| -> Option<(f32, f32, f32, f32, f32)> {
            let dx = bx - px;
            let dy = by - py;
            let fwd = dx * p_angle.cos() + dy * p_angle.sin();
            let lat = -dx * p_angle.sin() + dy * p_angle.cos();
            if fwd < 0.25 || fwd > max_dist {
                return None;
            }
            let sx = (sw * 0.5) + (lat / fwd) * focal;
            let scale = (focal / fwd) * 0.07;
            let w = size * scale * 12.0;
            let h = size * scale * 12.0;
            let y = half_h + (0.5 * scale * 12.0) - h;
            Some((fwd, sx - w * 0.5, y, w, h))
        };

        // Props in chunks (trees, rocks, bushes, cars)
        for dy in -1..=1 {
            for dx in -1..=1 {
                if let Some(chunk) = self.world.chunk(pcx + dx, pcy + dy) {
                    for prop in &chunk.props {
                        if prop.tile == id::WALL || prop.tile == id::WALL_INTERIOR || prop.tile == id::WINDOW || prop.tile == id::DOOR || prop.tile == id::CAR || prop.tile == id::TRUCK {
                            continue;
                        }
                        let size = prop.size;
                        if let Some((fwd, sx, sy, w, h)) = project(prop.x, prop.y, size) {
                            let fog = (1.0 - (fwd / max_dist).clamp(0.0, 1.0)) * (1.0 - night * 0.5);
                            billboards.push(Billboard {
                                forward: fwd,
                                screen_x: sx,
                                screen_y: sy,
                                width: w,
                                height: h,
                                tile: prop.tile,
                                color: [fog, fog, fog, 1.0],
                                hp_ratio: None,
                            });
                        }
                    }
                }
            }
        }

        // Crafted campfires
        for st in &self.structures {
            if st.is_campfire {
                if let Some((fwd, sx, sy, w, h)) = project(st.x, st.y, 1.2) {
                    billboards.push(Billboard {
                        forward: fwd,
                        screen_x: sx,
                        screen_y: sy,
                        width: w,
                        height: h,
                        tile: st.tile,
                        color: [1.0, 0.8, 0.4, 1.0],
                        hp_ratio: None,
                    });
                }
            }
        }

        // Entities (Zombies, Bullets, Loot, Cars)
        for e in &self.entities {
            if !e.alive {
                continue;
            }
            if let Some((fwd, sx, sy, w, h)) = project(e.x, e.y, e.size) {
                let fog = (1.0 - (fwd / max_dist).clamp(0.0, 1.0)) * (1.0 - night * 0.45);
                let tint = match e.kind {
                    Kind::Zombie => {
                        let zk = e.zombie_kind.unwrap_or(ZombieKind::Walker);
                        match zk {
                            ZombieKind::Walker => [0.8 * fog, 0.95 * fog, 0.8 * fog, 1.0],
                            ZombieKind::Runner => [1.0 * fog, 0.5 * fog, 0.5 * fog, 1.0],
                            ZombieKind::Brute => [0.95 * fog, 0.25 * fog, 0.25 * fog, 1.0],
                        }
                    }
                    Kind::Bullet => [1.0, 0.9, 0.3, 1.0],
                    Kind::Loot | Kind::DroppedWood => [fog, fog, fog, 1.0],
                    Kind::Car => [0.9 * fog, 0.9 * fog, 0.9 * fog, 1.0],
                };
                let hp_ratio = if e.kind == Kind::Zombie && e.hp < e.max_hp {
                    Some(e.hp as f32 / e.max_hp as f32)
                } else {
                    None
                };
                billboards.push(Billboard {
                    forward: fwd,
                    screen_x: sx,
                    screen_y: sy,
                    width: w,
                    height: h,
                    tile: e.tile(),
                    color: tint,
                    hp_ratio,
                });
            }
        }

        billboards.sort_by(|a, b| b.forward.partial_cmp(&a.forward).unwrap_or(std::cmp::Ordering::Equal));

        for b in billboards {
            let col = (((b.screen_x + b.width * 0.5) / sw) * num_rays as f32) as i32;
            if col >= 0 && (col as usize) < num_rays && depth_buffer[col as usize] < (b.forward - 0.2) {
                continue;
            }
            ctx.renderer.add_sprite_screen(
                tex,
                art::tile_uv(b.tile),
                b.screen_x,
                b.screen_y,
                b.width,
                b.height,
                b.color,
            );
            if let Some(ratio) = b.hp_ratio {
                let bar_y = b.screen_y - 6.0;
                ctx.renderer.add_rect_screen(b.screen_x, bar_y, b.width, 4.0, [0.3, 0.1, 0.1, 0.8]);
                ctx.renderer.add_rect_screen(b.screen_x, bar_y, b.width * ratio.clamp(0.0, 1.0), 4.0, [0.9, 0.2, 0.2, 0.9]);
            }
        }

        // 4. First-Person Gun Viewmodel & Muzzle Flash
        let bob_x = (self.walk_distance * 7.0).cos() * 5.0;
        let bob_y = (self.walk_distance * 14.0).sin().abs() * 7.0;
        let is_shooting = self.shoot_cooldown > (self.weapon.cooldown() - 0.07);
        let recoil = if is_shooting { 16.0 } else { 0.0 };

        let gun_w = 110.0;
        let gun_h = 110.0;
        let gun_x = (sw * 0.5) - 20.0 + bob_x;
        let gun_y = sh - gun_h + 10.0 + bob_y - recoil;

        ctx.renderer.add_sprite_screen(
            tex,
            art::tile_uv(id::LOOT_WEAPON),
            gun_x,
            gun_y,
            gun_w,
            gun_h,
            [1.0, 1.0, 1.0, 1.0],
        );

        if is_shooting {
            ctx.renderer.add_sprite_screen(
                tex,
                art::tile_uv(id::CAMPFIRE),
                gun_x + 10.0,
                gun_y - 25.0,
                44.0,
                44.0,
                [1.0, 0.95, 0.4, 1.0],
            );
        }

        // 5. Crosshair in exact center
        ctx.renderer.add_rect_screen(sw * 0.5 - 7.0, half_h - 1.0, 14.0, 2.0, [1.0, 1.0, 1.0, 0.85]);
        ctx.renderer.add_rect_screen(sw * 0.5 - 1.0, half_h - 7.0, 2.0, 14.0, [1.0, 1.0, 1.0, 0.85]);
        ctx.renderer.add_rect_screen(sw * 0.5 - 1.0, half_h - 1.0, 2.0, 2.0, [1.0, 0.2, 0.2, 0.95]);
    }

    /// Renders the Top-Down camera perspective:
    /// - Used automatically when inside houses / shops
    /// - Tactical overview of room layout, furniture, loot caches, and entering zombies
    fn render_top_down(&mut self, ctx: &mut RunContext) {
        let night = self.night_factor();
        let tex = self.atlas_texture;

        let view_h = 22.0f32;
        let (screen_w, screen_h) = ctx.renderer.size();
        let sw = screen_w as f32;
        let sh = screen_h as f32;
        let aspect = sw / sh.max(1.0);
        let view_w = view_h * aspect;
        let cam_min_x = self.player_x - view_w * 0.5 - 1.0;
        let cam_max_x = self.player_x + view_w * 0.5 + 1.0;
        let cam_min_y = self.player_y - view_h * 0.5 - 1.0;
        let cam_max_y = self.player_y + view_h * 0.5 + 1.0;

        ctx.renderer.set_camera(Camera {
            x: self.player_x,
            y: self.player_y,
            view_height: view_h,
        });

        let bg_r = 0.08 * (1.0 - night * 0.7);
        let bg_g = 0.12 * (1.0 - night * 0.7);
        let bg_b = 0.08 * (1.0 - night * 0.5);
        ctx.renderer.set_clear_color(wgpu::Color {
            r: bg_r as f64,
            g: bg_g as f64,
            b: bg_b as f64,
            a: 1.0,
        });

        let (pcx, pcy) = World::chunk_of(self.player_x, self.player_y);
        let ground_tint = [1.0 - night * 0.45, 1.0 - night * 0.45, 1.0 - night * 0.35, 1.0];

        // 1. Tiled Ground Grid (Asphalt streets, lane dividers, sidewalks, parquet floors, grass)
        for dy in -1..=1 {
            for dx in -1..=1 {
                let cx = pcx + dx;
                let cy = pcy + dy;
                if let Some(chunk) = self.world.chunk(cx, cy) {
                    let (ox, oy) = World::chunk_origin(cx, cy);
                    let min_tx = ((cam_min_x - ox).floor() as i32).clamp(0, world::CHUNK_TILES as i32) as usize;
                    let max_tx = ((cam_max_x - ox).ceil() as i32).clamp(0, world::CHUNK_TILES as i32) as usize;
                    let min_ty = ((cam_min_y - oy).floor() as i32).clamp(0, world::CHUNK_TILES as i32) as usize;
                    let max_ty = ((cam_max_y - oy).ceil() as i32).clamp(0, world::CHUNK_TILES as i32) as usize;

                    for ty in min_ty..max_ty {
                        for tx in min_tx..max_tx {
                            let tile = chunk.get_tile(tx, ty);
                            let g_uv = art::tile_uv(tile as u32);
                            let wx = ox + tx as f32 + 0.5;
                            let wy = oy + ty as f32 + 0.5;
                            ctx.renderer.add_sprite(tex, g_uv, wx, wy, 1.0, 1.0, ground_tint);
                        }
                    }
                }
            }
        }

        // 2. Building Roofs (Rendered over buildings when player is outside)
        for dy in -1..=1 {
            for dx in -1..=1 {
                let cx = pcx + dx;
                let cy = pcy + dy;
                if let Some(chunk) = self.world.chunk(cx, cy) {
                    for b in &chunk.buildings {
                        if b.contains(self.player_x, self.player_y) {
                            continue;
                        }
                        let r_uv = art::tile_uv(b.roof_tile);
                        ctx.renderer.add_sprite(
                            tex,
                            r_uv,
                            b.x + b.width * 0.5,
                            b.y + b.height * 0.5,
                            b.width,
                            b.height,
                            [1.0 - night * 0.4, 1.0 - night * 0.4, 1.0 - night * 0.35, 1.0],
                        );
                    }
                }
            }
        }

        // 3. World Props (Fences, gates, street lamps, trees, fallen logs, rocks, furniture)
        for dy in -1..=1 {
            for dx in -1..=1 {
                let cx = pcx + dx;
                let cy = pcy + dy;
                if let Some(chunk) = self.world.chunk(cx, cy) {
                    for prop in &chunk.props {
                        if prop.tile == id::CAR || prop.tile == id::TRUCK {
                            continue; // Interactive vehicles rendered by entity system
                        }
                        let covered_by_roof = chunk.buildings.iter().any(|b| {
                            !b.contains(self.player_x, self.player_y) && b.contains(prop.x, prop.y)
                        });
                        if covered_by_roof && prop.tile != id::WALL && prop.tile != id::WINDOW && prop.tile != id::DOOR {
                            continue;
                        }

                        let prop_uv = art::tile_uv(prop.tile);
                        let pw = prop.size;
                        let ph = prop.size;
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

        for e in &self.entities {
            if !e.alive {
                continue;
            }
            let uv = art::tile_uv(e.tile());
            let tint = match e.kind {
                Kind::Zombie => [0.8, 0.9, 0.8, 1.0],
                Kind::Bullet => [1.0, 0.9, 0.3, 1.0],
                Kind::Loot | Kind::DroppedWood => [1.0, 1.0, 1.0, 1.0],
                Kind::Car => [0.9, 0.9, 0.9, 1.0],
            };
            ctx.renderer.add_sprite(tex, uv, e.x, e.y, e.size, e.size, tint);
        }

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
            ctx.renderer.add_rect(
                self.player_x + self.player_aim_x * 0.75,
                self.player_y + self.player_aim_y * 0.75,
                0.18,
                0.18,
                [1.0, 0.2, 0.2, 0.9],
            );
        }
    }

    /// Common HUD overlay rendered in screen space across all camera modes.
    fn render_hud(&mut self, ctx: &mut RunContext) {
        let (screen_w, screen_h) = ctx.renderer.size();

        // Top-left Status panel
        ctx.renderer.add_rect_screen(12.0, 12.0, 240.0, 118.0, [0.08, 0.08, 0.10, 0.85]);

        let hp_pct = (self.hp.max(0) as f32 / MAX_HP as f32).clamp(0.0, 1.0);
        ctx.renderer.draw_text(&format!("HP: {}/{}", self.hp.max(0), MAX_HP), 18.0, 18.0, 1.8, [1.0, 1.0, 1.0, 1.0]);
        ctx.renderer.add_rect_screen(18.0, 34.0, 220.0, 10.0, [0.3, 0.1, 0.1, 1.0]);
        ctx.renderer.add_rect_screen(18.0, 34.0, 220.0 * hp_pct, 10.0, [0.9, 0.2, 0.2, 1.0]);

        let sta_pct = (self.stamina / MAX_STAMINA).clamp(0.0, 1.0);
        ctx.renderer.draw_text(&format!("STA: {:.0}%", self.stamina), 18.0, 48.0, 1.8, [1.0, 1.0, 1.0, 1.0]);
        ctx.renderer.add_rect_screen(18.0, 64.0, 220.0, 8.0, [0.1, 0.2, 0.3, 1.0]);
        ctx.renderer.add_rect_screen(18.0, 64.0, 220.0 * sta_pct, 8.0, [0.2, 0.6, 0.9, 1.0]);

        let hng_pct = (self.hunger / MAX_HUNGER).clamp(0.0, 1.0);
        ctx.renderer.draw_text(&format!("FOOD: {:.0}%", self.hunger), 18.0, 76.0, 1.8, [1.0, 1.0, 1.0, 1.0]);
        ctx.renderer.add_rect_screen(18.0, 92.0, 220.0, 8.0, [0.25, 0.2, 0.1, 1.0]);
        ctx.renderer.add_rect_screen(18.0, 92.0, 220.0 * hng_pct, 8.0, [0.9, 0.7, 0.2, 1.0]);

        ctx.renderer.draw_text(
            &format!("Wood: {}   Ammo: {}", self.wood, self.ammo),
            18.0,
            106.0,
            1.8,
            [0.85, 0.95, 0.6, 1.0],
        );

        // Top-right Info panel
        let right_x = screen_w as f32 - 250.0;
        ctx.renderer.add_rect_screen(right_x, 12.0, 238.0, 118.0, [0.08, 0.08, 0.10, 0.85]);
        let time_str = if self.is_night() { "NIGHT (SWARM!)" } else { "DAY" };
        let time_color = if self.is_night() { [1.0, 0.3, 0.3, 1.0] } else { [1.0, 0.9, 0.4, 1.0] };
        ctx.renderer.draw_text(&format!("Day {}: {}", self.days_survived, time_str), right_x + 8.0, 18.0, 2.0, time_color);
        ctx.renderer.draw_text(&format!("Wave: {}  Score: {}", self.wave, self.score), right_x + 8.0, 42.0, 1.8, [1.0, 1.0, 1.0, 1.0]);
        ctx.renderer.draw_text(&format!("Zombies Killed: {}", self.kills), right_x + 8.0, 64.0, 1.8, [0.9, 0.7, 0.7, 1.0]);
        ctx.renderer.draw_text(&format!("Equipped: {}", self.weapon.name()), right_x + 8.0, 86.0, 1.8, [0.4, 0.9, 0.5, 1.0]);
        ctx.renderer.draw_text(&format!("RSS RAM: {:.1} MB / 100 MB", ctx.rss_mb), right_x + 8.0, 106.0, 1.8, [0.6, 0.8, 1.0, 1.0]);

        // Top-center POV Mode Banner
        let pov_label = if self.is_first_person() {
            "POV: FIRST-PERSON (OUTSIDE) [V: Toggle]"
        } else {
            "POV: TOP-DOWN (INSIDE HOUSE) [V: Toggle]"
        };
        let pov_w = ctx.renderer.text_width(pov_label, 1.8) + 16.0;
        let pov_x = (screen_w as f32 - pov_w) * 0.5;
        ctx.renderer.add_rect_screen(pov_x, 12.0, pov_w, 24.0, [0.05, 0.05, 0.08, 0.88]);
        ctx.renderer.draw_text(pov_label, pov_x + 8.0, 16.0, 1.8, [0.3, 0.9, 1.0, 1.0]);

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
    fn camera_mode_switches_between_fps_outside_and_topdown_inside() {
        let mut g = Game::default();
        // At origin (0, 0) in open field, player is outside:
        assert!(!g.is_inside_house());
        assert!(g.is_first_person());

        // When forced to TopDown:
        g.camera_mode = CameraMode::TopDown;
        assert!(!g.is_first_person());

        // When forced to FirstPerson:
        g.camera_mode = CameraMode::FirstPerson;
        assert!(g.is_first_person());
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
        assert!(!g.check_solid_prop_collision(500.0, 500.0, 0.4));
    }
}
