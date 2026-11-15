//! Procedural pixel-art sprite atlas (16x16 high-fidelity tiles).
//!
//! Generates the entire sprite atlas procedurally at startup matching the
//! authentic Sombie visual style without shipping heavy binary texture assets,
//! staying strictly within the <= 100 MB RAM budget.

use image::{Rgba, RgbaImage};

/// Size of one atlas cell in pixels (16x16).
pub const TILE: u32 = 16;

/// Atlas grid dimensions (16 columns by 4 rows = 64 tile slots).
pub const ATLAS_COLS: u32 = 16;
pub const ATLAS_ROWS: u32 = 4;

/// Indices into the atlas grid.
pub mod id {
    pub const GRASS: u32 = 0;
    pub const DIRT: u32 = 1;
    pub const WATER: u32 = 2;
    pub const ROAD: u32 = 3;
    pub const ROAD_PLAIN: u32 = 4;
    pub const SIDEWALK: u32 = 5;
    pub const FLOOR: u32 = 6;
    pub const WALL: u32 = 7;
    pub const WALL_INTERIOR: u32 = 8;
    pub const DOOR: u32 = 9;
    pub const WINDOW: u32 = 10;
    pub const ROOF: u32 = 11;
    pub const TREE: u32 = 12;
    pub const TREE_PINE: u32 = 13;
    pub const FALLEN_LOG: u32 = 14;
    pub const STUMP: u32 = 15;
    pub const ROCK: u32 = 16;
    pub const BUSH: u32 = 17;
    pub const FENCE: u32 = 18;
    pub const GATE: u32 = 19;
    pub const SAND: u32 = 20;
    pub const DOCK: u32 = 21;
    pub const BED: u32 = 22;
    pub const TABLE: u32 = 23;
    pub const CHAIR: u32 = 24;
    pub const SHELF: u32 = 25;
    pub const COUNTER: u32 = 26;
    pub const CRATE: u32 = 27;
    pub const CAR: u32 = 28;
    pub const TRUCK: u32 = 29;
    pub const LAMP_POST: u32 = 30;
    pub const CAMPFIRE: u32 = 31;
    pub const BARRICADE: u32 = 32;
    pub const PLAYER: u32 = 33;
    pub const ZOMBIE: u32 = 34;
    pub const RUNNER: u32 = 35;
    pub const BRUTE: u32 = 36;
    pub const LOOT_FOOD: u32 = 37;
    pub const LOOT_AMMO: u32 = 38;
    pub const LOOT_WEAPON: u32 = 39;
    pub const LOOT_MATERIAL: u32 = 40;
    pub const FISHING_SPOT: u32 = 41;
    pub const BULLET: u32 = 42;
    pub const SHOP: u32 = 43;
    pub const HOUSE: u32 = 44;
}

/// Color parsing for procedural 16x16 pixel art.
fn parse_pixel(ch: char) -> Option<Rgba<u8>> {
    match ch {
        '.' => None,
        'X' => Some(Rgba([22, 20, 26, 255])),
        'g' => Some(Rgba([120, 148, 67, 255])), // Sombie olive lawn grass
        'G' => Some(Rgba([84, 112, 45, 255])),  // Dark shade grass
        'l' => Some(Rgba([148, 178, 88, 255])), // Light grass highlight
        'd' => Some(Rgba([142, 100, 62, 255])), // Rich earthy path dirt
        'D' => Some(Rgba([98, 68, 40, 255])),   // Dark moist soil
        'p' => Some(Rgba([175, 145, 115, 255])), // Gravel/pebbles
        'a' => Some(Rgba([78, 78, 80, 255])),   // Asphalt road gray
        'A' => Some(Rgba([58, 58, 60, 255])),   // Dark asphalt
        'Y' => Some(Rgba([230, 200, 50, 255])), // Road yellow center stripe
        'c' => Some(Rgba([185, 185, 178, 255])), // Concrete sidewalk
        'C' => Some(Rgba([135, 135, 130, 255])), // Sidewalk curb / shadow
        'f' => Some(Rgba([236, 215, 172, 255])), // Light parquet floorboards
        'F' => Some(Rgba([195, 170, 128, 255])), // Parquet board seams
        'w' => Some(Rgba([115, 78, 48, 255])),  // Wooden wall exterior
        'W' => Some(Rgba([72, 48, 30, 255])),   // Dark wall trim/shadow
        'i' => Some(Rgba([225, 220, 210, 255])), // Interior wall plaster
        'I' => Some(Rgba([150, 110, 80, 255])), // Interior baseboard
        'b' => Some(Rgba([45, 85, 140, 255])),  // Deep water blue
        'B' => Some(Rgba([80, 140, 205, 255])), // Shimmering water crest
        's' => Some(Rgba([215, 195, 140, 255])), // Shoreline sand
        'S' => Some(Rgba([175, 155, 110, 255])), // Wet beach sand
        'r' => Some(Rgba([175, 60, 48, 255])),  // Red roof shingle
        'R' => Some(Rgba([115, 36, 28, 255])),  // Dark roof shadow
        't' => Some(Rgba([52, 145, 48, 255])),  // Tree canopy green
        'T' => Some(Rgba([28, 92, 25, 255])),   // Dark canopy shadow
        'h' => Some(Rgba([88, 185, 65, 255])),  // Canopy sun highlight
        'k' => Some(Rgba([102, 68, 42, 255])),  // Tree bark brown
        'K' => Some(Rgba([65, 42, 24, 255])),   // Dark bark crease
        'm' => Some(Rgba([160, 165, 175, 255])), // Steel / metal
        'M' => Some(Rgba([40, 42, 48, 255])),   // Dark iron / tire rubber
        'u' => Some(Rgba([140, 190, 230, 240])), // Window glass blue reflection
        'o' => Some(Rgba([230, 185, 145, 255])), // Human skin
        'z' => Some(Rgba([140, 175, 130, 255])), // Zombie rot flesh
        'Z' => Some(Rgba([85, 120, 80, 255])),  // Dark zombie decay
        'v' => Some(Rgba([160, 25, 25, 255])),  // Blood crimson
        'e' => Some(Rgba([255, 160, 40, 255])), // Flame orange ember
        'E' => Some(Rgba([255, 230, 90, 255])), // Bright fire yellow
        'q' => Some(Rgba([245, 245, 240, 255])), // White sheet / pillow
        'Q' => Some(Rgba([55, 95, 165, 255])),  // Blue bed blanket
        _ => None,
    }
}

/// One atlas cell: 16 rows of 16 characters.
type Cell = [&'static str; 16];

const GRASS: Cell = [
    "gggggggggggggggg",
    "gggglgggggggglgg",
    "gggllGggggggllGg",
    "ggggGGggggggggGG",
    "gggggggggggggggg",
    "gglgggggggglgggg",
    "gllGggggggllGggg",
    "ggGGggggggggGGgg",
    "gggggggggggggggg",
    "ggggglgggggggggl",
    "ggggllGggggggllG",
    "ggggggGGggggggGG",
    "gggggggggggggggg",
    "gglgggggggglgggg",
    "gllGggggggllGggg",
    "gggggggggggggggg",
];

const DIRT: Cell = [
    "dddddddddddddddd",
    "ddpddddddDddpddd",
    "ddddDddddddddDdd",
    "dddddddddddddddd",
    "ddDddpdddddddddd",
    "ddddddddDddpdddd",
    "ddddddddddddDddd",
    "ddddpddddddddddd",
    "ddddddddddDddpdd",
    "ddDddddddddddddd",
    "ddddddpddddddDdd",
    "dddddddddddddddd",
    "ddpddddDdddddddd",
    "ddddddddddpddDdd",
    "ddDddpdddddddddd",
    "dddddddddddddddd",
];

const WATER: Cell = [
    "bbbbbbbbbbbbbbbb",
    "bbbBBbbbbbbbBBbb",
    "bbBBBBbbbbbBBBBb",
    "bbbbbbbbbbbbbbbb",
    "bbbbbbbbBBbbbbbb",
    "bbbbbbbBBBBbbbbb",
    "bbbbbbbbbbbbbbbb",
    "bBBbbbbbbbbbBBbb",
    "BBBBbbbbbbbBBBBb",
    "bbbbbbbbbbbbbbbb",
    "bbbbbbbbBBbbbbbb",
    "bbbbbbbBBBBbbbbb",
    "bbbbbbbbbbbbbbbb",
    "bbbBBbbbbbbbBBbb",
    "bbBBBBbbbbbBBBBb",
    "bbbbbbbbbbbbbbbb",
];

const ROAD: Cell = [
    "aaaaaaaaaaaaaaaa",
    "AAAAAAAAAAAAAAAA",
    "aaaaaaaaaaaaaaaa",
    "aaaaaaaaaaaaaaaa",
    "aaaaaaaaaaaaaaaa",
    "aaaaaaaaaaaaaaaa",
    "YYYYYYYYYYYYYYYY",
    "YYYYYYYYYYYYYYYY",
    "aaaaaaaaaaaaaaaa",
    "aaaaaaaaaaaaaaaa",
    "aaaaaaaaaaaaaaaa",
    "aaaaaaaaaaaaaaaa",
    "AAAAAAAAAAAAAAAA",
    "aaaaaaaaaaaaaaaa",
    "AAAAAAAAAAAAAAAA",
    "aaaaaaaaaaaaaaaa",
];

const ROAD_PLAIN: Cell = [
    "aaaaaaaaaaaaaaaa",
    "AAAAAAAAAAAAAAAA",
    "aaaaaaaaaaaaaaaa",
    "aaaaaaaaaaaaaaaa",
    "AAAAAAAAAAAAAAAA",
    "aaaaaaaaaaaaaaaa",
    "aaaaaaaaaaaaaaaa",
    "AAAAAAAAAAAAAAAA",
    "aaaaaaaaaaaaaaaa",
    "aaaaaaaaaaaaaaaa",
    "AAAAAAAAAAAAAAAA",
    "aaaaaaaaaaaaaaaa",
    "aaaaaaaaaaaaaaaa",
    "AAAAAAAAAAAAAAAA",
    "aaaaaaaaaaaaaaaa",
    "AAAAAAAAAAAAAAAA",
];

const SIDEWALK: Cell = [
    "CCCCCCCCCCCCCCCC",
    "cccccccccccccccc",
    "cccccccccccccccc",
    "CccccccccccCcccc",
    "cccccccccccccccc",
    "cccccccccccccccc",
    "cccccccccccccccc",
    "CCCCCCCCCCCCCCCC",
    "cccccccccccccccc",
    "cccccccccccccccc",
    "CccccccccccCcccc",
    "cccccccccccccccc",
    "cccccccccccccccc",
    "cccccccccccccccc",
    "CCCCCCCCCCCCCCCC",
    "CccccccccccCcccc",
];

const FLOOR: Cell = [
    "ffffffffffffffFF",
    "ffffffffffffffFF",
    "ffffffffffffffFF",
    "ffffffffffffffFF",
    "FFFFFFFFFFFFFFFF",
    "ffffffFFffffffff",
    "ffffffFFffffffff",
    "ffffffFFffffffff",
    "ffffffFFffffffff",
    "FFFFFFFFFFFFFFFF",
    "ffffffffffffffFF",
    "ffffffffffffffFF",
    "ffffffffffffffFF",
    "ffffffffffffffFF",
    "FFFFFFFFFFFFFFFF",
    "ffffffFFffffffff",
];

const WALL: Cell = [
    "XXXXXXXXXXXXXXXX",
    "XwwwwwwwwwwwwwwX",
    "XwwwwwwwwwwwwwwX",
    "XWWWWWWWWWWWWWWX",
    "XwwwwwwwwwwwwwwX",
    "XwwwwwwwwwwwwwwX",
    "XWWWWWWWWWWWWWWX",
    "XwwwwwwwwwwwwwwX",
    "XwwwwwwwwwwwwwwX",
    "XWWWWWWWWWWWWWWX",
    "XwwwwwwwwwwwwwwX",
    "XwwwwwwwwwwwwwwX",
    "XWWWWWWWWWWWWWWX",
    "XwwwwwwwwwwwwwwX",
    "XWWWWWWWWWWWWWWX",
    "XXXXXXXXXXXXXXXX",
];

const WALL_INTERIOR: Cell = [
    "XXXXXXXXXXXXXXXX",
    "XiiiiiiiiiiiiiiX",
    "XiiiiiiiiiiiiiiX",
    "XiiiiiiiiiiiiiiX",
    "XiiiiiiiiiiiiiiX",
    "XiiiiiiiiiiiiiiX",
    "XiiiiiiiiiiiiiiX",
    "XiiiiiiiiiiiiiiX",
    "XiiiiiiiiiiiiiiX",
    "XiiiiiiiiiiiiiiX",
    "XiiiiiiiiiiiiiiX",
    "XiiiiiiiiiiiiiiX",
    "XIIIIIIIIIIIIIIX",
    "XIIIIIIIIIIIIIIX",
    "XFFFFFFFFFFFFFFX",
    "XXXXXXXXXXXXXXXX",
];

const DOOR: Cell = [
    "XX............XX",
    "Xw............wX",
    "Xw...ffffff...wX",
    "Xw...ffffff...wX",
    "Xw...ffffff...wX",
    "Xw...ffffff...wX",
    "Xw...ffffff...wX",
    "Xw...ffffff...wX",
    "Xw...ffffff...wX",
    "Xw...ffffff...wX",
    "Xw...ffffff...wX",
    "Xw...ffffff...wX",
    "Xw...ffffff...wX",
    "Xw...ffffff...wX",
    "XW...FFFFFF...WX",
    "XX............XX",
];

const WINDOW: Cell = [
    "XXXXXXXXXXXXXXXX",
    "XwwwwwwwwwwwwwwX",
    "XwwXXuuuuuuXXwwX",
    "XwwXuuuuuuuuXwwX",
    "XwwXuuuXXuuuXwwX",
    "XwwXuuuXXuuuXwwX",
    "XwwXXXXXXXXXXwwX",
    "XwwXuuuXXuuuXwwX",
    "XwwXuuuXXuuuXwwX",
    "XwwXuuuuuuuuXwwX",
    "XwwXXuuuuuuXXwwX",
    "XwwwwwwwwwwwwwwX",
    "XWWWWWWWWWWWWWWX",
    "XwwwwwwwwwwwwwwX",
    "XWWWWWWWWWWWWWWX",
    "XXXXXXXXXXXXXXXX",
];

const ROOF: Cell = [
    "rrrrrrrrrrrrrrrr",
    "rrrrrrrrrrrrrrrr",
    "RRRRRRRRRRRRRRRR",
    "rrrrrrrrrrrrrrrr",
    "rrrrrrrrrrrrrrrr",
    "RRRRRRRRRRRRRRRR",
    "rrrrrrrrrrrrrrrr",
    "rrrrrrrrrrrrrrrr",
    "RRRRRRRRRRRRRRRR",
    "rrrrrrrrrrrrrrrr",
    "rrrrrrrrrrrrrrrr",
    "RRRRRRRRRRRRRRRR",
    "rrrrrrrrrrrrrrrr",
    "rrrrrrrrrrrrrrrr",
    "RRRRRRRRRRRRRRRR",
    "XXXXXXXXXXXXXXXX",
];

const TREE: Cell = [
    ".....hhhhhh.....",
    "...hhtttttthh...",
    "..htttttttttth..",
    ".htttttttttttth.",
    ".htttTTTTTtttth.",
    "htttTTTTTTTtttth",
    "htttTTTTTTTtttth",
    "httTTTTTTTTTttth",
    ".httTTTTTTTTtth.",
    "..htTTTTTTTTth..",
    "...htTTkkTTth...",
    "....hTTkkTTh....",
    "......kkkk......",
    "......kKKk......",
    ".....kkKKkk.....",
    "....kkkKKkkk....",
];

const TREE_PINE: Cell = [
    ".......hh.......",
    "......hhhh......",
    ".....hhtthh.....",
    "....hhtttthh....",
    "....hTTTTTTh....",
    "...hhttttttthh..",
    "..hhttttttttthh.",
    "..hTTTTTTTTTTTh.",
    ".hhttttttttttthh",
    "hhttttttttttttth",
    "hTTTTTTTTTTTTTTh",
    "TTTTTTTkkTTTTTTT",
    "......kkkk......",
    "......kKKk......",
    ".....kkKKkk.....",
    "....kkkKKkkk....",
];

const FALLEN_LOG: Cell = [
    "................",
    "................",
    "................",
    "................",
    "................",
    "..kKKKKKKKKKKk..",
    ".kKkkkkkkkkkkKk.",
    ".KkkGkkkGkkkkkK.",
    ".KkkkGkkkkkGkkK.",
    ".kKkkkkkkkkkkKk.",
    "..kKKKKKKKKKKk..",
    "................",
    "................",
    "................",
    "................",
    "................",
];

const STUMP: Cell = [
    "................",
    "................",
    "................",
    ".....KKKKKK.....",
    "...KKkddddkKK...",
    "..KkddpddpddkK..",
    "..KkdpdDDdpdkK..",
    "..KkddDDDDddkK..",
    "..KkdpdDDdpdkK..",
    "..KkddpddpddkK..",
    "...KKkddddkKK...",
    "....kKKKKKKk....",
    "...kkKKKKKKkk...",
    "................",
    "................",
    "................",
];

const ROCK: Cell = [
    "................",
    "................",
    "......mcm.......",
    "....mccccccm....",
    "...mccccccccm...",
    "..mccccccccccm..",
    ".mccccCccCccccm.",
    ".mcccCCccCCcccm.",
    ".mcccCCCCCCCCcm.",
    ".mccCCCCCCCCCCm.",
    "..mCCCCCCCCCCm..",
    "...mCCCCCCCCm...",
    "....mMMMMMMm....",
    "......mMMm......",
    "................",
    "................",
];

const BUSH: Cell = [
    "................",
    ".....thht.......",
    "...thhttttht....",
    "..thhtvvttttht..",
    ".thttttvvttttht.",
    ".thtvvttttvvtht.",
    "thttttttttttttht",
    "thttvvttttvvthtt",
    "thttttvvttttthtt",
    "tTttvvttttvvttTt",
    ".tTttttvvttttTt.",
    "..tTTttttttTTt..",
    "...tTTTTTTTTt...",
    ".....tTTTTt.....",
    "................",
    "................",
];

const FENCE: Cell = [
    ".ww..........ww.",
    ".ww..........ww.",
    ".ww..WWWWWW..ww.",
    ".ww..wwwwww..ww.",
    ".ww..wwwwww..ww.",
    ".ww..WWWWWW..ww.",
    ".ww..........ww.",
    ".ww..WWWWWW..ww.",
    ".ww..wwwwww..ww.",
    ".ww..wwwwww..ww.",
    ".ww..WWWWWW..ww.",
    ".ww..........ww.",
    ".ww..........ww.",
    ".ww..........ww.",
    ".WW..........WW.",
    ".WW..........WW.",
];

const GATE: Cell = [
    "ww............ww",
    "ww............ww",
    "ww..WWWWWWWW..ww",
    "ww..wwwwwwww..ww",
    "ww...WwwwwW...ww",
    "ww....WwwW....ww",
    "ww.....WW.....ww",
    "ww.....WW.....ww",
    "ww....WwwW....ww",
    "ww...WwwwwW...ww",
    "ww..wwwwwwww..ww",
    "ww..WWWWWWWW..ww",
    "ww............ww",
    "ww............ww",
    "WW............WW",
    "WW............WW",
];

const SAND: Cell = [
    "ssssssssssssssss",
    "ssSsssssssssSsss",
    "ssssssssSsssssss",
    "sssspsssssssssss",
    "ssssssssssssSsss",
    "sssSssssssssssss",
    "ssssssssspssssss",
    "ssssssssssssssss",
    "ssSssssssssssSss",
    "ssssssSsssssssss",
    "ssssssssssssssss",
    "ssspsssssssspsss",
    "ssssssssSsssssss",
    "ssSsssssssssssss",
    "ssssssssssSsssss",
    "ssssssssssssssss",
];

const DOCK: Cell = [
    "WWWWWWWWWWWWWWWW",
    "wwwwwwwwwwwwwwww",
    "wwwwwwwwwwwwwwww",
    "WWWWWWWWWWWWWWWW",
    "wwwwwwwwwwwwwwww",
    "wwwwwwwwwwwwwwww",
    "WWWWWWWWWWWWWWWW",
    "wwwwwwwwwwwwwwww",
    "wwwwwwwwwwwwwwww",
    "WWWWWWWWWWWWWWWW",
    "wwwwwwwwwwwwwwww",
    "wwwwwwwwwwwwwwww",
    "WWWWWWWWWWWWWWWW",
    "wwwwwwwwwwwwwwww",
    "wwwwwwwwwwwwwwww",
    "WWWWWWWWWWWWWWWW",
];

const BED: Cell = [
    ".XXXXXXXXXXXXXX.",
    ".XkkkkkkkkkkkkX.",
    ".XkqqqqqqqqqqkX.",
    ".XkqqqqqqqqqqkX.",
    ".XkqqqqqqqqqqkX.",
    ".XkQQQQQQQQQQkX.",
    ".XkQQQQQQQQQQkX.",
    ".XkQQQQQQQQQQkX.",
    ".XkQQQQQQQQQQkX.",
    ".XkQQQQQQQQQQkX.",
    ".XkQQQQQQQQQQkX.",
    ".XkQQQQQQQQQQkX.",
    ".XkQQQQQQQQQQkX.",
    ".XkQQQQQQQQQQkX.",
    ".XkKKKKKKKKKKkX.",
    ".XXXXXXXXXXXXXX.",
];

const TABLE: Cell = [
    "..XXXXXXXXXXXX..",
    "..XkkkkkkkkkkX..",
    "..XkkkkkkkkkkX..",
    "..XkKkkkkkkKkX..",
    "..XkkkkkkkkkkX..",
    "..XkkkKkkKkkkX..",
    "..XkkkkkkkkkkX..",
    "..XkkkkkkkkkkX..",
    "..XkKkkkkkkKkX..",
    "..XkkkkkkkkkkX..",
    "..XkkkkkkkkkkX..",
    "..XKKKKKKKKKKX..",
    "..XX..XXXX..XX..",
    "..kK........kK..",
    "..kK........kK..",
    "..XX........XX..",
];

const CHAIR: Cell = [
    ".....XXXXXX.....",
    ".....XkkkkX.....",
    ".....XkKkKX.....",
    ".....XkkkkX.....",
    "....XXXXXXXX....",
    "....XkkkkkkX....",
    "....XkkkkkkX....",
    "....XkKKKKkX....",
    "....XXXXXXXX....",
    "....XX....XX....",
    "....kK....kK....",
    "....kK....kK....",
    "....kK....kK....",
    "....XX....XX....",
    "................",
    "................",
];

const SHELF: Cell = [
    "XXXXXXXXXXXXXXXX",
    "XkkkkkkkkkkkkkkX",
    "XkQQ..vv..mm..kX",
    "XkQQ..vv..mm..kX",
    "XkKKKKKKKKKKKKkX",
    "Xkqq..YY..QQ..kX",
    "Xkqq..YY..QQ..kX",
    "XkKKKKKKKKKKKKkX",
    "Xkvv..qq..mm..kX",
    "Xkvv..qq..mm..kX",
    "XkKKKKKKKKKKKKkX",
    "Xkmm..QQ..vv..kX",
    "Xkmm..QQ..vv..kX",
    "XkKKKKKKKKKKKKkX",
    "XkkkkkkkkkkkkkkX",
    "XXXXXXXXXXXXXXXX",
];

const COUNTER: Cell = [
    "XXXXXXXXXXXXXXXX",
    "XmmmmmmmmmmmmmmX",
    "XmmXXuuuuuuXXmmX",
    "XmmXuuuuuuuuXmmX",
    "XmmXuuuuuuuuXmmX",
    "XmmXXuuuuuuXXmmX",
    "XmmmmmmmmmmmmmmX",
    "XkKKKKKKKKKKKKkX",
    "XkkkkkkkkkkkkkkX",
    "XkKKKKKKKKKKKKkX",
    "XkkkkkkkkkkkkkkX",
    "XkKKKKKKKKKKKKkX",
    "XkkkkkkkkkkkkkkX",
    "XkKKKKKKKKKKKKkX",
    "XkKKKKKKKKKKKKkX",
    "XXXXXXXXXXXXXXXX",
];

const CRATE: Cell = [
    "XXXXXXXXXXXXXXXX",
    "XmkkkkkkkkkkkkmX",
    "XkMkkkkkkkkkkMkX",
    "XkkMkkkkkkkkMkkX",
    "XkkkMkkkkkkMkkkX",
    "XkkkkMkkkkMkkkkX",
    "XkkkkkMkkMkkkkkX",
    "XkkkkkkMMkkkkkkX",
    "XkkkkkkMMkkkkkkX",
    "XkkkkkMkkMkkkkkX",
    "XkkkkMkkkkMkkkkX",
    "XkkkMkkkkkkMkkkX",
    "XkkMkkkkkkkkMkkX",
    "XkMkkkkkkkkkkMkX",
    "XmkkkkkkkkkkkkmX",
    "XXXXXXXXXXXXXXXX",
];

const CAR: Cell = [
    "...XXXXXXXXXX...",
    "..XmmMMMMMMmmX..",
    ".XmmmmmmmmmmmmX.",
    ".XmmmmmmmmmmmmX.",
    ".XmmXuuuuuuXmmX.",
    ".XmXuuuuuuuuXmX.",
    ".XmXuuuuuuuuXmX.",
    ".XmmXXXXXXXXmmX.",
    ".XmmmmmmmmmmmmX.",
    ".XmmmmmmmmmmmmX.",
    ".XmXuuuuuuuuXmX.",
    ".XmmXuuuuuuXmmX.",
    ".XmmmmmmmmmmmmX.",
    ".XmmrrrrrrrrXmmX",
    "..XmmMMMMMMmmX..",
    "...XXXXXXXXXX...",
];

const TRUCK: Cell = [
    "..XXXXXXXXXXXX..",
    ".XmmMMMMMMMMmmX.",
    "XmmmmmmmmmmmmmmX",
    "XmmmmmmmmmmmmmmX",
    "XmmXuuuuuuuuXmmX",
    "XmXuuuuuuuuuuXmX",
    "XmmXXXXXXXXXXmmX",
    "XmmmmmmmmmmmmmmX",
    "XkKKKKKKKKKKKKkX",
    "XkddddddddddddkX",
    "XkddddddddddddkX",
    "XkddddddddddddkX",
    "XkddddddddddddkX",
    "XkKKKKKKKKKKKKkX",
    ".XmmMMMMMMMMmmX.",
    "..XXXXXXXXXXXX..",
];

const LAMP_POST: Cell = [
    "......XXXX......",
    ".....XEEEE......",
    "....XEEeeeeX....",
    "....XeeEEeeX....",
    "....XeeEEeeX....",
    ".....XEEEE......",
    "......XXXX......",
    ".......MM.......",
    ".......MM.......",
    ".......MM.......",
    ".......MM.......",
    ".......MM.......",
    ".......MM.......",
    ".......MM.......",
    "......MMMM......",
    ".....XXXXXX.....",
];

const CAMPFIRE: Cell = [
    "................",
    "......EEEE......",
    ".....EEeeEE.....",
    "....EEeeeeEE....",
    "....EeeeeeeE....",
    "...EEeeeeeeEE...",
    "...EeeeeeEEeeE..",
    "..mCeeeeeEEeecm.",
    ".mCceeeeeeeecCcm",
    ".mCcKkkkkkkKcCcm",
    ".mCCkKkEEkKkcCCm",
    "..mCCkkEEkkCCm..",
    "...mMMMMMMMMm...",
    ".....mMMMMm.....",
    "................",
    "................",
];

const BARRICADE: Cell = [
    ".X..X....X..X...",
    ".k..k....k..k...",
    ".k..k....k..k...",
    "XXXXXXXXXXXXXXX.",
    "XkkkkkkkkkkkkkX.",
    "XkKkkkkkkkkkkkX.",
    "XkKkkkkkkkkkkkX.",
    "XXXXXXXXXXXXXXX.",
    ".k..k....k..k...",
    "XXXXXXXXXXXXXXX.",
    "XkkkkkkkkkkkkkX.",
    "XkKkkkkkkkkkkkX.",
    "XkKkkkkkkkkkkkX.",
    "XXXXXXXXXXXXXXX.",
    ".k..k....k..k...",
    ".K..K....K..K...",
];

const PLAYER: Cell = [
    "......XXXX......",
    ".....XooooX.....",
    "....XooooooX....",
    "....XoXooXoX....",
    "....XooooooX....",
    ".....XooooX.....",
    "....XXXXXXXX....",
    "...XkMMkkMMkX...",
    "..XkMMkkkkMMkX..",
    "..XkkkkkkkkkkX..",
    "..XkkkkkkkkkkX..",
    "...XXXXXXXXXX...",
    "....XMMXXMMX....",
    "....XMMXXMMX....",
    "....XMMXXMMX....",
    "....XXXX.XXXX...",
];

const ZOMBIE: Cell = [
    "......XXXX......",
    ".....XzzzzX.....",
    "....XzzvvzzX....",
    "....XzXzzXzX....",
    "....XzzzzzzX....",
    ".....XzzzzX.....",
    "....XXXXXXXX....",
    "...XzZZzzZZzX...",
    "..XzZZzzzzZZzX..",
    "..XzzzzvvzzzzX..",
    "..XzzzzzzzzzzX..",
    "...XXXXXXXXXX...",
    "....XZZXXZZX....",
    "....XZZXXZZX....",
    "....XZZXXZZX....",
    "....XXXX.XXXX...",
];

const RUNNER: Cell = [
    "......XXXX......",
    ".....XvvvvX.....",
    "....XvvzzvvX....",
    "....XvXvvXvX....",
    "....XvvvvvvX....",
    ".....XvvvvX.....",
    "....XXXXXXXX....",
    "...XvZZvvZZvX...",
    "..XvZZvvvvZZvX..",
    "..XvvvvvvvvvvX..",
    "..XvvvvvvvvvvX..",
    "...XXXXXXXXXX...",
    "....XZZXXZZX....",
    "....XZZXXZZX....",
    "....XZZXXZZX....",
    "....XXXX.XXXX...",
];

const BRUTE: Cell = [
    "....XXXXXXXX....",
    "...XzzzzzzzzX...",
    "..XzzvvvvvvzzX..",
    "..XzzXzzzzXzzX..",
    "..XzzzzzzzzzzX..",
    "...XzzzzzzzzX...",
    "..XXXXXXXXXXXX..",
    "..XZZZZZZZZZZX..",
    ".XZZZZzzzzZZZZX.",
    ".XZZZzzvvzzZZZX.",
    ".XZZZZzzzzZZZZX.",
    "..XXXXXXXXXXXX..",
    "..XZZZX..XZZZX..",
    "..XZZZX..XZZZX..",
    "..XZZZX..XZZZX..",
    "..XXXXX..XXXXX..",
];

const LOOT_FOOD: Cell = [
    "................",
    "................",
    ".....XXXXXX.....",
    "....XrrrrrrX....",
    "...XrvvvvvvrX...",
    "...XrqqqqqqrX...",
    "...XrqqqqqqrX...",
    "...XrvvvvvvrX...",
    "...XrvvvvvvrX...",
    "...XrqqqqqqrX...",
    "...XrqqqqqqrX...",
    "...XrvvvvvvrX...",
    "...XrrrrrrrrX...",
    "....XmmmmmmX....",
    ".....XXXXXX.....",
    "................",
];

const LOOT_AMMO: Cell = [
    "................",
    "................",
    "....XXXXXXXX....",
    "...XGGGGGGGGX...",
    "..XGgYYYYYYgGX..",
    "..XGYYYYYYYYGX..",
    "..XGYYYYYYYYGX..",
    "..XGgYYYYYYgGX..",
    "..XGgYYYYYYgGX..",
    "..XGYYYYYYYYGX..",
    "..XGYYYYYYYYGX..",
    "..XGgYYYYYYgGX..",
    "...XGGGGGGGGX...",
    "....XXXXXXXX....",
    "................",
    "................",
];

const LOOT_WEAPON: Cell = [
    "................",
    "................",
    "................",
    "................",
    "................",
    ".mmmmmmmm.......",
    ".mMMMMMMmmmmmm..",
    "..mmXkkXmmmmmm..",
    "...XkkkkX...mm..",
    "...XkkkX........",
    "....XkX.........",
    ".....X..........",
    "................",
    "................",
    "................",
    "................",
];

const LOOT_MATERIAL: Cell = [
    "................",
    "................",
    "...XXXXXX.......",
    "..XkkkkkkX......",
    "..XkKKKKkX......",
    "...XXXXXX.XXXXXX",
    "........XkkkkkkX",
    "XXXXXX..XkKKKKkX",
    "XkkkkkkX.XXXXXX.",
    "XkKKKKkX........",
    ".XXXXXX.........",
    "................",
    "................",
    "................",
    "................",
    "................",
];

const FISHING_SPOT: Cell = [
    "................",
    "......bbbb......",
    "....bbBBBBbb....",
    "...bBBBBBBBBb...",
    "..bBBbssssbBBb..",
    "..bBBssssssBBb..",
    "..bBBssssssBBb..",
    "..bBBbssssbBBb..",
    "...bBBBBBBBBb...",
    "....bbBBBBbb....",
    "......bbbb......",
    "................",
    ".......WW.......",
    ".......ww.......",
    ".......ww.......",
    ".......WW.......",
];

const BULLET: Cell = [
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "......YYYY......",
    ".....YYYYYY.....",
    ".....YYYYYY.....",
    "......YYYY......",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
];

const SHOP: Cell = [
    "XXXXXXXXXXXXXXXX",
    "XrrrrrrrrrrrrrrX",
    "XqqqqqqqqqqqqqqX",
    "XrrrrrrrrrrrrrrX",
    "XwwwwwwwwwwwwwwX",
    "XwwXXuuuuuuXXwwX",
    "XwwXuuuuuuuuXwwX",
    "XwwXuuuXXuuuXwwX",
    "XwwXXXXXXXXXXwwX",
    "Xw...ffffff...wX",
    "Xw...ffffff...wX",
    "Xw...ffffff...wX",
    "Xw...ffffff...wX",
    "Xw...ffffff...wX",
    "XW...FFFFFF...WX",
    "XXXXXXXXXXXXXXXX",
];

const HOUSE: Cell = [
    "XXXXXXXXXXXXXXXX",
    "XrrrrrrrrrrrrrrX",
    "XrrrrrrrrrrrrrrX",
    "XRRRRRRRRRRRRRRX",
    "XwwwwwwwwwwwwwwX",
    "XwwXXuuuuuuXXwwX",
    "XwwXuuuuuuuuXwwX",
    "XwwXXuuuuuuXXwwX",
    "XwwwwwwwwwwwwwwX",
    "Xw...ffffff...wX",
    "Xw...ffffff...wX",
    "Xw...ffffff...wX",
    "Xw...ffffff...wX",
    "Xw...ffffff...wX",
    "XW...FFFFFF...WX",
    "XXXXXXXXXXXXXXXX",
];

/// Table of all 45 cells ordered by tile id.
const CELLS: [Cell; 45] = [
    GRASS,         // 0
    DIRT,          // 1
    WATER,         // 2
    ROAD,          // 3
    ROAD_PLAIN,    // 4
    SIDEWALK,      // 5
    FLOOR,         // 6
    WALL,          // 7
    WALL_INTERIOR, // 8
    DOOR,          // 9
    WINDOW,        // 10
    ROOF,          // 11
    TREE,          // 12
    TREE_PINE,     // 13
    FALLEN_LOG,    // 14
    STUMP,         // 15
    ROCK,          // 16
    BUSH,          // 17
    FENCE,         // 18
    GATE,          // 19
    SAND,          // 20
    DOCK,          // 21
    BED,           // 22
    TABLE,         // 23
    CHAIR,         // 24
    SHELF,         // 25
    COUNTER,       // 26
    CRATE,         // 27
    CAR,           // 28
    TRUCK,         // 29
    LAMP_POST,     // 30
    CAMPFIRE,      // 31
    BARRICADE,     // 32
    PLAYER,        // 33
    ZOMBIE,        // 34
    RUNNER,        // 35
    BRUTE,         // 36
    LOOT_FOOD,     // 37
    LOOT_AMMO,     // 38
    LOOT_WEAPON,   // 39
    LOOT_MATERIAL, // 40
    FISHING_SPOT,  // 41
    BULLET,        // 42
    SHOP,          // 43
    HOUSE,         // 44
];

/// Build the full sprite atlas into an `RgbaImage`.
pub fn build_atlas() -> RgbaImage {
    let atlas_w = ATLAS_COLS * TILE;
    let atlas_h = ATLAS_ROWS * TILE;
    let mut img = RgbaImage::from_pixel(atlas_w, atlas_h, Rgba([0, 0, 0, 0]));

    for (tile_id, cell) in CELLS.iter().enumerate() {
        let col = (tile_id as u32) % ATLAS_COLS;
        let row = (tile_id as u32) / ATLAS_COLS;
        let base_x = col * TILE;
        let base_y = row * TILE;

        for (py, line) in cell.iter().enumerate() {
            for (px, ch) in line.chars().enumerate() {
                if let Some(rgba) = parse_pixel(ch) {
                    img.put_pixel(base_x + px as u32, base_y + py as u32, rgba);
                }
            }
        }
    }

    img
}

/// UV rectangle `[u0, v0, u1, v1]` for `tile_id`.
pub fn tile_uv(tile_id: u32) -> [f32; 4] {
    let col = (tile_id % ATLAS_COLS) as f32;
    let row = (tile_id / ATLAS_COLS) as f32;
    let u0 = col / ATLAS_COLS as f32;
    let v0 = row / ATLAS_ROWS as f32;
    let u1 = (col + 1.0) / ATLAS_COLS as f32;
    let v1 = (row + 1.0) / ATLAS_ROWS as f32;
    [u0, v0, u1, v1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atlas_builds_correctly() {
        let img = build_atlas();
        assert_eq!(img.width(), ATLAS_COLS * TILE);
        assert_eq!(img.height(), ATLAS_ROWS * TILE);
    }

    #[test]
    fn cells_are_16x16() {
        for (i, cell) in CELLS.iter().enumerate() {
            assert_eq!(cell.len(), 16, "Cell {} row count is not 16", i);
            for (r, row) in cell.iter().enumerate() {
                assert_eq!(row.len(), 16, "Cell {} row {} len is not 16", i, r);
            }
        }
    }

    #[test]
    fn tile_uv_bounds() {
        let uv = tile_uv(id::GRASS);
        assert!(uv[0] >= 0.0 && uv[2] <= 1.0);
        assert!(uv[1] >= 0.0 && uv[3] <= 1.0);
    }
}
