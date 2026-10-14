//! Combat machinery: projectiles (real meshes that fly, spin, home, bounce and stick into the ground),
//! timed area zones, traps, enemy missiles, and the eight primary weapons with their upgrades.
use crate::fx::*;
use crate::game::*;
use crate::models::*;
use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;
use std::f32::consts::{PI, TAU};

pub type PFn = Box<dyn FnMut(&mut Ctx, &mut Proj, Option<usize>) + Send + Sync>;
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Owner { Player, Enemy }
#[derive(Clone, Copy, PartialEq)]
pub enum Orient { Fixed, Vel, Spin(f32), Tumble }
pub struct Proj {
    pub pos: Vec3, pub vel: Vec3, pub grav: f32, pub r: f32, pub life: f32, pub age: f32, pub pierce: bool, pub no_world: bool, pub owner: Owner, pub dmg: f32,
    pub hit: Vec<u32>, pub e: Option<Entity>, pub stick: bool, pub orient: Orient, pub homing: f32, pub seek: f32, pub boomer: bool, pub back: bool, pub turn: f32,
    pub speed: f32, pub bnc: u32, pub on_hit: Option<PFn>, pub on_world: Option<PFn>, pub on_expire: Option<PFn>, pub trail: Option<PFn>, pub data: [f32; 4], pub dead: bool, pub ghost: bool,
}
impl Proj {
    pub fn new(pos: Vec3, vel: Vec3) -> Self {
        Proj { pos, vel, grav: 0.0, r: 0.3, life: 2.0, age: 0.0, pierce: false, no_world: false, owner: Owner::Player, dmg: 10.0, hit: vec![], e: None, stick: false, orient: Orient::Vel,
            homing: 0.0, seek: 12.0, boomer: false, back: false, turn: 0.0, speed: 0.0, bnc: 0, on_hit: None, on_world: None, on_expire: None, trail: None, data: [0.0; 4], dead: false, ghost: false }
    }
}
pub struct Stuck { pub e: Entity, pub t: f32 }

pub type ZFn = Box<dyn FnMut(&mut Ctx, &mut Zone) + Send + Sync>;
pub struct Zone { pub pos: Vec3, pub r: f32, pub life: f32, pub t: f32, pub every: f32, pub tick: f32, pub follow: bool, pub on_tick: Option<ZFn>, pub on_frame: Option<ZFn>, pub on_end: Option<ZFn>, pub ents: Vec<Entity>, pub n: u32, pub ghost: bool }
impl Zone { pub fn new(pos: Vec3, r: f32, life: f32) -> Self { Zone { pos, r, life, t: 0.0, every: 0.0, tick: 0.0, follow: false, on_tick: None, on_frame: None, on_end: None, ents: vec![], n: 0, ghost: false } } }

pub struct Trap { pub ghost: bool, pub pos: Vec3, pub life: f32, pub armed: f32, pub spiked: u32, pub boom: u32, pub field: bool, pub mini: bool, pub root: Entity, pub jaws: [Entity; 2], pub snap: f32 }

pub fn spawn_proj(cx: &mut Ctx, mut p: Proj) {
    if cx.g.projs.len() >= 300 { let o = cx.g.projs.remove(0); if let Some(e) = o.e { cx.c.entity(e).despawn_recursive(); } }
    if p.owner == Owner::Player && p.bnc == 0 && !cx.g.net.ghost { p.bnc = cx.m("x.bounce"); }
    p.ghost = cx.g.net.ghost;
    cx.g.projs.push(p);
}
pub fn zone(cx: &mut Ctx, mut z: Zone) {
    z.ghost = cx.g.net.ghost; if cx.g.zones.len() >= 60 { let mut o = cx.g.zones.remove(0); for e in o.ents.drain(..) { cx.c.entity(e).despawn_recursive(); } } cx.g.zones.push(z); }

// ---------------- meshes for projectiles ----------------
pub fn arrow_mesh(cx: &mut Ctx, shaft: u32, tip: u32, glow: Option<u32>) -> Entity {
    let mut b = cx.builder(); b.shadow = false;
    let root = b.root(Vec3::ZERO);
    b.bx(root, Vec3::new(0.045, 0.045, 0.8), shaft, Vec3::ZERO);
    if let Some(g) = glow { b.glow(root, Vec3::new(0.09, 0.09, 0.14), g, Vec3::new(0.0, 0.0, 0.43)); } else { b.bx(root, Vec3::new(0.08, 0.08, 0.14), tip, Vec3::new(0.0, 0.0, 0.43)); }
    b.bx(root, Vec3::new(0.16, 0.02, 0.14), 0xf0f0f0, Vec3::new(0.0, 0.0, -0.34)); b.bx(root, Vec3::new(0.02, 0.16, 0.14), 0xe0e0e0, Vec3::new(0.0, 0.0, -0.34));
    root
}
pub fn glow_ball(cx: &mut Ctx, hex: u32, r: f32) -> Entity {
    let core = cx.pal.mat(cx.mats, 0xffffff, Mk::Glow);
    let shell = cx.pal.mat(cx.mats, hex, Mk::Glow);
    let sph = cx.pal.sphere.clone();
    let root = cx.c.spawn((Transform::default(), Visibility::default())).id();
    let a = cx.c.spawn((Mesh3d(sph.clone()), MeshMaterial3d(core), Transform::from_scale(Vec3::splat(r * 0.5)), NotShadowCaster)).id();
    let b = cx.c.spawn((Mesh3d(sph), MeshMaterial3d(shell), Transform::from_scale(Vec3::splat(r)), NotShadowCaster)).id();
    cx.c.entity(root).add_children(&[a, b]);
    root
}
pub fn shard_mesh(cx: &mut Ctx, hex: u32, s: f32) -> Entity {
    let m = cx.pal.mat(cx.mats, hex, Mk::Glow); let o = cx.pal.octa.clone();
    let root = cx.c.spawn((Transform::default(), Visibility::default())).id();
    let a = cx.c.spawn((Mesh3d(o), MeshMaterial3d(m), Transform::from_scale(Vec3::new(0.1 * s, 0.1 * s, 0.32 * s)), NotShadowCaster)).id();
    cx.c.entity(root).add_child(a);
    root
}

// ---------------- the projectile loop ----------------
pub fn update_projs(cx: &mut Ctx) {
    let dt = cx.dt;
    let mut list = std::mem::take(&mut cx.g.projs);
    for p in list.iter_mut() {
        if p.dead { continue; }
        cx.g.net.ghost = p.ghost;
        p.age += dt;
        let mut end = false;
        if p.homing > 0.0 {
            let tgt = if p.owner == Owner::Enemy { Some(cx.p.body.pos + Vec3::Y) } else {
                let mut best = None; let mut bd = p.seek;
                for e in cx.g.enemies.iter() { if e.dead || e.t.passive || e.t.prop || p.hit.contains(&e.id) { continue; } let d = (e.body.pos + Vec3::Y * e.body.h * 0.5).distance(p.pos); if d < bd { bd = d; best = Some(e.body.pos + Vec3::Y * e.body.h * 0.5); } }
                best };
            if let Some(t) = tgt { let sp = p.vel.length(); let want = (t - p.pos).normalize() * sp; p.vel = p.vel.lerp(want, (p.homing * dt).min(1.0)).normalize() * sp; }
        }
        // boomerangs fly out, slow, then turn home to the thrower and can hit everything again
        if p.boomer {
            if !p.back && p.age > p.turn { p.back = true; p.hit.clear(); }
            if p.back { let d = cx.p.body.pos + Vec3::Y * 1.2 - p.pos; let l = d.length(); if l < 1.3 { end = true; } else { p.vel = p.vel.lerp(d / l * p.speed.max(16.0), (dt * 7.0).min(1.0)); } }
            else { p.vel *= 1.0 - dt * 1.2; }
        }
        let n = ((p.vel.length() * dt) / 0.3).ceil().max(1.0) as i32;
        let sdt = dt / n as f32;
        for _ in 0..n {
            if end { break; }
            p.vel.y -= p.grav * sdt;
            p.pos += p.vel * sdt;
            if !p.no_world && cx.w.peek(p.pos.x.floor() as i32, p.pos.y.floor() as i32, p.pos.z.floor() as i32) != 0 {
                p.pos -= p.vel * sdt * 0.5;
                if let Some(mut f) = p.on_world.take() { f(cx, p, None); p.on_world = Some(f); }
                end = true; break;
            }
            match p.owner {
                Owner::Player => {
                    let mut hit_i = None;
                    for (i, e) in cx.g.enemies.iter().enumerate() {
                        if e.dead || e.ai.hidden || p.hit.contains(&e.id) { continue; }
                        let hx = Vec2::new(e.body.pos.x - p.pos.x, e.body.pos.z - p.pos.z).length();
                        if hx < e.body.w + p.r + 0.1 && p.pos.y > e.body.pos.y - p.r && p.pos.y < e.body.pos.y + e.body.h + p.r { hit_i = Some(i); break; }
                    }
                    if let Some(i) = hit_i {
                        p.hit.push(cx.g.enemies[i].id);
                        if let Some(mut f) = p.on_hit.take() { f(cx, p, Some(i)); p.on_hit = Some(f); }
                        if !p.pierce { if p.bnc > 0 && bounce(cx, p) { p.bnc -= 1; } else { end = true; break; } }
                    }
                }
                Owner::Enemy => {
                    let b = &cx.p.body;
                    if !cx.p.dead && Vec2::new(b.pos.x - p.pos.x, b.pos.z - p.pos.z).length() < b.w + p.r && p.pos.y > b.pos.y - p.r && p.pos.y < b.pos.y + b.h + p.r {
                        if let Some(mut f) = p.on_hit.take() { f(cx, p, None); p.on_hit = Some(f); }
                        end = true; break;
                    }
                    // decoys soak shots
                    let pos = p.pos;
                    if let Some(d) = cx.g.rel.decoys.iter_mut().find(|d| d.t > 0.0 && d.pos.distance(pos) < 1.2) { d.hp -= p.dmg; end = true; break; }
                }
            }
        }
        if !end && p.age > p.life { if let Some(mut f) = p.on_expire.take() { f(cx, p, None); p.on_expire = Some(f); } end = true; }
        if let Some(e) = p.e { if let Ok(mut t) = cx.q.get_mut(e) {
            t.translation = p.pos;
            match p.orient {
                Orient::Vel => if p.vel.length_squared() > 0.01 { t.rotation = Transform::IDENTITY.looking_to(p.vel.normalize(), Vec3::Y).rotation * Quat::from_rotation_y(PI); },
                Orient::Spin(r) => t.rotation = Quat::from_rotation_y(p.age * r),
                Orient::Tumble => t.rotation = Quat::from_euler(EulerRot::XYZ, p.age * 7.0, p.age * 5.0, 0.0),
                Orient::Fixed => {}
            }
        } }
        if !end { if let Some(mut f) = p.trail.take() { f(cx, p, None); p.trail = Some(f); } }
        if end {
            p.dead = true;
            if let Some(e) = p.e { if p.stick { cx.g.rel.stuck.push(Stuck { e, t: 4.0 }); if cx.g.rel.stuck.len() > 120 { let o = cx.g.rel.stuck.remove(0); cx.c.entity(o.e).despawn_recursive(); } } else { cx.c.entity(e).despawn_recursive(); } }
        }
    }
    cx.g.net.ghost = false;
    list.retain(|p| !p.dead);
    list.append(&mut cx.g.projs);
    cx.g.projs = list;
    let mut i = 0;
    while i < cx.g.rel.stuck.len() { cx.g.rel.stuck[i].t -= dt; if cx.g.rel.stuck[i].t <= 0.0 { let s = cx.g.rel.stuck.swap_remove(i); cx.c.entity(s.e).despawn_recursive(); } else { i += 1; } }
}
fn bounce(cx: &mut Ctx, p: &mut Proj) -> bool {
    let mut best = None; let mut bd = 12.0;
    for e in cx.g.enemies.iter() { if e.dead || e.t.passive || e.t.prop || p.hit.contains(&e.id) { continue; } let d = Vec2::new(e.body.pos.x - p.pos.x, e.body.pos.z - p.pos.z).length(); if d < bd { bd = d; best = Some(e.body.pos + Vec3::Y * e.body.h * 0.5); } }
    let Some(t) = best else { return false };
    let sp = p.vel.length().max(18.0); p.vel = (t - p.pos).normalize() * sp; p.age = p.age.min(p.life * 0.5);
    cx.fx.sparks(p.pos, 0xff7ae8, 5, 6.0);
    true
}

pub fn update_zones(cx: &mut Ctx) {
    let dt = cx.dt;
    let mut list = std::mem::take(&mut cx.g.zones);
    for z in list.iter_mut() {
        cx.g.net.ghost = z.ghost;
        z.t += dt;
        if z.follow { z.pos = cx.p.body.pos; }
        if let Some(mut f) = z.on_frame.take() { f(cx, z); z.on_frame = Some(f); }
        if z.every > 0.0 { z.tick -= dt; if z.tick <= 0.0 { z.tick += z.every; z.n += 1; if let Some(mut f) = z.on_tick.take() { f(cx, z); z.on_tick = Some(f); } } }
        if z.t >= z.life { if let Some(mut f) = z.on_end.take() { f(cx, z); } for e in z.ents.drain(..) { cx.c.entity(e).despawn_recursive(); } }
    }
    cx.g.net.ghost = false;
    list.retain(|z| z.t < z.life);
    list.append(&mut cx.g.zones);
    cx.g.zones = list;
}

// ---------------- enemy missiles ----------------
#[derive(Clone, Copy, PartialEq)]
pub enum EProj { Arrow, Dark, Rock }
pub fn fire_enemy_proj(cx: &mut Ctx, k: EProj, from: Vec3, v: Vec3, dmg: f32) {
    if cx.g.net.host() { cx.g.net.send(crate::net::Msg::Fx { k: 5, a: from.to_array(), b: v.to_array(), f: [dmg, match k { EProj::Arrow => 0.0, EProj::Dark => 1.0, EProj::Rock => 2.0 }, 0.0, 0.0] }); }
    let e = match k { EProj::Arrow => arrow_mesh(cx, 0xc8c2b0, 0x5a5a5a, None), EProj::Dark => glow_ball(cx, 0xb04aff, 0.35), EProj::Rock => {
        let m = cx.pal.mat(cx.mats, 0x7d8088, Mk::Lit); let ico = cx.pal.ico.clone(); let root = cx.c.spawn((Transform::default(), Visibility::default())).id();
        let r = cx.c.spawn((Mesh3d(ico), MeshMaterial3d(m), Transform::from_scale(Vec3::splat(0.75)))).id(); cx.c.entity(root).add_child(r); root } };
    let mut p = Proj::new(from, v);
    p.owner = Owner::Enemy; p.dmg = dmg; p.e = Some(e);
    match k {
        EProj::Arrow => { p.grav = 9.0; p.r = 0.3; p.life = 4.0; p.stick = true;
            p.trail = Some(Box::new(|cx, p, _| { if cx.r() < 0.4 { cx.fx.emit(true, p.pos, Vec3::ZERO, 0.25, 0.2, 0.03, hexc(0xff8a6a) * 2.0, 0.0, 0.0, 0.6); } })); }
        EProj::Dark => { p.r = 0.45; p.life = 5.0; p.homing = 1.6; p.orient = Orient::Fixed;
            p.trail = Some(Box::new(|cx, p, _| { let v = Vec3::new(cx.rr(-0.5, 0.5), cx.rr(-0.5, 0.5), cx.rr(-0.5, 0.5)); let c = hexc(if cx.r() < 0.5 { 0xb04aff } else { 0x5a1a8a }) * 2.5; cx.fx.emit(true, p.pos, v, 0.5, 0.5, 0.05, c, 0.0, 1.0, 1.0); }));
            p.on_world = Some(Box::new(|cx, p, _| cx.fx.burst(p.pos, 0xb04aff, 14, 3.0, 0.35, 0.3, 0.0, 0.0))); }
        EProj::Rock => { p.grav = 14.0; p.r = 0.8; p.life = 4.0; p.orient = Orient::Tumble;
            p.trail = Some(Box::new(|cx, p, _| { if cx.r() < 0.5 { cx.fx.emit(false, p.pos, Vec3::ZERO, 0.6, 0.4, 0.9, hexc(0x8a8478), 0.0, 0.0, 0.5); } }));
            p.on_world = Some(Box::new(|cx, p, _| { rock_break(cx, p.pos); if p.pos.distance(cx.p.body.pos) < 2.5 { let d = p.dmg * 0.5; let s = p.pos; cx.hurt_player(d, Some(s), 5.0); } })); }
    }
    p.on_hit = Some(Box::new(move |cx, p, _| {
        let s = p.pos; let d = p.dmg;
        cx.hurt_player(d, Some(s), match k { EProj::Rock => 8.0, EProj::Dark => 4.0, _ => 2.0 });
        match k { EProj::Dark => cx.fx.burst(s, 0xb04aff, 20, 4.0, 0.4, 0.35, 0.0, 0.0), EProj::Rock => rock_break(cx, s), _ => {} }
    }));
    spawn_proj(cx, p);
}
fn rock_break(cx: &mut Ctx, p: Vec3) {
    cx.play("heavy");
    cx.gfx().debris(p, &[0x7d8088, 0x5c5f66], 10, 6.0, 0.3, 6.0);
    cx.fx.smoke(p, 0x8a8478, 6, 2.0, 1.0, 1.0);
    if cx.p.body.pos.distance(p) < 12.0 { cx.shake(0.2); }
}

// ---------------- primary weapons ----------------
pub fn weapon_rate(id: &str) -> f32 { match id { "longbow" => 0.25, "daggers" => 0.26, "firestaff" => 0.62, "frostwand" => 0.17, "greatsword" => 0.72, "swordshield" => 0.38, "mace" => 0.5, "scepter" => 0.42, _ => 0.4 } }
pub fn weapon_info(id: &str) -> (&'static str, &'static str, &'static str) {
    match id {
        "longbow" => ("Longbow", "bow", "Hold to draw, release to fire. Full draw pierces."),
        "daggers" => ("Twin Daggers", "daggers", "Rapid strikes. Every third hit deals double."),
        "firestaff" => ("Fire Staff", "fire", "Fireballs that explode and set enemies alight."),
        "frostwand" => ("Frost Wand", "ice", "Streams of ice shards. Five hits freeze a target."),
        "greatsword" => ("Greatsword", "gsword", "Huge, slow cleaves that knock enemies back."),
        "swordshield" => ("Sword & Shield", "shieldi", "Quick cuts. The shield blocks 30% of all damage."),
        "mace" => ("Holy Mace", "mace", "Blessed blows that heal you a little on every hit."),
        _ => ("Sacred Scepter", "scepter", "Seeking bolts of holy light."),
    }
}
/// melee stats: damage, range, arc, colour, heavy
fn melee(id: &str) -> (f32, f32, f32, u32, bool) {
    match id { "daggers" => (9.0, 2.7, 1.9, 0xd8f0ff, false), "greatsword" => (27.0, 3.7, 2.9, 0xfff1d6, true), "swordshield" => (14.0, 2.9, 2.0, 0xe8f4ff, false), _ => (16.0, 3.0, 2.1, 0xffe08a, false) }
}

pub fn update_primary(cx: &mut Ctx, lmb: bool) {
    let id = cx.p.weapon_id();
    let flurry = if id == "daggers" { 1.0 + 0.2 * cx.m("w.daggers.flurry") as f32 } else { 1.0 };
    cx.g.atk_t -= cx.dt * cx.g.st.atk_spd * flurry * crate::augments::atk_mul(cx);
    if cx.p.dash.is_some() || cx.p.leap.is_some() || cx.p.dead { return; }
    if id == "longbow" {
        if lmb && cx.g.atk_t <= 0.0 {
            if !cx.p.charging { cx.p.charging = true; cx.p.charge = 0.0; }
            cx.p.charge = (cx.p.charge + cx.dt * 1.55 * cx.g.st.charge_mul).min(1.0);
            if cx.p.charge >= 1.0 && cx.r() < 0.5 { let h = cx.hand(); let v = Vec3::new(cx.rr(-1.0, 1.0), cx.rr(-1.0, 1.0), cx.rr(-1.0, 1.0)); cx.fx.emit(true, h, v, 0.3, 0.3, 0.05, hexc(0xb8ff8a) * 2.5, 0.0, 3.0, 1.0); }
        } else if cx.p.charging { cx.p.charging = false; let c = cx.p.charge.max(0.15); fire_arrow(cx, c); cx.p.charge = 0.0; cx.g.atk_t = weapon_rate(id); let aim = cx.p.aim_pt.to_array(); let me = cx.g.net.my_id; cx.g.net.send(crate::net::Msg::Fire { id: me, charge: c, aim }); }
        return;
    }
    if !lmb || cx.g.atk_t > 0.0 { return; }
    cx.g.atk_t = weapon_rate(id);
    match id { "firestaff" | "frostwand" | "scepter" => cast_primary(cx, id), _ => melee_swing(cx, id) }
    let aim = cx.p.aim_pt.to_array(); let me = cx.g.net.my_id; cx.g.net.send(crate::net::Msg::Fire { id: me, charge: 0.0, aim });
}
/// a teammate's attack, replayed for the visuals
pub fn replay_primary(cx: &mut Ctx, charge: f32) {
    let id = cx.p.weapon_id();
    match id { "longbow" => fire_arrow(cx, charge.max(0.15)), "firestaff" | "frostwand" | "scepter" => cast_primary(cx, id), _ => melee_swing(cx, id) }
}

pub fn player_arrow(cx: &mut Ctx, from: Vec3, vel: Vec3, dmg: f32, grav: f32, pierce: bool, life: f32, shaft: u32, glow: Option<u32>, split: u32, fire: bool, blast: bool) {
    let e = arrow_mesh(cx, shaft, 0xd8dde4, glow);
    let mut p = Proj::new(from, vel);
    p.e = Some(e); p.dmg = dmg; p.grav = grav; p.pierce = pierce; p.life = life; p.r = 0.25; p.stick = true;
    p.data = [split as f32, if fire { 1.0 } else { 0.0 }, if blast { 1.0 } else { 0.0 }, 0.0];
    let gl = glow;
    p.trail = Some(Box::new(move |cx, p, _| {
        if let Some(g) = gl { cx.fx.emit(true, p.pos, Vec3::ZERO, 0.3, 0.32, 0.05, hexc(g) * 2.5, 0.0, 0.0, 0.85); }
        else if cx.r() < 0.5 { cx.fx.emit(true, p.pos, Vec3::ZERO, 0.25, 0.18, 0.03, Vec3::splat(1.5), 0.0, 0.0, 0.5); }
    }));
    p.on_hit = Some(Box::new(move |cx, p, i| {
        let Some(i) = i else { return };
        let s = p.pos; let d = p.dmg;
        cx.damage(i, d, Hit { kb: 3.0, src: Some(s), ..default() });
        cx.fx.sparks(s, gl.unwrap_or(0xffe0b0), 4, 6.0);
        if p.data[1] > 0.0 { cx.status(i, Status { burn: 3.0, ..default() }); cx.fx.burst(s, 0xff7a22, 10, 3.0, 0.3, 0.3, 0.0, 0.0); }
        if p.data[0] > 0.0 { split_arrows(cx, p, i); }
        if p.data[2] > 0.0 && !p.pierce { arrow_blast(cx, s); }
    }));
    p.on_world = Some(Box::new(|cx, p, _| { if p.data[2] > 0.0 { arrow_blast(cx, p.pos); } cx.fx.burst(p.pos, 0xc8b89a, 5, 2.0, 0.3, 0.15, 0.0, 8.0); }));
    spawn_proj(cx, p);
}
fn split_arrows(cx: &mut Ctx, p: &mut Proj, i: usize) {
    let n = p.data[0] as usize; p.data[0] = 0.0;
    let dir = Vec3::new(p.vel.x, 0.0, p.vel.z).normalize_or_zero();
    cx.fx.burst(p.pos, 0xb8ff8a, 12, 4.0, 0.3, 0.25, 0.0, 0.0);
    let hid = cx.g.enemies[i].id;
    for k in 0..n {
        let a = (k as f32 - (n as f32 - 1.0) / 2.0) * 0.55 + cx.rr(-0.1, 0.1);
        let mut d = Quat::from_rotation_y(a) * dir; d.y = 0.08; let d = d.normalize();
        let before = cx.g.projs.len();
        player_arrow(cx, p.pos + d * 0.5, d * 38.0, (p.dmg * 0.5).round().max(4.0), 6.0, false, 0.9, 0x4a7a3a, Some(0xb8ff8a), 0, false, false);
        if let Some(np) = cx.g.projs.get_mut(before) { np.hit.push(hid); }
    }
}
fn arrow_blast(cx: &mut Ctx, pos: Vec3) {
    cx.play("fire");
    cx.fx.burst(pos, 0xb8ff8a, 30, 7.0, 0.45, 0.45, 0.0, 0.0); cx.fx.sparks(pos, 0xeaffd0, 10, 10.0);
    { let mut g = cx.gfx(); g.ring(pos - Vec3::Y * 0.4, 0xb8ff8a, 0.4, 3.0, 0.35); g.flare(pos, 0xb8ff8a, 3.0, 0.25, true); }
    cx.aoe(pos, 3.0, &mut |cx, i, _| cx.damage(i, 26.0, Hit { kb: 6.0, src: Some(pos), col: 0xb8ff8a, ..default() }));
}
pub fn fire_arrow(cx: &mut Ctx, charge: f32) {
    let from = cx.hand();
    let mut dir = (cx.p.aim_pt - from).normalize(); dir.y += 0.02; let dir = dir.normalize();
    let full = charge >= 0.99;
    let sp = crate::noise::lerp(32.0, 72.0, charge);
    let dmg = crate::noise::lerp(10.0, 40.0, charge).round();
    cx.play("bow"); cx.p.cast_kick = 1.0;
    let split = if full && cx.m("w.longbow.split") > 0 { 3 } else { 0 };
    let blast = full && cx.m("w.longbow.blast") > 0;
    player_arrow(cx, from, dir * sp, dmg, if full { 3.0 } else { 9.0 }, full, 3.0, 0x6a4528, if full { Some(0xb8ff8a) } else { None }, split, false, blast);
    if full { cx.fx.burst(from, 0xb8ff8a, 12, 4.0, 0.3, 0.3, 0.0, 0.0); cx.shake(0.08); }
    let extra = 2 * cx.m("w.longbow.multi");
    for k in 1..=extra { let a = if k % 2 == 1 { 1.0 } else { -1.0 } * ((k + 1) / 2) as f32 * 0.07; let d = Quat::from_rotation_y(a) * dir; player_arrow(cx, from, d * sp, (dmg * 0.5).round(), if full { 3.0 } else { 9.0 }, false, 3.0, 0x6a4528, None, 0, false, false); }
}

fn melee_swing(cx: &mut Ctx, id: &'static str) {
    let (dmg0, range0, arc0, col, heavy) = melee(id);
    let rate = weapon_rate(id);
    cx.p.swing_t = 0.0; cx.p.swing_dur = rate * 0.85; cx.p.swing_side *= -1.0; cx.g.hits += 1;
    let n = cx.g.hits; let yaw = cx.p.yaw; let side = cx.p.swing_side;
    cx.play(if heavy { "heavy" } else { "swing" });
    let reach = if id == "greatsword" && cx.m("w.greatsword.reach") > 0 { 1.3 } else { 1.0 };
    let (range, arc) = (range0 * reach, arc0 * if reach > 1.0 { 1.15 } else { 1.0 });
    let c = cx.p.body.pos + Vec3::Y * 1.0;
    { let mut g = cx.gfx(); g.swoosh(c, yaw, range, arc, col, if heavy { 0.3 } else { 0.2 }, if id == "greatsword" { 0.0 } else { side * 0.25 }, side); if heavy { g.swoosh(c + Vec3::Y * 0.15, yaw, range * 0.8, arc * 0.9, 0xffffff, 0.18, 0.0, side); } }
    if id == "greatsword" && cx.m("w.greatsword.wave") > 0 { sword_wave(cx, c, 0xfff1d6, 18.0, false); }
    if cx.g.buff("cry") > 0.0 && cx.m("cry.warlord") > 0 { sword_wave(cx, c, 0xff5b4c, 22.0, true); }
    if id == "daggers" && n % 3 == 0 && cx.m("w.daggers.shadow") > 0 {
        let d = cx.p.look(); let from = cx.hand();
        let e = shard_mesh(cx, 0xa98aff, 2.2);
        let mut p = Proj::new(from, d * 40.0); p.e = Some(e); p.r = 0.35; p.life = 0.8; p.pierce = true;
        p.trail = Some(Box::new(|cx, p, _| cx.fx.emit(true, p.pos, Vec3::ZERO, 0.25, 0.3, 0.05, hexc(0xa98aff) * 2.5, 0.0, 0.0, 0.9)));
        p.on_hit = Some(Box::new(|cx, p, i| { if let Some(i) = i { let s = p.pos; cx.damage(i, 16.0, Hit { kb: 2.0, src: Some(s), col: 0xa98aff, ..default() }); cx.fx.sparks(s, 0xa98aff, 6, 6.0); } }));
        spawn_proj(cx, p);
    }
    cx.later(rate * 0.22, move |cx| {
        if cx.p.dead { return; }
        let f = cx.p.fwd(); let pp = cx.p.body.pos;
        let mut hit_n = 0;
        let cand: Vec<usize> = (0..cx.g.enemies.len()).collect();
        for i in cand {
            let (ep, w, dead, hp, mhp) = { let e = &cx.g.enemies[i]; (e.body.pos, e.body.w, e.dead, e.hp, e.max_hp) };
            if dead { continue; }
            let (dx, dz, dy) = (ep.x - pp.x, ep.z - pp.z, ep.y - pp.y);
            let hd = (dx * dx + dz * dz).sqrt(); let d = hd - w;
            if d > range || dy.abs() > 2.6 { continue; }
            let ang = ((dx * f.x + dz * f.z) / hd.max(0.01)).clamp(-1.0, 1.0).acos();
            if ang > arc / 2.0 && d > 0.6 { continue; }
            let mut dm = dmg0;
            if id == "daggers" && n % 3 == 0 { dm *= 2.0; }
            if id == "mace" { dm *= 1.0 + 0.25 * cx.m("w.mace.heavy") as f32; }
            if id == "greatsword" && cx.m("w.greatsword.exec") > 0 && hp / mhp < 0.35 { dm *= 1.6; }
            cx.damage(i, dm, Hit { kb: if heavy { 9.0 } else { 4.0 }, src: Some(pp), col, ..default() });
            hit_n += 1;
            let h = cx.g.enemies.get(i).map(|e| e.body.h).unwrap_or(1.0);
            cx.fx.sparks(ep + Vec3::Y * h * 0.6, col, if heavy { 8 } else { 4 }, if heavy { 10.0 } else { 7.0 });
            if id == "daggers" && cx.m("w.daggers.venom") > 0 { cx.status(i, Status { poison: 3.0, ..default() }); }
            if id == "swordshield" && cx.m("w.swordshield.bash") > 0 && n % 3 == 0 { cx.status(i, Status { stun: 1.0, ..default() }); }
            if id == "mace" && cx.m("w.mace.thunder") > 0 && n % 3 == 0 && hit_n == 1 { lightning_strike(cx, ep, 26.0, 1.6, 0xa8dcff); }
        }
        if hit_n > 0 && id == "mace" { cx.sustain(2.0); if cx.m("w.mace.nova") > 0 { let k = cx.g.flags.entry("mace_n").or_insert(0.0); *k += 1.0; if (*k as i32) % 4 == 0 { holy_nova(cx); } } }
        if hit_n > 0 { cx.g.hitstop = cx.g.hitstop.max(if heavy { 0.07 } else { 0.035 }); cx.shake(if heavy { 0.25 } else { 0.1 }); }
    });
}
fn sword_wave(cx: &mut Ctx, c: Vec3, col: u32, dmg: f32, big: bool) {
    let d = cx.p.fwd();
    let key = ((if big { 2.6 } else { 2.2 }) * 10.0) as i32;
    let mesh = if let Some(m) = cx.fx.arcs.get(&(key, 999)) { m.clone() } else { let m = cx.meshes.add(annulus(0.55, 24, -1.1, 2.2, false)); cx.fx.arcs.insert((key, 999), m.clone()); m };
    let mat = cx.pal.mat(cx.mats, col, Mk::Glow);
    let s = if big { 2.6 } else { 2.2 };
    let root = cx.c.spawn((Transform::default(), Visibility::default())).id();
    let a = cx.c.spawn((Mesh3d(mesh), MeshMaterial3d(mat), Transform::from_scale(Vec3::splat(s)), NotShadowCaster)).id();
    cx.c.entity(root).add_child(a);
    let mut p = Proj::new(c + d, d * 24.0); p.e = Some(root); p.r = if big { 1.7 } else { 1.4 }; p.life = 0.6; p.pierce = true; p.no_world = true; p.orient = Orient::Fixed;
    let yaw = d.x.atan2(d.z);
    if let Ok(mut t) = cx.q.get_mut(root) { t.rotation = Quat::from_rotation_y(yaw); }
    cx.c.entity(root).insert(Transform::from_translation(c + d).with_rotation(Quat::from_rotation_y(yaw)));
    p.trail = Some(Box::new(move |cx, p, _| { let j = Vec3::new(cx.rr(-1.0, 1.0), 0.0, cx.rr(-1.0, 1.0)); cx.fx.emit(true, p.pos + j, Vec3::ZERO, 0.25, 0.35, 0.05, hexc(col) * 2.5, 0.0, 0.0, 0.8); }));
    p.on_hit = Some(Box::new(move |cx, p, i| { if let Some(i) = i { let s = p.pos; cx.damage(i, dmg, Hit { kb: 5.0, src: Some(s), col, ..default() }); } }));
    spawn_proj(cx, p);
}
pub fn holy_nova(cx: &mut Ctx) {
    let c = cx.p.body.pos; cx.play("holy");
    { let mut g = cx.gfx(); g.ring(c, 0xffe48a, 0.5, 4.5, 0.4); g.flare(c + Vec3::Y, 0xfff3b8, 4.0, 0.3, true); }
    cx.fx.burst(c + Vec3::Y, 0xffe48a, 30, 6.0, 0.5, 0.35, 0.0, 0.0);
    cx.aoe(c, 4.5, &mut |cx, i, _| cx.damage(i, 20.0, Hit { kb: 3.0, src: Some(c), col: 0xffe48a, ..default() }));
    cx.sustain(8.0);
}

pub fn lightning_strike(cx: &mut Ctx, pos: Vec3, dmg: f32, r: f32, col: u32) -> Vec3 {
    let mut g = pos; g.y = cx.ground(g.x, g.z, g.y + 1.5);
    let top = Vec3::new(g.x + cx.rr(-3.0, 3.0), g.y + 24.0, g.z + cx.rr(-3.0, 3.0));
    { let mut gx = cx.gfx(); gx.lightning(top, g + Vec3::Y * 0.1, col, 0.3, 0.32); gx.ring(g, col, 0.3, r * 1.7, 0.3); gx.flare(g + Vec3::Y * 0.6, col, 4.0, 0.25, true); gx.decal(g, false, 1.4, 4.0, None); }
    cx.fx.sparks(g + Vec3::Y * 0.3, 0xffffff, 10, 9.0); cx.fx.burst(g + Vec3::Y * 0.3, col, 14, 4.0, 0.35, 0.35, 0.0, 0.0);
    cx.play("zap"); cx.shake(0.1);
    cx.aoe(g, r, &mut |cx, i, _| { cx.damage(i, dmg, Hit { kb: 2.0, up: 3.0, src: Some(g), col, ..default() }); cx.status(i, Status { stun: 0.35, ..default() }); });
    g
}

pub fn explode_fire(cx: &mut Ctx, pos: Vec3, rad: f32, dmg: f32, cluster: u32, small: bool) {
    cx.play("fire");
    cx.fx.burst(pos, 0xff7a22, if small { 16 } else { 40 }, rad * 2.6, 0.55, if small { 0.5 } else { 0.7 }, 0.0, 0.0);
    cx.fx.burst(pos, 0xffd27a, if small { 6 } else { 16 }, rad * 1.5, 0.35, 0.5, 0.0, 0.0);
    cx.fx.smoke(pos, 0x3a3330, if small { 3 } else { 8 }, 1.4, 1.4, 1.2);
    { let mut g = cx.gfx(); g.ring(pos - Vec3::Y * 0.4, 0xff8a3a, 0.4, rad, 0.35); g.flare(pos, 0xffa050, if small { 2.5 } else { 5.0 }, 0.3, false); }
    if !small { let gy = cx.ground(pos.x, pos.z, pos.y + 1.0); if (gy - pos.y).abs() < 3.0 { cx.gfx().decal(Vec3::new(pos.x, gy, pos.z), false, rad * 0.7, 6.0, Some(0xff6a1a)); } cx.shake(0.12); }
    let fr = cx.g.st.fire_rad;
    cx.aoe(pos, rad, &mut |cx, i, _| { cx.damage(i, dmg, Hit { kb: if small { 3.0 } else { 6.0 }, src: Some(pos), col: 0xff8a3a, ..default() }); cx.status(i, Status { burn: 3.0 * fr, ..default() }); });
    if cluster > 0 {
        let n = 1 + 2 * cluster;
        for k in 0..n {
            let a = k as f32 / n as f32 * TAU + cx.rr(-0.3, 0.3);
            let v = Vec3::new(a.cos() * cx.rr(5.0, 8.0), cx.rr(5.0, 8.0), a.sin() * cx.rr(5.0, 8.0));
            let e = glow_ball(cx, 0xff8a2a, 0.16);
            let mut p = Proj::new(pos + Vec3::Y * 0.4, v); p.e = Some(e); p.grav = 18.0; p.life = 1.6; p.orient = Orient::Fixed;
            p.trail = Some(Box::new(|cx, p, _| cx.fx.emit(true, p.pos, Vec3::Y * 0.5, 0.3, 0.3, 0.05, hexc(0xffb03a) * 2.5, 0.0, 0.0, 1.0)));
            let boom: fn(&mut Ctx, &mut Proj, Option<usize>) = |cx, p, _| { let r = p.data[0]; explode_fire(cx, p.pos, r, 9.0, 0, true); };
            p.data[0] = rad * 0.55;
            p.on_hit = Some(Box::new(boom)); p.on_world = Some(Box::new(boom)); p.on_expire = Some(Box::new(boom));
            spawn_proj(cx, p);
        }
    }
}
/// burning ground: Phoenix Fire, the Firebomb relic and Trail of Ruin
pub fn fire_pool(cx: &mut Ctx, pos: Vec3, r: f32, life: f32, dmg: f32, slow: bool) {
    { let mut g = cx.gfx(); g.disk(pos, 0xff5a1a, r, life, 0.26); g.ring(pos, 0xff8a3a, 0.4, r, 0.3); g.decal(pos, false, r, life + 3.0, None); }
    let mut z = Zone::new(pos, r, life); z.every = 0.4;
    z.on_tick = Some(Box::new(move |cx, z| { let (p, rr) = (z.pos, z.r); cx.aoe(p, rr, &mut |cx, i, _| { cx.damage(i, dmg, Hit { quiet: true, col: 0xff7a22, ..default() }); cx.status(i, Status { burn: 2.0, slow: if slow { 1.0 } else { 0.0 }, ..default() }); }); }));
    z.on_frame = Some(Box::new(|cx, z| { for _ in 0..2 { let a = cx.rr(0.0, TAU); let rr = cx.r().sqrt() * z.r; let p = z.pos + Vec3::new(a.cos() * rr, 0.15, a.sin() * rr);
        let c = hexc([0xff6a1a, 0xffb03a, 0xff3a10][(cx.r() * 3.0) as usize % 3]) * 2.5; let v = Vec3::new(cx.rr(-0.3, 0.3), cx.rr(1.2, 3.0), cx.rr(-0.3, 0.3)); cx.fx.emit(true, p, v, 0.55, 0.5, 0.06, c, -1.0, 0.5, 1.0); } }));
    zone(cx, z);
}

fn cast_primary(cx: &mut Ctx, id: &str) {
    let from = cx.hand();
    let dir = (cx.p.aim_pt - from).normalize();
    cx.p.cast_kick = 1.0;
    match id {
        "firestaff" => {
            cx.play("fire");
            let rad = 3.0 * cx.g.st.fire_rad * cx.g.st.area * (1.0 + 0.3 * cx.m("w.firestaff.big") as f32);
            let cl = cx.m("w.firestaff.cluster"); let tr = cx.m("w.firestaff.trail") > 0;
            let e = glow_ball(cx, 0xff6a1a, 0.32);
            let mut p = Proj::new(from, dir * 30.0); p.e = Some(e); p.r = 0.35; p.life = 2.2; p.orient = Orient::Fixed;
            p.trail = Some(Box::new(|cx, p, _| { for _ in 0..3 { let j = Vec3::new(cx.rr(-0.1, 0.1), cx.rr(-0.1, 0.1), cx.rr(-0.1, 0.1)); let v = Vec3::new(cx.rr(-0.6, 0.6), cx.rr(0.0, 1.2), cx.rr(-0.6, 0.6));
                let c = hexc([0xff6a1a, 0xffb03a, 0xff3a10][(cx.r() * 3.0) as usize % 3]) * 3.0; cx.fx.emit(true, p.pos + j, v, 0.45, 0.6, 0.08, c, -1.0, 1.0, 1.0); }
                if cx.r() < 0.3 { cx.fx.emit(false, p.pos, Vec3::Y * 0.8, 0.8, 0.3, 0.9, hexc(0x3a3330), -0.5, 1.0, 0.4); } }));
            let boom = move |cx: &mut Ctx, p: &mut Proj, _: Option<usize>| { explode_fire(cx, p.pos, rad, 20.0, cl, false); if tr { let mut g = p.pos; g.y = cx.ground(g.x, g.z, g.y + 1.0); fire_pool(cx, g, rad * 0.75, 3.0, 5.0, false); } };
            p.on_hit = Some(Box::new(boom)); p.on_world = Some(Box::new(boom)); p.on_expire = Some(Box::new(boom));
            spawn_proj(cx, p);
            cx.fx.burst(from, 0xff8a2a, 6, 2.0, 0.25, 0.3, 0.0, 0.0);
        }
        "frostwand" => {
            cx.play("ice");
            let n = 1 + 2 * cx.m("w.frostwand.fork"); let thr = if cx.m("w.frostwand.brittle") > 0 { 3 } else { 5 }; let lance = cx.m("w.frostwand.lance") > 0;
            for k in 0..n {
                let a = if k == 0 { 0.0 } else { (if k % 2 == 1 { 1.0 } else { -1.0 }) * ((k + 1) / 2) as f32 * 0.08 };
                let d = (Quat::from_rotation_y(a) * dir + Vec3::new(cx.rr(-0.02, 0.02), cx.rr(-0.02, 0.02), 0.0)).normalize();
                let e = shard_mesh(cx, 0xbff4ff, 1.6);
                let mut p = Proj::new(from, d * 46.0); p.e = Some(e); p.r = 0.25; p.life = 1.4; p.pierce = lance;
                p.trail = Some(Box::new(|cx, p, _| { let v = Vec3::new(cx.rr(-0.3, 0.3), cx.rr(-0.3, 0.3), cx.rr(-0.3, 0.3)); cx.fx.emit(true, p.pos, v, 0.3, 0.3, 0.04, hexc(0x8fe6ff) * 2.5, 0.0, 1.0, 0.8); }));
                p.on_hit = Some(Box::new(move |cx, p, i| { if let Some(i) = i { let s = p.pos; cx.damage(i, 6.0, Hit { kb: 1.0, src: Some(s), col: 0x8fe6ff, ..default() }); cx.status(i, Status { slow: 2.0, chill: 1, thr, ..default() }); cx.fx.burst(s, 0xbff4ff, 6, 3.0, 0.3, 0.2, 0.0, 0.0); } }));
                p.on_world = Some(Box::new(|cx, p, _| cx.fx.burst(p.pos, 0xbff4ff, 6, 3.0, 0.3, 0.2, 0.0, 0.0)));
                spawn_proj(cx, p);
            }
        }
        _ => {
            cx.play("holy");
            let n = 1 + cx.m("w.scepter.twin"); let split = cx.m("w.scepter.split") > 0; let nova = cx.m("w.scepter.nova") > 0;
            for k in 0..n { let a = if k == 0 { 0.0 } else { (if k % 2 == 1 { 1.0 } else { -1.0 }) * ((k + 1) / 2) as f32 * 0.12 }; holy_bolt(cx, from, Quat::from_rotation_y(a) * dir * 30.0, 14.0, 0.24, split, nova); }
        }
    }
}
pub fn holy_bolt(cx: &mut Ctx, from: Vec3, vel: Vec3, dmg: f32, size: f32, split: bool, nova: bool) {
    let e = glow_ball(cx, 0xffd35a, size);
    let mut p = Proj::new(from, vel); p.e = Some(e); p.r = 0.35; p.life = 2.0; p.homing = 3.2; p.orient = Orient::Fixed; p.dmg = dmg;
    p.trail = Some(Box::new(move |cx, p, _| { let v = Vec3::new(cx.rr(-0.4, 0.4), cx.rr(-0.4, 0.4), cx.rr(-0.4, 0.4)); cx.fx.emit(true, p.pos, v, 0.5, size * 1.8, 0.05, hexc(0xffe48a) * 2.5, 0.0, 1.5, 0.9); }));
    p.on_hit = Some(Box::new(move |cx, p, i| {
        let Some(i) = i else { return };
        let s = p.pos; let d = p.dmg; let id = cx.g.enemies[i].id;
        cx.damage(i, d, Hit { kb: 2.0, src: Some(s), col: 0xffe48a, ..default() }); cx.fx.burst(s, 0xffe48a, 14, 4.0, 0.4, 0.3, 0.0, 0.0);
        if nova { cx.gfx().ring(s - Vec3::Y * 0.5, 0xffe48a, 0.4, 2.6, 0.3); cx.fx.sparks(s, 0xfff3b8, 8, 8.0); cx.aoe(s, 2.6, &mut |cx, j, _| { if cx.g.enemies[j].id != id { cx.damage(j, 10.0, Hit { kb: 3.0, src: Some(s), col: 0xffe48a, quiet: true, ..default() }); } }); }
        if split { for _ in 0..3 { let a = cx.rr(0.0, TAU); let before = cx.g.projs.len(); holy_bolt(cx, s, Vec3::new(a.cos(), 0.3, a.sin()) * 18.0, 7.0, 0.14, false, false); if let Some(np) = cx.g.projs.get_mut(before) { np.hit.push(id); np.life = 1.2; } } }
    }));
    p.on_world = Some(Box::new(|cx, p, _| cx.fx.burst(p.pos, 0xffe48a, 10, 3.0, 0.35, 0.3, 0.0, 0.0)));
    spawn_proj(cx, p);
}
/// Riposte: taking a hit with sword and shield blasts everything around
pub fn on_player_hurt(cx: &mut Ctx) {
    if cx.p.weapon_id() == "swordshield" && cx.m("w.swordshield.riposte") > 0 && cx.g.run_t > *cx.g.flags.get("rip").unwrap_or(&0.0) {
        let t = cx.g.run_t + 2.0; cx.g.flags.insert("rip", t);
        let c = cx.p.body.pos; cx.play("crit");
        { let mut g = cx.gfx(); g.shock(c, 0xe8f4ff, 0.5, 4.0, 0.35, 1.2); g.ring(c, 0xe8f4ff, 0.5, 4.0, 0.35); }
        cx.aoe(c, 4.0, &mut |cx, i, _| { cx.damage(i, 28.0, Hit { kb: 9.0, src: Some(c), col: 0xe8f4ff, ..default() }); cx.status(i, Status { stun: 0.6, ..default() }); });
    }
}
pub fn _unused() -> f32 { PI }
