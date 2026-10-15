//! Enemies: kinds, models, the hunting AI, spawning (they claw up out of the ground), deaths (they keel over
//! and shrink away) and loot. The horde always hunts the player; only camp and ruin guards hold position.
use crate::combat::{fire_enemy_proj, EProj};
use crate::fx::hexc;
use crate::game::*;
use crate::models::*;
use crate::physics::*;
use bevy::prelude::*;
use std::f32::consts::{PI, TAU};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind { Zombie, Skeleton, Slime, Spider, Cultist, Golem, Bat, Bloater, Brute, Urn, Pig, Sheep, Chicken,
    Spiderling, Gorehorn, Shieldbearer, Shaman, Imp, Necromancer, Wurm, FrostGiant, Colossus, Broodmother, EmberKing, Goblin }
pub const ALL_KINDS: [Kind; 25] = [Kind::Zombie, Kind::Skeleton, Kind::Slime, Kind::Spider, Kind::Cultist, Kind::Golem, Kind::Bat, Kind::Bloater, Kind::Brute, Kind::Urn, Kind::Pig, Kind::Sheep, Kind::Chicken,
    Kind::Spiderling, Kind::Gorehorn, Kind::Shieldbearer, Kind::Shaman, Kind::Imp, Kind::Necromancer, Kind::Wurm, Kind::FrostGiant, Kind::Colossus, Kind::Broodmother, Kind::EmberKing, Kind::Goblin];
pub struct EType { pub name: &'static str, pub hp: f32, pub dmg: f32, pub speed: f32, pub reach: f32, pub cd: f32, pub wind: f32, pub xp: f32, pub w: f32, pub h: f32,
    pub ranged: bool, pub range: f32, pub passive: bool, pub prop: bool, pub fly: bool, pub kb_res: f32, pub elite: bool, pub blood: &'static [u32] }
pub const fn et(name: &'static str, hp: f32, dmg: f32, speed: f32, xp: f32, w: f32, h: f32, blood: &'static [u32]) -> EType {
    EType { name, hp, dmg, speed, reach: 1.7, cd: 1.2, wind: 0.38, xp, w, h, ranged: false, range: 0.0, passive: false, prop: false, fly: false, kb_res: 0.0, elite: false, blood }
}
pub static ZOMBIE: EType = et("Zombie", 40.0, 9.0, 3.3, 3.0, 0.32, 1.85, &[0x5f9a4a, 0x3a6ea5, 0x2a4a2a]);
pub static SKELETON: EType = EType { ranged: true, range: 20.0, cd: 2.7, wind: 0.65, ..et("Skeleton", 30.0, 8.0, 3.5, 4.0, 0.28, 1.85, &[0xe8e4d8, 0xc8c2b0]) };
pub static SLIME: EType = et("Slime", 30.0, 7.0, 4.4, 2.0, 0.45, 0.9, &[0x6fd46a, 0x3fa83a]);
pub static SPIDER: EType = EType { reach: 1.6, cd: 1.0, wind: 0.25, ..et("Spider", 26.0, 8.0, 5.8, 3.0, 0.5, 0.8, &[0x2a2228, 0x6a2a2a]) };
pub static CULTIST: EType = EType { ranged: true, range: 16.0, cd: 3.4, wind: 0.8, ..et("Cultist", 40.0, 11.0, 3.1, 5.0, 0.32, 1.85, &[0x4a2a6a, 0x9a4aff]) };
pub static GOLEM: EType = EType { reach: 3.2, cd: 3.0, wind: 1.0, elite: true, kb_res: 1.0, ..et("Ancient Golem", 620.0, 26.0, 2.8, 90.0, 1.0, 3.7, &[0x7d8088, 0x5c7a42, 0x6ff3ff]) };
pub static BAT: EType = EType { fly: true, cd: 1.1, ..et("Bat", 12.0, 5.0, 7.4, 1.0, 0.35, 0.6, &[0x3a2a40, 0x8a2a3a]) };
pub static BLOATER: EType = EType { wind: 0.75, ..et("Bloater", 55.0, 22.0, 2.7, 4.0, 0.45, 1.9, &[0x9ab860, 0xc8ff5a, 0x5a7a32]) };
pub static BRUTE: EType = EType { reach: 2.8, cd: 2.3, wind: 0.7, kb_res: 0.7, ..et("Brute", 230.0, 20.0, 3.0, 16.0, 0.55, 2.9, &[0x9a7a62, 0x6a4a34, 0xb03a2a]) };
pub static URN: EType = EType { prop: true, ..et("Ancient Urn", 14.0, 0.0, 0.0, 0.0, 0.4, 1.1, &[0xb0683e, 0x7a4428, 0xe0b84a]) };
pub static PIG: EType = EType { passive: true, ..et("Pig", 20.0, 0.0, 2.6, 2.0, 0.4, 0.9, &[0xf0a0a8, 0xd88088]) };
pub static SHEEP: EType = EType { passive: true, ..et("Sheep", 22.0, 0.0, 2.4, 2.0, 0.42, 1.0, &[0xf2f0ea, 0xd0ccc4]) };
pub static CHICKEN: EType = EType { passive: true, ..et("Chicken", 8.0, 0.0, 3.0, 1.0, 0.22, 0.6, &[0xffffff, 0xf0e0c0]) };
pub fn etype(k: Kind) -> &'static EType {
    match k { Kind::Zombie => &ZOMBIE, Kind::Skeleton => &SKELETON, Kind::Slime => &SLIME, Kind::Spider => &SPIDER, Kind::Cultist => &CULTIST, Kind::Golem => &GOLEM, Kind::Bat => &BAT,
        Kind::Bloater => &BLOATER, Kind::Brute => &BRUTE, Kind::Urn => &URN, Kind::Pig => &PIG, Kind::Sheep => &SHEEP, Kind::Chicken => &CHICKEN,
        k => crate::horde::etype(k).unwrap() }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Affix { Swift, Brute, Splitter, Vampiric, Volatile }
pub const AFFIXES: [Affix; 5] = [Affix::Swift, Affix::Brute, Affix::Splitter, Affix::Vampiric, Affix::Volatile];
pub fn affix_info(a: Affix) -> (&'static str, u32) {
    match a { Affix::Swift => ("Swift", 0x7fd4ff), Affix::Brute => ("Brute", 0xff7a3a), Affix::Splitter => ("Splitting", 0x9be07a), Affix::Vampiric => ("Vampiric", 0xff3a5a), Affix::Volatile => ("Volatile", 0xffd35a) }
}

pub struct Enemy {
    pub id: u32, pub kind: Kind, pub t: &'static EType, pub body: Body, pub rig: Rig, pub yaw: f32, pub hp: f32, pub max_hp: f32, pub dmg: f32,
    pub atk_t: f32, pub wind: f32, pub slow: f32, pub frozen: f32, pub stun: f32, pub burn: f32, pub poison: f32, pub fear: f32, pub flee: f32, pub flash: f32, pub flashing: bool, pub hit_p: f32,
    pub ph: f32, pub rise: f32, pub elite: bool, pub affix: Option<Affix>, pub boss: bool, pub size: f32, pub color: u32, pub speed_mul: f32, pub sc: f32, pub poi: Option<(i32, i32)>,
    pub leap_t: f32, pub hop_t: f32, pub throw_t: f32, pub leaping: f32, pub swell: f32, pub squash: f32, pub smash_at: Vec3, pub chill: u32, pub dead: bool, pub alert: bool,
    pub wander_t: f32, pub wdir: Vec3, pub burn_acc: f32, pub ice: Option<Entity>, pub aiming: bool, pub was_air: bool, pub force: bool, pub ai: crate::horde::Ai,
}

pub struct Corpse { pub root: Entity, pub t: f32, pub sc: f32, pub side: f32, pub fly: bool, pub pos: Vec3, pub yaw: f32 }

/// build the model for a kind; returns the rig
pub fn enemy_model(b: &mut Builder, k: Kind, pos: Vec3, color: u32, size: f32, rng: &mut impl FnMut() -> f32) -> Rig {
    let v = Vec3::new;
    let mut pick = |a: &[u32]| a[((rng()) * a.len() as f32) as usize % a.len()];
    match k {
        Kind::Zombie => {
            let lk = Look { skin: pick(&[0x6aa050, 0x7aa860, 0x5e9448]), shirt: pick(&[0x3a6ea5, 0x7a4a3a, 0x4a5a3a]), pants: 0x3b3355, eye: 0x101010, eye_glow: if pick(&[0, 0, 1]) == 1 { Some(0xd8ff5a) } else { None }, ..default() };
            let r = humanoid(b, pos, &lk);
            b.bx(r.torso.unwrap(), v(0.2, 0.18, 0.04), 0x2a3a22, v(0.12, 0.18, 0.16)); b.bx(r.head.unwrap(), v(0.2, 0.12, 0.04), 0x4a7a38, v(-0.1, 0.42, 0.2));
            r
        }
        Kind::Skeleton => {
            let r = humanoid(b, pos, &Look { skin: 0xe8e4d8, shirt: 0xd8d4c6, pants: 0xd0ccbc, thin: true, eye_glow: Some(0xff5a3a), ..default() });
            for i in 0..3 { b.bx(r.torso.unwrap(), v(0.42, 0.04, 0.04), 0x8a8478, v(0.0, 0.5 - i as f32 * 0.13, 0.12)); }
            weapon_model(b, r.hand_l.unwrap(), "ebow");
            r
        }
        Kind::Cultist => {
            let r = humanoid(b, pos, &Look { skin: 0x2a1f33, shirt: 0x3a2350, pants: 0x241634, robe: Some(0x2e1c44), belt: Some(0x9a4aff), eye_glow: Some(0xd07aff), ..default() });
            let h = r.head.unwrap();
            b.bx(h, v(0.5, 0.14, 0.5), 0x2e1c44, v(0.0, 0.47, -0.02)); b.bx(h, v(0.5, 0.46, 0.1), 0x2e1c44, v(0.0, 0.24, -0.25)); b.bx(h, v(0.12, 0.12, 0.4), 0x2e1c44, v(0.0, 0.58, -0.18));
            b.glow(r.torso.unwrap(), v(0.1, 0.1, 0.02), 0xd07aff, v(0.0, 0.42, 0.16));
            r
        }
        Kind::Bloater => {
            let mut r = humanoid(b, pos, &Look { skin: 0x87a856, shirt: 0x5d7a3a, pants: 0x3a4a2a, eye: 0x10200a, ..default() });
            let belly = b.pivot(r.torso.unwrap(), v(0.0, 0.32, 0.05));
            b.bx(belly, v(0.86, 0.82, 0.7), 0x9ab860, Vec3::ZERO); b.glow(belly, v(0.5, 0.46, 0.06), 0xc8ff5a, v(0.0, -0.02, 0.35));
            for (x, y, z) in [(-0.34, 0.24, 0.3), (0.3, -0.22, 0.32), (0.38, 0.2, -0.1), (-0.3, -0.25, -0.2)] { b.bx(belly, v(0.16, 0.16, 0.16), 0x5a7a32, v(x, y, z)); }
            r.extra.push(belly);
            r
        }
        Kind::Brute => {
            let r = humanoid(b, pos, &Look { skin: 0x9a7a62, shirt: 0x6a4a34, pants: 0x3a2a22, boots: Some(0x2a1e18), belt: Some(0x2a1e18), eye_glow: Some(0xffb03a), ..default() });
            let (t, h) = (r.torso.unwrap(), r.head.unwrap());
            b.bx(t, v(0.66, 0.5, 0.42), 0x6a4a34, v(0.0, 0.48, 0.0));
            b.bx(r.arm_l.unwrap(), v(0.36, 0.22, 0.36), 0x5a5f66, v(0.0, 0.04, 0.0)); b.bx(r.arm_r.unwrap(), v(0.36, 0.22, 0.36), 0x5a5f66, v(0.0, 0.04, 0.0));
            let h1 = b.bx(h, v(0.06, 0.18, 0.06), 0xe8dcc0, v(-0.2, 0.5, 0.05)); b.rot(h1, 0.0, 0.0, 0.5);
            let h2 = b.bx(h, v(0.06, 0.18, 0.06), 0xe8dcc0, v(0.2, 0.5, 0.05)); b.rot(h2, 0.0, 0.0, -0.5);
            let club = b.pivot(r.hand_r.unwrap(), v(0.0, -0.05, 0.0)); b.rot(club, PI / 2.0, 0.0, 0.0);
            b.bx(club, v(0.12, 0.12, 0.9), 0x5a3a22, v(0.0, 0.0, 0.3)); b.bx(club, v(0.3, 0.3, 0.42), 0x5a3a22, v(0.0, 0.0, 0.82));
            for (x, y) in [(0.16, 0.0), (-0.16, 0.0), (0.0, 0.16), (0.0, -0.16)] { b.bx(club, v(0.08, 0.08, 0.08), 0x8a8f96, v(x, y, 0.86)); }
            r
        }
        Kind::Slime => {
            let root = b.root(pos); let body = b.pivot(root, Vec3::ZERO);
            b.bxk(body, v(size, size * 0.9, size), color, v(0.0, size * 0.45, 0.0), Mk::Clear);
            b.bx(body, v(size * 0.45, size * 0.4, size * 0.45), 0x1a3a1a, v(0.0, size * 0.4, 0.0));
            for x in [-0.2, 0.2] { b.bx(body, v(0.12 * size, 0.16 * size, 0.02), 0x10161a, v(x * size, size * 0.6, size * 0.5)); }
            b.bx(body, v(0.3 * size, 0.06 * size, 0.02), 0x10161a, v(0.0, size * 0.35, size * 0.5));
            Rig { root: Some(root), body: Some(body), ..default() }
        }
        Kind::Spider => {
            let root = b.root(pos); let body = b.pivot(root, v(0.0, 0.45, 0.0));
            b.bx(body, v(0.85, 0.5, 0.95), 0x2a2228, v(0.0, 0.1, -0.35));
            let head = b.pivot(body, v(0.0, 0.05, 0.3)); b.bx(head, v(0.55, 0.4, 0.45), 0x3a2e33, v(0.0, 0.0, 0.1));
            for (x, y) in [(-0.14, 0.06), (0.14, 0.06), (-0.07, 0.12), (0.07, 0.12)] { b.glow(head, v(0.07, 0.07, 0.02), 0xff3030, v(x, y, 0.335)); }
            let mut extra = vec![];
            for i in 0..8 { let side = if i < 4 { -1.0 } else { 1.0 }; let k = (i % 4) as f32;
                let lp = b.pivot(body, v(side * 0.35, 0.0, 0.25 - k * 0.22));
                let l = b.bx(lp, v(0.62, 0.07, 0.07), 0x3a2e33, v(side * 0.3, 0.05, 0.0)); b.rot(l, 0.0, 0.0, side * 0.5);
                b.bx(lp, v(0.07, 0.5, 0.07), 0x3a2e33, v(side * 0.6, -0.2, 0.0)); extra.push(lp); }
            Rig { root: Some(root), body: Some(body), head: Some(head), extra, ..default() }
        }
        Kind::Golem => {
            let root = b.root(pos); let body = b.pivot(root, Vec3::ZERO);
            let (st, dk, ms, gl) = (0x7d8088, 0x5c5f66, 0x5c7a42, 0x6ff3ff);
            let leg_l = b.pivot(body, v(-0.45, 1.3, 0.0)); let leg_r = b.pivot(body, v(0.45, 1.3, 0.0));
            b.bx(leg_l, v(0.62, 1.3, 0.66), dk, v(0.0, -0.65, 0.0)); b.bx(leg_r, v(0.62, 1.3, 0.66), dk, v(0.0, -0.65, 0.0));
            let torso = b.pivot(body, v(0.0, 1.3, 0.0));
            b.bx(torso, v(1.7, 1.3, 1.0), st, v(0.0, 0.7, 0.0)); b.bx(torso, v(1.2, 0.3, 0.9), ms, v(0.1, 1.42, -0.05)); b.glow(torso, v(0.3, 0.3, 0.05), gl, v(0.0, 0.75, 0.51));
            for (x, y) in [(-0.5, 1.1), (0.55, 0.4), (-0.3, 0.25)] { b.glow(torso, v(0.12, 0.12, 0.04), gl, v(x, y, 0.51)); }
            b.bx(torso, v(0.5, 0.35, 0.45), ms, v(-0.6, 1.3, 0.0)); b.bx(torso, v(0.3, 0.25, 0.3), dk, v(0.65, 1.5, -0.1));
            let head = b.pivot(torso, v(0.0, 1.35, 0.25)); b.bx(head, v(0.72, 0.6, 0.62), dk, v(0.0, 0.25, 0.0));
            b.glow(head, v(0.14, 0.08, 0.02), gl, v(-0.16, 0.3, 0.32)); b.glow(head, v(0.14, 0.08, 0.02), gl, v(0.16, 0.3, 0.32));
            let arm_l = b.pivot(torso, v(-1.05, 1.15, 0.0)); let arm_r = b.pivot(torso, v(1.05, 1.15, 0.0));
            for a in [arm_l, arm_r] { b.bx(a, v(0.52, 1.5, 0.56), st, v(0.0, -0.75, 0.0)); b.bx(a, v(0.66, 0.55, 0.66), dk, v(0.0, -1.55, 0.0)); }
            b.bx(arm_l, v(0.4, 0.2, 0.4), ms, v(0.0, -0.05, 0.0));
            Rig { root: Some(root), body: Some(body), torso: Some(torso), head: Some(head), arm_l: Some(arm_l), arm_r: Some(arm_r), leg_l: Some(leg_l), leg_r: Some(leg_r), ..default() }
        }
        Kind::Bat => {
            let root = b.root(pos); let body = b.pivot(root, v(0.0, 0.3, 0.0));
            b.bx(body, v(0.34, 0.3, 0.42), 0x3a2a40, Vec3::ZERO);
            let head = b.pivot(body, v(0.0, 0.1, 0.24)); b.bx(head, v(0.28, 0.24, 0.24), 0x3a2a40, v(0.0, 0.0, 0.04));
            b.bx(head, v(0.07, 0.16, 0.05), 0x7a4a6a, v(-0.09, 0.17, 0.0)); b.bx(head, v(0.07, 0.16, 0.05), 0x7a4a6a, v(0.09, 0.17, 0.0));
            b.glow(head, v(0.06, 0.05, 0.02), 0xff4a5a, v(-0.065, 0.03, 0.165)); b.glow(head, v(0.06, 0.05, 0.02), 0xff4a5a, v(0.065, 0.03, 0.165));
            let wl = b.pivot(body, v(-0.15, 0.06, 0.0)); let wr = b.pivot(body, v(0.15, 0.06, 0.0));
            for (wg, s) in [(wl, -1.0), (wr, 1.0)] { b.bx(wg, v(0.5, 0.04, 0.38), 0x5c3a62, v(s * 0.27, 0.0, 0.0)); b.bx(wg, v(0.36, 0.04, 0.26), 0x7a4a6a, v(s * 0.66, 0.0, -0.06)); b.bx(wg, v(0.06, 0.07, 0.5), 0x3a2a40, v(s * 0.5, 0.03, 0.04)); }
            Rig { root: Some(root), body: Some(body), head: Some(head), extra: vec![wl, wr], ..default() }
        }
        Kind::Urn => {
            let root = b.root(pos); let body = b.pivot(root, Vec3::ZERO);
            b.bx(body, v(0.5, 0.18, 0.5), 0x7a4428, v(0.0, 0.09, 0.0)); b.bx(body, v(0.72, 0.5, 0.72), 0xb0683e, v(0.0, 0.42, 0.0)); b.bx(body, v(0.76, 0.08, 0.76), 0xe0b84a, v(0.0, 0.56, 0.0));
            b.bx(body, v(0.5, 0.24, 0.5), 0xb0683e, v(0.0, 0.78, 0.0)); b.bx(body, v(0.36, 0.1, 0.36), 0x7a4428, v(0.0, 0.94, 0.0));
            let g = b.glow(body, v(0.14, 0.14, 0.14), 0xffd36a, v(0.0, 1.02, 0.0)); b.rot(g, 0.6, 0.6, 0.0);
            for s in [-1.0, 1.0] { b.bx(body, v(0.08, 0.26, 0.12), 0x7a4428, v(s * 0.4, 0.62, 0.0)); }
            Rig { root: Some(root), body: Some(body), ..default() }
        }
        Kind::Pig | Kind::Sheep | Kind::Chicken => {
            struct Q { body: u32, head: u32, snout: Option<u32>, leg: u32, bw: f32, bh: f32, bl: f32, hw: f32, hh: f32, hl: f32, leg_h: f32, lw: f32, comb: bool, two: bool }
            let o = match k {
                Kind::Pig => Q { body: 0xf0a0a8, head: 0xf2adb4, snout: Some(0xd88088), leg: 0xe090a0, bw: 0.7, bh: 0.55, bl: 1.0, hw: 0.5, hh: 0.45, hl: 0.42, leg_h: 0.35, lw: 0.18, comb: false, two: false },
                Kind::Sheep => Q { body: 0xf4f2ec, head: 0x6a625a, snout: None, leg: 0x5a524a, bw: 0.8, bh: 0.62, bl: 1.0, hw: 0.38, hh: 0.4, hl: 0.42, leg_h: 0.42, lw: 0.16, comb: false, two: false },
                _ => Q { body: 0xffffff, head: 0xffffff, snout: Some(0xf2b030), leg: 0xf2b030, bw: 0.34, bh: 0.32, bl: 0.42, hw: 0.22, hh: 0.26, hl: 0.22, leg_h: 0.25, lw: 0.06, comb: true, two: true },
            };
            let root = b.root(pos); let body = b.pivot(root, v(0.0, o.leg_h, 0.0));
            b.bx(body, v(o.bw, o.bh, o.bl), o.body, v(0.0, o.bh / 2.0, 0.0));
            let head = b.pivot(body, v(0.0, o.bh * 0.75, o.bl / 2.0)); b.bx(head, v(o.hw, o.hh, o.hl), o.head, v(0.0, 0.0, o.hl / 2.0 - 0.05));
            if let Some(s) = o.snout { b.bx(head, v(o.hw * 0.5, o.hh * 0.4, 0.1), s, v(0.0, -o.hh * 0.15, o.hl - 0.02)); }
            for x in [-0.3, 0.3] { b.bx(head, v(0.06, 0.06, 0.02), 0x111111, v(x * o.hw, o.hh * 0.15, o.hl - 0.04)); }
            if o.comb { b.bx(head, v(0.06, 0.12, 0.16), 0xd83a3a, v(0.0, o.hh / 2.0 + 0.05, o.hl * 0.5)); }
            let mut extra = vec![];
            for (x, z) in [(-1.0, 1.0), (1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)] { if o.two && z < 0.0 { continue; }
                let lp = b.pivot(root, v(x * o.bw * 0.3, o.leg_h, z * o.bl * 0.32 * if o.two { 0.0 } else { 1.0 })); b.bx(lp, v(o.lw, o.leg_h, o.lw), o.leg, v(0.0, -o.leg_h / 2.0, 0.0)); extra.push(lp); }
            Rig { root: Some(root), body: Some(body), head: Some(head), extra, ..default() }
        }
        _ => crate::horde::model(b, k, pos, rng),
    }
}

pub struct SpawnOpts { pub size: f32, pub color: u32, pub affix: Option<Affix>, pub boss: bool, pub poi: Option<(i32, i32)>, pub no_rise: bool, pub force: bool, pub id: Option<u32> }
impl Default for SpawnOpts { fn default() -> Self { SpawnOpts { size: 1.0, color: 0x6fd46a, affix: None, boss: false, poi: None, no_rise: false, force: false, id: None } } }

pub fn spawn_enemy(cx: &mut Ctx, k: Kind, pos: Vec3, o: SpawnOpts) -> Option<usize> {
    let live = cx.g.enemies.iter().filter(|e| !e.dead).count();
    if live >= 160 + if o.force { 24 } else { 0 } { return None; }
    let t = etype(k);
    let mi = cx.g.minutes();
    // health grows both linearly and by a compounding rate, so late-game builds have to keep scaling too
    let diff = (1.0 + mi * 0.08) * 1.045f32.powf(mi) * (1.0 + (cx.dc.day as f32 - 1.0) * 0.06) * cx.g.mods_hp();
    let quiet = t.passive || t.prop;
    let mut seed = cx.fx.rng.load(std::sync::atomic::Ordering::Relaxed);
    let mut rng = move || { seed ^= seed << 13; seed ^= seed >> 17; seed ^= seed << 5; (seed % 10000) as f32 / 10000.0 };
    let shadow = o.affix.is_some() || matches!(k, Kind::Brute | Kind::Golem) || live < 45;
    let rig = { let mut b = cx.builder(); b.shadow = shadow; let mut r = enemy_model(&mut b, k, pos, o.color, o.size, &mut rng); r.parts = std::mem::take(&mut b.parts); r };
    let _ = cx.fx.r();
    let mut sc = if k == Kind::Brute { 1.55 } else { crate::horde::scale(k) };
    let (mut w, mut h) = (t.w, t.h);
    if k == Kind::Slime { w = 0.45 * o.size; h = 0.9 * o.size; }
    let curse = if cx.g.flags.contains_key("curse") { 1.4 } else { 1.0 };
    let boss_hp = (1.0 + mi * 0.05) * cx.g.mods_hp() * if cx.g.ember() >= 5 { 1.4 } else { 1.0 } * (1.0 + *cx.g.flags.get("endless_n").unwrap_or(&0.0) * 0.5);
    let mut hp = t.hp * if quiet { 1.0 } else if o.boss { boss_hp } else { diff * curse } * if k == Kind::Slime { o.size } else { 1.0 } * if o.boss { 1.5 } else { 1.0 };
    let mut dmg = t.dmg * (0.75 + mi * 0.07) * 1.02f32.powf(mi) * (1.0 + (cx.dc.day as f32 - 1.0) * 0.05) * cx.g.mods_dmg();
    let mut speed_mul = if quiet { 1.0 } else { (1.0 + (mi * 0.02).min(0.3)) * if cx.g.ember() >= 9 { 1.15 } else { 1.0 } };
    if let Some(a) = o.affix { hp *= 2.6; dmg *= 1.3; if a == Affix::Swift { speed_mul *= 1.5; } if a == Affix::Brute { hp *= 1.5; } let s = if a == Affix::Brute { 1.4 } else { 1.2 }; sc *= s; w *= s; h *= s; }
    if let Some(r) = rig.root { cx.c.entity(r).insert(Transform::from_translation(pos).with_scale(Vec3::splat(sc))); }
    let id = match o.id { Some(id) => id, None => { let id = cx.g.next_id; cx.g.next_id += 1; id } };
    let rise = if o.no_rise || quiet || t.fly || crate::horde::flies(k) || k == Kind::Wurm { 0.0 } else { 0.7 };
    if rise > 0.0 { cx.gfx().debris(pos + Vec3::Y * 0.1, &[0x6a4a2c, 0x8a5b34, 0x4a3a2a], if o.affix.is_some() { 10 } else { 4 }, 3.0, 0.18, 5.0); }
    let yaw = cx.rr(0.0, TAU); let ph = cx.rr(0.0, 6.0); let atk = cx.rr(0.5, 1.5);
    let mut body = Body::new(pos, w, h); body.fly = t.fly || crate::horde::flies(k);
    let mut ai = crate::horde::Ai::default(); if k == Kind::Wurm { ai.hidden = true; ai.sink = 1.0; ai.cd = 1.5; }
    cx.g.enemies.push(Enemy { id, kind: k, t, body, rig, yaw, hp, max_hp: hp, dmg, atk_t: atk, wind: 0.0, slow: 0.0, frozen: 0.0, stun: 0.0, burn: 0.0, poison: 0.0, fear: 0.0, flee: 0.0,
        flash: 0.0, flashing: false, hit_p: 0.0, ph, rise, elite: t.elite || o.affix.is_some(), affix: o.affix, boss: o.boss, size: o.size, color: o.color, speed_mul, sc, poi: o.poi,
        leap_t: 2.0, hop_t: 0.0, throw_t: 4.0, leaping: 0.0, swell: 0.0, squash: 0.0, smash_at: pos, chill: 0, dead: false, alert: false, wander_t: 0.0, wdir: Vec3::ZERO, burn_acc: 0.0,
        ice: None, aiming: false, was_air: false, force: o.force, ai });
    Some(cx.g.enemies.len() - 1)
}

fn despawn_rig(cx: &mut Ctx, e: &Enemy) { if let Some(r) = e.rig.root { cx.c.entity(r).despawn_recursive(); } if let Some(i) = e.ice { cx.c.entity(i).despawn_recursive(); } }

pub fn kill_enemy(cx: &mut Ctx, i: usize, no_xp: bool, self_det: bool) {
    if cx.g.enemies[i].dead { return; }
    cx.g.enemies[i].dead = true;
    let (pos, h, kind, elite, affix, t, size, color, sc, yaw, fly, boss) = { let e = &cx.g.enemies[i]; (e.body.pos, e.body.h, e.kind, e.elite, e.affix, e.t, e.size, e.color, e.sc, e.yaw, e.t.fly, e.boss) };
    let c = pos + Vec3::Y * h * 0.5;
    // the death burst
    cx.gfx().debris(c, t.blood, if elite { 40 } else { 9 }, if elite { 8.0 } else { 5.0 }, if elite { 0.35 } else { 0.2 }, 6.0);
    cx.fx.smoke(c, 0x4a4448, if elite { 12 } else { 3 }, 1.2, 0.9, if elite { 1.6 } else { 0.8 });
    cx.fx.burst(c, 0xffffff, 8, 4.0, 0.3, 0.3, 0.0, 0.0);
    cx.fx.sparks(c, t.blood[0], 6, 7.0);
    if !t.passive { cx.play(if t.prop { "glass" } else { "die" }); }
    if elite { cx.shake(0.6); let col = affix.map(|a| affix_info(a).1).unwrap_or(0x6ff3ff); let mut g = cx.gfx(); g.ring(pos, col, 0.5, 5.0, 0.5); g.flare(c, col, 5.0, 0.5, true); }
    if !t.passive && !t.prop { cx.g.kills += 1; if !cx.g.net.client() { crate::augments::on_kill(cx, c); } }
    if !cx.g.net.client() {
    let xp = if no_xp { 0.0 } else { (t.xp * (1.0 + cx.g.minutes() * 0.05) * if affix.is_some() { 4.0 } else { 1.0 } * if kind == Kind::Slime { size } else { 1.0 }).round() };
    if xp > 0.0 && !cx.p.dead { cx.drop_gems(c, xp); }
    if t.passive { let n = if kind == Kind::Chicken { 1 } else { 2 } + if cx.g.st.food > 1.0 { 1 } else { 0 }; for _ in 0..n { cx.drop_pick(c, PickKind::Meat, 0.0); } }
    else if t.prop { urn_loot(cx, c); }
    else { if cx.r() < 0.04 { let k = if cx.r() < 0.5 { PickKind::Meat } else { PickKind::Berries }; cx.drop_pick(c, k, 0.0); } if !no_xp { rare_drop(cx, c, elite); } }
    if kind == Kind::Slime && size > 1.1 { for _ in 0..2 { let p = pos + Vec3::new(cx.rr(-0.5, 0.5), 0.3, cx.rr(-0.5, 0.5)); if let Some(j) = spawn_enemy(cx, Kind::Slime, p, SpawnOpts { size: size * 0.55, color, no_rise: true, force: true, ..default() }) { let v = Vec3::new(cx.rr(-4.0, 4.0), 6.0, cx.rr(-4.0, 4.0)); cx.g.enemies[j].body.vel = v; cx.g.enemies[j].alert = true; } } }
    if affix == Some(Affix::Splitter) { for _ in 0..3 { let p = pos + Vec3::new(cx.rr(-1.0, 1.0), 0.3, cx.rr(-1.0, 1.0)); let k2 = if kind == Kind::Golem { Kind::Zombie } else { kind }; if let Some(j) = spawn_enemy(cx, k2, p, SpawnOpts { size: 0.8, color, no_rise: true, force: true, ..default() }) { let v = Vec3::new(cx.rr(-5.0, 5.0), 6.0, cx.rr(-5.0, 5.0)); cx.g.enemies[j].body.vel = v; } } }
    if affix == Some(Affix::Volatile) {
        crate::net::tele(cx, pos, 0xffd35a, 4.0, 1.0, 0);
        cx.later(1.0, move |cx| { cx.play("boom"); let mut g = cx.gfx(); g.ring(pos, 0xffd35a, 0.5, 4.0, 0.4); g.flare(pos + Vec3::Y, 0xffd35a, 5.0, 0.4, true); g.decal(pos, false, 3.0, 6.0, None); cx.fx.burst(pos + Vec3::Y * 0.6, 0xffd35a, 60, 10.0, 0.6, 0.6, 2.0, 0.0); cx.shake(0.3);
            crate::net::slam(cx, pos, 4.0, 22.0, 8.0); });
    }
    if kind == Kind::Bloater { bloater_pop(cx, i, self_det); }
    crate::horde::on_death(cx, kind, pos);
    if elite { let (nm, _) = affix.map(affix_info).unwrap_or((t.name, 0)); let label = if affix.is_some() { format!("{} {} slain", nm, t.name) } else { format!("{} slain", t.name) }; cx.toast(&label, &format!("+{} experience", xp), 2.5); crate::director::drop_chest(cx, pos, false); }
    if boss { cx.g.boss = None; *cx.g.flags.entry("bosses").or_insert(0.0) += 1.0; crate::director::stage_boss_down(cx, kind, pos); for _ in 0..8 { let g = (5.0 + cx.g.minutes()).round(); cx.drop_pick(c, PickKind::Gold, g); } }
    if cx.g.net.host() { let id = cx.g.enemies[i].id; let killer = cx.g.net.last_hitter.remove(&id).unwrap_or(cx.g.net.my_id); let sx = if t.passive || t.prop { 0.0 } else { xp }; cx.g.net.send(crate::net::Msg::Die { id, killer, xp: sx }); }
    } else if boss { *cx.g.flags.entry("bosses").or_insert(0.0) += 1.0; }
    let e = std::mem::replace(&mut cx.g.enemies[i].rig, Rig::default());
    if let Some(ice) = cx.g.enemies[i].ice.take() { cx.c.entity(ice).despawn_recursive(); cx.gfx().debris(c, &[0xbff4ff, 0x8fe6ff, 0xffffff], 12, 6.0, 0.2, 6.0); }
    if t.prop || kind == Kind::Bloater { if let Some(r) = e.root { cx.c.entity(r).despawn_recursive(); } }
    else if let Some(r) = e.root {
        if cx.g.corpses.len() >= 40 { let o = cx.g.corpses.remove(0); cx.c.entity(o.root).despawn_recursive(); }
        let side = if cx.r() < 0.5 { -1.0 } else { 1.0 };
        cx.g.corpses.push(Corpse { root: r, t: 0.0, sc, side, fly, pos, yaw });
    }
}
/// a bloater bursts in a cloud of bile: it always hurts nearby monsters (chain reactions), and hurts the player only when it detonates on its own
fn bloater_pop(cx: &mut Ctx, i: usize, self_det: bool) {
    let p = cx.g.enemies[i].body.pos; let dmg = cx.g.enemies[i].dmg; let r = 3.8;
    cx.play("squelch");
    cx.fx.burst(p + Vec3::Y * 0.8, 0xb8ff4a, 50, r * 2.4, 0.6, 0.6, 2.0, 6.0);
    cx.fx.smoke(p, 0x6a8a3a, 10, 2.2, 1.2, 1.4);
    { let mut g = cx.gfx(); g.ring(p, 0x9aff4a, 0.5, r, 0.4); g.shock(p, 0x9aff4a, 0.4, r, 0.35, 1.4); g.sphere(p + Vec3::Y * 0.8, 0x9aff4a, 0.5, r, 0.4); g.debris(p + Vec3::Y * 0.5, &[0x9ab860, 0x5a7a32, 0xc8ff5a], 14, 6.0, 0.22, 6.0); g.decal(p, false, r * 0.8, 8.0, Some(0x6aa83a)); }
    if Vec3::distance(cx.p.body.pos, p) < 20.0 { cx.shake(0.25); }
    let dm = (30.0 * (1.0 + cx.g.minutes() * 0.08)).round();
    for (j, _) in cx.near(p, r) { if j == i || cx.g.enemies[j].t.passive || cx.g.enemies[j].t.prop { continue; } cx.apply_damage(j, dm, Hit { kb: 7.0, up: 5.0, src: Some(p), ..default() }); }
    if self_det { crate::net::slam(cx, p, r, dmg, 9.0); }
}

pub fn update_corpses(cx: &mut Ctx) {
    let dt = cx.dt;
    let mut i = 0;
    while i < cx.g.corpses.len() {
        let c = &mut cx.g.corpses[i];
        c.t += dt;
        let k = (c.t / 0.45).min(1.0);
        if c.fly { c.pos.y -= dt * 6.0; }
        let s = c.sc * if k < 0.6 { 1.0 } else { 1.0 - (k - 0.6) / 0.4 * 0.95 };
        let (root, pos, yaw, side, done) = (c.root, c.pos, c.yaw, c.side, c.t >= 0.45);
        if let Ok(mut t) = cx.q.get_mut(root) { t.translation = pos; t.rotation = Quat::from_rotation_y(yaw) * Quat::from_euler(EulerRot::XYZ, -k * 0.3, 0.0, side * k * 1.3); t.scale = Vec3::new(s, s * (1.0 - k * 0.3), s); }
        if done { cx.c.entity(root).despawn_recursive(); cx.g.corpses.swap_remove(i); } else { i += 1; }
    }
}

fn set_rot(q: &mut Query<&mut Transform>, e: Option<Entity>, x: f32, y: f32, z: f32) { if let Some(e) = e { if let Ok(mut t) = q.get_mut(e) { t.rotation = Quat::from_euler(EulerRot::XYZ, x, y, z); } } }

fn animate(cx: &mut Ctx, i: usize, moving: bool) {
    let dt = cx.dt;
    let e = &mut cx.g.enemies[i];
    e.ph += dt * if moving { 9.0 } else { 2.0 };
    let s = if moving { e.ph.sin() } else { 0.0 };
    let r = &e.rig;
    let q = &mut *cx.q;
    match e.kind {
        Kind::Zombie | Kind::Skeleton | Kind::Cultist | Kind::Bloater | Kind::Brute => {
            set_rot(q, r.leg_l, s * 0.7, 0.0, 0.0); set_rot(q, r.leg_r, -s * 0.7, 0.0, 0.0);
            let c = if moving { e.ph.cos() } else { 0.0 };
            set_rot(q, r.knee_l, (-c).max(0.0) * 0.9 + 0.05, 0.0, 0.0); set_rot(q, r.knee_r, c.max(0.0) * 0.9 + 0.05, 0.0, 0.0);
            set_rot(q, r.elbow_l, -0.25, 0.0, 0.0); set_rot(q, r.elbow_r, -0.25, 0.0, 0.0);
            let (mut al, mut ar, mut tx, mut tz, mut hz) = (Vec3::new(-s * 0.5, 0.0, 0.0), Vec3::new(s * 0.5, 0.0, 0.0), 0.0, s * 0.05, 0.0);
            match e.kind {
                Kind::Zombie => { al = Vec3::new(-PI / 2.0 + (e.ph * 0.5).sin() * 0.15, 0.0, 0.0); ar = Vec3::new(-PI / 2.0 - (e.ph * 0.5).sin() * 0.15, 0.0, 0.0); hz = (e.ph * 0.3).sin() * 0.15;
                    if e.wind > 0.0 { al.x = -2.6; ar.x = -2.6; tx = -0.2; } else { tx = 0.15; } }
                Kind::Skeleton => { if e.aiming { al = Vec3::new(-PI / 2.0, 0.0, 0.0); ar = Vec3::new(-PI / 2.0, -0.5, 0.0); } }
                Kind::Cultist => { if e.wind > 0.0 { al = Vec3::new(-2.7, 0.0, 0.4); ar = Vec3::new(-2.7, 0.0, -0.4); } }
                Kind::Bloater => { al = Vec3::new(-s * 0.3 - 0.3, 0.0, 0.5); ar = Vec3::new(s * 0.3 - 0.3, 0.0, -0.5); tx = -0.1;
                    let sw = 1.0 + e.swell * 0.55 + (e.ph * 0.7).sin() * 0.04; if let Some(b) = r.extra.first() { if let Ok(mut t) = q.get_mut(*b) { t.scale = Vec3::splat(sw); } } }
                Kind::Brute => { ar = Vec3::new(if e.wind > 0.0 { -2.9 + (1.0 - e.wind / e.t.wind) * 0.4 } else { s * 0.5 - 0.3 }, 0.0, 0.0); tx = if e.wind > 0.0 { -0.3 } else { 0.15 }; }
                _ => {}
            }
            set_rot(q, r.arm_l, al.x, al.y, al.z); set_rot(q, r.arm_r, ar.x, ar.y, ar.z); set_rot(q, r.torso, tx, 0.0, tz); set_rot(q, r.head, 0.0, 0.0, hz);
            if e.kind == Kind::Brute { if let Some(b) = r.body { if let Ok(mut t) = q.get_mut(b) { t.translation.y = s.abs() * 0.06; } } }
        }
        Kind::Golem => {
            set_rot(q, r.leg_l, s * 0.5, 0.0, 0.0); set_rot(q, r.leg_r, -s * 0.5, 0.0, 0.0);
            let a = if e.wind > 0.0 { -2.8 } else { -s * 0.4 }; set_rot(q, r.arm_l, a, 0.0, 0.0); set_rot(q, r.arm_r, if e.wind > 0.0 { -2.8 } else { s * 0.4 }, 0.0, 0.0);
            if let Some(b) = r.body { if let Ok(mut t) = q.get_mut(b) { t.translation.y = s.abs() * 0.08; } }
        }
        Kind::Bat => { let f = (e.ph * 2.6).sin(); if r.extra.len() == 2 { set_rot(q, Some(r.extra[0]), 0.0, 0.0, f * 0.9); set_rot(q, Some(r.extra[1]), 0.0, 0.0, -f * 0.9); }
            if let Some(b) = r.body { if let Ok(mut t) = q.get_mut(b) { t.translation.y = 0.3 + f * 0.06; t.rotation = Quat::from_rotation_x(0.2); } } }
        Kind::Spider => { for (k, l) in r.extra.iter().enumerate() { set_rot(q, Some(*l), 0.0, (e.ph * 1.5 + k as f32).sin() * 0.4 * if moving { 1.0 } else { 0.2 }, 0.0); } }
        Kind::Slime => { let sq = if e.body.on_ground { 1.0 - e.squash.max(0.0) } else { 1.0 }; if let Some(b) = r.body { if let Ok(mut t) = q.get_mut(b) { t.scale = Vec3::new(1.0 + (1.0 - sq) * 0.5, sq * if e.body.on_ground { 1.0 } else { 1.15 }, 1.0 + (1.0 - sq) * 0.5); } } e.squash = (e.squash - dt * 3.0).max(0.0); }
        Kind::Urn => { if let Some(b) = r.body { if let Ok(mut t) = q.get_mut(b) { t.rotation = Quat::from_rotation_y(e.ph * 0.15); } } }
        Kind::Pig | Kind::Sheep | Kind::Chicken => {
            for (k, l) in r.extra.iter().enumerate() { set_rot(q, Some(*l), (e.ph + if k % 2 == 1 { PI } else { 0.0 } + if k > 1 { PI } else { 0.0 }).sin() * 0.6 * if moving { 1.0 } else { 0.0 }, 0.0, 0.0); }
            if !moving { set_rot(q, r.head, (e.ph * 0.4).sin().max(0.0) * 0.5, 0.0, 0.0); }
        }
        _ => crate::horde::animate(e, q, moving),
    }
}

pub fn visual(cx: &mut Ctx, i: usize, moving: bool) {
    let dt = cx.dt;
    animate(cx, i, moving);
    let rnd = cx.r();
    let (r0, r1, r2) = (cx.r(), cx.r(), cx.r());
    let e = &mut cx.g.enemies[i];
    let mut pos = e.body.pos;
    if e.rise > 0.0 { pos.y -= crate::noise::smooth(0.0, 0.7, e.rise) * (e.body.h + 0.1); }
    pos.y -= e.ai.sink * (e.body.h + 0.6);
    let hp = if e.hit_p > 0.0 { e.hit_p / 0.14 } else { 0.0 };
    let (root, yaw, sc, ice, frozen) = (e.rig.root, e.yaw, e.sc, e.ice, e.frozen > 0.0);
    if let Some(r) = root { if let Ok(mut t) = cx.q.get_mut(r) {
        let cur = t.rotation.to_euler(EulerRot::YXZ).0;
        let dy = ((yaw - cur) % TAU + TAU * 1.5) % TAU - PI;
        t.translation = pos; t.rotation = Quat::from_rotation_y(cur + dy * (dt * 10.0).min(1.0));
        t.scale = Vec3::new(sc * (1.0 + hp * 0.18), sc * (1.0 - hp * 0.16), sc * (1.0 + hp * 0.18));
    } }
    if let Some(ic) = ice { if let Ok(mut t) = cx.q.get_mut(ic) { t.translation = e.body.pos + Vec3::Y * e.body.h * 0.5; } }
    if ice.is_some() && !frozen { let ic = e.ice.take().unwrap(); let p = e.body.pos + Vec3::Y * e.body.h * 0.5; cx.c.entity(ic).despawn_recursive(); cx.gfx().debris(p, &[0xbff4ff, 0x8fe6ff, 0xffffff], 8, 4.0, 0.16, 4.0); cx.play("ice"); }
    // hit flash: swap every part to a white-hot material for a moment
    let e = &mut cx.g.enemies[i];
    let want = e.flash > 0.0;
    if want != e.flashing {
        e.flashing = want;
        let parts = e.rig.parts.clone();
        for (pe, col, k) in parts { let m = if want { cx.pal.mat(cx.mats, 0xffffff, Mk::Flash) } else { cx.pal.mat(cx.mats, col, k) }; cx.c.entity(pe).insert(MeshMaterial3d(m)); }
    }
    let e = &cx.g.enemies[i];
    let (pos, w, h) = (e.body.pos, e.body.w, e.body.h);
    let near = pos.distance(cx.p.body.pos) < 50.0;
    if near && e.burn > 0.0 && rnd < 0.7 { let c = hexc(if e.poison > 0.0 { 0x7aff5a } else if r2 < 0.5 { 0xff6a1a } else { 0xffb03a }) * 2.5; cx.fx.emit(true, pos + Vec3::new((r0 - 0.5) * 2.0 * w, r1 * h, (r2 - 0.5) * 2.0 * w), Vec3::Y * 2.0, 0.4, 0.4, 0.05, c, -1.0, 0.0, 1.0); }
    let e = &cx.g.enemies[i];
    if near && e.elite && rnd < 0.4 { let col = e.affix.map(|a| affix_info(a).1).unwrap_or(0x6ff3ff); cx.fx.emit(true, pos + Vec3::new((r0 - 0.5) * 2.0, 0.5 + r1 * h, (r2 - 0.5) * 2.0), Vec3::Y, 0.6, 0.3, 0.05, hexc(col) * 2.0, 0.0, 0.0, 0.8); }
    let e = &cx.g.enemies[i];
    if near && e.stun > 0.0 && rnd < 0.25 { let a = e.ph * 4.0; cx.fx.emit(true, pos + Vec3::new(a.cos() * 0.4, h + 0.25, a.sin() * 0.4), Vec3::ZERO, 0.3, 0.25, 0.05, hexc(0xffe48a) * 2.5, 0.0, 0.0, 1.0); }
    let e = &cx.g.enemies[i];
    if near && e.kind == Kind::Bloater && rnd < 0.15 { cx.fx.emit(false, pos + Vec3::new((r0 - 0.5) * 0.6, 1.3, (r1 - 0.5) * 0.6), Vec3::Y * 0.6, 1.0, 0.3, 0.6, hexc(0x8ab84a), -0.3, 0.5, 0.5); }
    let e = &cx.g.enemies[i];
    if e.rise > 0.0 && rnd < 0.5 { cx.fx.emit(false, pos + Vec3::new((r0 - 0.5), 0.1, (r1 - 0.5)), Vec3::new((r2 - 0.5) * 2.0, 1.8, (r0 - 0.5) * 2.0), 0.5, 0.25, 0.4, hexc(0x7a5a3a), 8.0, 1.0, 0.9); }
}

/// relocate a straggler that fell far behind to the edge of the fight so the horde keeps pressing in
fn recycle(cx: &mut Ctx, i: usize) -> bool {
    let a = cx.p.body.pos; let yaw = cx.p.yaw;
    let Some(s) = crate::director::ring_spot(cx, a, yaw, 26.0, 34.0) else { return false };
    let e = &mut cx.g.enemies[i];
    e.body.pos = s; e.body.vel = Vec3::ZERO; e.rise = if e.t.fly { 0.0 } else { 0.7 }; e.wind = 0.0; e.leaping = 0.0;
    true
}

pub fn update_enemies(cx: &mut Ctx) {
    let dt = cx.dt;
    let pp = cx.p.body.pos;
    let pdead = cx.p.dead || !cx.p.spawned;
    let remotes = crate::net::targets(cx.g);
    let n = cx.g.enemies.len();
    for i in 0..n {
        if cx.g.enemies[i].dead { continue; }
        { let e = &mut cx.g.enemies[i]; e.flash -= dt; if e.hit_p > 0.0 { e.hit_p = (e.hit_p - dt).max(0.0); } }
        let (epos, t, poi, boss, fly) = { let e = &cx.g.enemies[i]; (e.body.pos, e.t, e.poi, e.boss, e.t.fly) };
        let near = remotes.iter().map(|r| Vec2::new(r.1.x - epos.x, r.1.z - epos.z).length()).fold(Vec2::new(pp.x - epos.x, pp.z - epos.z).length(), f32::min);
        if t.prop { if near > 90.0 { despawn(cx, i); continue; } let mut b = cx.g.enemies[i].body.clone(); move_body(cx.w, &mut b, dt, 26.0); cx.g.enemies[i].body = b; visual(cx, i, false); continue; }
        if t.passive && near > 96.0 { despawn(cx, i); continue; }
        if poi.is_some() && near > 120.0 { despawn(cx, i); continue; }
        if !t.passive && poi.is_none() && near > if boss { 95.0 } else { 60.0 } && !recycle(cx, i) && near > 130.0 { despawn(cx, i); continue; }
        if cx.g.enemies[i].rise > 0.0 {
            let e = &mut cx.g.enemies[i]; e.rise -= dt; e.body.vel = Vec3::new(0.0, e.body.vel.y.min(0.0), 0.0);
            if !fly { let mut b = e.body.clone(); move_body(cx.w, &mut b, dt, 26.0); cx.g.enemies[i].body = b; }
            visual(cx, i, false); continue;
        }
        // hunt the nearest living player (teammates included), and let decoys draw aggro
        let mut tp = pp; let mut target_decoy = false; let mut tgt_remote: Option<u32> = None;
        { let mut best = if pdead { f32::MAX } else { Vec2::new(pp.x - epos.x, pp.z - epos.z).length() }; for &(rid, rpos) in remotes.iter() { let d = Vec2::new(rpos.x - epos.x, rpos.z - epos.z).length(); if d < best { best = d; tp = rpos; tgt_remote = Some(rid); } } }
        if let Some(d) = cx.g.rel.decoys.iter().filter(|d| d.t > 0.0).min_by(|a, b| a.pos.distance(epos).partial_cmp(&b.pos.distance(epos)).unwrap()) {
            if d.pos.distance(epos) * 0.5 < near || pdead { tp = d.pos; target_decoy = true; } }
        let has_tgt = !pdead || target_decoy || tgt_remote.is_some();
        let (dx, dz) = (tp.x - epos.x, tp.z - epos.z);
        let dist = Vec2::new(dx, dz).length().max(0.001);
        let dy = tp.y - epos.y;
        let mi = cx.g.minutes();
        {
            let e = &mut cx.g.enemies[i];
            e.slow -= dt; e.stun -= dt; e.fear -= dt; e.atk_t -= dt; e.leap_t -= dt; e.hop_t -= dt; e.throw_t -= dt; e.ai.haste -= dt;
            if e.frozen > 0.0 { e.frozen -= dt; }
        }
        let (burn, poison) = (cx.g.enemies[i].burn, cx.g.enemies[i].poison);
        if burn > 0.0 || poison > 0.0 {
            let e = &mut cx.g.enemies[i]; e.burn -= dt; e.poison -= dt; e.burn_acc += dt;
            if e.burn_acc > 0.5 { e.burn_acc = 0.0; let bd = ((if poison > 0.0 { 3.0 } else { 4.0 }) * (1.0 + mi * 0.06)).round(); let p = e.body.pos + Vec3::Y * e.body.h;
                cx.apply_damage(i, bd, Hit::default()); if cx.r() < 0.5 { cx.pop(p, format!("{}", bd as i32), 0); } if cx.g.enemies[i].dead { continue; } }
        }
        { let e = &mut cx.g.enemies[i]; if e.affix == Some(Affix::Vampiric) && e.hp < e.max_hp { e.hp = (e.hp + dt * e.max_hp * 0.01).min(e.max_hp); } }
        let (frozen, stun, fear, slow, kind, alert) = { let e = &cx.g.enemies[i]; (e.frozen, e.stun, e.fear, e.slow, e.kind, e.alert) };
        let mut speed = t.speed * cx.g.enemies[i].speed_mul * if slow > 0.0 { 0.55 } else { 1.0 } * if cx.g.enemies[i].ai.haste > 0.0 { 1.35 } else { 1.0 };
        let can_move = frozen <= 0.0 && stun <= 0.0 && has_tgt;
        let aggro = !t.passive && has_tgt && (poi.is_none() || dist < 26.0 || alert);
        let (mut mvx, mut mvz) = (0.0f32, 0.0f32);
        let (tdx, tdz) = (dx / dist, dz / dist);
        let mut hurt: Option<(f32, f32)> = None; // (damage, knockback) to the target
        if t.passive {
            let e = &mut cx.g.enemies[i];
            e.wander_t -= dt;
            if e.flee > 0.0 && !pdead { e.flee -= dt; mvx = -tdx; mvz = -tdz; speed *= 2.1; }
            else { if e.wander_t <= 0.0 { e.wander_t = 2.0 + cx.fx.r() * 4.0; let a = cx.fx.r() * TAU; e.wdir = if cx.fx.r() < 0.4 { Vec3::ZERO } else { Vec3::new(a.cos(), 0.0, a.sin()) }; } mvx = e.wdir.x; mvz = e.wdir.z; }
        } else if aggro && can_move {
            if fear > 0.0 { mvx = -tdx; mvz = -tdz; }
            else if t.ranged {
                let want = if kind == Kind::Skeleton { 10.0 } else { 9.0 };
                if dist > want + 3.0 { mvx = tdx; mvz = tdz; } else if dist < want - 4.0 { mvx = -tdx; mvz = -tdz; } else { let sg = (cx.g.enemies[i].ph * 0.2).sin().signum(); mvx = -tdz * 0.6 * sg; mvz = tdx * 0.6 * sg; }
                let e = &mut cx.g.enemies[i]; e.aiming = dist < t.range + 4.0;
                if e.wind > 0.0 { e.wind -= dt; mvx *= 0.2; mvz *= 0.2;
                    if kind == Kind::Cultist { let hp = e.body.pos + Vec3::Y * 2.2; let c = hexc(0xd07aff) * 2.5; let j = Vec3::new(cx.fx.r() - 0.5, 0.0, cx.fx.r() - 0.5) * 0.8; cx.fx.emit(true, hp + j, Vec3::Y, 0.4, 0.35, 0.05, c, 0.0, 0.0, 1.0); }
                    if cx.g.enemies[i].wind <= 0.0 { enemy_shoot(cx, i, tp); } }
                else if e.atk_t <= 0.0 && dist < t.range { e.wind = t.wind; e.atk_t = t.cd * (0.8 + cx.fx.r() * 0.4) / e.speed_mul; }
            } else if t.fly {
                mvx = tdx; mvz = tdz;
                let e = &mut cx.g.enemies[i];
                let ty = tp.y + 1.1 + (e.ph * 0.9).sin() * 0.5;
                e.body.vel.y = crate::noise::lerp(e.body.vel.y, ((ty - e.body.pos.y) * 3.0).clamp(-6.0, 6.0), (dt * 4.0).min(1.0));
                if dist < 0.9 + e.body.w && (tp.y + 1.0 - e.body.pos.y).abs() < 1.5 && e.atk_t <= 0.0 { e.atk_t = t.cd; hurt = Some((e.dmg, 3.0)); e.body.vel.x -= tdx * 9.0; e.body.vel.z -= tdz * 9.0; e.body.vel.y = 4.0; }
            } else {
                match kind {
                    k if crate::horde::etype(k).is_some() => {
                        let (a, b, h2, sm) = crate::horde::ai(cx, i, tp, dist, tdx, tdz, dy);
                        mvx = a; mvz = b; if h2.is_some() { hurt = h2; } speed *= sm;
                        if cx.g.enemies[i].dead { continue; }
                    }
                    Kind::Slime => { mvx = tdx; mvz = tdz; let e = &mut cx.g.enemies[i]; if dist < e.body.w + 0.7 && dy.abs() < 1.5 && e.atk_t <= 0.0 { e.atk_t = 0.9; hurt = Some((e.dmg * e.size, 5.0)); } }
                    Kind::Bloater => {
                        mvx = tdx; mvz = tdz; if dist < 9.0 { speed *= 1.7; }
                        let e = &mut cx.g.enemies[i];
                        if e.wind > 0.0 { e.wind -= dt; mvx = 0.0; mvz = 0.0; e.swell = 1.0 - e.wind / t.wind; if e.wind <= 0.0 { kill_enemy(cx, i, true, true); continue; } }
                        else if dist < 2.3 && dy.abs() < 2.0 { e.wind = t.wind; let p = e.body.pos; crate::net::tele(cx, p, 0x9aff4a, 3.8, t.wind, 4); cx.play("swell"); }
                    }
                    Kind::Brute => {
                        mvx = tdx; mvz = tdz;
                        let e = &mut cx.g.enemies[i];
                        if e.wind > 0.0 { e.wind -= dt; mvx = 0.0; mvz = 0.0; if e.wind <= 0.0 { brute_smash(cx, i); } }
                        else if dist < t.reach + 0.6 && e.atk_t <= 0.0 && dy.abs() < 2.5 { e.wind = t.wind; e.atk_t = t.cd / e.speed_mul; e.smash_at = e.body.pos + Vec3::new(tdx, 0.0, tdz) * 1.8; let p = e.smash_at; crate::net::tele(cx, p, 0xff7a3a, 2.8, t.wind, 1); }
                    }
                    Kind::Golem => {
                        mvx = tdx; mvz = tdz;
                        let e = &mut cx.g.enemies[i];
                        if e.wind > 0.0 { e.wind -= dt; mvx = 0.0; mvz = 0.0; if e.wind <= 0.0 { golem_slam(cx, i); } }
                        else if dist < t.reach + 1.5 && e.atk_t <= 0.0 { e.wind = t.wind; e.atk_t = t.cd; let p = e.body.pos; crate::net::tele(cx, p, 0xff6a3a, 5.2, t.wind, 1); }
                        else if dist > 10.0 && e.throw_t <= 0.0 && dist < 34.0 { e.throw_t = 4.0 + cx.fx.r() * 2.0; golem_throw(cx, i, tp); }
                    }
                    _ => {
                        mvx = tdx; mvz = tdz;
                        let e = &mut cx.g.enemies[i];
                        if kind == Kind::Spider && dist < 7.0 && dist > 2.2 && e.leap_t <= 0.0 && e.body.on_ground { e.leap_t = 2.5 + cx.fx.r() * 1.5; e.body.vel = Vec3::new(tdx * 13.0, 7.5, tdz * 13.0); e.leaping = 0.8; }
                        if e.leaping > 0.0 { e.leaping -= dt; if dist < 1.4 && dy.abs() < 1.6 { e.leaping = 0.0; hurt = Some((e.dmg, 5.0)); } }
                        if e.wind > 0.0 { e.wind -= dt; mvx *= 0.25; mvz *= 0.25;
                            if e.wind <= 0.0 { if dist < t.reach + 0.7 && dy.abs() < 2.0 { hurt = Some((e.dmg, 5.0)); } let p = e.body.pos + Vec3::Y; let yaw = dx.atan2(dz); cx.gfx().swoosh(p, yaw, t.reach + 0.5, 1.6, 0xff6a5a, 0.18, 0.0, 1.0); } }
                        else if dist < t.reach + 0.3 && e.atk_t <= 0.0 && dy.abs() < 2.0 { e.wind = t.wind; e.atk_t = t.cd / e.speed_mul; }
                    }
                }
            }
            cx.g.enemies[i].yaw = dx.atan2(dz);
        } else if !t.passive && poi.is_some() {
            let e = &mut cx.g.enemies[i]; e.wander_t -= dt;
            if e.wander_t <= 0.0 { e.wander_t = 2.0 + cx.fx.r() * 3.0; let a = cx.fx.r() * TAU; e.wdir = if cx.fx.r() < 0.5 { Vec3::ZERO } else { Vec3::new(a.cos(), 0.0, a.sin()) }; }
            mvx = e.wdir.x * 0.5; mvz = e.wdir.z * 0.5;
        }
        if let Some((d, kb)) = hurt {
            if target_decoy { if let Some(dd) = cx.g.rel.decoys.iter_mut().min_by(|a, b| a.pos.distance(epos).partial_cmp(&b.pos.distance(epos)).unwrap()) { dd.hp -= d; } }
            else if let Some(rid) = tgt_remote { cx.g.net.out.push((Some(rid), crate::net::Msg::Hurt { to: rid, dmg: d, src: epos.to_array(), kb })); }
            else { cx.hurt_player(d, Some(epos), kb);
                if cx.g.flags.remove("parried").is_some() { let e = &mut cx.g.enemies[i]; e.stun = e.stun.max(if e.boss { 0.5 } else { 1.5 }); if !e.boss { e.body.vel += Vec3::new(-tdx, 0.4, -tdz) * 9.0; } }
                if cx.g.enemies[i].affix == Some(Affix::Vampiric) { let e = &mut cx.g.enemies[i]; e.hp = (e.hp + d * 1.5).min(e.max_hp); } }
        }
        let e = &mut cx.g.enemies[i];
        if e.frozen > 0.0 || e.stun > 0.0 { mvx = 0.0; mvz = 0.0; if fly { e.body.vel.y = (e.body.vel.y - dt * 12.0).max(-6.0); } }
        if (mvx != 0.0 || mvz != 0.0) && (t.passive || !aggro || e.fear > 0.0) { e.yaw = mvx.atan2(mvz); }
        let moving = mvx != 0.0 || mvz != 0.0;
        if kind == Kind::Slime {
            if e.body.on_ground { let k = 1.0 - (dt * 8.0).min(1.0); e.body.vel.x *= k; e.body.vel.z *= k;
                if e.hop_t <= 0.0 && moving { e.hop_t = 0.7 + cx.fx.r() * 0.4; e.body.vel = Vec3::new(mvx * speed * 1.6, 7.5, mvz * speed * 1.6); e.squash = 0.4; } }
        } else if e.leaping <= 0.0 {
            let acc = if e.body.on_ground || fly { 10.0 } else { 2.0 };
            e.body.vel.x = crate::noise::lerp(e.body.vel.x, mvx * speed, (acc * dt).min(1.0));
            e.body.vel.z = crate::noise::lerp(e.body.vel.z, mvz * speed, (acc * dt).min(1.0));
        }
        if e.body.hit_wall && moving { if fly { e.body.vel.y = 7.0; } else if e.body.on_ground && kind != Kind::Slime { e.body.vel.y = 8.6; } }
        if e.body.in_water && !e.body.on_ground && !fly { e.body.vel.y = e.body.vel.y.max(2.0); }
        let mut b = e.body.clone();
        move_body(cx.w, &mut b, dt, 26.0);
        let e = &mut cx.g.enemies[i];
        if kind == Kind::Slime && b.on_ground && e.was_air { e.squash = 0.45; }
        e.was_air = !b.on_ground;
        e.body = b;
        visual(cx, i, moving && (cx.g.enemies[i].body.on_ground || fly));
    }
    // keep the horde from stacking into one blob
    let n = cx.g.enemies.len();
    if n < 240 { for a in 0..n { if cx.g.enemies[a].dead || cx.g.enemies[a].t.prop { continue; } for b in a + 1..n {
        if cx.g.enemies[b].dead || cx.g.enemies[b].t.prop { continue; }
        let (pa, pb) = (cx.g.enemies[a].body.pos, cx.g.enemies[b].body.pos);
        let r = cx.g.enemies[a].body.w + cx.g.enemies[b].body.w + 0.1;
        let (dx, dz) = (pb.x - pa.x, pb.z - pa.z);
        if dx.abs() > r || dz.abs() > r || (pa.y - pb.y).abs() > 1.5 { continue; }
        let d = (dx * dx + dz * dz).sqrt();
        if d < r && d > 0.001 { let push = (r - d) * 0.5; let (nx, nz) = (dx / d * push, dz / d * push);
            let (wa, ha, wb, hb) = (cx.g.enemies[a].body.w, cx.g.enemies[a].body.h, cx.g.enemies[b].body.w, cx.g.enemies[b].body.h);
            if !collides(cx.w, pa.x - nx, pa.y, pa.z - nz, wa, ha) { cx.g.enemies[a].body.pos.x -= nx; cx.g.enemies[a].body.pos.z -= nz; }
            if !collides(cx.w, pb.x + nx, pb.y, pb.z + nz, wb, hb) { cx.g.enemies[b].body.pos.x += nx; cx.g.enemies[b].body.pos.z += nz; } }
    } } }
    // sweep the dead
    let mut k = 0;
    while k < cx.g.enemies.len() { if cx.g.enemies[k].dead { let e = cx.g.enemies.swap_remove(k); if e.rig.root.is_some() { despawn_rig(cx, &e); } } else { k += 1; } }
}
fn despawn(cx: &mut Ctx, i: usize) {
    let e = &mut cx.g.enemies[i];
    e.dead = true;
    if let Some(p) = e.poi { cx.g.dir.guarded.remove(&p); }
    let r = std::mem::take(&mut cx.g.enemies[i].rig);
    if let Some(root) = r.root { cx.c.entity(root).despawn_recursive(); }
    if let Some(ic) = cx.g.enemies[i].ice.take() { cx.c.entity(ic).despawn_recursive(); }
}
pub fn rand01(s: &mut u32) -> f32 { *s ^= *s << 13; *s ^= *s >> 17; *s ^= *s << 5; (*s % 1_000_000) as f32 / 1_000_000.0 }

fn enemy_shoot(cx: &mut Ctx, i: usize, tp: Vec3) {
    let (pos, kind, dmg) = { let e = &cx.g.enemies[i]; (e.body.pos, e.kind, e.dmg) };
    let from = pos + Vec3::Y * 1.5;
    let mut tpos = tp + Vec3::Y;
    if kind == Kind::Skeleton {
        cx.play("bow");
        let sp = 30.0; let t = from.distance(tpos) / sp;
        tpos.x += cx.p.body.vel.x * t * 0.7; tpos.z += cx.p.body.vel.z * t * 0.7;
        let mut v = (tpos - from).normalize() * sp; v.y += 0.5 * 9.0 * t;
        fire_enemy_proj(cx, EProj::Arrow, from, v, dmg);
    } else {
        cx.play("arcane");
        let v = (tpos - from).normalize() * 10.0;
        fire_enemy_proj(cx, EProj::Dark, from, v, dmg);
    }
}
fn golem_slam(cx: &mut Ctx, i: usize) {
    let (c, dmg) = (cx.g.enemies[i].body.pos, cx.g.enemies[i].dmg);
    crate::net::slam(cx, c, 5.5, dmg, 12.0);
}
fn brute_smash(cx: &mut Ctx, i: usize) {
    let (mut c, dmg) = (cx.g.enemies[i].smash_at, cx.g.enemies[i].dmg);
    c.y = cx.ground(c.x, c.z, cx.g.enemies[i].body.pos.y + 1.0);
    crate::net::slam(cx, c, 2.9, dmg, 10.0);
}
pub fn slam_fx(cx: &mut Ctx, c: Vec3, r: f32) {
    cx.play("boom"); cx.shake(if r > 4.0 { 0.6 } else { 0.3 });
    let mut g = cx.gfx(); g.ring(c, 0xffa070, 0.5, r, 0.5); g.shock(c, 0xffa070, 0.5, r, 0.4, 1.6); g.decal(c, true, r * 0.8, 6.0, None);
    g.debris(c + Vec3::Y * 0.3, &[0x7d8088, 0x5c5f66, 0x8a5b34], if r > 4.0 { 20 } else { 10 }, 8.0, 0.4, 9.0);
    cx.fx.smoke(c, 0x8a8478, if r > 4.0 { 14 } else { 7 }, 3.0, 1.2, 1.4);
}
fn golem_throw(cx: &mut Ctx, i: usize, tp: Vec3) {
    let (pos, dmg) = (cx.g.enemies[i].body.pos, cx.g.enemies[i].dmg);
    let from = pos + Vec3::Y * 4.0; let to = tp + Vec3::Y * 0.5;
    let sp = 22.0; let t = from.distance(to) / sp;
    let mut v = (to - from).normalize() * sp; v.y += 0.5 * 14.0 * t;
    cx.play("heavy");
    fire_enemy_proj(cx, EProj::Rock, from, v, dmg * 0.8);
}
