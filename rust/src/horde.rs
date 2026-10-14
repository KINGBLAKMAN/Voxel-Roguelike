//! The extended bestiary: monsters with a job (a charging beast, a shield wall, a healer, a bomber, a summoner),
//! biome natives (desert sand wurms that hunt underground, snowbound frost giants), and the bosses that give a run
//! its shape: the Ancient Golem at 5:00, the Bone Colossus at 10:00, the Broodmother at 15:00 and the Ember King,
//! whose fall at 20:00 wins the run.
use crate::combat::*;
use crate::enemies::*;
use crate::fx::hexc;
use crate::game::*;
use crate::models::*;
use bevy::prelude::*;
use std::f32::consts::{PI, TAU};

/// per-enemy state machine for the monsters that need one
#[derive(Default, Clone, Copy)]
pub struct Ai { pub st: u8, pub t: f32, pub cd: f32, pub cd2: f32, pub cd3: f32, pub v: Vec3, pub phase: u8, pub hidden: bool, pub sink: f32, pub haste: f32 }

pub static SPIDERLING: EType = EType { reach: 1.3, cd: 0.9, wind: 0.2, ..et("Spiderling", 10.0, 5.0, 7.0, 1.0, 0.3, 0.45, &[0x2a2228, 0x6a2a2a]) };
pub static GOREHORN: EType = EType { kb_res: 0.5, reach: 2.0, cd: 1.4, ..et("Gorehorn", 110.0, 16.0, 3.4, 9.0, 0.6, 1.7, &[0x5a3a2a, 0x8a2a2a]) };
pub static SHIELDBEARER: EType = EType { kb_res: 0.6, reach: 2.0, cd: 1.5, wind: 0.5, ..et("Shieldbearer", 90.0, 12.0, 2.7, 8.0, 0.42, 2.0, &[0xe8e4d8, 0x6a6e76]) };
pub static SHAMAN: EType = EType { ..et("Bone Shaman", 45.0, 6.0, 3.3, 9.0, 0.3, 1.6, &[0x6a8a4a, 0xaaff5a]) };
pub static IMP: EType = EType { ..et("Ember Imp", 26.0, 12.0, 5.2, 5.0, 0.4, 0.9, &[0xb03a2a, 0xffa040]) };
pub static NECRO: EType = EType { ..et("Necromancer", 60.0, 8.0, 2.8, 11.0, 0.32, 1.9, &[0x2a2034, 0x9a5aff]) };
pub static WURM: EType = EType { kb_res: 1.0, reach: 2.4, ..et("Sand Wurm", 170.0, 20.0, 7.0, 14.0, 0.75, 4.0, &[0xc8a868, 0x8a6a40]) };
pub static FROST_GIANT: EType = EType { kb_res: 0.85, reach: 3.2, cd: 2.6, wind: 1.0, ..et("Frost Giant", 340.0, 24.0, 2.7, 24.0, 0.8, 3.6, &[0x8ab8d8, 0xbff4ff]) };
pub static COLOSSUS: EType = EType { kb_res: 1.0, elite: true, ..et("Bone Colossus", 2600.0, 18.0, 2.5, 160.0, 1.2, 5.6, &[0xe8e4d8, 0xc8c2b0, 0x9a5aff]) };
pub static BROOD: EType = EType { kb_res: 1.0, elite: true, reach: 3.0, cd: 1.3, ..et("Broodmother", 3600.0, 20.0, 3.7, 200.0, 1.5, 2.2, &[0x2a1a24, 0xff3a3a, 0x6a2a2a]) };
pub static KING: EType = EType { kb_res: 1.0, elite: true, reach: 5.0, ..et("The Ember King", 9000.0, 30.0, 3.1, 400.0, 1.2, 5.0, &[0x2a1a14, 0xffa040, 0xff5a1a]) };

pub fn etype(k: Kind) -> Option<&'static EType> {
    Some(match k { Kind::Spiderling => &SPIDERLING, Kind::Gorehorn => &GOREHORN, Kind::Shieldbearer => &SHIELDBEARER, Kind::Shaman => &SHAMAN, Kind::Imp => &IMP,
        Kind::Necromancer => &NECRO, Kind::Wurm => &WURM, Kind::FrostGiant => &FROST_GIANT, Kind::Colossus => &COLOSSUS, Kind::Broodmother => &BROOD, Kind::EmberKing => &KING, _ => return None })
}
pub fn scale(k: Kind) -> f32 { match k { Kind::Spiderling => 0.5, Kind::Shaman => 0.85, Kind::FrostGiant => 1.9, Kind::Colossus => 3.0, Kind::Broodmother => 2.6, Kind::EmberKing => 2.7, Kind::Shieldbearer => 1.08, _ => 1.0 } }
pub fn flies(k: Kind) -> bool { k == Kind::Imp }
pub fn kind_by_name(s: &str) -> Option<Kind> {
    Some(match s { "zombie" => Kind::Zombie, "skeleton" => Kind::Skeleton, "spider" => Kind::Spider, "golem" => Kind::Golem, "spiderling" => Kind::Spiderling, "gorehorn" => Kind::Gorehorn,
        "shield" => Kind::Shieldbearer, "shaman" => Kind::Shaman, "imp" => Kind::Imp, "necro" => Kind::Necromancer, "wurm" => Kind::Wurm, "giant" => Kind::FrostGiant,
        "colossus" => Kind::Colossus, "brood" => Kind::Broodmother, "king" => Kind::EmberKing, _ => return None })
}
pub fn is_boss_kind(k: Kind) -> bool { matches!(k, Kind::Golem | Kind::Colossus | Kind::Broodmother | Kind::EmberKing) }

pub fn model(b: &mut Builder, k: Kind, pos: Vec3, rng: &mut impl FnMut() -> f32) -> Rig {
    let v = Vec3::new;
    match k {
        Kind::Spiderling => enemy_model(b, Kind::Spider, pos, 0, 1.0, rng),
        Kind::Broodmother => {
            let r = enemy_model(b, Kind::Spider, pos, 0, 1.0, rng);
            let body = r.body.unwrap();
            b.bx(body, v(1.25, 0.95, 1.4), 0x2a1a24, v(0.0, 0.3, -0.9));
            b.glow(body, v(0.3, 0.5, 0.05), 0xff3a3a, v(0.0, 0.35, -1.62));
            for (x, y, z) in [(-0.45, 0.6, -0.6), (0.4, 0.75, -1.1), (-0.2, 0.8, -1.3), (0.5, 0.3, -0.5)] { b.glow(body, v(0.14, 0.14, 0.14), 0xffd06a, v(x, y, z)); }
            for (x, z) in [(-0.3, 0.25), (0.3, 0.25)] { let f = b.bx(body, v(0.08, 0.08, 0.35), 0x1a1014, v(x, -0.05, 0.55 + z)); b.rot(f, 0.4, 0.0, 0.0); }
            r
        }
        Kind::Gorehorn => {
            let root = b.root(pos); let body = b.pivot(root, v(0.0, 0.95, 0.0));
            b.bx(body, v(1.1, 0.9, 1.8), 0x5a3a2a, Vec3::ZERO); b.bx(body, v(1.25, 0.55, 0.9), 0x3e281c, v(0.0, 0.5, 0.45));
            b.bx(body, v(0.3, 0.3, 0.5), 0x3e281c, v(0.0, 0.1, -1.05));
            let head = b.pivot(body, v(0.0, 0.15, 0.95));
            b.bx(head, v(0.72, 0.62, 0.75), 0x3e281c, v(0.0, 0.0, 0.3)); b.bx(head, v(0.5, 0.3, 0.3), 0x2a1a12, v(0.0, -0.18, 0.7));
            for s in [-1.0f32, 1.0] {
                let h1 = b.bx(head, v(0.18, 0.18, 0.55), 0xe8dcc0, v(s * 0.42, 0.22, 0.3)); b.rot(h1, -0.3, s * 0.6, 0.0);
                let h2 = b.bx(head, v(0.14, 0.14, 0.45), 0xf4ecd8, v(s * 0.62, 0.4, 0.62)); b.rot(h2, -0.9, s * 0.2, 0.0);
                b.glow(head, v(0.1, 0.08, 0.04), 0xff3a1a, v(s * 0.2, 0.1, 0.68));
            }
            let mut extra = vec![];
            for (x, z) in [(-0.38, 0.6), (0.38, 0.6), (-0.38, -0.6), (0.38, -0.6)] { let lp = b.pivot(body, v(x, -0.3, z)); b.bx(lp, v(0.3, 0.75, 0.3), 0x3a261a, v(0.0, -0.37, 0.0)); b.bx(lp, v(0.32, 0.15, 0.34), 0x1a120c, v(0.0, -0.72, 0.02)); extra.push(lp); }
            Rig { root: Some(root), body: Some(body), head: Some(head), extra, ..default() }
        }
        Kind::Shieldbearer => {
            let r = humanoid(b, pos, &Look { skin: 0xe8e4d8, shirt: 0x6a6e76, pants: 0x4a4e56, boots: Some(0x2a2c30), belt: Some(0x8a6a3a), sleeve: Some(0x5a5e66), eye_glow: Some(0x6ad4ff), thin: true, ..default() });
            let (h, hl, hr) = (r.head.unwrap(), r.hand_l.unwrap(), r.hand_r.unwrap());
            b.bx(h, v(0.52, 0.3, 0.52), 0x7a7e86, v(0.0, 0.4, 0.0)); b.bx(h, v(0.08, 0.2, 0.4), 0xb03a2a, v(0.0, 0.62, 0.0)); b.bx(h, v(0.52, 0.08, 0.06), 0x3a3c40, v(0.0, 0.24, 0.24));
            let sh = b.pivot(hl, v(-0.05, 0.1, 0.25));
            b.bx(sh, v(0.12, 1.35, 0.95), 0x5a4a3a, Vec3::ZERO); b.bx(sh, v(0.14, 1.4, 0.08), 0x8a8e96, v(0.0, 0.0, 0.47)); b.bx(sh, v(0.14, 1.4, 0.08), 0x8a8e96, v(0.0, 0.0, -0.47));
            b.bx(sh, v(0.14, 0.08, 1.0), 0x8a8e96, v(0.0, 0.68, 0.0)); b.bx(sh, v(0.14, 0.08, 1.0), 0x8a8e96, v(0.0, -0.68, 0.0)); b.glow(sh, v(0.15, 0.3, 0.3), 0x6ad4ff, v(0.0, 0.05, 0.0));
            b.rot(sh, 0.0, -PI / 2.0, 0.0);
            let sw = b.pivot(hr, v(0.0, -0.05, 0.0)); b.rot(sw, PI / 2.0, 0.0, 0.0);
            b.bx(sw, v(0.08, 0.06, 1.0), 0xc8ccd4, v(0.0, 0.0, 0.55)); b.bx(sw, v(0.3, 0.06, 0.06), 0x8a6a3a, v(0.0, 0.0, 0.05));
            let mut r = r; r.extra.push(sh); r
        }
        Kind::Shaman => {
            let r = humanoid(b, pos, &Look { skin: 0x6a8a4a, shirt: 0x7a5a3a, pants: 0x4a3a2a, robe: Some(0x5a3a2a), belt: Some(0xaaff5a), eye_glow: Some(0xaaff5a), thin: true, ..default() });
            let (h, hr) = (r.head.unwrap(), r.hand_r.unwrap());
            b.bx(h, v(0.5, 0.36, 0.12), 0xe8e4d8, v(0.0, 0.24, 0.25));
            for (k, c) in [0xd83a3a, 0xf0c64a, 0x3a8ad8, 0x6ad84a].iter().enumerate() { let f = b.bx(h, v(0.08, 0.5, 0.06), *c, v(-0.24 + k as f32 * 0.16, 0.6, -0.05)); b.rot(f, 0.0, 0.0, (k as f32 - 1.5) * 0.25); }
            let st = b.pivot(hr, v(0.0, -0.05, 0.0));
            b.bx(st, v(0.07, 1.8, 0.07), 0x6a4528, v(0.0, 0.4, 0.0)); b.bx(st, v(0.24, 0.22, 0.24), 0xe8e4d8, v(0.0, 1.35, 0.0)); b.glow(st, v(0.16, 0.16, 0.16), 0x9aff5a, v(0.0, 1.55, 0.0));
            r
        }
        Kind::Imp => {
            let root = b.root(pos); let body = b.pivot(root, v(0.0, 0.4, 0.0));
            b.bx(body, v(0.42, 0.5, 0.34), 0xb03a2a, Vec3::ZERO);
            let head = b.pivot(body, v(0.0, 0.38, 0.05)); b.bx(head, v(0.36, 0.32, 0.32), 0xc84a32, Vec3::ZERO);
            for s in [-1.0f32, 1.0] { let hn = b.bx(head, v(0.06, 0.2, 0.06), 0x2a1a14, v(s * 0.14, 0.24, -0.02)); b.rot(hn, 0.0, 0.0, -s * 0.4); b.glow(head, v(0.07, 0.05, 0.02), 0xffe04a, v(s * 0.08, 0.04, 0.165)); }
            let wl = b.pivot(body, v(-0.18, 0.15, -0.1)); let wr = b.pivot(body, v(0.18, 0.15, -0.1));
            for (wg, s) in [(wl, -1.0f32), (wr, 1.0)] { b.bx(wg, v(0.6, 0.04, 0.42), 0x6a1a14, v(s * 0.32, 0.0, 0.0)); b.bx(wg, v(0.06, 0.07, 0.5), 0x2a1a14, v(s * 0.58, 0.03, 0.0)); }
            b.bx(body, v(0.06, 0.06, 0.5), 0x8a2a1a, v(0.0, -0.2, -0.35));
            b.glow(body, v(0.22, 0.22, 0.22), 0xff8a2a, v(0.0, -0.4, 0.15));
            Rig { root: Some(root), body: Some(body), head: Some(head), extra: vec![wl, wr], ..default() }
        }
        Kind::Necromancer => {
            let r = humanoid(b, pos, &Look { skin: 0xd8d0c8, shirt: 0x2a2034, pants: 0x1a1424, robe: Some(0x1a1424), belt: Some(0x6a2a8a), eye_glow: Some(0x9a5aff), thin: true, ..default() });
            let (h, hr) = (r.head.unwrap(), r.hand_r.unwrap());
            b.bx(h, v(0.52, 0.16, 0.52), 0x1a1424, v(0.0, 0.47, -0.02)); b.bx(h, v(0.52, 0.48, 0.1), 0x1a1424, v(0.0, 0.24, -0.25)); b.bx(h, v(0.14, 0.14, 0.4), 0x1a1424, v(0.0, 0.6, -0.2));
            let st = b.pivot(hr, v(0.0, -0.05, 0.0));
            b.bx(st, v(0.08, 2.0, 0.08), 0x2a1a1a, v(0.0, 0.5, 0.0)); b.bx(st, v(0.4, 0.08, 0.08), 0x2a1a1a, v(0.0, 1.45, 0.0));
            let o = b.pal.octa.clone(); b.mesh(st, o, 0xb06aff, Mk::Glow, Transform::from_xyz(0.0, 1.7, 0.0).with_scale(Vec3::new(0.18, 0.32, 0.18)));
            r
        }
        Kind::Wurm => {
            let root = b.root(pos); let body = b.pivot(root, Vec3::ZERO);
            let mut extra = vec![];
            for k in 0..7 {
                let w = 1.15 - k as f32 * 0.06;
                let seg = b.pivot(body, v(0.0, k as f32 * 0.55, 0.0));
                b.bx(seg, v(w, 0.5, w), 0xc8a868, Vec3::ZERO); b.bx(seg, v(w + 0.04, 0.1, w + 0.04), 0x8a6a40, v(0.0, 0.22, 0.0));
                extra.push(seg);
            }
            let head = b.pivot(body, v(0.0, 3.95, 0.0));
            b.bx(head, v(1.25, 0.8, 1.25), 0xb89858, Vec3::ZERO); b.glow(head, v(0.7, 0.1, 0.7), 0xff4a2a, v(0.0, 0.42, 0.0));
            for k in 0..4 { let a = k as f32 * PI / 2.0 + PI / 4.0; let f = b.bx(head, v(0.16, 0.7, 0.16), 0xf0e6d0, v(a.cos() * 0.5, 0.65, a.sin() * 0.5)); b.rot(f, a.sin() * 0.4, 0.0, -a.cos() * 0.4); }
            Rig { root: Some(root), body: Some(body), head: Some(head), extra, ..default() }
        }
        Kind::FrostGiant => {
            let r = humanoid(b, pos, &Look { skin: 0x8ab8d8, shirt: 0x5a6a7a, pants: 0x3a4a5a, boots: Some(0x2a3a4a), belt: Some(0x6a5a4a), sleeve: Some(0x8ab8d8), eye_glow: Some(0xbff4ff), ..default() });
            let (h, t, hr) = (r.head.unwrap(), r.torso.unwrap(), r.hand_r.unwrap());
            b.bx(h, v(0.46, 0.3, 0.12), 0xf0f6ff, v(0.0, 0.02, 0.24)); b.bx(h, v(0.3, 0.3, 0.1), 0xf0f6ff, v(0.0, -0.2, 0.24));
            b.bx(t, v(0.6, 0.3, 0.36), 0xd8d0c0, v(0.0, 0.6, 0.0));
            let o = b.pal.octa.clone();
            for s in [-1.0f32, 1.0] { b.mesh(t, o.clone(), 0xbff4ff, Mk::Ice, Transform::from_xyz(s * 0.36, 0.85, 0.0).with_scale(Vec3::new(0.16, 0.38, 0.16)).with_rotation(Quat::from_rotation_z(-s * 0.4))); }
            let club = b.pivot(hr, v(0.0, -0.05, 0.0)); b.rot(club, PI / 2.0, 0.0, 0.0);
            b.bx(club, v(0.14, 0.14, 1.0), 0x6a5a4a, v(0.0, 0.0, 0.35));
            b.mesh(club, o.clone(), 0xbff4ff, Mk::Ice, Transform::from_xyz(0.0, 0.0, 0.95).with_scale(Vec3::new(0.36, 0.36, 0.5)));
            r
        }
        Kind::Colossus => {
            let r = enemy_model(b, Kind::Skeleton, pos, 0, 1.0, rng);
            let h = r.head.unwrap();
            for k in 0..5 { let x = -0.2 + k as f32 * 0.1; let s = b.bx(h, v(0.06, 0.22 + (k % 2) as f32 * 0.12, 0.06), 0xe8e4d8, v(x, 0.5, 0.0)); b.rot(s, 0.0, 0.0, (k as f32 - 2.0) * -0.15); }
            b.glow(h, v(0.5, 0.06, 0.06), 0x9a5aff, v(0.0, 0.42, 0.2));
            b.glow(r.torso.unwrap(), v(0.16, 0.16, 0.05), 0x9a5aff, v(0.0, 0.42, 0.13));
            r
        }
        Kind::EmberKing => {
            let r = humanoid(b, pos, &Look { skin: 0x2a1a14, shirt: 0x3a2a2a, pants: 0x2a1e1a, boots: Some(0x1a1210), belt: Some(0xffa040), sleeve: Some(0x4a3a34), eye_glow: Some(0xffd060), ..default() });
            let (h, t, hr) = (r.head.unwrap(), r.torso.unwrap(), r.hand_r.unwrap());
            b.bx(h, v(0.5, 0.1, 0.5), 0xe0b84a, v(0.0, 0.47, 0.0));
            for k in 0..5 { let a = k as f32 / 5.0 * TAU; b.bx(h, v(0.08, 0.2, 0.08), 0xe0b84a, v(a.cos() * 0.22, 0.6, a.sin() * 0.22)); }
            b.glow(h, v(0.1, 0.1, 0.1), 0xff4a2a, v(0.0, 0.58, 0.24));
            for s in [-1.0f32, 1.0] { b.bx(t, v(0.4, 0.22, 0.42), 0x4a3a34, v(s * 0.38, 0.72, 0.0)); b.glow(t, v(0.08, 0.3, 0.02), 0xff6a1a, v(s * 0.12, 0.3, 0.16)); }
            b.glow(t, v(0.3, 0.06, 0.02), 0xff6a1a, v(0.0, 0.45, 0.16));
            b.bx(t, v(0.62, 1.3, 0.06), 0x6a1a14, v(0.0, -0.1, -0.2));
            let sw = b.pivot(hr, v(0.0, -0.05, 0.0)); b.rot(sw, PI / 2.0, 0.0, 0.0);
            b.bx(sw, v(0.1, 0.1, 0.5), 0x2a1a14, v(0.0, 0.0, 0.0)); b.bx(sw, v(0.6, 0.1, 0.12), 0xe0b84a, v(0.0, 0.0, 0.25));
            b.bx(sw, v(0.08, 0.32, 2.0), 0x2a2020, v(0.0, 0.0, 1.3)); b.glow(sw, v(0.1, 0.12, 2.0), 0xff7a2a, v(0.0, 0.17, 1.3)); b.glow(sw, v(0.1, 0.12, 2.0), 0xff7a2a, v(0.0, -0.17, 1.3));
            r
        }
        _ => enemy_model(b, Kind::Zombie, pos, 0, 1.0, rng),
    }
}

fn set_rot(q: &mut Query<&mut Transform>, e: Option<Entity>, x: f32, y: f32, z: f32) { if let Some(e) = e { if let Ok(mut t) = q.get_mut(e) { t.rotation = Quat::from_euler(EulerRot::XYZ, x, y, z); } } }
pub fn animate(e: &mut Enemy, q: &mut Query<&mut Transform>, moving: bool) {
    let s = if moving { e.ph.sin() } else { 0.0 };
    let r = &e.rig;
    match e.kind {
        Kind::Spiderling | Kind::Broodmother => { for (k, l) in r.extra.iter().enumerate() { set_rot(q, Some(*l), 0.0, (e.ph * 1.5 + k as f32).sin() * 0.4 * if moving { 1.0 } else { 0.2 }, 0.0); } }
        Kind::Gorehorn => {
            let charging = e.ai.st == 2;
            for (k, l) in r.extra.iter().enumerate() { let ph = e.ph * if charging { 2.0 } else { 1.0 } + if k % 2 == 1 { PI } else { 0.0 } + if k > 1 { PI } else { 0.0 }; set_rot(q, Some(*l), ph.sin() * 0.7 * if moving || charging { 1.0 } else { 0.0 }, 0.0, 0.0); }
            set_rot(q, r.head, if e.ai.st == 1 { 0.35 + (e.ph * 6.0).sin() * 0.1 } else if charging { 0.4 } else { 0.0 }, 0.0, 0.0);
        }
        Kind::Imp | Kind::Wurm => {
            if e.kind == Kind::Imp { let f = (e.ph * 3.0).sin(); if r.extra.len() == 2 { set_rot(q, Some(r.extra[0]), 0.0, 0.0, f * 0.9); set_rot(q, Some(r.extra[1]), 0.0, 0.0, -f * 0.9); } }
            else { for (k, l) in r.extra.iter().enumerate() { set_rot(q, Some(*l), (e.ph * 0.6 + k as f32 * 0.5).sin() * 0.06 * k as f32, 0.0, (e.ph * 0.5 + k as f32 * 0.4).cos() * 0.05 * k as f32); } }
        }
        _ => {
            // humanoid bosses and specialists
            set_rot(q, r.leg_l, s * 0.6, 0.0, 0.0); set_rot(q, r.leg_r, -s * 0.6, 0.0, 0.0);
            let winding = e.wind > 0.0 || e.ai.st == 1;
            let (al, ar) = match e.kind {
                Kind::Shieldbearer => (Vec3::new(-1.2, 0.0, 0.2), if winding { Vec3::new(-2.6, 0.0, 0.0) } else { Vec3::new(-0.4 + s * 0.3, 0.0, 0.0) }),
                Kind::Shaman | Kind::Necromancer => if winding { (Vec3::new(-2.7, 0.0, 0.4), Vec3::new(-2.7, 0.0, -0.4)) } else { (Vec3::new(-s * 0.4, 0.0, 0.1), Vec3::new(-0.8, 0.0, -0.1)) },
                Kind::Colossus => (Vec3::new(-PI / 2.0, 0.12, 0.0), Vec3::new(-PI / 2.0, -0.5 - if winding { 0.4 } else { 0.0 }, 0.0)),
                _ => if winding { (Vec3::new(-2.8, 0.0, 0.3), Vec3::new(-2.8, 0.0, -0.3)) } else { (Vec3::new(-s * 0.5, 0.0, 0.08), Vec3::new(s * 0.5 - 0.3, 0.0, -0.08)) },
            };
            set_rot(q, r.arm_l, al.x, al.y, al.z); set_rot(q, r.arm_r, ar.x, ar.y, ar.z);
            set_rot(q, r.torso, if winding { -0.25 } else { 0.08 }, 0.0, s * 0.04);
        }
    }
}

// ---------------- attacks ----------------
fn enemy_ball(cx: &mut Ctx, from: Vec3, vel: Vec3, dmg: f32, col: u32, grav: f32, web: bool) {
    let e = glow_ball(cx, col, if web { 0.3 } else { 0.35 });
    let mut p = Proj::new(from, vel); p.owner = Owner::Enemy; p.dmg = dmg; p.e = Some(e); p.r = 0.45; p.life = 4.0; p.grav = grav; p.orient = Orient::Fixed;
    p.trail = Some(Box::new(move |cx, p, _| { let c = hexc(col) * 2.5; let v = Vec3::new(cx.rr(-0.5, 0.5), cx.rr(-0.2, 0.8), cx.rr(-0.5, 0.5)); cx.fx.emit(true, p.pos, v, 0.4, 0.45, 0.05, c, -1.0, 1.0, 1.0); }));
    p.on_hit = Some(Box::new(move |cx, p, _| { let (s, d) = (p.pos, p.dmg); cx.hurt_player(d, Some(s), 4.0); if web { cx.g.buffs.insert("web", 2.5); cx.toast("Webbed", "You're slowed", 1.0); } cx.fx.burst(s, col, 18, 4.0, 0.4, 0.35, 0.0, 0.0); }));
    p.on_world = Some(Box::new(move |cx, p, _| cx.fx.burst(p.pos, col, 12, 3.0, 0.35, 0.3, 0.0, 0.0)));
    spawn_proj(cx, p);
}
/// a lobbed bomb: the landing spot is marked, then it bursts
fn bomb(cx: &mut Ctx, from: Vec3, to: Vec3, dmg: f32) {
    let tt = 1.0; let g = 18.0;
    let v = Vec3::new((to.x - from.x) / tt, (to.y - from.y + 0.5 * g * tt * tt) / tt, (to.z - from.z) / tt);
    cx.gfx().circle(to, 0xff6a1a, 2.6, tt + 0.1, "fire", 3.0, None, None);
    let e = glow_ball(cx, 0xff8a2a, 0.3);
    let mut p = Proj::new(from, v); p.owner = Owner::Enemy; p.dmg = dmg; p.e = Some(e); p.r = 0.35; p.life = 3.0; p.grav = g; p.orient = Orient::Fixed;
    p.trail = Some(Box::new(|cx, p, _| { let c = hexc(0xffb03a) * 2.5; cx.fx.emit(true, p.pos, Vec3::Y * 0.5, 0.3, 0.3, 0.05, c, 0.0, 0.0, 1.0); }));
    let boom = |cx: &mut Ctx, p: &mut Proj, _: Option<usize>| {
        let (c, d) = (p.pos, p.dmg);
        cx.play("boom"); cx.fx.burst(c, 0xff7a22, 30, 6.0, 0.5, 0.5, 2.0, 0.0); cx.fx.smoke(c, 0x3a3330, 5, 1.4, 1.2, 1.1);
        cx.gfx().ring(c - Vec3::Y * 0.3, 0xff8a3a, 0.4, 2.6, 0.35);
        if Vec2::new(cx.p.body.pos.x - c.x, cx.p.body.pos.z - c.z).length() < 2.6 && (cx.p.body.pos.y - c.y).abs() < 2.5 { cx.hurt_player(d, Some(c), 7.0); }
    };
    p.on_hit = Some(Box::new(boom)); p.on_world = Some(Box::new(boom)); p.on_expire = Some(Box::new(boom));
    spawn_proj(cx, p);
}
/// a burning patch that hurts the player
fn hot_ground(cx: &mut Ctx, pos: Vec3, r: f32, life: f32, dmg: f32) {
    { let mut g = cx.gfx(); g.disk(pos, 0xff3a10, r, life, 0.3); g.decal(pos, false, r, life + 2.0, Some(0xff4a1a)); }
    let mut z = Zone::new(pos, r, life); z.every = 0.5;
    z.on_tick = Some(Box::new(move |cx, z| { let (p, rr) = (z.pos, z.r); if !cx.p.dead && Vec2::new(cx.p.body.pos.x - p.x, cx.p.body.pos.z - p.z).length() < rr && (cx.p.body.pos.y - p.y).abs() < 2.0 { cx.hurt_player(dmg, None, 0.0); } }));
    z.on_frame = Some(Box::new(|cx, z| { let a = cx.rr(0.0, TAU); let rr = cx.r().sqrt() * z.r; let c = hexc(cx.pick(&[0xff6a1a, 0xffb03a])) * 2.5; let v = Vec3::new(0.0, cx.rr(1.0, 3.0), 0.0); cx.fx.emit(true, z.pos + Vec3::new(a.cos() * rr, 0.15, a.sin() * rr), v, 0.5, 0.45, 0.06, c, -1.0, 0.5, 1.0); }));
    zone(cx, z);
}
fn summon(cx: &mut Ctx, at: Vec3, k: Kind, n: usize, r: f32, col: u32) {
    cx.gfx().circle(at, col, r + 1.0, 1.2, "blood", 3.0, None, None);
    cx.play("rise");
    for j in 0..n {
        let a = j as f32 / n as f32 * TAU + cx.rr(-0.3, 0.3);
        if let Some(s) = cx.w.surface_spot(at.x + a.cos() * r, at.z + a.sin() * r) { spawn_enemy(cx, k, s, SpawnOpts { force: true, ..default() }); }
    }
}

/// state-machine AI for the extended bestiary; returns (move x, move z, damage to the target, speed multiplier)
pub fn ai(cx: &mut Ctx, i: usize, tp: Vec3, dist: f32, tdx: f32, tdz: f32, dy: f32) -> (f32, f32, Option<(f32, f32)>, f32) {
    let dt = cx.dt;
    let (kind, epos, dmg, h) = { let e = &mut cx.g.enemies[i]; e.ai.t -= dt; e.ai.cd -= dt; e.ai.cd2 -= dt; e.ai.cd3 -= dt; (e.kind, e.body.pos, e.dmg, e.body.h) };
    let ai = cx.g.enemies[i].ai;
    let dir = Vec3::new(tdx, 0.0, tdz);
    let mut out = (tdx, tdz, None, 1.0f32);
    let melee = |cx: &mut Ctx, i: usize, reach: f32| -> Option<(f32, f32)> {
        let e = &mut cx.g.enemies[i];
        if dist < reach && dy.abs() < 2.2 && e.atk_t <= 0.0 { e.atk_t = e.t.cd / e.speed_mul; let yaw = tdx.atan2(tdz); let p = e.body.pos + Vec3::Y; let d = e.dmg; cx.gfx().swoosh(p, yaw, reach, 1.6, 0xff6a5a, 0.18, 0.0, 1.0); Some((d, 5.0)) } else { None }
    };
    match kind {
        Kind::Spiderling => { out.2 = melee(cx, i, 1.3); }
        Kind::Gorehorn => match ai.st {
            1 => { out.0 = 0.0; out.1 = 0.0; cx.g.enemies[i].yaw = ai.v.x.atan2(ai.v.z); if cx.r() < 0.4 { cx.fx.emit(false, epos, Vec3::new(cx.rr(-1.0, 1.0), 1.0, cx.rr(-1.0, 1.0)), 0.5, 0.2, 0.5, hexc(0x8a7458), 2.0, 1.0, 0.7); }
                if ai.t <= 0.0 { let e = &mut cx.g.enemies[i]; e.ai.st = 2; e.ai.t = 1.1; cx.play("dash"); } }
            2 => {
                out.0 = ai.v.x; out.1 = ai.v.z; out.3 = 19.0 / 3.4;
                if dist < 1.8 && dy.abs() < 2.0 { out.2 = Some((dmg * 1.5, 14.0)); let e = &mut cx.g.enemies[i]; e.ai.st = 0; e.ai.cd = 4.0; }
                else if cx.g.enemies[i].body.hit_wall { cx.play("heavy"); cx.shake(0.15); let e = &mut cx.g.enemies[i]; e.ai.st = 0; e.ai.cd = 4.0; e.stun = 1.4; cx.fx.sparks(epos + Vec3::Y, 0xffe0a0, 10, 7.0); }
                else if ai.t <= 0.0 { let e = &mut cx.g.enemies[i]; e.ai.st = 0; e.ai.cd = 3.5; }
                if cx.r() < 0.6 { cx.fx.smoke(epos, 0xb8a888, 1, 1.0, 0.6, 0.8); }
            }
            _ => {
                if dist > 5.0 && dist < 18.0 && ai.cd <= 0.0 {
                    let e = &mut cx.g.enemies[i]; e.ai.st = 1; e.ai.t = 0.9; e.ai.v = dir;
                    cx.gfx().streak(epos + Vec3::Y * 0.2, epos + dir * 20.0 + Vec3::Y * 0.2, 0xff3a1a, 0.9, 0.5); cx.play("shout");
                } else { out.2 = melee(cx, i, 2.2); }
            }
        },
        Kind::Shieldbearer => { out.2 = melee(cx, i, 2.2); }
        Kind::Shaman => {
            if dist < 9.0 { out.0 = -tdx; out.1 = -tdz; } else if dist < 13.0 { out.0 = -tdz * 0.5; out.1 = tdx * 0.5; }
            if ai.cd <= 0.0 {
                cx.g.enemies[i].ai.cd = 5.0; cx.g.enemies[i].wind = 0.6;
                cx.play("holy"); cx.gfx().circle(epos, 0x9aff5a, 3.0, 0.8, "nature", 3.0, None, None);
                let mut healed = 0;
                for j in 0..cx.g.enemies.len() {
                    if j == i || healed >= 8 { continue; }
                    let o = &cx.g.enemies[j]; if o.dead || o.t.passive || o.t.prop || o.body.pos.distance(epos) > 9.0 { continue; }
                    let to = o.body.pos + Vec3::Y * o.body.h * 0.6;
                    let o = &mut cx.g.enemies[j]; o.hp = (o.hp + o.max_hp * 0.3).min(o.max_hp); o.ai.haste = 4.0;
                    cx.gfx().streak(epos + Vec3::Y * 1.8, to, 0x9aff5a, 0.4, 0.08); cx.fx.burst(to, 0x9aff5a, 6, 2.0, 0.3, 0.25, 0.0, 0.0);
                    healed += 1;
                }
            }
        }
        Kind::Imp => {
            // circle above the player and drop bombs where they're heading
            let a = cx.g.enemies[i].ph * 0.6;
            let want = tp + Vec3::new(a.cos() * 4.0, 6.0, a.sin() * 4.0);
            let d = want - epos;
            out.0 = d.x.signum() * (d.x.abs() / 2.0).min(1.0); out.1 = d.z.signum() * (d.z.abs() / 2.0).min(1.0);
            { let e = &mut cx.g.enemies[i]; e.body.vel.y = crate::noise::lerp(e.body.vel.y, (d.y * 2.0).clamp(-6.0, 6.0), (dt * 4.0).min(1.0)); }
            if ai.cd <= 0.0 && dist < 14.0 { cx.g.enemies[i].ai.cd = 3.2; let mut to = tp + cx.p.body.vel * 1.0; to.y = cx.ground(to.x, to.z, tp.y + 2.0); bomb(cx, epos + Vec3::Y * 0.2, to, dmg); }
        }
        Kind::Necromancer => {
            if dist < 10.0 { out.0 = -tdx; out.1 = -tdz; } else if dist < 15.0 { out.0 = -tdz * 0.5; out.1 = tdx * 0.5; }
            if ai.cd <= 0.0 && dist < 26.0 { cx.g.enemies[i].ai.cd = 7.0; cx.g.enemies[i].wind = 0.8; summon(cx, epos, Kind::Zombie, 3, 2.5, 0x9a5aff); }
        }
        Kind::Wurm => match ai.st {
            1 => { // telegraph under the target
                out = (0.0, 0.0, None, 1.0);
                if cx.r() < 0.5 { let p = ai.v + Vec3::new(cx.rr(-1.5, 1.5), 0.2, cx.rr(-1.5, 1.5)); let v = Vec3::new(0.0, cx.rr(3.0, 6.0), 0.0); cx.gfx().debris_one(p, v, 0xc8a868, 0.15, 0.6); }
                if ai.t <= 0.0 {
                    let e = &mut cx.g.enemies[i]; e.ai.st = 2; e.ai.t = 2.4; e.ai.hidden = false; e.body.pos = ai.v;
                    cx.play("boom"); cx.shake(0.35); cx.gfx().debris(ai.v + Vec3::Y * 0.3, &[0xc8a868, 0xe0cd8c, 0x8a6a40], 18, 7.0, 0.3, 10.0); cx.fx.smoke(ai.v, 0xd8c8a0, 10, 2.5, 1.2, 1.4);
                    if Vec2::new(tp.x - ai.v.x, tp.z - ai.v.z).length() < 2.6 { out.2 = Some((dmg, 3.0)); cx.p.body.vel.y = 12.0; }
                }
            }
            2 => { out.0 = 0.0; out.1 = 0.0; cx.g.enemies[i].ai.sink = (cx.g.enemies[i].ai.sink - dt * 6.0).max(0.0); out.2 = melee(cx, i, 3.0);
                if ai.t <= 0.0 { let e = &mut cx.g.enemies[i]; e.ai.st = 3; e.ai.t = 0.5; cx.play("rise"); } }
            3 => { out.0 = 0.0; out.1 = 0.0; cx.g.enemies[i].ai.sink = (cx.g.enemies[i].ai.sink + dt * 2.5).min(1.0);
                if ai.t <= 0.0 { let e = &mut cx.g.enemies[i]; e.ai.st = 0; e.ai.hidden = true; e.ai.cd = 2.5; e.ai.sink = 1.0; } }
            _ => { // tunnelling: a mound of sand races toward you
                let e = &mut cx.g.enemies[i]; e.ai.hidden = true; e.ai.sink = 1.0;
                let step = dir * 7.5 * dt; e.body.pos.x += step.x; e.body.pos.z += step.z; e.body.vel = Vec3::ZERO;
                let p = e.body.pos; let gy = cx.ground(p.x, p.z, p.y + 3.0); cx.g.enemies[i].body.pos.y = gy;
                out = (0.0, 0.0, None, 0.0);
                if cx.r() < 0.6 { cx.fx.emit(false, Vec3::new(p.x, gy + 0.2, p.z), Vec3::new(cx.rr(-1.0, 1.0), cx.rr(1.0, 2.5), cx.rr(-1.0, 1.0)), 0.6, 0.3, 0.7, hexc(0xc8b080), 3.0, 1.0, 0.8); }
                if cx.r() < 0.2 { let v = Vec3::new(cx.rr(-1.0, 1.0), 3.0, cx.rr(-1.0, 1.0)); cx.gfx().debris_one(Vec3::new(p.x, gy + 0.2, p.z), v, 0xc8a868, 0.14, 0.6); }
                if dist < 3.0 && ai.cd <= 0.0 { let e = &mut cx.g.enemies[i]; e.ai.st = 1; e.ai.t = 1.0; let mut at = tp; at.y = cx.ground(tp.x, tp.z, tp.y + 2.0); cx.g.enemies[i].ai.v = at; cx.play("rumble"); cx.gfx().circle(at, 0xd8b868, 2.6, 1.1, "war", 4.0, None, None); }
            }
        },
        Kind::FrostGiant => {
            if cx.g.enemies[i].wind > 0.0 {
                out.0 = 0.0; out.1 = 0.0; cx.g.enemies[i].wind -= dt;
                if cx.g.enemies[i].wind <= 0.0 {
                    let c = cx.g.enemies[i].ai.v; slam_fx(cx, c, 3.6); cx.gfx().spikes(c, 0xbff4ff, 12, 3.0, 1.6, 1.6, true);
                    if Vec2::new(tp.x - c.x, tp.z - c.z).length() < 3.6 && (tp.y - c.y).abs() < 2.5 { out.2 = Some((dmg, 9.0)); cx.g.buffs.insert("chill", 2.5); }
                }
            } else if dist < 4.0 && cx.g.enemies[i].atk_t <= 0.0 {
                let e = &mut cx.g.enemies[i]; e.wind = 1.0; e.atk_t = e.t.cd; let c = e.body.pos + dir * 1.6; e.ai.v = c; cx.gfx().circle(c, 0xbff4ff, 3.6, 1.0, "frost", 3.0, None, None);
            }
        }
        Kind::Colossus => {
            if dist > 18.0 { } else if dist < 8.0 { out.0 = -tdx; out.1 = -tdz; } else { out.0 = -tdz * 0.4; out.1 = tdx * 0.4; }
            if cx.g.enemies[i].wind > 0.0 { out.0 = 0.0; out.1 = 0.0; cx.g.enemies[i].wind -= dt;
                if cx.g.enemies[i].wind <= 0.0 { match cx.g.enemies[i].ai.st {
                    1 => { cx.play("bow"); let from = epos + Vec3::Y * h * 0.75; for k in 0..7 { let a = (k as f32 - 3.0) * 0.12; let d = (Quat::from_rotation_y(a) * (tp + Vec3::Y - from).normalize()).normalize(); let mut v = d * 34.0; v.y += 3.0; fire_enemy_proj(cx, EProj::Arrow, from, v, dmg * 0.6); } }
                    _ => { slam_fx(cx, epos, 5.5); if dist < 5.5 { out.2 = Some((dmg * 1.4, 12.0)); } }
                } } }
            else if dist < 5.0 && ai.cd2 <= 0.0 { let e = &mut cx.g.enemies[i]; e.ai.st = 2; e.wind = 1.0; e.ai.cd2 = 5.0; cx.gfx().circle(epos, 0xff6a3a, 5.5, 1.0, "war", 3.0, None, None); }
            else if ai.cd <= 0.0 && dist < 30.0 { let e = &mut cx.g.enemies[i]; e.ai.st = 1; e.wind = 0.7; e.ai.cd = 3.0; }
            else if ai.cd3 <= 0.0 { cx.g.enemies[i].ai.cd3 = 12.0; summon(cx, epos, Kind::Skeleton, 4, 4.0, 0x9a5aff); }
        }
        Kind::Broodmother => {
            let e = &mut cx.g.enemies[i];
            if e.leaping > 0.0 { e.leaping -= dt; out.0 = 0.0; out.1 = 0.0;
                if e.body.on_ground && e.leaping < 0.6 { e.leaping = 0.0; let c = e.body.pos; slam_fx(cx, c, 4.0); if Vec2::new(tp.x - c.x, tp.z - c.z).length() < 4.0 { out.2 = Some((dmg * 1.2, 10.0)); } } }
            else if ai.cd <= 0.0 && dist > 5.0 && dist < 20.0 && e.body.on_ground { e.ai.cd = 5.0; e.leaping = 1.2; e.body.vel = Vec3::new(tdx * dist.min(16.0) * 0.9, 11.0, tdz * dist.min(16.0) * 0.9); cx.play("whoosh"); }
            else if ai.cd2 <= 0.0 && dist < 22.0 { cx.g.enemies[i].ai.cd2 = 4.0; let from = epos + Vec3::Y * 1.2; let v = (tp + Vec3::Y - from).normalize() * 20.0; enemy_ball(cx, from, v, dmg * 0.4, 0xf0f0f0, 0.0, true); }
            else if ai.cd3 <= 0.0 { cx.g.enemies[i].ai.cd3 = 9.0; summon(cx, epos, Kind::Spiderling, 5, 2.5, 0xff3a3a); }
            else { out.2 = melee(cx, i, 3.2); }
        }
        Kind::EmberKing => {
            // phases by remaining health: each one adds attacks and speed
            let (hp, mhp) = (cx.g.enemies[i].hp, cx.g.enemies[i].max_hp);
            let phase = if hp < mhp * 0.15 && cx.g.ember() >= 10 { 3 } else if hp < mhp * 0.33 { 2 } else if hp < mhp * 0.66 { 1 } else { 0 };
            if phase > ai.phase {
                cx.g.enemies[i].ai.phase = phase;
                cx.toast(if phase == 1 { "The Ember King burns hotter" } else if phase == 3 { "THE EMBER KING UNLEASHED" } else { "The Ember King is enraged" }, if phase == 1 { "Fire rains from the sky" } else { "He will not fall quietly" }, 2.5);
                cx.play("bomb"); cx.shake(0.8);
                { let mut g = cx.gfx(); g.shock(epos, 0xff6a1a, 0.5, 12.0, 0.6, 3.0); g.ring(epos, 0xffb03a, 0.5, 12.0, 0.6); }
                if dist < 12.0 { cx.p.body.vel += dir * 14.0 + Vec3::Y * 6.0; }
            }
            out.3 = 1.0 + 0.25 * phase as f32;
            if dist < 3.5 { out.0 = 0.0; out.1 = 0.0; }
            match ai.st {
                1 => { out.0 = 0.0; out.1 = 0.0;
                    if ai.t <= 0.0 { cx.g.enemies[i].ai.st = 0;
                        let yaw = ai.v.x.atan2(ai.v.z); let c = epos + Vec3::Y * 2.0; cx.gfx().swoosh(c, yaw, 6.5, 2.4, 0xff7a2a, 0.3, 0.0, 1.0); cx.play("heavy"); cx.shake(0.4);
                        let to = (tp - epos).with_y(0.0); let ang = to.normalize_or_zero().dot(ai.v);
                        if to.length() < 6.5 && ang > 0.3 { out.2 = Some((dmg * 1.5, 12.0)); }
                        hot_ground(cx, epos + ai.v * 3.0, 2.5, 3.0, dmg * 0.15);
                    } }
                2 => { out.0 = 0.0; out.1 = 0.0;
                    if ai.t <= 0.0 { cx.g.enemies[i].ai.st = 0; cx.play("fire");
                        let rings = 1 + phase.min(1) as usize; for r in 0..rings { let n = 16; for k in 0..n { let a = (k as f32 + r as f32 * 0.5) / n as f32 * TAU; let from = epos + Vec3::Y * 1.5; enemy_ball(cx, from, Vec3::new(a.cos(), 0.0, a.sin()) * (11.0 + r as f32 * 3.0), dmg * 0.5, 0xff6a1a, 0.0, false); } } } }
                3 => { // the charge
                    out.0 = ai.v.x; out.1 = ai.v.z; out.3 = 18.0 / 3.1;
                    if cx.r() < 0.5 { let mut g = epos; g.y = cx.ground(g.x, g.z, g.y + 1.0); hot_ground(cx, g, 1.4, 3.5, dmg * 0.12); }
                    if dist < 2.5 { out.2 = Some((dmg * 1.3, 14.0)); cx.g.enemies[i].ai.st = 0; }
                    else if ai.t <= 0.0 || cx.g.enemies[i].body.hit_wall { cx.g.enemies[i].ai.st = 0; }
                }
                _ => {
                    if dist < 6.0 && ai.cd <= 0.0 { let e = &mut cx.g.enemies[i]; e.ai.st = 1; e.ai.t = 0.8 - 0.15 * phase as f32; e.ai.cd = 2.2; e.ai.v = dir; cx.gfx().circle(epos + dir * 3.0, 0xff3a1a, 4.0, 0.8, "fire", 3.0, None, None); }
                    else if ai.cd2 <= 0.0 { let e = &mut cx.g.enemies[i]; e.ai.st = 2; e.ai.t = 0.6; e.ai.cd2 = 6.0 - phase as f32; cx.gfx().circle(epos, 0xff6a1a, 3.0, 0.6, "fire", 5.0, None, None); }
                    else if phase >= 2 && ai.cd3 <= 0.0 && dist > 6.0 { let e = &mut cx.g.enemies[i]; e.ai.st = 3; e.ai.t = 1.2; e.ai.v = dir; e.ai.cd3 = 5.0; cx.gfx().streak(epos + Vec3::Y * 0.2, epos + dir * 22.0 + Vec3::Y * 0.2, 0xff3a1a, 0.8, 0.8); cx.play("shout"); }
                    else if phase >= 1 && ai.cd3 <= 0.0 {
                        cx.g.enemies[i].ai.cd3 = 9.0;
                        for k in 0..6 { let a = cx.rr(0.0, TAU); let r = if k == 0 { 0.0 } else { cx.rr(3.0, 9.0) }; let mut c = tp + Vec3::new(a.cos() * r, 0.0, a.sin() * r); c.y = cx.ground(c.x, c.z, tp.y + 3.0); crate::director::eruption(cx, c, 2.4, 1.0 + k as f32 * 0.2); }
                        if cx.r() < 0.5 { summon(cx, epos, Kind::Imp, 3, 3.0, 0xff6a1a); }
                    }
                }
            }
        }
        _ => {}
    }
    out
}

/// bosses leave something behind when they die
pub fn on_death(cx: &mut Ctx, kind: Kind, pos: Vec3) {
    match kind {
        Kind::Broodmother => { for _ in 0..8 { let p = pos + Vec3::new(cx.rr(-2.0, 2.0), 0.4, cx.rr(-2.0, 2.0)); if let Some(j) = spawn_enemy(cx, Kind::Spiderling, p, SpawnOpts { no_rise: true, force: true, ..default() }) { cx.g.enemies[j].body.vel = Vec3::new(cx.rr(-6.0, 6.0), 6.0, cx.rr(-6.0, 6.0)); } } }
        Kind::EmberKing => { cx.g.flags.insert("victory", cx.g.run_t); cx.toast("The Ember King has fallen", "The wild is yours", 4.0); cx.play("relic"); cx.g.flash = 0.4;
            let mut g = cx.gfx(); g.pillar(pos, 0xffd35a, 3.0, 40.0, 3.0); g.shock(pos, 0xffd35a, 0.5, 20.0, 1.0, 4.0); }
        _ => {}
    }
}
