//! The horde director. Monsters don't live in the world: the director keeps a crowd of them hunting you. It spawns
//! packs on a ring just past the edge of the fight, raises the headcount and unlocks tougher kinds as the minutes
//! pass, throws set-piece horde events and bosses at you, scatters loot urns and animals, guards ruins and camps,
//! and runs the points of interest (chests, shrines, altar trials).
use crate::enemies::*;
use crate::game::*;
use crate::models::*;
use crate::world::*;
use bevy::prelude::*;
use std::collections::HashSet;
use std::f32::consts::TAU;

pub struct Chest { pub root: Entity, pub lid: Entity, pub pos: Vec3, pub open: bool, pub t: f32, pub relic: bool, pub poi: Option<(i32, i32)> }
pub struct Trial { pub poi: (i32, i32), pub c: Vec3, pub t: f32, pub spawn: f32 }

pub struct Director {
    pub pack: f32, pub event: f32, pub boss: f32, pub urn: f32, pub animal: f32, pub ev_i: u32,
    pub guarded: HashSet<(i32, i32)>, pub cleared: HashSet<(i32, i32)>, pub used: HashSet<(i32, i32)>, pub chest_out: HashSet<(i32, i32)>,
    pub chests: Vec<Chest>, pub trial: Option<Trial>, pub heat: f32, pub hazard: f32, pub next_boss: usize, pub warned: bool, pub endless_t: f32,
}
impl Default for Director {
    fn default() -> Self {
        Director { pack: 1.5, event: 70.0, boss: 300.0, urn: 2.0, animal: 3.0, ev_i: 0, guarded: HashSet::new(), cleared: HashSet::new(), used: HashSet::new(), chest_out: HashSet::new(),
            chests: vec![], trial: None, heat: 0.0, hazard: 10.0, next_boss: 0, warned: false, endless_t: 120.0 }
    }
}

pub const THREAT: [(f32, &str, u32); 6] = [(0.0, "Calm", 0x9fd8ff), (3.0, "Restless", 0xc8e070), (6.0, "Dangerous", 0xf0c64a), (10.0, "Deadly", 0xff8a3d), (15.0, "Nightmare", 0xff4a3a), (21.0, "Apocalypse", 0xd04aff)];
pub fn threat(m: f32) -> (&'static str, u32) { let mut t = THREAT[0]; for q in THREAT { if m >= q.0 { t = q; } } (t.1, t.2) }

/// how many hostiles should be hunting right now
fn horde_target(cx: &Ctx) -> f32 {
    let m = cx.g.minutes();
    let mut n = 6.0 + m * 6.5 + (cx.dc.day as f32 - 1.0) * 3.0;
    if cx.dc.night { n *= 1.3; }
    if cx.dc.blood { n *= 1.4; }
    if cx.g.dir.trial.is_some() { n += 10.0; }
    if cx.g.ember() >= 6 { n *= 1.25; }
    n.min(150.0)
}

/// a spawn point on a ring around `a`; spots in front of the player are pushed further out so the horde rises at the edge of sight
pub fn ring_spot(cx: &mut Ctx, a: Vec3, yaw: f32, r0: f32, r1: f32) -> Option<Vec3> {
    let (fx, fz) = (yaw.sin(), yaw.cos());
    for _ in 0..10 {
        let an = cx.rr(0.0, TAU);
        let mut d = cx.rr(r0, r1);
        if an.cos() * fx + an.sin() * fz > 0.6 { d += 8.0; }
        if let Some(s) = cx.w.surface_spot(a.x + an.cos() * d, a.z + an.sin() * d) {
            if (s.y - a.y).abs() < 14.0 && cx.w.poi_near(s.x, s.z, 2.0).is_none() { return Some(s); }
        }
    }
    None
}

/// [kind, minute it joins the horde, weight]
const ROSTER: &[(Kind, f32, f32)] = &[(Kind::Zombie, 0.0, 6.0), (Kind::Slime, 0.0, 4.0), (Kind::Bat, 1.5, 3.5), (Kind::Skeleton, 2.5, 2.4), (Kind::Spider, 3.5, 3.0), (Kind::Wurm, 3.0, 2.2),
    (Kind::Gorehorn, 4.0, 1.5), (Kind::Bloater, 4.5, 2.2), (Kind::Shieldbearer, 5.0, 1.6), (Kind::Cultist, 6.0, 2.0), (Kind::Imp, 6.5, 1.4), (Kind::Shaman, 7.0, 1.0),
    (Kind::FrostGiant, 7.0, 0.8), (Kind::Brute, 8.0, 1.4), (Kind::Necromancer, 9.0, 0.9)];
fn pack_size(k: Kind) -> (u32, u32) { match k { Kind::Zombie => (2, 5), Kind::Slime => (2, 4), Kind::Bat => (4, 8), Kind::Skeleton => (1, 3), Kind::Spider => (2, 3), Kind::Bloater | Kind::Cultist | Kind::Gorehorn => (1, 2), Kind::Shieldbearer | Kind::Imp => (2, 3), _ => (1, 1) } }
fn pick_kind(cx: &mut Ctx, b: Bio) -> Kind {
    let m = cx.g.minutes();
    let mut opts = vec![]; let mut tot = 0.0;
    for &(k, at, w) in ROSTER {
        if m < at { continue; }
        // biome natives: sand wurms only hunt the desert, frost giants only the snow
        if k == Kind::Wurm && b != Bio::Desert { continue; }
        if k == Kind::FrostGiant && !matches!(b, Bio::Snow | Bio::Peak) { continue; }
        let mut q = w * (0.35 + (m - at) * 0.4).min(1.0);
        if k == Kind::Skeleton && b == Bio::Desert { q *= 2.0; }
        if k == Kind::Spider && b == Bio::Forest { q *= 2.0; }
        if k == Kind::Cultist && matches!(b, Bio::Snow | Bio::Peak) { q *= 1.8; }
        if k == Kind::Slime && matches!(b, Bio::Plains | Bio::Forest) { q *= 1.4; }
        if cx.dc.night && matches!(k, Kind::Zombie | Kind::Skeleton | Kind::Bat | Kind::Necromancer) { q *= 1.5; }
        if cx.dc.blood && matches!(k, Kind::Shaman | Kind::Imp | Kind::Gorehorn) { q *= 1.8; }
        opts.push((k, q)); tot += q;
    }
    let mut r = cx.r() * tot;
    for (k, q) in opts { r -= q; if r <= 0.0 { return k; } }
    Kind::Zombie
}
fn slime_opts(cx: &mut Ctx, b: Bio) -> SpawnOpts {
    let size = if cx.r() < 0.3 { 1.6 } else { 1.0 };
    let color = match b { Bio::Desert => 0xe0b85a, Bio::Snow => 0x8fd8ff, _ => cx.pick(&[0x6fd46a, 0x6fd46a, 0xff7a9a]) };
    SpawnOpts { size, color, ..default() }
}
pub struct PackOpts { pub at: Option<Vec3>, pub r0: f32, pub r1: f32, pub elite: bool, pub force: bool }
impl Default for PackOpts { fn default() -> Self { PackOpts { at: None, r0: 26.0, r1: 36.0, elite: false, force: false } } }
pub fn spawn_pack(cx: &mut Ctx, around: Vec3, yaw: f32, k: Kind, n: u32, o: PackOpts) -> u32 {
    let s = match o.at { Some(s) => s, None => match ring_spot(cx, around, yaw, o.r0, o.r1) { Some(s) => s, None => return 0 } };
    let b = cx.w.column(s.x.floor() as i32, s.z.floor() as i32).b;
    let mut got = 0;
    for i in 0..n {
        let (jx, jz) = (cx.rr(-2.5, 2.5), cx.rr(-2.5, 2.5));
        let sp = cx.w.surface_spot(s.x + jx, s.z + jz).unwrap_or(s);
        let mut eo = if k == Kind::Slime { slime_opts(cx, b) } else { SpawnOpts::default() };
        eo.force = o.force;
        if i == 0 && o.elite { eo.affix = Some(cx.pick(&AFFIXES)); }
        let y = if k == Kind::Bat { sp.y + cx.rr(2.0, 4.0) } else { sp.y };
        if spawn_enemy(cx, k, Vec3::new(sp.x, y, sp.z), eo).is_some() { got += 1; }
    }
    if got > 0 && cx.r() < 0.25 { cx.play("rise"); }
    got
}

pub fn update_director(cx: &mut Ctx) {
    if cx.p.dead { return; }
    let dt = cx.dt;
    let (pp, yaw) = (cx.p.body.pos, cx.p.yaw);
    let (mut hostile, mut animals, mut urns, mut ranged, mut near) = (0.0f32, 0, 0, 0.0f32, 0.0f32);
    for e in cx.g.enemies.iter() {
        if e.dead { continue; }
        if e.t.prop { urns += 1; } else if e.t.passive { animals += 1; } else {
            if e.poi.is_none() { hostile += 1.0; if e.t.ranged { ranged += 1.0; } }
            if Vec2::new(e.body.pos.x - pp.x, e.body.pos.z - pp.z).length() < 22.0 { near += 1.0; }
        }
    }
    let boss_up = cx.g.boss.is_some();
    let heat = (near / 22.0 + if boss_up { 0.5 } else { 0.0 }).clamp(0.0, 1.0);
    cx.g.dir.heat += (heat - cx.g.dir.heat) * (dt * 0.6).min(1.0);
    let m = cx.g.minutes();
    cx.g.dir.pack -= dt;
    if cx.g.dir.pack <= 0.0 {
        cx.g.dir.pack = (1.0 - m * 0.05).max(0.3);
        let want = horde_target(cx) * if final_boss_up(cx) { 0.5 } else { 1.0 };
        if hostile < want {
            let b = cx.w.column(pp.x.floor() as i32, pp.z.floor() as i32).b;
            let mut k = pick_kind(cx, b);
            if etype(k).ranged && ranged > want * 0.22 { k = cx.pick(&[Kind::Zombie, Kind::Spider, Kind::Slime]); }
            let (lo, hi) = pack_size(k);
            let n = ((want - hostile).ceil() as u32).min(lo + (cx.r() * (hi - lo + 1) as f32) as u32 + (m / 4.0) as u32);
            let elite = cx.r() < ((0.02 + m * 0.012 + if cx.dc.blood { 0.08 } else { 0.0 }) * if cx.g.ember() >= 3 { 2.0 } else { 1.0 }).clamp(0.0, 0.45);
            spawn_pack(cx, pp, yaw, k, n, PackOpts { elite, ..default() });
        }
    }
    if !final_boss_up(cx) { cx.g.dir.event -= dt; }
    if cx.g.dir.event <= 0.0 { cx.g.dir.event = cx.rr(75.0, 100.0) * (1.0 - m * 0.02).max(0.6); let i = cx.g.dir.ev_i; cx.g.dir.ev_i += 1; horde_event(cx, i); }
    boss_schedule(cx);
    // from the Dangerous tier on, the ground itself turns on you: telegraphed eruptions land where you're heading
    let hz0 = if cx.g.ember() >= 7 { 3.0 } else { 6.0 };
    if m >= hz0 {
        cx.g.dir.hazard -= dt;
        if cx.g.dir.hazard <= 0.0 {
            cx.g.dir.hazard = (14.0 - (m - hz0) * 0.55).max(5.0) * cx.rr(0.8, 1.2) * if cx.g.ember() >= 7 { 0.75 } else { 1.0 };
            let n = 1 + (m / 8.0) as usize;
            let lead = cx.p.body.vel * 1.3;
            for k in 0..n {
                let off = if k == 0 { lead.with_y(0.0) } else { let a = cx.rr(0.0, TAU); Vec3::new(a.cos(), 0.0, a.sin()) * cx.rr(3.0, 9.0) };
                let mut c = pp + off; c.y = cx.ground(c.x, c.z, pp.y + 3.0);
                eruption(cx, c, 2.6, 1.3 + k as f32 * 0.25);
            }
        }
    }
    cx.g.dir.urn -= dt;
    if cx.g.dir.urn <= 0.0 { cx.g.dir.urn = 6.0; if urns < 4 { if let Some(s) = ring_spot(cx, pp, yaw, 14.0, 34.0) { spawn_enemy(cx, Kind::Urn, s, SpawnOpts::default()); } } }
    cx.g.dir.animal -= dt;
    if cx.g.dir.animal <= 0.0 {
        cx.g.dir.animal = 4.0;
        if animals < 8 { if let Some(s) = ring_spot(cx, pp, yaw, 22.0, 40.0) {
            let tb = cx.w.get(s.x.floor() as i32, s.y.floor() as i32 - 1, s.z.floor() as i32);
            if matches!(tb, b::GRASS | b::FGRASS | b::SNOW) {
                let k = if tb == b::SNOW { Kind::Sheep } else { cx.pick(&[Kind::Pig, Kind::Pig, Kind::Sheep, Kind::Chicken, Kind::Chicken]) };
                let n = 2 + (cx.r() * 3.0) as u32;
                for _ in 0..n { let (jx, jz) = (cx.rr(-3.0, 3.0), cx.rr(-3.0, 3.0)); if let Some(sp) = cx.w.surface_spot(s.x + jx, s.z + jz) { spawn_enemy(cx, k, sp, SpawnOpts::default()); } }
            }
        } }
    }
}

/// a rune burns into the ground, then the earth bursts upward in fire
pub fn eruption(cx: &mut Ctx, c: Vec3, r: f32, delay: f32) {
    cx.gfx().circle(c, 0xff3a1a, r, delay + 0.1, "fire", 2.5, None, None);
    cx.gfx().decal(c, true, r * 0.9, delay + 3.0, Some(0xff5a1a));
    cx.later(delay, move |cx| {
        cx.play("boom"); cx.shake(0.25);
        let pts: Vec<(Vec3, f32)> = (0..5).map(|i| { let a = i as f32 / 5.0 * TAU + cx.rr(-0.2, 0.2); let rr = r * cx.rr(0.2, 0.7); let mut p = c + Vec3::new(a.cos() * rr, 0.0, a.sin() * rr); p.y = cx.ground(p.x, p.z, c.y + 2.0); (p, cx.rr(1.2, 2.2)) }).collect();
        crate::abilities::earth_pillars(cx, &pts, 0.9);
        cx.fx.burst(c + Vec3::Y * 0.5, 0xff6a1a, 40, 7.0, 0.6, 0.5, 5.0, 0.0);
        cx.fx.smoke(c, 0x3a3430, 8, 2.0, 1.2, 1.3);
        cx.gfx().ring(c, 0xff8a3a, 0.4, r * 1.3, 0.4);
        let m = cx.g.minutes();
        if !cx.p.dead && Vec2::new(cx.p.body.pos.x - c.x, cx.p.body.pos.z - c.z).length() < r && (cx.p.body.pos.y - c.y).abs() < 3.0 { cx.hurt_player(10.0 + 2.0 * m, Some(c), 9.0); }
        // the horde gets caught in it too
        cx.aoe(c, r, &mut |cx, i, _| { if !cx.g.enemies[i].t.prop { cx.apply_damage(i, 30.0, Hit { kb: 4.0, up: 6.0, src: Some(c), ..default() }); } });
    });
}

/// set pieces that break up the steady stream
fn horde_event(cx: &mut Ctx, i: u32) {
    let m = cx.g.minutes(); let pp = cx.p.body.pos; let yaw = cx.p.yaw;
    cx.play("horn");
    match i % 3 {
        0 => {
            // encircled: a full ring of the dead rises all around you at once
            let n = (14 + (m * 3.0) as u32).min(44); let r = cx.rr(19.0, 23.0);
            cx.toast("The horde closes in!", "They rise all around you", 2.8);
            for k in 0..n {
                let an = k as f32 / n as f32 * TAU;
                let Some(s) = cx.w.surface_spot(pp.x + an.cos() * r, pp.z + an.sin() * r) else { continue };
                let kind = if m > 5.0 && k % 4 == 0 { Kind::Skeleton } else if m > 8.0 && k % 7 == 0 { Kind::Bloater } else { Kind::Zombie };
                spawn_enemy(cx, kind, s, SpawnOpts { force: true, ..default() });
            }
        }
        1 => {
            // a shrieking swarm from one direction
            let n = (10 + (m * 2.5) as u32).min(40); let an = cx.rr(0.0, TAU);
            let dir = ["east", "south", "west", "north"][((an / (TAU / 4.0)).round() as usize) % 4];
            cx.toast("A swarm descends!", &format!("Bats pour in from the {}", dir), 2.8);
            for _ in 0..n {
                let d = cx.rr(26.0, 34.0); let sp = an + cx.rr(-0.35, 0.35);
                if let Some(s) = cx.w.surface_spot(pp.x + sp.cos() * d, pp.z + sp.sin() * d) { let y = s.y + cx.rr(3.0, 6.0); spawn_enemy(cx, Kind::Bat, Vec3::new(s.x, y, s.z), SpawnOpts { force: true, ..default() }); }
            }
        }
        _ => {
            cx.toast("Elites approach", "A war band hunts you", 2.8);
            let lead = if m > 8.0 { Kind::Brute } else if m > 4.0 { cx.pick(&[Kind::Zombie, Kind::Spider]) } else { Kind::Zombie };
            let at = ring_spot(cx, pp, yaw, 24.0, 30.0);
            if at.is_none() { return; }
            spawn_pack(cx, pp, yaw, lead, 1 + (m / 6.0) as u32, PackOpts { at, elite: true, force: true, ..default() });
            let follow = if lead == Kind::Brute { Kind::Zombie } else { lead };
            spawn_pack(cx, pp, yaw, follow, 4 + (m / 2.0) as u32, PackOpts { at, elite: true, force: true, ..default() });
        }
    }
}
/// the run's shape: a boss every five minutes, and the Ember King at twenty
pub const BOSSES: [(f32, Kind, &str, &str); 4] = [
    (300.0, Kind::Golem, "An Ancient Golem awakens", "Bring it down for a relic"),
    (600.0, Kind::Colossus, "The Bone Colossus rises", "Its arrows blot out the sun"),
    (900.0, Kind::Broodmother, "The Broodmother descends", "Her children are hungry"),
    (1200.0, Kind::EmberKing, "THE EMBER KING", "Defeat him to win the run"),
];
pub const RUN_LEN: f32 = 1200.0;
pub fn final_boss_up(cx: &Ctx) -> bool { cx.g.boss.and_then(|id| cx.g.enemies.iter().find(|e| e.id == id && !e.dead)).map(|e| e.kind == Kind::EmberKing).unwrap_or(false) }
/// (seconds until, name) of the next scheduled boss
pub fn next_boss(g: &Game) -> Option<(f32, &'static str)> {
    if g.flags.contains_key("endless") { return Some((g.dir.endless_t, "Boss")); }
    BOSSES.get(g.dir.next_boss).map(|b| (b.0 - g.run_t, crate::enemies::etype(b.1).name))
}
fn boss_schedule(cx: &mut Ctx) {
    let t = cx.g.run_t;
    if cx.g.flags.contains_key("endless") {
        cx.g.dir.endless_t -= cx.dt;
        if cx.g.dir.endless_t <= 0.0 && cx.g.boss.is_none() {
            let k = cx.pick(&[Kind::Golem, Kind::Colossus, Kind::Broodmother, Kind::EmberKing]);
            if spawn_boss(cx, k, "A champion of the wild appears", "Endless: every boss grows stronger") { cx.g.dir.endless_t = 240.0; *cx.g.flags.entry("endless_n").or_insert(0.0) += 1.0; }
        }
        return;
    }
    let Some(&(at, k, title, sub)) = BOSSES.get(cx.g.dir.next_boss) else { return };
    if t >= at - 8.0 && !cx.g.dir.warned { cx.g.dir.warned = true; cx.play("rumble"); cx.toast("The ground trembles...", "Something huge is coming", 3.0); cx.shake(0.3); }
    if t >= at && spawn_boss(cx, k, title, sub) { cx.g.dir.next_boss += 1; cx.g.dir.warned = false; }
}
fn spawn_boss(cx: &mut Ctx, k: Kind, title: &str, sub: &str) -> bool {
    let (pp, yaw) = (cx.p.body.pos, cx.p.yaw);
    let Some(s) = ring_spot(cx, pp, yaw, 22.0, 28.0) else { return false };
    if let Some(i) = spawn_enemy(cx, k, s, SpawnOpts { boss: true, force: true, ..default() }) {
        let id = cx.g.enemies[i].id; cx.g.boss = Some(id);
        cx.toast(title, sub, 3.5); cx.play("horn"); cx.play("boom"); cx.shake(0.7);
        let mut g = cx.gfx(); g.pillar(s, 0xff3a1a, 2.5, 30.0, 1.5); g.shock(s, 0xff6a3a, 0.5, 14.0, 0.8, 3.0);
        true
    } else { false }
}

// ---------------- chests, shrines and altars ----------------
fn chest_model(cx: &mut Ctx, pos: Vec3, yaw: f32) -> (Entity, Entity) {
    let mut b = cx.builder();
    let root = b.root(pos);
    let (w, gd, dk) = (0x7a4e2a, 0xe0b84a, 0x3a2618);
    let v = Vec3::new;
    b.bx(root, v(0.9, 0.5, 0.6), w, v(0.0, 0.25, 0.0)); b.bx(root, v(0.94, 0.06, 0.64), gd, v(0.0, 0.08, 0.0)); b.bx(root, v(0.94, 0.06, 0.64), gd, v(0.0, 0.46, 0.0));
    let lid = b.pivot(root, v(0.0, 0.5, -0.3));
    b.bx(lid, v(0.92, 0.22, 0.62), w, v(0.0, 0.11, 0.3)); b.bx(lid, v(0.14, 0.18, 0.06), gd, v(0.0, 0.05, 0.62));
    b.bx(root, v(0.06, 0.5, 0.64), dk, v(-0.3, 0.25, 0.0)); b.bx(root, v(0.06, 0.5, 0.64), dk, v(0.3, 0.25, 0.0));
    b.glow(root, v(0.7, 0.04, 0.5), 0xffd35a, v(0.0, 0.49, 0.0));
    b.rot(root, 0.0, yaw, 0.0);
    (root, lid)
}
/// a treasure chest drops where an elite or boss fell (relic chests come from altar trials)
pub fn drop_chest(cx: &mut Ctx, pos: Vec3, relic: bool) {
    let y = cx.ground(pos.x, pos.z, pos.y + 1.0);
    let p = Vec3::new(pos.x, y, pos.z);
    let yaw = cx.rr(0.0, TAU);
    // it slams down out of the sky with a golden beam
    let (root, lid) = chest_model(cx, p + Vec3::Y * 6.0, yaw);
    cx.g.dir.chests.push(Chest { root, lid, pos: p, open: false, t: -0.35, relic, poi: None });
    cx.gfx().pillar(p, 0xffd35a, 0.8, 10.0, 1.2);
}
fn open_chest(cx: &mut Ctx, i: usize) {
    let (pos, relic, poi) = { let c = &mut cx.g.dir.chests[i]; c.open = true; c.t = 0.0; (c.pos, c.relic, c.poi) };
    cx.play("chest");
    cx.fx.burst(pos + Vec3::Y * 0.6, 0xffd35a, 50, 6.0, 0.8, 0.35, 4.0, 4.0);
    { let mut g = cx.gfx(); g.pillar(pos, 0xffd35a, 0.7, 8.0, 0.8); g.flare(pos + Vec3::Y * 0.8, 0xffd35a, 4.0, 0.5, true); }
    // coins and gems spill out over the lip
    for _ in 0..14 { let v = Vec3::new(cx.rr(-2.5, 2.5), cx.rr(5.0, 8.0), cx.rr(-2.5, 2.5)); cx.gfx().debris_one(pos + Vec3::Y * 0.6, v, 0xffd35a, 0.12, 1.4); }
    if let Some(p) = poi { cx.g.dir.cleared.insert(p); }
    cx.g.food_meat += (cx.r() * 3.0) as u32; cx.g.food_berries += 1 + (cx.r() * 3.0) as u32;
    let xp = 10.0 + cx.dc.day as f32 * 4.0; cx.gain_xp(xp);
    cx.later(0.6, move |cx| { cx.g.chest_pending += 1; if relic { cx.g.flags.insert("relic_chest", 1.0); } });
}
fn use_shrine(cx: &mut Ctx, p: &Poi) {
    cx.g.dir.used.insert((p.gx, p.gz));
    cx.play("holy");
    let mh = cx.p.max_hp; cx.heal(mh, false);
    cx.g.buffs.insert("shrine", 90.0);
    let c = Vec3::new(p.x as f32 + 0.5, p.y as f32 + 1.0, p.z as f32 + 0.5);
    { let mut g = cx.gfx(); g.pillar(c, 0xb98aff, 2.5, 40.0, 2.0); g.ring(c, 0xb98aff, 0.5, 8.0, 0.8); g.circle(c, 0xb98aff, 5.0, 2.0, "arcane", 2.0, None, None); }
    cx.fx.burst(c + Vec3::Y * 1.5, 0xb98aff, 60, 6.0, 1.0, 0.4, 3.0, 0.0);
    cx.toast("Blessing of the old stones", "Full heal and +25% damage for 90s", 3.0);
}
fn begin_trial(cx: &mut Ctx, p: &Poi) {
    if cx.g.dir.trial.is_some() { return; }
    cx.g.dir.used.insert((p.gx, p.gz));
    let c = Vec3::new(p.x as f32 + 0.5, p.y as f32 + 1.0, p.z as f32 + 0.5);
    cx.g.dir.trial = Some(Trial { poi: (p.gx, p.gz), c, t: 35.0, spawn: 1.0 });
    cx.toast("The altar awakens", "Survive 35 seconds near the altar", 3.0); cx.play("horn");
    { let mut g = cx.gfx(); g.pillar(c, 0xff3a4a, 2.0, 30.0, 1.5); g.ring(c, 0xff3a4a, 0.5, 12.0, 0.8); g.shock(c, 0xff3a4a, 0.5, 12.0, 0.6, 3.0); g.circle(c, 0xff3a4a, 12.0, 35.0, "blood", 0.4, None, None); }
    cx.shake(0.5);
}
fn update_trial(cx: &mut Ctx) {
    let Some(tr) = cx.g.dir.trial.as_mut() else { return };
    tr.t -= cx.dt; tr.spawn -= cx.dt;
    let (c, t, spawn) = (tr.c, tr.t, tr.spawn);
    if cx.r() < 0.7 { let an = cx.rr(0.0, TAU); let r = cx.rr(0.0, 12.0); let v = Vec3::new(0.0, cx.rr(2.0, 4.0), 0.0); cx.fx.emit(true, c + Vec3::new(an.cos() * r, 0.0, an.sin() * r), v, 0.8, 0.4, 0.05, crate::fx::hexc(0xff3a4a) * 2.5, 0.0, 0.0, 1.0); }
    if spawn <= 0.0 {
        if let Some(tr) = cx.g.dir.trial.as_mut() { tr.spawn = 2.2; }
        let m = cx.g.minutes();
        let k = cx.pick(&[Kind::Zombie, Kind::Skeleton, Kind::Spider, Kind::Zombie, if m > 4.0 { Kind::Bloater } else { Kind::Slime }]);
        let elite = cx.r() < 0.3;
        spawn_pack(cx, c, 0.0, k, 3 + (m / 3.0) as u32, PackOpts { r0: 12.0, r1: 16.0, force: true, elite, ..default() });
    }
    let d = Vec2::new(cx.p.body.pos.x - c.x, cx.p.body.pos.z - c.z).length();
    if !cx.p.dead && d > 40.0 { cx.toast("The trial is broken", "You strayed too far from the altar", 2.5); cx.g.dir.trial = None; return; }
    if t <= 0.0 {
        cx.g.dir.trial = None;
        cx.toast("Trial complete", "The altar yields a relic", 3.0); cx.play("chest");
        let cp = c + Vec3::new(0.0, 0.0, 3.0);
        drop_chest(cx, cp, true);
        cx.gfx().pillar(c, 0xffd35a, 2.0, 30.0, 1.5);
        let xp = (20.0 + cx.g.minutes() * 6.0).round(); cx.drop_gems(cp, xp);
    }
}

/// what the player can interact with right now
#[derive(Clone)]
pub enum Interact { Chest(usize), Shrine(Poi), Altar(Poi) }
pub fn find_interact(cx: &mut Ctx) -> Option<(String, Interact)> {
    if cx.p.dead { return None; }
    let pp = cx.p.body.pos;
    let mut best = 3.2; let mut out = None;
    for (i, c) in cx.g.dir.chests.iter().enumerate() { if c.open || c.t < 0.0 { continue; } let d = c.pos.distance(pp); if d < best { best = d; out = Some(("Open chest".to_string(), Interact::Chest(i))); } }
    if out.is_some() { return out; }
    let p = cx.w.poi((pp.x / 64.0).floor() as i32, (pp.z / 64.0).floor() as i32)?;
    if cx.g.dir.used.contains(&(p.gx, p.gz)) { return None; }
    let d = Vec2::new(pp.x - p.x as f32 - 0.5, pp.z - p.z as f32 - 0.5).length();
    match p.kind {
        PoiKind::Shrine if d < 3.5 => Some(("Pray at the shrine".into(), Interact::Shrine(p))),
        PoiKind::Altar if d < 4.5 && cx.g.dir.trial.is_none() => Some(("Awaken the altar: survive its trial for a relic".into(), Interact::Altar(p))),
        _ => None,
    }
}
pub fn interact(cx: &mut Ctx, it: Interact) { match it { Interact::Chest(i) => open_chest(cx, i), Interact::Shrine(p) => use_shrine(cx, &p), Interact::Altar(p) => begin_trial(cx, &p) } }

/// ruins and camps get guards and a chest when you come near; shrines and altars give off motes
pub fn update_pois(cx: &mut Ctx) {
    let pp = cx.p.body.pos;
    let (gx0, gz0) = ((pp.x / 64.0).floor() as i32, (pp.z / 64.0).floor() as i32);
    for gz in gz0 - 1..=gz0 + 1 { for gx in gx0 - 1..=gx0 + 1 {
        let Some(p) = cx.w.poi(gx, gz) else { continue };
        let key = (gx, gz);
        let (px, py, pz) = (p.x as f32 + 0.5, p.y as f32, p.z as f32 + 0.5);
        let d = Vec2::new(pp.x - px, pp.z - pz).length();
        let used = cx.g.dir.used.contains(&key);
        if p.kind == PoiKind::Shrine && !used && d < 60.0 && cx.r() < 0.5 { let an = cx.rr(0.0, TAU); let v = Vec3::new(0.0, cx.rr(0.5, 1.5), 0.0); cx.fx.emit(true, Vec3::new(px + an.cos() * 0.6, py + 2.6, pz + an.sin() * 0.6), v, 1.2, 0.4, 0.05, crate::fx::hexc(0xb98aff) * 2.5, 0.0, 0.0, 1.0); }
        if p.kind == PoiKind::Altar && !used && d < 60.0 && cx.r() < 0.6 {
            let k = (cx.r() * 4.0) as i32; let (ox, oz) = (if k & 1 == 1 { 4.0 } else { -4.0 }, if k & 2 == 2 { 4.0 } else { -4.0 });
            let v = Vec3::new(0.0, cx.rr(0.8, 2.0), 0.0); cx.fx.emit(true, Vec3::new(px + ox, py + 5.0, pz + oz), v, 1.2, 0.4, 0.05, crate::fx::hexc(0xff3a4a) * 2.5, 0.0, 0.0, 1.0);
        }
        let looted = cx.g.dir.cleared.contains(&key);
        if matches!(p.kind, PoiKind::Ruin | PoiKind::Camp) {
            if !cx.g.dir.chest_out.contains(&key) && d < 52.0 && !looted {
                cx.g.dir.chest_out.insert(key);
                let cp = if p.kind == PoiKind::Ruin { Vec3::new(px, py + 1.0, pz) } else { Vec3::new(px + 2.0, py + 1.0, pz + 1.0) };
                let yaw = crate::noise::hash2(gx, gz, 7) * TAU;
                let (root, lid) = chest_model(cx, cp, yaw);
                cx.g.dir.chests.push(Chest { root, lid, pos: cp, open: false, t: 0.0, relic: false, poi: Some(key) });
            }
            if cx.g.dir.chest_out.contains(&key) && d > 110.0 {
                let mut i = 0;
                while i < cx.g.dir.chests.len() { if cx.g.dir.chests[i].poi == Some(key) && !cx.g.dir.chests[i].open { let c = cx.g.dir.chests.swap_remove(i); cx.c.entity(c.root).despawn_recursive(); } else { i += 1; } }
                cx.g.dir.chest_out.remove(&key);
            }
            if !cx.g.dir.guarded.contains(&key) && !looted && d < 52.0 {
                cx.g.dir.guarded.insert(key);
                let mut kinds = if p.kind == PoiKind::Ruin { vec![Kind::Skeleton, Kind::Skeleton, Kind::Zombie] } else { vec![Kind::Zombie, Kind::Zombie, Kind::Skeleton, Kind::Zombie] };
                if p.kind == PoiKind::Ruin && cx.dc.day >= 3 && crate::noise::hash2(gx, gz, cx.w.seed.wrapping_add(999)) < 0.4 { kinds.push(Kind::Golem); }
                for k in kinds {
                    let an = cx.rr(0.0, TAU); let r = if k == Kind::Golem { 0.0 } else { cx.rr(2.0, 5.0) };
                    let s = cx.w.surface_spot(px + an.cos() * r, pz + an.sin() * r).unwrap_or(Vec3::new(px, py + 1.0, pz));
                    spawn_enemy(cx, k, s, SpawnOpts { poi: Some(key), ..default() });
                }
            }
        }
    } }
    // chests: drop in, idle sparkle, lids swing open
    let dt = cx.dt;
    for i in 0..cx.g.dir.chests.len() {
        let (root, lid, pos, open) = { let c = &mut cx.g.dir.chests[i]; c.t += dt; (c.root, c.lid, c.pos, c.open) };
        let t = cx.g.dir.chests[i].t;
        if t < 0.0 { if let Ok(mut tr) = cx.q.get_mut(root) { tr.translation = pos + Vec3::Y * (-t / 0.35).powi(2) * 6.0; } continue; }
        if !open && t < dt * 1.5 && t >= 0.0 && cx.g.dir.chests[i].poi.is_none() {
            if let Ok(mut tr) = cx.q.get_mut(root) { tr.translation = pos; }
            cx.play("heavy"); cx.fx.smoke(pos, 0xc8b89a, 6, 1.5, 0.8, 0.8); cx.gfx().ring(pos, 0xffd35a, 0.3, 2.5, 0.4);
        }
        if open { if let Ok(mut tr) = cx.q.get_mut(lid) { tr.rotation = Quat::from_rotation_x(-(t * 6.0).min(1.9)); } }
        else if cx.r() < 0.1 { let j = Vec3::new(cx.rr(-0.4, 0.4), 0.6, cx.rr(-0.4, 0.4)); cx.fx.emit(true, pos + j, Vec3::Y * 0.8, 0.8, 0.25, 0.05, crate::fx::hexc(0xffd35a) * 2.5, 0.0, 0.0, 0.9); }
    }
    // opened chests fade after a while
    let mut i = 0;
    while i < cx.g.dir.chests.len() { if cx.g.dir.chests[i].open && cx.g.dir.chests[i].t > 30.0 { let c = cx.g.dir.chests.swap_remove(i); cx.c.entity(c.root).despawn_recursive(); } else { i += 1; } }
    update_trial(cx);
}
