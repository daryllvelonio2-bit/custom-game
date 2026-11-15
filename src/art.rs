//! Procedural pixel-art sprite atlas.
//!
//! The game ships no binary assets: every sprite is drawn from ASCII art at
//! startup and uploaded to the GPU as a single atlas texture.

use image::{Rgba, RgbaImage};

/// Size of one atlas cell in pixels.
pub const TILE: u32 = 8;

/// Atlas grid dimensions (tiles per row / column).
pub const ATLAS_COLS: u32 = 16;
pub const ATLAS_ROWS: u32 = 2;

/// Indices into the atlas grid.
pub mod id {
    pub const GRASS: u32 = 0;
    pub const DIRT: u32 = 1;
    pub const WATER: u32 = 2;
    pub const ROAD: u32 = 3;
    pub const FLOOR: u32 = 4;
    pub const WALL: u32 = 5;
    pub const DOOR: u32 = 6;
    pub const TREE: u32 = 7;
    pub const STUMP: u32 = 8;
    pub const ROCK: u32 = 9;
    pub const BUSH: u32 = 10;
    pub const PLAYER: u32 = 11;
    pub const ZOMBIE: u32 = 12;
    pub const RUNNER: u32 = 13;
    pub const BRUTE: u32 = 14;
    pub const CAR: u32 = 15;
    pub const CRATE: u32 = 16;
    pub const LOOT_FOOD: u32 = 17;
    pub const LOOT_AMMO: u32 = 18;
    pub const LOOT_WEAPON: u32 = 19;
    pub const LOOT_MATERIAL: u32 = 20;
    pub const FISH: u32 = 21;
    pub const BARREL: u32 = 22;
    pub const CAMPFIRE: u32 = 23;
    pub const FENCE: u32 = 24;
    pub const SHOP: u32 = 25;
    pub const HOUSE: u32 = 26;
    pub const FISHING_SPOT: u32 = 27;
    pub const BULLET: u32 = 28;
    pub const BED: u32 = 29;
    pub const TABLE: u32 = 30;
    pub const SHELF: u32 = 31;
}

/// Colour key: `.` transparent, `X` black, plus named colours.
fn parse_pixel(ch: char) -> Option<Rgba<u8>> {
    match ch {
        '.' => None,
        'X' => Some(Rgba([20, 18, 24, 255])),
        'w' => Some(Rgba([240, 240, 235, 255])),
        'g' => Some(Rgba([92, 148, 66, 255])),
        'G' => Some(Rgba([58, 104, 48, 255])),
        'd' => Some(Rgba([142, 100, 62, 255])),
        'D' => Some(Rgba([104, 72, 44, 255])),
        'b' => Some(Rgba([56, 86, 150, 255])),
        'B' => Some(Rgba([34, 56, 104, 255])),
        'r' => Some(Rgba([178, 58, 52, 255])),
        'R' => Some(Rgba([118, 34, 34, 255])),
        'p' => Some(Rgba([196, 150, 190, 255])),
        'y' => Some(Rgba([226, 196, 84, 255])),
        'o' => Some(Rgba([214, 134, 52, 255])),
        'c' => Some(Rgba([206, 206, 198, 255])),
        'C' => Some(Rgba([150, 150, 144, 255])),
        's' => Some(Rgba([188, 188, 196, 255])),
        'k' => Some(Rgba([44, 38, 48, 255])),
        'm' => Some(Rgba([150, 90, 170, 255])),
        'e' => Some(Rgba([120, 80, 50, 255])),
        'n' => Some(Rgba([70, 130, 180, 255])),
        'f' => Some(Rgba([90, 180, 90, 255])),
        'h' => Some(Rgba([220, 180, 120, 255])),
        'Y' => Some(Rgba([255, 232, 110, 255])),
        'O' => Some(Rgba([240, 165, 70, 255])),
        _ => None,
    }
}

/// One atlas cell: 8 rows of 8 characters.
type Cell = [&'static str; 8];

const GRASS: Cell = [
    "........",
    "..g..g..",
    ".gG.g.G.",
    "..g..g..",
    ".g.G..g.",
    "..g..g..",
    ".G..g.G.",
    "........",
];

const DIRT: Cell = [
    "........",
    "..d..d..",
    ".dD.d.d.",
    "..d..d..",
    ".d.d..d.",
    "..d..d..",
    ".D..d.D.",
    "........",
];

const WATER: Cell = [
    "........",
    ".n.n.n..",
    "..nnn...",
    ".n.nn.n.",
    "..nnn...",
    ".n.n.n..",
    "..nnn...",
    "........",
];

const ROAD: Cell = [
    "cccccccc",
    "ccCccccc",
    "cccccccc",
    "cccccccc",
    "ccCccccc",
    "cccccccc",
    "cccccccc",
    "ccCccccc",
];

const FLOOR: Cell = [
    "........",
    ".c.c.c..",
    "........",
    "..c.c.c.",
    "........",
    ".c.c.c..",
    "........",
    "..c.c.c.",
];

const WALL: Cell = [
    "XXXXXXXX",
    "XwwwwwwX",
    "XwddddwX",
    "XwddddwX",
    "XwddddwX",
    "XwddddwX",
    "XwddddwX",
    "XXXXXXXX",
];

const DOOR: Cell = [
    "........",
    "..dddd..",
    "..d..d..",
    "..d..d..",
    "..d..d..",
    "..d..d..",
    "..dddd..",
    "........",
];

const TREE: Cell = [
    "........",
    "...G....",
    "..GGG...",
    ".GGGGG..",
    ".GgggG..",
    "GGgggGG.",
    "GgggggG.",
    ".DDDDD..",
];

const STUMP: Cell = [
    "........",
    "........",
    "........",
    "........",
    "..dddd..",
    "..d..d..",
    "..dddd..",
    "........",
];

const ROCK: Cell = [
    "........",
    "........",
    "...CC...",
    "..CCCC..",
    ".CCCCCC.",
    ".CCCCCC.",
    "..CCCC..",
    "........",
];

const BUSH: Cell = [
    "........",
    "........",
    "..g..g..",
    ".gGggGg.",
    "GgGgggGG",
    ".gggggg.",
    "..gggg..",
    "........",
];

const PLAYER: Cell = [
    "..hhhh..",
    ".hhhhhh.",
    ".kokoko.",
    ".khhkok.",
    ".kokoko.",
    "..kok...",
    ".d..d...",
    "dd..dd..",
];

const ZOMBIE: Cell = [
    "..gggg..",
    ".gppppg.",
    ".kokoko.",
    ".kggkok.",
    ".kokoko.",
    "..kok...",
    ".d..d...",
    "dd..dd..",
];

const RUNNER: Cell = [
    "..pppp..",
    ".pppppp.",
    ".kokoko.",
    ".kppkok.",
    ".kokoko.",
    "..kok...",
    ".d..d...",
    "dd..dd..",
];

const BRUTE: Cell = [
    "..rrrr..",
    ".rrrrrr.",
    ".kokoko.",
    ".krrkok.",
    ".kokoko.",
    "..kok...",
    ".d..d...",
    "dddddd..",
];

const CAR: Cell = [
    "........",
    ".kkkkkk.",
    "krrrrrrk",
    "krssssrk",
    "krssssrk",
    "krrrrrrk",
    ".kkkkkk.",
    "........",
];

const CRATE: Cell = [
    "XXXXXXXX",
    "XddddddX",
    "XdwwwwdX",
    "XdwddwdX",
    "XdwddwdX",
    "XdwwwwdX",
    "XddddddX",
    "XXXXXXXX",
];

const LOOT_FOOD: Cell = [
    "........",
    "........",
    "...oo...",
    "..oOeo..",
    "..oeeo..",
    "...oo...",
    "........",
    "........",
];

const LOOT_AMMO: Cell = [
    "........",
    "........",
    ".yy.yy..",
    ".yyyy...",
    ".y..y...",
    ".yy.yy..",
    "........",
    "........",
];

const LOOT_WEAPON: Cell = [
    "........",
    "........",
    ".kkkk...",
    ".k..kkk.",
    ".k....k.",
    ".kkkkk..",
    "........",
    "........",
];

const LOOT_MATERIAL: Cell = [
    "........",
    "........",
    ".dd.dd..",
    ".dD.Dd..",
    ".dD.Dd..",
    ".dd.dd..",
    "........",
    "........",
];

const FISH: Cell = [
    "........",
    "........",
    "..nnn...",
    ".nnnnn..",
    "..nnn...",
    "..n..n..",
    "........",
    "........",
];

const BARREL: Cell = [
    "........",
    ".oooooo.",
    ".oXooXo.",
    ".oooooo.",
    ".oooooo.",
    ".oXooXo.",
    ".oooooo.",
    "........",
];

const CAMPFIRE: Cell = [
    "........",
    "....y...",
    "...yYo..",
    "..oyYo..",
    ".eoyyoe.",
    ".eeooee.",
    "..eeee..",
    "........",
];

const FENCE: Cell = [
    "........",
    ".d.d.d..",
    "ddddddd.",
    ".d.d.d..",
    "ddddddd.",
    ".d.d.d..",
    "ddddddd.",
    "........",
];

const SHOP: Cell = [
    "XXXXXXXX",
    "XrrrrrrX",
    "XrrrrrrX",
    "XwwwwwwX",
    "XwwwwwwX",
    "XwwwwwwX",
    "XwwwwwwX",
    "XXXXXXXX",
];

const HOUSE: Cell = [
    "XXXXXXXX",
    "XddddddX",
    "XddddddX",
    "XddddddX",
    "XwwwwwwX",
    "XwwwwwwX",
    "XwwwwwwX",
    "XXXXXXXX",
];

const FISHING_SPOT: Cell = [
    "........",
    "........",
    "........",
    "...n....",
    "..nnn...",
    "...n....",
    "........",
    "........",
];

const BULLET: Cell = [
    "........",
    "........",
    "........",
    "..yyyy..",
    "...yy...",
    "........",
    "........",
    "........",
];

const BED: Cell = [
    "........",
    "........",
    ".ccccc..",
    "csssssc.",
    "csssssc.",
    ".ccccc..",
    "........",
    "........",
];

const TABLE: Cell = [
    "........",
    ".dddddd.",
    "d......d",
    "d......d",
    "d......d",
    "d......d",
    ".dddddd.",
    "........",
];

const SHELF: Cell = [
    "........",
    "dddddddd",
    "d.w..w.d",
    "dddddddd",
    "d.w..w.d",
    "dddddddd",
    "d.w..w.d",
    "dddddddd",
];

/// All cells in atlas order (row-major, 16 per row).
fn all_cells() -> Vec<Cell> {
    vec![
        GRASS, DIRT, WATER, ROAD, FLOOR, WALL, DOOR, TREE, STUMP, ROCK, BUSH, PLAYER,
        ZOMBIE, RUNNER, BRUTE, CAR, CRATE, LOOT_FOOD, LOOT_AMMO, LOOT_WEAPON, LOOT_MATERIAL,
        FISH, BARREL, CAMPFIRE, FENCE, SHOP, HOUSE, FISHING_SPOT, BULLET, BED, TABLE, SHELF,
    ]
}

/// Build the atlas image.
pub fn build_atlas() -> RgbaImage {
    let width = ATLAS_COLS * TILE;
    let height = ATLAS_ROWS * TILE;
    let mut img = RgbaImage::new(width, height);
    for (index, cell) in all_cells().into_iter().enumerate() {
        let ox = (index as u32 % ATLAS_COLS) * TILE;
        let oy = (index as u32 / ATLAS_COLS) * TILE;
        for (row, line) in cell.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                if let Some(pixel) = parse_pixel(ch) {
                    img.put_pixel(ox + col as u32, oy + row as u32, pixel);
                }
            }
        }
    }
    img
}

/// UV rectangle for an atlas tile id.
pub fn tile_uv(id: u32) -> [f32; 4] {
    let width = ATLAS_COLS * TILE;
    let height = ATLAS_ROWS * TILE;
    let ox = (id % ATLAS_COLS) * TILE;
    let oy = (id / ATLAS_COLS) * TILE;
    let u0 = ox as f32 / width as f32;
    let v0 = oy as f32 / height as f32;
    let u1 = (ox + TILE) as f32 / width as f32;
    let v1 = (oy + TILE) as f32 / height as f32;
    [u0, v0, u1, v1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atlas_contains_every_tile() {
        assert_eq!(all_cells().len() as u32, ATLAS_COLS * ATLAS_ROWS);
    }

    #[test]
    fn tile_uv_is_normalised() {
        let uv = tile_uv(id::PLAYER);
        assert!(uv[0] >= 0.0 && uv[2] <= 1.0);
        assert!(uv[1] >= 0.0 && uv[3] <= 1.0);
        assert!(uv[2] > uv[0] && uv[3] > uv[1]);
    }

    #[test]
    fn atlas_pixels_are_drawn() {
        let img = build_atlas();
        let uv = tile_uv(id::PLAYER);
        let px = (uv[0] * img.width() as f32) as u32;
        let py = (uv[1] * img.height() as f32) as u32;
        assert_ne!(img.get_pixel(px + 1, py + 1).0, [0, 0, 0, 0]);
    }
}
