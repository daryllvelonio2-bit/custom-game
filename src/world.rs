//! Infinite, deterministic world generation.
//!
//! The world is an endless grid of chunks. Each chunk is derived purely from
//! its coordinates and the world seed, so it can be generated on demand and
//! thrown away when the player walks away. Chunks hold towns (with enterable
//! buildings), forests, lakes and open fields.

use std::collections::HashMap;

use crate::art::id;

/// Side length of a chunk in world units.
pub const CHUNK_SIZE: f32 = 40.0;

/// How many chunks around the player stay resident.
pub const VIEW_RADIUS: i32 = 2;

/// Biome of a chunk.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Biome {
    Field,
    Forest,
    Town,
    Lake,
}

/// A prop standing in the world.
#[derive(Clone, Debug)]
pub struct Prop {
    pub x: f32,
    pub y: f32,
    /// Atlas tile to draw.
    pub tile: u32,
    /// Blocks movement (buildings, trees, cars).
    pub solid: bool,
    /// Can be chopped for materials.
    pub tree: bool,
    /// Loot contents of a building, spawned when the player enters.
    pub loot: Option<Vec<LootSpawn>>,
    /// Interior floor extent (for buildings).
    pub interior: Option<(f32, f32, f32, f32)>,
}

/// A loot item placed inside a building.
#[derive(Clone, Copy, Debug)]
pub struct LootSpawn {
    pub x: f32,
    pub y: f32,
    pub kind: LootKind,
}

/// Kinds of loot that can be found.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LootKind {
    Food,
    Ammo,
    Weapon,
    Material,
}

/// A generated chunk.
#[derive(Clone, Debug)]
pub struct Chunk {
    pub cx: i32,
    pub cy: i32,
    pub biome: Biome,
    pub props: Vec<Prop>,
    /// Roads run horizontally and vertically through town chunks.
    pub roads: bool,
    /// Lake centre and radius, when this chunk holds a lake.
    pub lake_center: Option<(f32, f32, f32)>,
}

/// The streaming world: keeps a bounded number of chunks resident.
pub struct World {
    pub seed: u64,
    chunks: HashMap<(i32, i32), Chunk>,
    /// Chunks whose loot has already been handed to the entity list.
    pub looted: std::collections::HashSet<(i32, i32)>,
}

impl World {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            chunks: HashMap::new(),
            looted: std::collections::HashSet::new(),
        }
    }

    /// World-space position of a chunk's origin.
    pub fn chunk_origin(cx: i32, cy: i32) -> (f32, f32) {
        (cx as f32 * CHUNK_SIZE, cy as f32 * CHUNK_SIZE)
    }

    /// Chunk coordinate containing a world position.
    pub fn chunk_of(x: f32, y: f32) -> (i32, i32) {
        (
            (x / CHUNK_SIZE).floor() as i32,
            (y / CHUNK_SIZE).floor() as i32,
        )
    }

    /// Make sure chunks around the player exist, and drop distant ones.
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
        // Evict chunks that fell outside the resident window.
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

    /// All resident chunks.
    pub fn chunks(&self) -> impl Iterator<Item = &Chunk> {
        self.chunks.values()
    }

    /// Mutable iterator over all resident chunks.
    pub fn chunks_mut(&mut self) -> impl Iterator<Item = &mut Chunk> {
        self.chunks.values_mut()
    }

    /// Generate one chunk deterministically.
    pub fn generate(&self, cx: i32, cy: i32) -> Chunk {
        let mut rng = Rng::new(hash_seed(self.seed, cx, cy));
        let roll = rng.next_f32();

        let biome = if roll < 0.10 {
            Biome::Town
        } else if roll < 0.26 {
            Biome::Lake
        } else if roll < 0.62 {
            Biome::Forest
        } else {
            Biome::Field
        };

        let mut chunk = Chunk {
            cx,
            cy,
            biome,
            props: Vec::new(),
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

    fn build_town(&self, chunk: &mut Chunk, ox: f32, oy: f32, rng: &mut Rng) {
        // A cross of roads through the middle.
        let mid = CHUNK_SIZE * 0.5;
        // Four building plots around the crossroads.
        let spots = [
            (CHUNK_SIZE * 0.24, CHUNK_SIZE * 0.24),
            (CHUNK_SIZE * 0.76, CHUNK_SIZE * 0.24),
            (CHUNK_SIZE * 0.24, CHUNK_SIZE * 0.76),
            (CHUNK_SIZE * 0.76, CHUNK_SIZE * 0.76),
        ];
        for (index, (sx, sy)) in spots.iter().enumerate() {
            let bx = ox + sx - 5.0;
            let by = oy + sy - 4.0;
            let is_shop = index == 3;
            let width = 10.0;
            let height = 8.0;
            let tile = if is_shop { id::SHOP } else { id::HOUSE };

            // Walls around the building, with a gap for the door.
            let door_x = bx + width * 0.5;
            let door_y = by + height;
            for i in 0..(width as i32) {
                let x = bx + i as f32;
                // Top wall
                chunk.props.push(Prop {
                    x,
                    y: by,
                    tile: id::WALL,
                    solid: true,
                    tree: false,
                    loot: None,
                    interior: None,
                });
                // Bottom wall, leaving a door gap in the middle.
                if (x - door_x).abs() > 1.0 {
                    chunk.props.push(Prop {
                        x,
                        y: by + height,
                        tile: id::WALL,
                        solid: true,
                        tree: false,
                        loot: None,
                        interior: None,
                    });
                }
            }
            for i in 0..(height as i32) {
                let y = by + i as f32;
                chunk.props.push(Prop {
                    x: bx,
                    y,
                    tile: id::WALL,
                    solid: true,
                    tree: false,
                    loot: None,
                    interior: None,
                });
                chunk.props.push(Prop {
                    x: bx + width,
                    y,
                    tile: id::WALL,
                    solid: true,
                    tree: false,
                    loot: None,
                    interior: None,
                });
            }
            // Door.
            chunk.props.push(Prop {
                x: door_x,
                y: door_y,
                tile: id::DOOR,
                solid: false,
                tree: false,
                loot: None,
                interior: None,
            });

            // Interior: floor marker plus furniture and loot.
            let mut loot = Vec::new();
            let items = 1 + rng.next_u32() % 3;
            for _ in 0..items {
                let lx = bx + 2.0 + rng.next_f32() * (width - 4.0);
                let ly = by + 2.0 + rng.next_f32() * (height - 4.0);
                let kind = match rng.next_u32() % 10 {
                    0..=3 => LootKind::Ammo,
                    4..=6 => LootKind::Food,
                    7 => LootKind::Weapon,
                    _ => LootKind::Material,
                };
                loot.push(LootSpawn { x: lx, y: ly, kind });
            }
            // Furniture.
            chunk.props.push(Prop {
                x: bx + 2.0,
                y: by + 2.0,
                tile: id::TABLE,
                solid: true,
                tree: false,
                loot: None,
                interior: None,
            });
            chunk.props.push(Prop {
                x: bx + width - 2.0,
                y: by + height - 2.0,
                tile: id::BED,
                solid: true,
                tree: false,
                loot: None,
                interior: None,
            });
            chunk.props.push(Prop {
                x: bx + width - 2.0,
                y: by + 2.0,
                tile: id::SHELF,
                solid: true,
                tree: false,
                loot: None,
                interior: None,
            });

            chunk.props.push(Prop {
                x: bx + width * 0.5,
                y: by + height * 0.5,
                tile,
                solid: false,
                tree: false,
                loot: Some(loot),
                interior: Some((bx, by, width, height)),
            });
        }

        // Cars parked by the roadside.
        let car_count = 1 + rng.next_u32() % 2;
        for _ in 0..car_count {
            let cx = ox + 4.0 + rng.next_f32() * (CHUNK_SIZE - 8.0);
            let cy = oy + mid - 1.5;
            // Cars parked by the roadside (tracked separately as interactive entities if spawned, or props).
            chunk.props.push(Prop {
                x: cx,
                y: cy,
                tile: id::CAR,
                solid: false,
                tree: false,
                loot: None,
                interior: None,
            });
        }
    }

    fn build_forest(&self, chunk: &mut Chunk, ox: f32, oy: f32, rng: &mut Rng) {
        let count = 26 + rng.next_u32() % 22;
        for _ in 0..count {
            let x = ox + rng.next_f32() * CHUNK_SIZE;
            let y = oy + rng.next_f32() * CHUNK_SIZE;
            chunk.props.push(Prop {
                x,
                y,
                tile: id::TREE,
                solid: true,
                tree: true,
                loot: None,
                interior: None,
            });
        }
        // A few rocks and bushes for variety.
        for _ in 0..6 {
            let x = ox + rng.next_f32() * CHUNK_SIZE;
            let y = oy + rng.next_f32() * CHUNK_SIZE;
            let tile = if rng.next_u32() % 2 == 0 { id::ROCK } else { id::BUSH };
            chunk.props.push(Prop {
                x,
                y,
                tile,
                solid: tile == id::ROCK,
                tree: false,
                loot: None,
                interior: None,
            });
        }
    }

    fn build_lake(&self, chunk: &mut Chunk, ox: f32, oy: f32, rng: &mut Rng) {
        let cx = ox + CHUNK_SIZE * (0.35 + rng.next_f32() * 0.3);
        let cy = oy + CHUNK_SIZE * (0.35 + rng.next_f32() * 0.3);
        let radius = 9.0 + rng.next_f32() * 4.0;
        // Fishing spots sit on the lake edge.
        for i in 0..4 {
            let angle = (i as f32 / 4.0) * std::f32::consts::TAU + rng.next_f32() * 0.4;
            chunk.props.push(Prop {
                x: cx + angle.cos() * (radius + 1.2),
                y: cy + angle.sin() * (radius + 1.2),
                tile: id::FISHING_SPOT,
                solid: false,
                tree: false,
                loot: None,
                interior: None,
            });
        }
        for _ in 0..5 {
            let x = ox + rng.next_f32() * CHUNK_SIZE;
            let y = oy + rng.next_f32() * CHUNK_SIZE;
            chunk.props.push(Prop {
                x,
                y,
                tile: id::TREE,
                solid: true,
                tree: true,
                loot: None,
                interior: None,
            });
        }
        chunk.lake_center = Some((cx, cy, radius));
    }

    fn build_field(&self, chunk: &mut Chunk, ox: f32, oy: f32, rng: &mut Rng) {
        let count = 4 + rng.next_u32() % 8;
        for _ in 0..count {
            let x = ox + rng.next_f32() * CHUNK_SIZE;
            let y = oy + rng.next_f32() * CHUNK_SIZE;
            chunk.props.push(Prop {
                x,
                y,
                tile: id::TREE,
                solid: true,
                tree: true,
                loot: None,
                interior: None,
            });
        }
        for _ in 0..4 {
            let x = ox + rng.next_f32() * CHUNK_SIZE;
            let y = oy + rng.next_f32() * CHUNK_SIZE;
            chunk.props.push(Prop {
                x,
                y,
                tile: id::ROCK,
                solid: true,
                tree: false,
                loot: None,
                interior: None,
            });
        }
    }

    /// Is a world position inside a lake? Returns (centre, radius) when so.
    pub fn lake_at(&self, x: f32, y: f32) -> Option<(f32, f32, f32)> {
        let (cx, cy) = Self::chunk_of(x, y);
        let chunk = self.chunk(cx, cy)?;
        let (lx, ly, r) = chunk.lake_center?;
        let dx = x - lx;
        let dy = y - ly;
        if dx * dx + dy * dy <= r * r {
            Some((lx, ly, r))
        } else {
            None
        }
    }
}

/// Hash a seed with chunk coordinates into a deterministic 64-bit value.
pub fn hash_seed(seed: u64, cx: i32, cy: i32) -> u64 {
    let mut h = seed
        ^ (cx as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (cy as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51_afd7_ed55_8ccd);
    h ^= h >> 33;
    h = h.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    h ^= h >> 33;
    h
}

/// Small deterministic PRNG (xorshift64*).
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0x9E37_79B9_7F4A_7C15 } else { seed },
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    pub fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u32 << 24) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generation_is_deterministic() {
        let a = World::new(42).generate(3, -2);
        let b = World::new(42).generate(3, -2);
        assert_eq!(a.biome, b.biome);
        assert_eq!(a.props.len(), b.props.len());
        assert_eq!(a.props[0].x, b.props[0].x);
    }

    #[test]
    fn different_seeds_differ() {
        let a = World::new(1).generate(0, 0);
        let b = World::new(2).generate(0, 0);
        let same = a.biome == b.biome
            && a.props.len() == b.props.len()
            && a.props.first().map(|p| p.x) == b.props.first().map(|p| p.x);
        assert!(!same, "different seeds should not generate identical chunks");
    }

    #[test]
    fn streaming_is_bounded() {
        let mut world = World::new(7);
        world.stream_around(0.0, 0.0);
        let first = world.chunks().count();
        world.stream_around(CHUNK_SIZE * 100.0, CHUNK_SIZE * 100.0);
        let second = world.chunks().count();
        assert!(first <= (2 * VIEW_RADIUS + 1).pow(2) as usize);
        assert!(second <= (2 * VIEW_RADIUS + 3).pow(2) as usize);
    }

    #[test]
    fn chunk_coordinates_round_trip() {
        assert_eq!(World::chunk_of(0.0, 0.0), (0, 0));
        assert_eq!(World::chunk_of(-0.1, -0.1), (-1, -1));
        assert_eq!(World::chunk_of(CHUNK_SIZE * 2.0, 0.0), (2, 0));
    }

    #[test]
    fn rng_stays_in_range() {
        let mut rng = Rng::new(99);
        for _ in 0..1000 {
            let v = rng.next_f32();
            assert!((0.0..1.0).contains(&v));
        }
    }
}
