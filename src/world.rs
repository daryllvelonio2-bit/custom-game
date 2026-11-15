//! Infinite, deterministic tile-based world generation matching Sombie.
//!
//! Every chunk is a 40x40 tile grid (1.0 meter per tile) derived deterministically
//! from its coordinates and world seed.
//! Features:
//! - Realistic town grids with asphalt avenues, lane stripes, sidewalks, and street lamps
//! - Enterable multi-room houses (living room, bedroom, kitchen) with parquet floors
//! - Fenced yard plots, driveways with parked cars/trucks, and garden paths
//! - Deep forests with deciduous/pine trees, fallen logs, stumps, and campsites
//! - Shimmering lakes with sandy beaches and wooden fishing piers
//! - Rolling farmland fields with dirt trails and agricultural fencing

use std::collections::{HashMap, HashSet};

use crate::art::id;

/// Number of tiles along one axis of a chunk.
pub const CHUNK_TILES: usize = 40;

/// Physical size of one tile in world units (meters).
pub const TILE_SIZE: f32 = 1.0;

/// Side length of a chunk in world units.
pub const CHUNK_SIZE: f32 = (CHUNK_TILES as f32) * TILE_SIZE; // 40.0

/// How many chunks around the player stay resident.
pub const VIEW_RADIUS: i32 = 2;

/// Biomes that can generate in the world.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Biome {
    Town,
    Forest,
    Lake,
    Field,
}

/// A prop standing in the world.
#[derive(Clone, Debug)]
pub struct Prop {
    pub x: f32,
    pub y: f32,
    /// Atlas tile to draw.
    pub tile: u32,
    /// Blocks movement.
    pub solid: bool,
    /// Can be chopped for wood/materials.
    pub tree: bool,
    /// Visual rendering size in world units.
    pub size: f32,
    /// Loot contents when searched.
    pub loot: Option<Vec<LootSpawn>>,
    /// Interior building bounds if applicable.
    pub interior: Option<(f32, f32, f32, f32)>,
}

/// A building structure within a chunk.
#[derive(Clone, Debug)]
pub struct Building {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub is_shop: bool,
    pub roof_tile: u32,
}

impl Building {
    /// Check whether a world coordinate is inside this building.
    pub fn contains(&self, wx: f32, wy: f32) -> bool {
        wx >= self.x && wx <= self.x + self.width && wy >= self.y && wy <= self.y + self.height
    }
}

/// A loot item placed inside a building or crate.
#[derive(Clone, Copy, Debug)]
pub struct LootSpawn {
    pub x: f32,
    pub y: f32,
    pub kind: LootKind,
}

/// Kinds of loot that can be scavenged.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LootKind {
    Food,
    Ammo,
    Weapon,
    Material,
}

/// A generated chunk with a 40x40 ground tile grid, props, and buildings.
#[derive(Clone, Debug)]
pub struct Chunk {
    pub cx: i32,
    pub cy: i32,
    pub biome: Biome,
    pub tiles: [u8; CHUNK_TILES * CHUNK_TILES],
    pub props: Vec<Prop>,
    pub buildings: Vec<Building>,
    pub roads: bool,
    pub lake_center: Option<(f32, f32, f32)>,
}

impl Chunk {
    /// Get the tile ID at local tile coordinates `(tx, ty)`.
    pub fn get_tile(&self, tx: usize, ty: usize) -> u8 {
        if tx < CHUNK_TILES && ty < CHUNK_TILES {
            self.tiles[ty * CHUNK_TILES + tx]
        } else {
            id::GRASS as u8
        }
    }

    /// Set the tile ID at local tile coordinates `(tx, ty)`.
    pub fn set_tile(&mut self, tx: usize, ty: usize, tile: u8) {
        if tx < CHUNK_TILES && ty < CHUNK_TILES {
            self.tiles[ty * CHUNK_TILES + tx] = tile;
        }
    }

    /// Fill a rectangular tile region.
    pub fn fill_rect(&mut self, x0: usize, y0: usize, w: usize, h: usize, tile: u8) {
        for dy in 0..h {
            let y = y0 + dy;
            if y >= CHUNK_TILES {
                break;
            }
            for dx in 0..w {
                let x = x0 + dx;
                if x >= CHUNK_TILES {
                    break;
                }
                self.tiles[y * CHUNK_TILES + x] = tile;
            }
        }
    }
}

/// The streaming world: keeps a bounded window of chunks resident.
pub struct World {
    pub seed: u64,
    chunks: HashMap<(i32, i32), Chunk>,
    /// Chunks whose loot has already been harvested/spawned into entity lists.
    pub looted: HashSet<(i32, i32)>,
}

impl World {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            chunks: HashMap::new(),
            looted: HashSet::new(),
        }
    }

    /// World-space position of a chunk's bottom-left origin.
    pub fn chunk_origin(cx: i32, cy: i32) -> (f32, f32) {
        (cx as f32 * CHUNK_SIZE, cy as f32 * CHUNK_SIZE)
    }

    /// Chunk coordinate containing a given world position.
    pub fn chunk_of(x: f32, y: f32) -> (i32, i32) {
        (
            (x / CHUNK_SIZE).floor() as i32,
            (y / CHUNK_SIZE).floor() as i32,
        )
    }

    /// Ensure chunks around the player are loaded and evict distant ones.
    pub fn stream_around(&mut self, px: f32, py: f32) {
        let (pcx, pcy) = Self::chunk_of(px, py);
        for dy in -VIEW_RADIUS..=VIEW_RADIUS {
            for dx in -VIEW_RADIUS..=VIEW_RADIUS {
                let key = (pcx + dx, pcy + dy);
                if !self.chunks.contains_key(&key) {
                    let chunk = self.generate(key.0, key.1);
                    self.chunks.insert(key, chunk);
                }
            }
        }
        self.chunks.retain(|&(cx, cy), _| {
            (cx - pcx).abs() <= VIEW_RADIUS + 1 && (cy - pcy).abs() <= VIEW_RADIUS + 1
        });
        self.looted.retain(|&(cx, cy)| {
            (cx - pcx).abs() <= VIEW_RADIUS + 1 && (cy - pcy).abs() <= VIEW_RADIUS + 1
        });
    }

    pub fn chunk(&self, cx: i32, cy: i32) -> Option<&Chunk> {
        self.chunks.get(&(cx, cy))
    }

    pub fn chunk_mut(&mut self, cx: i32, cy: i32) -> Option<&mut Chunk> {
        self.chunks.get_mut(&(cx, cy))
    }

    pub fn chunks(&self) -> impl Iterator<Item = &Chunk> {
        self.chunks.values()
    }

    pub fn chunks_mut(&mut self) -> impl Iterator<Item = &mut Chunk> {
        self.chunks.values_mut()
    }

    /// Check whether a world position is near a lake in resident chunks.
    pub fn lake_at(&self, wx: f32, wy: f32) -> Option<(f32, f32, f32)> {
        let (pcx, pcy) = Self::chunk_of(wx, wy);
        for dy in -1..=1 {
            for dx in -1..=1 {
                if let Some(chunk) = self.chunk(pcx + dx, pcy + dy) {
                    if let Some((cx, cy, r)) = chunk.lake_center {
                        let d2 = (wx - cx).powi(2) + (wy - cy).powi(2);
                        if d2 <= (r + 3.0).powi(2) {
                            return Some((cx, cy, r));
                        }
                    }
                }
            }
        }
        None
    }

    /// Generate one chunk deterministically from its coordinate and seed.
    pub fn generate(&self, cx: i32, cy: i32) -> Chunk {
        let mut rng = Rng::new(hash_seed(self.seed, cx, cy));
        let roll = rng.next_f32();

        // Balanced biomes: Towns with roads & loot, lush Forests, Lakes, and Fields
        let biome = if roll < 0.22 {
            Biome::Town
        } else if roll < 0.40 {
            Biome::Lake
        } else if roll < 0.75 {
            Biome::Forest
        } else {
            Biome::Field
        };

        let mut chunk = Chunk {
            cx,
            cy,
            biome,
            tiles: [id::GRASS as u8; CHUNK_TILES * CHUNK_TILES],
            props: Vec::with_capacity(96),
            buildings: Vec::with_capacity(4),
            roads: biome == Biome::Town,
            lake_center: None,
        };
        let (ox, oy) = Self::chunk_origin(cx, cy);

        match biome {
            Biome::Town => self.build_town(&mut chunk, ox, oy, &mut rng),
            Biome::Forest => self.build_forest(&mut chunk, ox, oy, &mut rng),
            Biome::Lake => self.build_lake(&mut chunk, ox, oy, &mut rng),
            Biome::Field => self.build_field(&mut chunk, ox, oy, &mut rng),
        }

        chunk
    }

    /// Build an authentic Sombie town with an asphalt street crossroad,
    /// sidewalks, street lamps, fenced yards, and enterable multi-room houses.
    fn build_town(&self, chunk: &mut Chunk, ox: f32, oy: f32, rng: &mut Rng) {
        // 1. Horizontal Avenue (tx = 0..40, ty = 18..22)
        // Center lane stripes at ty = 20
        chunk.fill_rect(0, 18, 40, 4, id::ROAD_PLAIN as u8);
        for x in 0..40 {
            if x % 3 != 0 {
                chunk.set_tile(x, 20, id::ROAD as u8);
            }
        }
        // North & South sidewalks
        chunk.fill_rect(0, 17, 40, 1, id::SIDEWALK as u8);
        chunk.fill_rect(0, 22, 40, 1, id::SIDEWALK as u8);

        // 2. Vertical Cross Street (tx = 18..22, ty = 0..40)
        chunk.fill_rect(18, 0, 4, 40, id::ROAD_PLAIN as u8);
        for y in 0..40 {
            if y % 3 != 0 && (y < 17 || y > 22) {
                chunk.set_tile(20, y, id::ROAD as u8);
            }
        }
        // West & East sidewalks
        for y in 0..40 {
            if y < 17 || y > 22 {
                chunk.set_tile(17, y, id::SIDEWALK as u8);
                chunk.set_tile(22, y, id::SIDEWALK as u8);
            }
        }

        // 3. Street Lamps at intersections
        let lamps = [
            (ox + 16.5, oy + 16.5),
            (ox + 23.5, oy + 16.5),
            (ox + 16.5, oy + 23.5),
            (ox + 23.5, oy + 23.5),
        ];
        for (lx, ly) in lamps {
            chunk.props.push(Prop {
                x: lx,
                y: ly,
                tile: id::LAMP_POST,
                solid: true,
                tree: false,
                size: 1.2,
                loot: None,
                interior: None,
            });
        }

        // 4. Four Property Plots around the intersection
        let plots = [
            (3, 3, 13, 13, false),    // South-West plot
            (24, 3, 13, 13, false),   // South-East plot
            (3, 24, 13, 13, false),   // North-West plot
            (24, 24, 13, 13, true),   // North-East plot (Corner General Store)
        ];

        for (plot_x, plot_y, plot_w, plot_h, is_shop) in plots {
            // A. Fenced yard boundaries with a gate
            let gate_facing_x = plot_x < 20;
            let gate_x = if gate_facing_x { plot_x + plot_w } else { plot_x };
            let gate_y = plot_y + plot_h / 2;

            for x in plot_x..=(plot_x + plot_w) {
                for y in [plot_y, plot_y + plot_h] {
                    chunk.props.push(Prop {
                        x: ox + x as f32 + 0.5,
                        y: oy + y as f32 + 0.5,
                        tile: id::FENCE,
                        solid: true,
                        tree: false,
                        size: 1.0,
                        loot: None,
                        interior: None,
                    });
                }
            }
            for y in plot_y..=(plot_y + plot_h) {
                for x in [plot_x, plot_x + plot_w] {
                    if x == gate_x && y == gate_y {
                        // Gate opening
                        chunk.props.push(Prop {
                            x: ox + x as f32 + 0.5,
                            y: oy + y as f32 + 0.5,
                            tile: id::GATE,
                            solid: false,
                            tree: false,
                            size: 1.0,
                            loot: None,
                            interior: None,
                        });
                    } else {
                        chunk.props.push(Prop {
                            x: ox + x as f32 + 0.5,
                            y: oy + y as f32 + 0.5,
                            tile: id::FENCE,
                            solid: true,
                            tree: false,
                            size: 1.0,
                            loot: None,
                            interior: None,
                        });
                    }
                }
            }

            // B. Garden path from gate to house
            let house_x = plot_x + 2;
            let house_y = plot_y + 2;
            let house_w = 9;
            let house_h = 8;
            let door_x = house_x + house_w / 2;
            let door_y = house_y + house_h - 1;

            // Connect path
            let path_y = gate_y;
            for px in (gate_x.min(door_x))..=(gate_x.max(door_x)) {
                chunk.set_tile(px, path_y, id::DIRT as u8);
            }
            for py in (path_y.min(door_y))..=(path_y.max(door_y)) {
                chunk.set_tile(door_x, py, id::DIRT as u8);
            }

            // C. Building Floor & Multi-Room Layout
            chunk.fill_rect(house_x, house_y, house_w, house_h, id::FLOOR as u8);

            chunk.buildings.push(Building {
                x: ox + house_x as f32,
                y: oy + house_y as f32,
                width: house_w as f32,
                height: house_h as f32,
                is_shop,
                roof_tile: if is_shop { id::SHOP } else { id::ROOF },
            });

            // Exterior walls, windows, and door
            for x in house_x..(house_x + house_w) {
                for y in [house_y, house_y + house_h - 1] {
                    if x == door_x && y == door_y {
                        chunk.props.push(Prop {
                            x: ox + x as f32 + 0.5,
                            y: oy + y as f32 + 0.5,
                            tile: id::DOOR,
                            solid: false,
                            tree: false,
                            size: 1.0,
                            loot: None,
                            interior: None,
                        });
                    } else {
                        let is_win = (x == house_x + 2 || x == house_x + house_w - 3) && y == house_y;
                        chunk.props.push(Prop {
                            x: ox + x as f32 + 0.5,
                            y: oy + y as f32 + 0.5,
                            tile: if is_win { id::WINDOW } else { id::WALL },
                            solid: true,
                            tree: false,
                            size: 1.0,
                            loot: None,
                            interior: None,
                        });
                    }
                }
            }
            for y in (house_y + 1)..(house_y + house_h - 1) {
                for x in [house_x, house_x + house_w - 1] {
                    let is_win = y == house_y + house_h / 2;
                    chunk.props.push(Prop {
                        x: ox + x as f32 + 0.5,
                        y: oy + y as f32 + 0.5,
                        tile: if is_win { id::WINDOW } else { id::WALL },
                        solid: true,
                        tree: false,
                        size: 1.0,
                        loot: None,
                        interior: None,
                    });
                }
            }

            // Interior partition wall dividing Bedroom from Living Room
            let partition_x = house_x + 4;
            for y in (house_y + 1)..(house_y + house_h - 2) {
                chunk.props.push(Prop {
                    x: ox + partition_x as f32 + 0.5,
                    y: oy + y as f32 + 0.5,
                    tile: id::WALL_INTERIOR,
                    solid: true,
                    tree: false,
                    size: 1.0,
                    loot: None,
                    interior: None,
                });
            }

            // Furniture inside rooms
            // Room 1 (Bedroom): Bed & Shelf
            chunk.props.push(Prop {
                x: ox + house_x as f32 + 1.5,
                y: oy + house_y as f32 + 1.5,
                tile: id::BED,
                solid: true,
                tree: false,
                size: 1.2,
                loot: None,
                interior: None,
            });
            chunk.props.push(Prop {
                x: ox + house_x as f32 + 3.0,
                y: oy + house_y as f32 + 1.5,
                tile: id::SHELF,
                solid: true,
                tree: false,
                size: 1.0,
                loot: None,
                interior: None,
            });

            // Room 2 (Living / Kitchen): Table, Chair, Counter / Shelves
            chunk.props.push(Prop {
                x: ox + house_x as f32 + 6.0,
                y: oy + house_y as f32 + 3.0,
                tile: id::TABLE,
                solid: true,
                tree: false,
                size: 1.2,
                loot: None,
                interior: None,
            });
            chunk.props.push(Prop {
                x: ox + house_x as f32 + 6.0,
                y: oy + house_y as f32 + 4.2,
                tile: id::CHAIR,
                solid: false,
                tree: false,
                size: 0.9,
                loot: None,
                interior: None,
            });
            chunk.props.push(Prop {
                x: ox + house_x as f32 + 7.5,
                y: oy + house_y as f32 + 1.5,
                tile: if is_shop { id::CRATE } else { id::COUNTER },
                solid: true,
                tree: false,
                size: 1.0,
                loot: None,
                interior: None,
            });

            // Loot caches inside the house
            let mut loot = Vec::with_capacity(4);
            let loot_count = 2 + rng.next_u32() % 3;
            for _ in 0..loot_count {
                let lx = ox + house_x as f32 + 1.5 + rng.next_f32() * (house_w as f32 - 3.0);
                let ly = oy + house_y as f32 + 1.5 + rng.next_f32() * (house_h as f32 - 3.0);
                let kind = match rng.next_u32() % 10 {
                    0..=3 => LootKind::Ammo,
                    4..=6 => LootKind::Food,
                    7 => LootKind::Weapon,
                    _ => LootKind::Material,
                };
                loot.push(LootSpawn { x: lx, y: ly, kind });
            }

            // Building interior center marker
            chunk.props.push(Prop {
                x: ox + house_x as f32 + (house_w as f32) * 0.5,
                y: oy + house_y as f32 + (house_h as f32) * 0.5,
                tile: if is_shop { id::SHOP } else { id::HOUSE },
                solid: false,
                tree: false,
                size: 1.0,
                loot: Some(loot),
                interior: Some((
                    ox + house_x as f32,
                    oy + house_y as f32,
                    house_w as f32,
                    house_h as f32,
                )),
            });

            // Yard bush / tree
            chunk.props.push(Prop {
                x: ox + plot_x as f32 + 1.5,
                y: oy + plot_y as f32 + plot_h as f32 - 1.5,
                tile: id::BUSH,
                solid: false,
                tree: false,
                size: 1.0,
                loot: None,
                interior: None,
            });
        }

        // 5. Parked Cars and Pickup Trucks along the avenue
        let vehicles = [
            (ox + 8.0, oy + 23.5, id::CAR),
            (ox + 31.0, oy + 16.5, id::TRUCK),
        ];
        for (vx, vy, vtile) in vehicles {
            chunk.props.push(Prop {
                x: vx,
                y: vy,
                tile: vtile,
                solid: false, // interactive vehicle
                tree: false,
                size: 2.2,
                loot: None,
                interior: None,
            });
        }
    }

    /// Build deep atmospheric forests with multi-tiered canopies,
    /// evergreen pine trees, fallen logs, mossy stumps, and hunter campsites.
    fn build_forest(&self, chunk: &mut Chunk, ox: f32, oy: f32, rng: &mut Rng) {
        // Winding dirt path cutting across the woods
        let start_y = 10 + (rng.next_u32() % 20) as usize;
        let mut py = start_y;
        for x in 0..CHUNK_TILES {
            chunk.set_tile(x, py, id::DIRT as u8);
            if x % 4 == 0 {
                let drift = (rng.next_u32() % 3) as i32 - 1;
                py = (py as i32 + drift).clamp(4, (CHUNK_TILES - 5) as i32) as usize;
            }
        }

        // Deciduous and pine trees
        let tree_count = 32 + (rng.next_u32() % 16) as usize;
        for _ in 0..tree_count {
            let tx = 2.0 + rng.next_f32() * (CHUNK_SIZE - 4.0);
            let ty = 2.0 + rng.next_f32() * (CHUNK_SIZE - 4.0);
            let is_pine = rng.next_u32() % 3 == 0;
            chunk.props.push(Prop {
                x: ox + tx,
                y: oy + ty,
                tile: if is_pine { id::TREE_PINE } else { id::TREE },
                solid: true,
                tree: true,
                size: if is_pine { 2.6 } else { 2.8 },
                loot: None,
                interior: None,
            });
        }

        // Fallen logs and stumps
        for _ in 0..6 {
            let lx = 3.0 + rng.next_f32() * (CHUNK_SIZE - 6.0);
            let ly = 3.0 + rng.next_f32() * (CHUNK_SIZE - 6.0);
            chunk.props.push(Prop {
                x: ox + lx,
                y: oy + ly,
                tile: id::FALLEN_LOG,
                solid: true,
                tree: true,
                size: 2.2,
                loot: None,
                interior: None,
            });
        }
        for _ in 0..5 {
            let sx = 3.0 + rng.next_f32() * (CHUNK_SIZE - 6.0);
            let sy = 3.0 + rng.next_f32() * (CHUNK_SIZE - 6.0);
            chunk.props.push(Prop {
                x: ox + sx,
                y: oy + sy,
                tile: id::STUMP,
                solid: true,
                tree: false,
                size: 1.1,
                loot: None,
                interior: None,
            });
        }

        // Granite rocks and berry bushes
        for _ in 0..10 {
            let bx = 2.0 + rng.next_f32() * (CHUNK_SIZE - 4.0);
            let by = 2.0 + rng.next_f32() * (CHUNK_SIZE - 4.0);
            let is_rock = rng.next_u32() % 2 == 0;
            chunk.props.push(Prop {
                x: ox + bx,
                y: oy + by,
                tile: if is_rock { id::ROCK } else { id::BUSH },
                solid: is_rock,
                tree: false,
                size: if is_rock { 1.3 } else { 1.0 },
                loot: None,
                interior: None,
            });
        }

        // Occasional survivor campsite in the forest
        if rng.next_f32() < 0.6 {
            let camp_x = ox + 15.0 + rng.next_f32() * 10.0;
            let camp_y = oy + 15.0 + rng.next_f32() * 10.0;
            chunk.props.push(Prop {
                x: camp_x,
                y: camp_y,
                tile: id::CAMPFIRE,
                solid: true,
                tree: false,
                size: 1.3,
                loot: None,
                interior: None,
            });
            chunk.props.push(Prop {
                x: camp_x + 2.0,
                y: camp_y,
                tile: id::CRATE,
                solid: true,
                tree: false,
                size: 1.1,
                loot: Some(vec![
                    LootSpawn { x: camp_x + 2.0, y: camp_y, kind: LootKind::Material },
                    LootSpawn { x: camp_x + 2.0, y: camp_y, kind: LootKind::Ammo },
                ]),
                interior: None,
            });
        }
    }

    /// Build shimmering lakes with sand coastlines, piers, and fishing spots.
    fn build_lake(&self, chunk: &mut Chunk, ox: f32, oy: f32, rng: &mut Rng) {
        let center_tx = 16 + (rng.next_u32() % 8) as usize;
        let center_ty = 16 + (rng.next_u32() % 8) as usize;
        let radius = 10.0 + rng.next_f32() * 4.0;
        let sand_ring = radius + 2.5;

        chunk.lake_center = Some((
            ox + center_tx as f32,
            oy + center_ty as f32,
            radius,
        ));

        // Fill water and sand shoreline tiles
        for ty in 0..CHUNK_TILES {
            for tx in 0..CHUNK_TILES {
                let dx = tx as f32 - center_tx as f32;
                let dy = ty as f32 - center_ty as f32;
                let d = (dx * dx + dy * dy).sqrt();
                if d <= radius {
                    chunk.set_tile(tx, ty, id::WATER as u8);
                } else if d <= sand_ring {
                    chunk.set_tile(tx, ty, id::SAND as u8);
                }
            }
        }

        // Wooden fishing pier extending into the lake
        let dock_y = center_ty;
        let dock_start_x = (center_tx as f32 - sand_ring - 1.0).max(1.0) as usize;
        let dock_end_x = (center_tx as f32 - radius * 0.4) as usize;
        for dx in dock_start_x..=dock_end_x {
            chunk.set_tile(dx, dock_y, id::DOCK as u8);
        }

        // Fishing spots at the end of the dock and on shores
        chunk.props.push(Prop {
            x: ox + dock_end_x as f32 + 0.5,
            y: oy + dock_y as f32 + 0.5,
            tile: id::FISHING_SPOT,
            solid: false,
            tree: false,
            size: 1.2,
            loot: None,
            interior: None,
        });

        for i in 0..4 {
            let angle = (i as f32 / 4.0) * std::f32::consts::TAU + rng.next_f32() * 0.3;
            let fx = ox + center_tx as f32 + angle.cos() * (radius + 0.8);
            let fy = oy + center_ty as f32 + angle.sin() * (radius + 0.8);
            chunk.props.push(Prop {
                x: fx,
                y: fy,
                tile: id::FISHING_SPOT,
                solid: false,
                tree: false,
                size: 1.1,
                loot: None,
                interior: None,
            });
        }
    }

    /// Build open countryside fields with dirt paths, split fences, and hay crates.
    fn build_field(&self, chunk: &mut Chunk, ox: f32, oy: f32, rng: &mut Rng) {
        // Dirt path across the field
        let mid_y = 18 + (rng.next_u32() % 5) as usize;
        for x in 0..CHUNK_TILES {
            chunk.set_tile(x, mid_y, id::DIRT as u8);
        }

        // Split-rail fence lines dividing grazing plots
        for x in 4..36 {
            if x != 16 && x != 17 {
                chunk.props.push(Prop {
                    x: ox + x as f32 + 0.5,
                    y: oy + 10.5,
                    tile: id::FENCE,
                    solid: true,
                    tree: false,
                    size: 1.0,
                    loot: None,
                    interior: None,
                });
            }
        }

        // Hay crates and supply caches
        for _ in 0..4 {
            let cx = 5.0 + rng.next_f32() * (CHUNK_SIZE - 10.0);
            let cy = 5.0 + rng.next_f32() * (CHUNK_SIZE - 10.0);
            chunk.props.push(Prop {
                x: ox + cx,
                y: oy + cy,
                tile: id::CRATE,
                solid: true,
                tree: false,
                size: 1.1,
                loot: Some(vec![LootSpawn {
                    x: ox + cx,
                    y: oy + cy,
                    kind: LootKind::Material,
                }]),
                interior: None,
            });
        }

        // Occasional standalone shade trees
        for _ in 0..5 {
            let tx = 3.0 + rng.next_f32() * (CHUNK_SIZE - 6.0);
            let ty = 3.0 + rng.next_f32() * (CHUNK_SIZE - 6.0);
            chunk.props.push(Prop {
                x: ox + tx,
                y: oy + ty,
                tile: id::TREE,
                solid: true,
                tree: true,
                size: 2.8,
                loot: None,
                interior: None,
            });
        }
    }
}

/// Simple deterministic pseudo-random generator.
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0xdeadbeefcafe } else { seed },
        }
    }

    pub fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.state >> 32) as u32
    }

    pub fn next_f32(&mut self) -> f32 {
        (self.next_u32() as f32) / (u32::MAX as f32)
    }
}

fn hash_seed(seed: u64, cx: i32, cy: i32) -> u64 {
    let mut h = seed ^ ((cx as u64) << 32) ^ (cy as u64);
    h = h.wrapping_mul(0x517cc1b727220a95);
    h ^= h >> 32;
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_streams_deterministic_chunks() {
        let w = World::new(42);
        let c1 = w.generate(0, 0);
        let c2 = w.generate(0, 0);
        assert_eq!(c1.biome, c2.biome);
        assert_eq!(c1.tiles[0], c2.tiles[0]);
        assert_eq!(c1.props.len(), c2.props.len());
    }

    #[test]
    fn chunk_grid_accessors() {
        let mut c = Chunk {
            cx: 0,
            cy: 0,
            biome: Biome::Town,
            tiles: [0; CHUNK_TILES * CHUNK_TILES],
            props: Vec::new(),
            buildings: Vec::new(),
            roads: true,
            lake_center: None,
        };
        c.set_tile(5, 5, id::ROAD as u8);
        assert_eq!(c.get_tile(5, 5), id::ROAD as u8);
        assert_eq!(c.get_tile(0, 0), 0);
    }
}
