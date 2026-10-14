//! Relics: auto-firing weapons any class picks up on level-up (up to 4 at once, 5 ranks each), so a build keeps
//! fighting on its own while you aim your abilities. Ranks live in the mods map as "r.<id>". Every relic is a real
//! object in the world: steel blades orbiting you, a spinning axe that flies out and back, a corked flask that
//! shatters into flame, a wisp that floats at your shoulder, a crown of ice shards.
use crate::combat::*;
use crate::fx::hexc;
use crate::game::*;
use crate::models::*;
use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;
use std::collections::HashMap;
use std::f32::consts::{PI, TAU};

pub struct Decoy { pub pos: Vec3, pub t: f32, pub hp: f32, pub e: Entity }

pub struct RelicInfo { pub id: &'static str, pub name: &'static str, pub color: u32, pub ranks: [&'static str; 5] }
pub const RELICS: [RelicInfo; 7] = [
    RelicInfo { id: "blades", name: "Whirling Blades", color: 0xcfe6ff, ranks: ["Two blades orbit you and cut whatever they touch.", "+1 blade.", "+35% blade damage.", "+1 blade and a wider orbit.", "+1 blade, and they spin 40% faster."] },
    RelicInfo { id: "thunder", name: "Thunder Call", color: 0x9fd8ff, ranks: ["Lightning strikes a nearby enemy every 2.5s.", "+1 strike per volley.", "Strikes come 30% more often.", "+1 strike and +30% damage.", "Every strike chains to two more enemies."] },
    RelicInfo { id: "aura", name: "Ember Aura", color: 0xff9a4a, ranks: ["Embers sear every enemy within 3m of you.", "+20% radius.", "+40% damage.", "The aura also slows what it burns.", "+25% radius and +40% damage."] },
    RelicInfo { id: "wisp", name: "Spirit Wisp", color: 0xb8f0ff, ranks: ["A wisp follows you and fires at the nearest enemy.", "Fires 35% faster.", "Fires two bolts at a time.", "+40% damage.", "A second wisp joins, and bolts pierce."] },
    RelicInfo { id: "axe", name: "Ricochet Axe", color: 0xe8c890, ranks: ["Hurl a spinning axe that flies out and comes back.", "+1 axe.", "+35% damage and bigger axes.", "+1 axe.", "Throw 35% more often."] },
    RelicInfo { id: "flask", name: "Firebomb Flask", color: 0xff6a3a, ranks: ["Lob a flask at the horde that leaves a burning pool.", "+1 flask.", "Pools are 30% larger and last longer.", "+1 flask and +30% damage.", "Pools burn hotter and slow."] },
    RelicInfo { id: "frost", name: "Frost Halo", color: 0x9fe6ff, ranks: ["A ring of ice shards bursts from you every 4s.", "+4 shards.", "Shards chill: three chills freeze.", "+40% damage.", "Bursts come 40% faster and shards pierce."] },
];
pub const RMAX: u32 = 5;
pub const RSLOTS: usize = 4;

#[derive(Default)]
pub struct RelicState {
    pub decoys: Vec<Decoy>, pub stuck: Vec<Stuck>, pub t: HashMap<&'static str, f32>, pub objs: HashMap<&'static str, Vec<Entity>>, pub hit_t: HashMap<u32, f32>, pub ang: f32,
    pub wisp_pos: Vec<Vec3>,
}
pub fn rk(g: &Game, k: &str) -> u32 { g.m(&format!("r.{}", k)) }
pub fn relic_count(g: &Game) -> usize { RELICS.iter().filter(|r| rk(g, r.id) > 0).count() }

fn blade_mesh(cx: &mut Ctx) -> Entity {
    let mut b = cx.builder(); b.shadow = false;
    let root = b.root(Vec3::ZERO);
    let v = Vec3::new;
    b.bx(root, v(0.14, 0.05, 0.9), 0xd8e2ec, v(0.0, 0.0, 0.25)); b.glow(root, v(0.22, 0.06, 0.18), 0xcfe6ff, v(0.0, 0.0, 0.72));
    b.bx(root, v(0.32, 0.07, 0.07), 0xb08a3a, v(0.0, 0.0, -0.22)); b.bx(root, v(0.06, 0.06, 0.22), 0x4a3a2a, v(0.0, 0.0, -0.36));
    root
}
fn axe_mesh(cx: &mut Ctx, big: bool) -> Entity {
    let mut b = cx.builder(); b.shadow = false;
    let root = b.root(Vec3::ZERO);
    let inner = b.pivot(root, Vec3::ZERO);
    let v = Vec3::new;
    b.bx(inner, v(0.07, 0.8, 0.07), 0x6a4528, Vec3::ZERO); b.bx(inner, v(0.06, 0.3, 0.42), 0xc9d0d8, v(0.0, 0.32, 0.12)); b.glow(inner, v(0.07, 0.34, 0.06), 0xe8f4ff, v(0.0, 0.32, 0.34));
    b.bx(inner, v(0.09, 0.09, 0.09), 0x3a2618, v(0.0, -0.4, 0.0));
    if big { b.c.entity(root).insert(Transform::from_scale(Vec3::splat(1.35))); }
    root
}
fn flask_mesh(cx: &mut Ctx) -> Entity {
    let mut b = cx.builder(); b.shadow = false;
    let root = b.root(Vec3::ZERO);
    let v = Vec3::new;
    b.bxk(root, v(0.24, 0.24, 0.24), 0xff6a3a, Vec3::ZERO, Mk::Glow); b.bxk(root, v(0.28, 0.28, 0.28), 0xd8e2ec, Vec3::ZERO, Mk::Clear);
    b.bx(root, v(0.1, 0.14, 0.1), 0xd8e2ec, v(0.0, 0.2, 0.0)); b.bx(root, v(0.08, 0.07, 0.08), 0x6a4528, v(0.0, 0.3, 0.0));
    root
}
/// keep exactly `n` persistent objects for a relic
fn objs(cx: &mut Ctx, k: &'static str, n: usize, make: fn(&mut Ctx) -> Entity) -> Vec<Entity> {
    let mut l = cx.g.rel.objs.remove(k).unwrap_or_default();
    while l.len() < n { let e = make(cx); l.push(e); }
    while l.len() > n { let e = l.pop().unwrap(); cx.c.entity(e).despawn_recursive(); }
    cx.g.rel.objs.insert(k, l.clone());
    l
}
fn tick(cx: &mut Ctx, k: &'static str, init: f32) -> f32 { let dt = cx.dt; let t = cx.g.rel.t.entry(k).or_insert(init); *t -= dt; *t }
fn reset(cx: &mut Ctx, k: &'static str, v: f32) { cx.g.rel.t.insert(k, v); }

/// a glowing bolt that seeks enemies (wisps, Arcane Surge)
pub fn seek_bolt(cx: &mut Ctx, from: Vec3, dir: Vec3, dmg: f32, col: u32, pierce: bool, size: f32, speed: f32, homing: f32) {
    let e = glow_ball(cx, col, size);
    let mut p = Proj::new(from, dir.normalize_or_zero() * speed); p.e = Some(e); p.r = 0.3; p.life = 1.8; p.homing = homing; p.orient = Orient::Fixed; p.dmg = dmg; p.pierce = pierce; p.seek = 18.0;
    p.trail = Some(Box::new(move |cx, p, _| { let v = Vec3::new(cx.rr(-0.3, 0.3), cx.rr(-0.3, 0.3), cx.rr(-0.3, 0.3)); cx.fx.emit(true, p.pos, v, 0.35, size * 1.8, 0.04, hexc(col) * 2.5, 0.0, 1.0, 0.9); }));
    p.on_hit = Some(Box::new(move |cx, p, i| { if let Some(i) = i { let s = p.pos; let d = p.dmg; cx.damage(i, d, Hit { kb: 1.5, src: Some(s), col, ..default() }); cx.fx.burst(s, col, 8, 3.0, 0.3, 0.25, 0.0, 0.0); } }));
    p.on_world = Some(Box::new(move |cx, p, _| cx.fx.burst(p.pos, col, 6, 2.0, 0.3, 0.2, 0.0, 0.0)));
    spawn_proj(cx, p);
}

fn flask_dmg(r: u32) -> f32 { let mut d = 7.0f32; if r >= 4 { d *= 1.3; } if r >= 5 { d *= 1.3; } d.floor() + if d.fract() >= 0.5 { 1.0 } else { 0.0 } }
fn flask_land(cx: &mut Ctx, pos: Vec3, rad: f32, life: f32, dmg: f32, slow: bool) {
    let mut c = pos; c.y = cx.ground(c.x, c.z, c.y + 1.0);
    cx.play("glass");
    cx.fx.burst(c + Vec3::Y * 0.4, 0xff8a3a, 26, 6.0, 0.5, 0.45, 2.0, 0.0);
    cx.gfx().debris(c + Vec3::Y * 0.4, &[0xd8e2ec, 0xff6a3a], 6, 4.0, 0.12, 5.0);
    fire_pool(cx, c, rad, life, dmg, slow);
}
pub fn update_relics(cx: &mut Ctx) {
    let dt = cx.dt;
    // decoys tick down
    for d in cx.g.rel.decoys.iter_mut() { d.t -= dt; if d.hp <= 0.0 { d.t = 0.0; } }
    let mut k = 0;
    while k < cx.g.rel.decoys.len() {
        if cx.g.rel.decoys[k].t > 0.0 { k += 1; continue; }
        let d = cx.g.rel.decoys.swap_remove(k);
        cx.c.entity(d.e).despawn_recursive();
        cx.fx.smoke(d.pos, 0xd8d0c0, 6, 1.2, 0.8, 0.8);
        cx.gfx().debris(d.pos + Vec3::Y, &[0xe0c070, 0xc8a85a, 0x6a4528], 10, 3.0, 0.12, 4.0);
    }
    let dead = cx.p.dead;
    let vis = if dead { Visibility::Hidden } else { Visibility::Inherited };
    for l in cx.g.rel.objs.values() { for e in l { cx.c.entity(*e).insert(vis); } }
    if dead || relic_count(cx.g) == 0 { return; }
    if cx.g.rel.hit_t.len() > 400 { cx.g.rel.hit_t.clear(); }
    let (cd, area) = (cx.g.st.cd, cx.g.st.area);
    let pp = cx.p.body.pos; let now = cx.g.run_t;

    // Whirling Blades: orbiting steel that cuts on contact
    let r = rk(cx.g, "blades");
    let bl = objs(cx, "blades", [0, 2, 3, 3, 4, 5][r.min(5) as usize], blade_mesh);
    if !bl.is_empty() {
        cx.g.rel.ang += dt * 3.1 * if r >= 5 { 1.4 } else { 1.0 };
        let ang = cx.g.rel.ang;
        let rad = 2.6 * if r >= 4 { 1.25 } else { 1.0 } * area; let dmg = 12.0 * if r >= 3 { 1.35 } else { 1.0 }; let n = bl.len();
        for (i, e) in bl.iter().enumerate() {
            let an = ang + i as f32 / n as f32 * TAU;
            let p = Vec3::new(pp.x + an.cos() * rad, pp.y + 1.05 + (ang * 2.0 + i as f32).sin() * 0.15, pp.z + an.sin() * rad);
            if let Ok(mut t) = cx.q.get_mut(*e) { t.translation = p; t.rotation = Quat::from_euler(EulerRot::YXZ, -an, 0.0, (ang * 3.0 + i as f32).sin() * 0.4); }
            if cx.r() < 0.5 { cx.fx.emit(true, p, Vec3::ZERO, 0.22, 0.28, 0.04, hexc(0xcfe6ff) * 2.0, 0.0, 0.0, 0.7); }
            let hits: Vec<usize> = cx.g.enemies.iter().enumerate().filter(|(_, e)| !e.dead && !e.t.passive && Vec2::new(e.body.pos.x - p.x, e.body.pos.z - p.z).length() < 0.75 + e.body.w && p.y > e.body.pos.y - 0.4 && p.y < e.body.pos.y + e.body.h + 0.4).map(|(i, _)| i).collect();
            for j in hits {
                let id = cx.g.enemies[j].id;
                if now < *cx.g.rel.hit_t.get(&id).unwrap_or(&0.0) { continue; }
                cx.g.rel.hit_t.insert(id, now + 0.42);
                cx.damage(j, dmg, Hit { kb: 4.0, src: Some(pp), col: 0xcfe6ff, ..default() });
                cx.fx.sparks(p, 0xe8f4ff, 5, 7.0);
            }
        }
    }
    // Thunder Call: lightning on random nearby foes
    let r = rk(cx.g, "thunder");
    if r > 0 && tick(cx, "thunder", 1.0) <= 0.0 {
        reset(cx, "thunder", 2.5 * if r >= 3 { 0.7 } else { 1.0 } * cd);
        let mut fs: Vec<usize> = cx.foes(pp, 20.0).into_iter().take(10).collect();
        let n = fs.len().min(1 + (r >= 2) as usize + (r >= 4) as usize);
        let dmg = 34.0 * if r >= 4 { 1.3 } else { 1.0 };
        let mut ids = vec![];
        for _ in 0..n { let k = (cx.r() * fs.len() as f32) as usize % fs.len(); let i = fs.swap_remove(k); ids.push(cx.g.enemies[i].id); }
        for id in ids {
            let Some(i) = cx.find(id) else { continue };
            let ep = cx.g.enemies[i].body.pos;
            let g = lightning_strike(cx, ep, dmg, 1.9, 0xa8dcff);
            if r >= 5 {
                let mut from = g + Vec3::Y; let mut hit = vec![id];
                for _ in 0..2 {
                    let Some(nx) = cx.foes(from, 8.0).into_iter().find(|&q| !hit.contains(&cx.g.enemies[q].id)) else { break };
                    hit.push(cx.g.enemies[nx].id);
                    let to = cx.g.enemies[nx].body.pos + Vec3::Y * cx.g.enemies[nx].body.h * 0.5;
                    cx.gfx().lightning(from, to, 0xa8dcff, 0.22, 0.2);
                    cx.damage(nx, (dmg * 0.6).round(), Hit { kb: 2.0, src: Some(from), col: 0xa8dcff, ..default() });
                    from = to;
                }
            }
        }
    }
    // Ember Aura: a ring of embers that follows you
    let r = rk(cx.g, "aura");
    let au = objs(cx, "aura", if r > 0 { 1 } else { 0 }, |cx| {
        let ring = cx.fx.ring.clone(); let disk = cx.fx.disk.clone();
        let m1 = cx.mats.add(StandardMaterial { base_color: srgb(0xff8a3a).with_alpha(0.55), emissive: lrgb(0xff8a3a, 3.0), unlit: true, alpha_mode: AlphaMode::Add, double_sided: true, cull_mode: None, ..default() });
        let m2 = cx.mats.add(StandardMaterial { base_color: srgb(0xff5a1a).with_alpha(0.1), emissive: lrgb(0xff5a1a, 0.4), unlit: true, alpha_mode: AlphaMode::Add, double_sided: true, cull_mode: None, ..default() });
        let root = cx.c.spawn((Transform::default(), Visibility::default())).id();
        let a = cx.c.spawn((Mesh3d(ring), MeshMaterial3d(m1), Transform::default(), NotShadowCaster)).id();
        let b = cx.c.spawn((Mesh3d(disk), MeshMaterial3d(m2), Transform::default(), NotShadowCaster)).id();
        cx.c.entity(root).add_children(&[a, b]); root
    });
    if let Some(&e) = au.first() {
        let rad = 3.0 * if r >= 2 { 1.2 } else { 1.0 } * if r >= 5 { 1.25 } else { 1.0 } * area; let dmg = 5.0 * if r >= 3 { 1.4 } else { 1.0 } * if r >= 5 { 1.4 } else { 1.0 };
        if let Ok(mut t) = cx.q.get_mut(e) { t.translation = pp + Vec3::Y * 0.12; t.scale = Vec3::splat(rad); t.rotation = Quat::from_rotation_y(now * 0.8); }
        for _ in 0..2 { let an = cx.rr(0.0, TAU); let rr = rad * cx.rr(0.7, 1.0); let v = Vec3::new(-an.sin() * 1.5, cx.rr(1.0, 2.5), an.cos() * 1.5); let c = hexc(cx.pick(&[0xff6a1a, 0xffb03a, 0xff3a10])) * 2.5; cx.fx.emit(true, pp + Vec3::new(an.cos() * rr, 0.2, an.sin() * rr), v, 0.6, 0.35, 0.05, c, -1.0, 0.5, 1.0); }
        if tick(cx, "aura", 0.0) <= 0.0 { reset(cx, "aura", 0.5);
            cx.aoe(pp, rad, &mut |cx, i, _| { if cx.g.enemies[i].t.passive { return; } cx.damage(i, dmg, Hit { kb: 1.2, src: Some(pp), col: 0xff9a4a, quiet: true, ..default() }); cx.status(i, Status { burn: if r >= 4 { 1.5 } else { 1.0 }, slow: if r >= 4 { 0.8 } else { 0.0 }, ..default() }); }); }
    }
    // Spirit Wisp: a lantern-like spirit at your shoulder that shoots the nearest foe
    let r = rk(cx.g, "wisp");
    let ws = objs(cx, "wisp", if r == 0 { 0 } else if r >= 5 { 2 } else { 1 }, |cx| {
        let e = glow_ball(cx, 0xb8f0ff, 0.24);
        // a little crown of orbiting motes makes it read as a creature, not a bullet
        let m = cx.pal.mat(cx.mats, 0xffffff, Mk::Glow); let cube = cx.pal.cube.clone();
        for k in 0..3 { let a = k as f32 / 3.0 * TAU; let c = cx.c.spawn((Mesh3d(cube.clone()), MeshMaterial3d(m.clone()), Transform::from_translation(Vec3::new(a.cos() * 0.4, 0.1, a.sin() * 0.4)).with_scale(Vec3::splat(0.07)), NotShadowCaster)).id(); cx.c.entity(e).add_child(c); }
        e
    });
    if !ws.is_empty() {
        while cx.g.rel.wisp_pos.len() < ws.len() { cx.g.rel.wisp_pos.push(pp + Vec3::Y * 2.3); }
        let yaw = cx.p.yaw;
        for (i, e) in ws.iter().enumerate() {
            let an = now * 1.3 + i as f32 * PI;
            let tgt = Vec3::new(pp.x + an.cos() * 1.3 - yaw.sin() * 0.6, pp.y + 2.3 + (now * 3.0 + i as f32).sin() * 0.2, pp.z + an.sin() * 1.3 - yaw.cos() * 0.6);
            let p = cx.g.rel.wisp_pos[i].lerp(tgt, (dt * 6.0).min(1.0)); cx.g.rel.wisp_pos[i] = p;
            if let Ok(mut t) = cx.q.get_mut(*e) { t.translation = p; t.rotation = Quat::from_rotation_y(now * 4.0); }
            if cx.r() < 0.6 { let v = Vec3::new(cx.rr(-0.3, 0.3), cx.rr(-0.2, 0.4), cx.rr(-0.3, 0.3)); cx.fx.emit(true, p, v, 0.5, 0.3, 0.03, hexc(0xb8f0ff) * 2.5, 0.0, 1.0, 0.9); }
        }
        if tick(cx, "wisp", 0.5) <= 0.0 {
            reset(cx, "wisp", 1.15 * if r >= 2 { 0.65 } else { 1.0 } * cd);
            let f = cx.foes(pp, 22.0);
            if !f.is_empty() {
                cx.play("arcane");
                for wi in 0..ws.len() { for k in 0..if r >= 3 { 2 } else { 1 } {
                    let e = &cx.g.enemies[f[k % f.len()]]; let from = cx.g.rel.wisp_pos[wi];
                    let d = e.body.pos + Vec3::Y * e.body.h * 0.5 - from;
                    seek_bolt(cx, from, d, (12.0f32 * if r >= 4 { 1.4 } else { 1.0 }).round(), 0xb8f0ff, r >= 5, 0.16, 30.0, 4.0);
                } }
            }
        }
    }
    // Ricochet Axe: spinning axes that fly out through the horde and come back
    let r = rk(cx.g, "axe");
    if r > 0 && tick(cx, "axe", 1.2) <= 0.0 {
        reset(cx, "axe", 2.6 * if r >= 5 { 0.65 } else { 1.0 } * cd);
        let n = 1 + (r >= 2) as usize + (r >= 4) as usize; let big = r >= 3;
        let f = cx.foes(pp, 18.0);
        cx.play("whoosh");
        for i in 0..n {
            let mut d = if let Some(&j) = f.get(i % f.len().max(1)) { let ep = cx.g.enemies[j].body.pos; Vec3::new(ep.x - pp.x, 0.0, ep.z - pp.z).normalize_or_zero() } else { Quat::from_rotation_y(i as f32 * TAU / n as f32) * cx.p.fwd() };
            if !f.is_empty() && i >= f.len() { d = Quat::from_rotation_y(if i % 2 == 1 { 0.5 } else { -0.5 }) * d; }
            let e = axe_mesh(cx, big);
            let mut p = Proj::new(pp + Vec3::Y * 1.2, d * 20.0); p.e = Some(e); p.speed = 20.0; p.r = if big { 0.9 } else { 0.7 }; p.life = 4.0; p.turn = 0.55; p.boomer = true; p.pierce = true; p.no_world = true;
            p.orient = Orient::Fixed;
            p.trail = Some(Box::new(move |cx, p, _| {
                let yaw = p.vel.x.atan2(p.vel.z);
                if let Some(e) = p.e { if let Ok(mut t) = cx.q.get_mut(e) { t.rotation = Quat::from_rotation_y(yaw) * Quat::from_rotation_x(p.age * 16.0); } }
                if cx.r() < 0.5 { cx.fx.emit(true, p.pos, Vec3::ZERO, 0.2, 0.3, 0.04, hexc(0xe8c890) * 2.0, 0.0, 0.0, 0.7); }
            }));
            p.on_hit = Some(Box::new(move |cx, p, i| { if let Some(i) = i { let s = p.pos; cx.damage(i, (22.0f32 * if big { 1.35 } else { 1.0 }).round(), Hit { kb: 5.0, src: Some(s), col: 0xe8c890, ..default() }); cx.fx.sparks(s, 0xffe0b0, 5, 7.0); } }));
            spawn_proj(cx, p);
        }
    }
    // Firebomb Flask: a corked bottle lobbed into the thick of the horde; it shatters into a burning pool
    let r = rk(cx.g, "flask");
    if r > 0 && tick(cx, "flask", 2.0) <= 0.0 {
        reset(cx, "flask", 3.4 * cd);
        let n = 1 + (r >= 2) as usize + (r >= 4) as usize;
        let f: Vec<usize> = cx.foes(pp, 16.0).into_iter().take(8).collect();
        for _ in 0..n {
            let tp = if f.is_empty() { pp + Vec3::new(cx.rr(-8.0, 8.0), 0.0, cx.rr(-8.0, 8.0)) } else { let j = cx.pick(&f); cx.g.enemies[j].body.pos };
            let from = pp + Vec3::Y * 1.6; let (tt, g) = (0.75, 20.0);
            let v = Vec3::new((tp.x - from.x) / tt, (tp.y - from.y + 0.5 * g * tt * tt) / tt, (tp.z - from.z) / tt);
            let e = flask_mesh(cx);
            let mut p = Proj::new(from, v); p.e = Some(e); p.grav = g; p.r = 0.3; p.life = 3.0; p.orient = Orient::Tumble;
            p.trail = Some(Box::new(|cx, p, _| { if cx.r() < 0.5 { cx.fx.emit(true, p.pos + Vec3::Y * 0.25, Vec3::Y * 0.5, 0.3, 0.25, 0.04, hexc(0xffb03a) * 2.5, 0.0, 0.0, 1.0); } }));
            let (prad, plife, pdmg, pslow) = (2.2 * if r >= 3 { 1.3 } else { 1.0 } * area, 3.0 + if r >= 3 { 1.5 } else { 0.0 }, flask_dmg(r), r >= 5);
            let land = move |cx: &mut Ctx, p: &mut Proj, _: Option<usize>| flask_land(cx, p.pos, prad, plife, pdmg, pslow);
            p.on_hit = Some(Box::new(land)); p.on_world = Some(Box::new(land)); p.on_expire = Some(Box::new(land));
            spawn_proj(cx, p);
        }
    }
    // Frost Halo: a crown of ice shards that bursts outward
    let r = rk(cx.g, "frost");
    if r > 0 && tick(cx, "frost", 1.5) <= 0.0 {
        reset(cx, "frost", 4.0 * if r >= 5 { 0.6 } else { 1.0 } * cd);
        let n = 10 + if r >= 2 { 4 } else { 0 }; let dmg = (11.0f32 * if r >= 4 { 1.4 } else { 1.0 }).round();
        let c = pp + Vec3::Y; cx.play("ice");
        cx.gfx().ring(pp, 0x9fe6ff, 0.5, 4.0, 0.35); cx.fx.burst(c, 0xbff4ff, 20, 5.0, 0.4, 0.35, 0.0, 0.0);
        let ang = cx.g.rel.ang;
        for i in 0..n {
            let an = i as f32 / n as f32 * TAU + ang * 0.3;
            let e = shard_mesh(cx, 0xbff4ff, 2.4);
            let mut p = Proj::new(c, Vec3::new(an.cos(), 0.0, an.sin()) * 24.0); p.e = Some(e); p.r = 0.3; p.life = 0.9; p.pierce = r >= 5;
            p.trail = Some(Box::new(|cx, p, _| cx.fx.emit(true, p.pos, Vec3::ZERO, 0.25, 0.3, 0.04, hexc(0x9fe6ff) * 2.5, 0.0, 0.0, 0.8)));
            p.on_hit = Some(Box::new(move |cx, p, i| { if let Some(i) = i { let s = p.pos; cx.damage(i, dmg, Hit { kb: 2.0, src: Some(s), col: 0x9fe6ff, ..default() }); cx.status(i, Status { slow: 2.0, chill: if r >= 3 { 1 } else { 0 }, thr: 3, ..default() }); } }));
            spawn_proj(cx, p);
        }
    }
}
