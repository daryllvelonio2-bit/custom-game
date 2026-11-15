//! Entities: zombies, bullets, loot, vehicles and dropped resources.

use crate::art::id;
use crate::world::LootKind;

/// What an entity is.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Zombie,
    Bullet,
    Loot,
    Car,
    DroppedWood,
}

/// Zombie variants with different behaviour.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ZombieKind {
    Walker,
    Runner,
    Brute,
}

impl ZombieKind {
    pub fn tile(self) -> u32 {
        match self {
            ZombieKind::Walker => id::ZOMBIE,
            ZombieKind::Runner => id::RUNNER,
            ZombieKind::Brute => id::BRUTE,
        }
    }

    pub fn max_hp(self) -> i32 {
        match self {
            ZombieKind::Walker => 40,
            ZombieKind::Runner => 26,
            ZombieKind::Brute => 150,
        }
    }

    pub fn speed(self) -> f32 {
        match self {
            ZombieKind::Walker => 2.6,
            ZombieKind::Runner => 5.4,
            ZombieKind::Brute => 2.0,
        }
    }

    pub fn damage(self) -> i32 {
        match self {
            ZombieKind::Walker => 8,
            ZombieKind::Runner => 6,
            ZombieKind::Brute => 22,
        }
    }

    pub fn size(self) -> f32 {
        match self {
            ZombieKind::Walker => 1.0,
            ZombieKind::Runner => 0.9,
            ZombieKind::Brute => 1.5,
        }
    }

    pub fn aggro_range(self) -> f32 {
        match self {
            ZombieKind::Walker => 7.0,
            ZombieKind::Runner => 11.0,
            ZombieKind::Brute => 6.0,
        }
    }
}

/// A world entity.
#[derive(Clone, Debug)]
pub struct Entity {
    pub kind: Kind,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub size: f32,
    pub hp: i32,
    pub max_hp: i32,
    pub damage: i32,
    pub alive: bool,
    /// Seconds remaining before the entity disappears.
    pub ttl: f32,
    /// Zombie variant.
    pub zombie_kind: Option<ZombieKind>,
    /// Zombie is actively chasing.
    pub aggro: bool,
    /// Loot contents.
    pub loot: Option<LootKind>,
    /// Vehicle integrity, 0..100.
    pub integrity: f32,
    /// Vehicle heading in radians.
    pub angle: f32,
}

impl Entity {
    pub fn zombie(x: f32, y: f32, kind: ZombieKind) -> Self {
        let hp = kind.max_hp();
        Self {
            kind: Kind::Zombie,
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            size: kind.size(),
            hp,
            max_hp: hp,
            damage: kind.damage(),
            alive: true,
            ttl: 0.0,
            zombie_kind: Some(kind),
            aggro: false,
            loot: None,
            integrity: 100.0,
            angle: 0.0,
        }
    }

    pub fn bullet(x: f32, y: f32, vx: f32, vy: f32, damage: i32) -> Self {
        Self {
            kind: Kind::Bullet,
            x,
            y,
            vx,
            vy,
            size: 0.4,
            hp: 1,
            max_hp: 1,
            damage,
            alive: true,
            ttl: 0.9,
            zombie_kind: None,
            aggro: false,
            loot: None,
            integrity: 100.0,
            angle: 0.0,
        }
    }

    pub fn loot(x: f32, y: f32, loot: LootKind) -> Self {
        Self {
            kind: Kind::Loot,
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            size: 0.7,
            hp: 1,
            max_hp: 1,
            damage: 0,
            alive: true,
            ttl: 0.0,
            zombie_kind: None,
            aggro: false,
            loot: Some(loot),
            integrity: 100.0,
            angle: 0.0,
        }
    }

    pub fn car(x: f32, y: f32, angle: f32) -> Self {
        Self {
            kind: Kind::Car,
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            size: 2.4,
            hp: 100,
            max_hp: 100,
            damage: 0,
            alive: true,
            ttl: 0.0,
            zombie_kind: None,
            aggro: false,
            loot: None,
            integrity: 100.0,
            angle,
        }
    }

    pub fn dropped_wood(x: f32, y: f32) -> Self {
        Self {
            kind: Kind::DroppedWood,
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            size: 0.7,
            hp: 1,
            max_hp: 1,
            damage: 0,
            alive: true,
            ttl: 0.0,
            zombie_kind: None,
            aggro: false,
            loot: None,
            integrity: 100.0,
            angle: 0.0,
        }
    }

    pub fn tile(&self) -> u32 {
        match self.kind {
            Kind::Zombie => self.zombie_kind.unwrap_or(ZombieKind::Walker).tile(),
            Kind::Bullet => id::BULLET,
            Kind::Loot => match self.loot {
                Some(LootKind::Food) => id::LOOT_FOOD,
                Some(LootKind::Ammo) => id::LOOT_AMMO,
                Some(LootKind::Weapon) => id::LOOT_WEAPON,
                Some(LootKind::Material) => id::LOOT_MATERIAL,
                None => id::LOOT_MATERIAL,
            },
            Kind::Car => id::CAR,
            Kind::DroppedWood => id::LOOT_MATERIAL,
        }
    }
}

/// Axis-aligned rectangle overlap test.
pub fn rects_overlap(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zombie_stats_differ_by_variant() {
        let walker = Entity::zombie(0.0, 0.0, ZombieKind::Walker);
        let brute = Entity::zombie(0.0, 0.0, ZombieKind::Brute);
        assert!(brute.max_hp > walker.max_hp);
        assert!(brute.damage > walker.damage);
        assert!(ZombieKind::Runner.speed() > ZombieKind::Walker.speed());
    }

    #[test]
    fn overlap_detects_touching_rects() {
        assert!(rects_overlap(0.0, 0.0, 1.0, 1.0, 0.5, 0.5, 1.0, 1.0));
        assert!(!rects_overlap(0.0, 0.0, 1.0, 1.0, 2.0, 2.0, 1.0, 1.0));
    }

    #[test]
    fn loot_tiles_differ() {
        let food = Entity::loot(0.0, 0.0, LootKind::Food);
        let ammo = Entity::loot(0.0, 0.0, LootKind::Ammo);
        assert_ne!(food.tile(), ammo.tile());
    }
}
