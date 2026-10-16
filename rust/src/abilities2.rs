//! The Necromancer and the Gunslinger: their eight abilities and the Necromancer's risen skeletons.
//! Raise Dead claws real skeleton warriors (and archers) out of the ground that hunt the horde for you; Bone Spear
//! drives a line of bone spikes through the earth; Corpse Explosion detonates the bodies the horde leaves behind;
//! Soul Drain tethers enemies and pulls their life into yours. Fan the Hammer empties a revolver in a heartbeat,
//! Smoke Roll vanishes in a cloud, Dynamite is a lit bundle with a fuse, and Deadeye slows time to mark and shoot.
use crate::combat::*;
use crate::fx::hexc;
use crate::game::*;
use crate::models::*;
use crate::physics::*;
use crate::player::*;
use bevy::prelude::*;
use std::f32::consts::{PI, TAU};

pub fn cast(cx: &mut Ctx, id: &str) -> bool {
    match id {
        "raise" => raise(cx), "bonespear" => bone_spear(cx), "corpse" => corpse_explosion(cx), "drain" => soul_drain(cx),
        "fan" => fan(cx), "smoke" => smoke_roll(cx), "dynamite" => dynamite(cx), "deadeye" => deadeye(cx),
        _ => false,
    }
}

use crate::abilities::{alv, pw};
fn clamp_aim(cx: &Ctx, max: f32) -> Vec3 {
    let pp = cx.p.body.pos; let a = cx.p.aim_pt;
    let d = Vec2::new(a.x - pp.x, a.z - pp.z);
    if d.length() <= max { a } else { let n = d.normalize() * max; Vec3::new(pp.x + n.x, pp.y, pp.z + n.y) }
}
fn set_tf(cx: &mut Ctx, e: Entity, f: impl FnOnce(&mut Transform)) { if let Ok(mut t) = cx.q.get_mut(e) { f(&mut t); } }
fn flat_aim(cx: &Ctx) -> Vec3 { let d = cx.p.aim_pt - cx.p.body.pos; let d = Vec3::new(d.x, 0.0, d.z); if d.length_squared() < 0.01 { cx.p.fwd() } else { d.normalize() } }

// ================= the risen dead =================
pub struct Minion { pub body: Body, pub rig: Rig, pub yaw: f32, pub life: f32, pub atk: f32, pub archer: bool, pub ghost: bool, pub ph: f32, pub rise: f32, pub swing: f32 }
const MINION_CAP: usize = 24;
/// the army grows with Raise Dead: +1 skeleton allowed per 2 levels (up to 48)
fn minion_cap(cx: &Ctx) -> usize { MINION_CAP + (crate::abilities::alv(cx, "raise") as usize / 2).min(24) }
/// Raise Dead's level multiplier, for skeleton damage, attack speed and staying power
fn rpw(cx: &Ctx) -> f32 { crate::abilities::pw(cx, "raise") }

/// a skeleton claws up out of the ground beside you
pub fn spawn_minion(cx: &mut Ctx, at: Vec3, archer: bool) {
    let mut pos = at; pos.y = cx.ground(at.x, at.z, at.y + 2.0);
    if cx.g.minions.len() >= minion_cap(cx) { let m = cx.g.minions.remove(0); if let Some(r) = m.rig.root { cx.c.entity(r).despawn_recursive(); } }
    let rig = {
        let mut b = cx.builder();
        let r = humanoid(&mut b, pos, &Look { skin: 0xe8e4d8, shirt: 0xd8d4c6, pants: 0xd0ccbc, thin: true, eye_glow: Some(0x7affb0), ..default() });
        for i in 0..3 { b.bx(r.torso.unwrap(), Vec3::new(0.42, 0.04, 0.04), 0x8a8478, Vec3::new(0.0, 0.5 - i as f32 * 0.13, 0.12)); }
        b.glow(r.torso.unwrap(), Vec3::new(0.1, 0.1, 0.04), 0x7affb0, Vec3::new(0.0, 0.42, 0.06));
        if archer { weapon_model(&mut b, r.hand_l.unwrap(), "ebow"); } else { let w = weapon_model(&mut b, r.hand_r.unwrap(), "sword"); b.rot(w.root, 0.9, 0.0, 0.0); }
        r
    };
    let n1 = cx.g.perks.iter().any(|p| p == "Grave Pact");
    let life = (15.0 + 6.0 * cx.m("raise.long") as f32 + if n1 { 4.0 } else { 0.0 }) * rpw(cx).sqrt();
    let yaw = cx.p.yaw;
    cx.g.minions.push(Minion { body: Body::new(pos, 0.3, 1.7), rig, yaw, life, atk: 0.4, archer, ghost: cx.g.net.ghost, ph: cx.rr(0.0, TAU), rise: 0.6, swing: 1.0 });
    cx.gfx().decal(pos, true, 1.0, 3.0, Some(0x7affb0));
    cx.fx.burst(pos + Vec3::Y * 0.2, 0x7affb0, 10, 3.0, 0.4, 0.3, 2.0, 0.0);
    cx.gfx().debris(pos + Vec3::Y * 0.1, &[0x6a4a2c, 0x8a5b34], 6, 4.0, 0.14, 5.0);
}
fn minion_die(cx: &mut Ctx, m: Minion) {
    let p = m.body.pos + Vec3::Y * 0.8;
    cx.gfx().debris(p, &[0xe8e4d8, 0xd8d4c6, 0x8a8478], 10, 4.0, 0.14, 3.0);
    cx.fx.burst(p, 0x7affb0, 12, 3.0, 0.4, 0.3, 0.0, 0.0);
    // Army of the Dead: the bones burst outward
    if cx.m("raise.legion") > 0 && !m.ghost {
        cx.play("bone");
        cx.gfx().ring(m.body.pos, 0x7affb0, 0.3, 3.0, 0.3);
        let dm = (22.0 * rpw(cx)).round();
        cx.aoe(m.body.pos, 3.0, &mut |cx, i, _| cx.damage(i, dm, Hit { kb: 5.0, src: Some(p), col: 0x7affb0, quiet: true, ..default() }));
    }
    if let Some(r) = m.rig.root { cx.c.entity(r).despawn_recursive(); }
}
/// skeletons hunt the nearest enemy, archers keep their distance and shoot, and they all follow you when the field is clear
pub fn update_minions(cx: &mut Ctx) {
    if cx.g.minions.is_empty() { return; }
    let dt = cx.dt;
    let mut list = std::mem::take(&mut cx.g.minions);
    let was_ghost = cx.g.net.ghost;
    let n1 = cx.g.perks.iter().any(|p| p == "Grave Pact");
    let legion = cx.m("raise.legion") > 0;
    let (cap, haste) = (minion_cap(cx), 1.0 / rpw(cx).sqrt());
    let mut keep = vec![];
    for mut m in list.drain(..) {
        cx.g.net.ghost = m.ghost;
        m.life -= dt; m.atk -= dt; m.rise = (m.rise - dt).max(0.0); m.swing = (m.swing + dt * 3.0).min(1.0);
        if m.life <= 0.0 { minion_die(cx, m); continue; }
        let pos = m.body.pos;
        let tgt = cx.foes(pos, 22.0).into_iter().next();
        let (mut mv, mut want_yaw) = (Vec3::ZERO, m.yaw);
        if m.rise <= 0.0 {
            if let Some(i) = tgt {
                let (ep, ew, eh) = { let e = &cx.g.enemies[i]; (e.body.pos, e.body.w, e.body.h) };
                let d = Vec3::new(ep.x - pos.x, 0.0, ep.z - pos.z); let dist = d.length();
                want_yaw = d.x.atan2(d.z);
                let reach = if m.archer { 9.0 } else { 1.3 + ew };
                if dist > reach { mv = d / dist.max(0.01); } else if m.archer && dist < 5.0 { mv = -d / dist.max(0.01) * 0.6; }
                if dist <= reach + 0.3 && m.atk <= 0.0 {
                    let dmg = (9.0 * (1.0 + cx.g.minutes() * 0.08) * if n1 { 1.3 } else { 1.0 } * cx.g.flags.get("raise_pw").copied().unwrap_or(1.0)).round();
                    if m.archer {
                        m.atk = 1.3 * haste;
                        let from = pos + Vec3::Y * 1.4; let to = ep + Vec3::Y * eh * 0.6;
                        player_arrow(cx, from, (to - from).normalize() * 40.0, dmg, 3.0, false, 1.2, 0x8a8478, Some(0x7affb0), 0, false, false);
                    } else {
                        m.atk = 0.8 * haste; m.swing = 0.0;
                        cx.damage(i, dmg, Hit { kb: 3.0, src: Some(pos), col: 0x9affc0, quiet: true, ..default() });
                        let hp = ep + Vec3::Y * eh * 0.6; cx.fx.sparks(hp, 0x9affc0, 3, 5.0);
                    }
                    // Army of the Dead: their kills can rise to join them
                    if legion && cx.g.enemies[i].dead && !m.ghost && list_len(&keep, cx) < cap && cx.r() < 0.35 { let at = ep; cx.later(0.5, move |cx| spawn_minion(cx, at, false)); }
                }
            } else {
                let d = Vec3::new(cx.p.body.pos.x - pos.x, 0.0, cx.p.body.pos.z - pos.z); let dist = d.length();
                if dist > 4.0 { mv = d / dist; want_yaw = d.x.atan2(d.z); }
                if dist > 40.0 { m.body.pos = cx.p.body.pos - cx.p.fwd() * 2.0; }
            }
        }
        let speed = if m.archer { 5.0 } else { 6.2 };
        m.body.vel.x = crate::noise::lerp(m.body.vel.x, mv.x * speed, (dt * 10.0).min(1.0));
        m.body.vel.z = crate::noise::lerp(m.body.vel.z, mv.z * speed, (dt * 10.0).min(1.0));
        if m.body.hit_wall && mv != Vec3::ZERO && m.body.on_ground { m.body.vel.y = 8.6; }
        if m.rise <= 0.0 { move_body(cx.w, &mut m.body, dt, 26.0); }
        let dy = ((want_yaw - m.yaw) % TAU + TAU * 1.5) % TAU - PI;
        m.yaw += dy * (dt * 10.0).min(1.0);
        // a stiff, jerky walk; the sword arm chops on each attack
        let hs = Vec2::new(m.body.vel.x, m.body.vel.z).length();
        m.ph += dt * hs * 2.2;
        let s = m.ph.sin() * (hs / 6.0).min(1.0);
        let r = m.rig.clone();
        let mut root_pos = m.body.pos; root_pos.y -= crate::noise::smooth(0.0, 0.6, m.rise) * 1.8;
        if let Some(e) = r.root { set_tf(cx, e, |t| { t.translation = root_pos; t.rotation = Quat::from_rotation_y(m.yaw); }); }
        for (e, a) in [(r.leg_l, s * 0.6), (r.leg_r, -s * 0.6)] { if let Some(e) = e { set_tf(cx, e, |t| t.rotation = Quat::from_rotation_x(a)); } }
        let chop = if m.swing < 1.0 { (1.0 - m.swing) * 2.4 } else { 0.0 };
        if let Some(e) = r.arm_r { set_tf(cx, e, |t| t.rotation = Quat::from_rotation_x(if m.archer { -0.3 } else { -0.6 - chop + s * 0.3 })); }
        if let Some(e) = r.arm_l { set_tf(cx, e, |t| t.rotation = Quat::from_rotation_x(if m.archer { -1.5 } else { -s * 0.4 })); }
        if m.rise > 0.0 && cx.r() < 0.5 { cx.fx.emit(false, m.body.pos + Vec3::new(cx.rr(-0.4, 0.4), 0.1, cx.rr(-0.4, 0.4)), Vec3::new(cx.rr(-1.0, 1.0), 2.0, cx.rr(-1.0, 1.0)), 0.5, 0.2, 0.3, hexc(0x6a4a2c), 8.0, 1.0, 0.9); }
        if m.life < 1.0 && cx.r() < 0.3 { cx.fx.emit(true, m.body.pos + Vec3::Y * cx.rr(0.2, 1.6), Vec3::Y * 1.5, 0.4, 0.25, 0.04, hexc(0x7affb0) * 2.0, 0.0, 0.5, 0.9); }
        keep.push(m);
    }
    cx.g.net.ghost = was_ghost;
    keep.append(&mut cx.g.minions);
    cx.g.minions = keep;
}
fn list_len(keep: &[Minion], cx: &Ctx) -> usize { keep.len() + cx.g.minions.len() }

// ================= NECROMANCER =================
fn raise(cx: &mut Ctx) -> bool {
    let lv = alv(cx, "raise");
    let n = 3 + cx.m("raise.more") + lv / 2;
    let archers = if cx.m("raise.archer") > 0 { 2 } else { 0 } + lv / 4;
    cx.g.flags.insert("raise_pw", pw(cx, "raise"));
    let pp = cx.p.body.pos; let f = cx.p.fwd(); let r = cx.p.right();
    cx.play("rise"); cx.play("bone"); cx.p.pose(Pose::Raise, 0.9);
    { let c = pp + f * 2.0; let mut g = cx.gfx(); g.circle(c, 0x7affb0, 3.2, 1.2, "blood", 2.0, None, None); g.ring(c, 0x7affb0, 0.5, 4.0, 0.5); }
    for k in 0..n + archers {
        let archer = k >= n;
        let lane = k as f32 - (n + archers - 1) as f32 / 2.0;
        let at = pp + f * if archer { 0.5 } else { 2.2 } + r * lane * 1.2;
        cx.later(0.1 + k as f32 * 0.08, move |cx| spawn_minion(cx, at, archer));
    }
    true
}

/// a line of bone spikes races out of the ground toward the aim
fn bone_spear(cx: &mut Ctx) -> bool {
    let lv = alv(cx, "bonespear");
    let len = 14.0 * (1.0 + 0.5 * cx.m("bonespear.long") as f32 + 0.08 * lv as f32);
    cx.g.flags.insert("spear_pw", pw(cx, "bonespear"));
    let dir = flat_aim(cx);
    cx.play("bone"); cx.p.pose(Pose::Slam, 0.5); cx.shake(0.2);
    let lines = 1 + cx.m("bonespear.twin") + lv / 4;
    let dirs: Vec<Vec3> = (0..lines).map(|k| if lines == 1 { dir } else { Quat::from_rotation_y((k as f32 / (lines - 1) as f32 - 0.5) * (0.56 * (lines - 1) as f32).min(TAU * 0.9)) * dir }).collect();
    for d in dirs { spike_line(cx, cx.p.body.pos + d * 1.2, d, len); }
    true
}
fn spike_line(cx: &mut Ctx, from: Vec3, dir: Vec3, len: f32) {
    let wall = cx.m("bonespear.wall") > 0; let shards = cx.m("bonespear.shards") > 0;
    let dmg = (30.0 * (1.0 + cx.g.minutes() * 0.05) * cx.g.flags.get("spear_pw").copied().unwrap_or(1.0)).round();
    let n = (len / 0.9) as usize;
    let travel = n as f32 * 0.025;
    let hold = if wall { 3.0 } else { 0.7 };
    let mut z = crate::combat::Zone::new(from, 0.0, travel + hold);
    let mut hit: Vec<u32> = vec![];
    let mut spikes: Vec<(Entity, Vec3, f32, Quat, f32)> = vec![];
    let mut next = 0usize;
    let mut slow_t = 0.0f32;
    z.on_frame = Some(Box::new(move |cx, z| {
        let t = z.t;
        while next < n && t >= next as f32 * 0.025 {
            let k = next; next += 1;
            let side = Vec3::new(-dir.z, 0.0, dir.x) * if k % 2 == 0 { 0.25 } else { -0.25 };
            let mut p = from + dir * (k as f32 * 0.9) + side; p.y = cx.ground(p.x, p.z, p.y + 2.0);
            let h = cx.rr(1.4, 2.3) * if k + 2 >= n { 0.7 } else { 1.0 };
            let tilt = Quat::from_euler(EulerRot::XYZ, dir.z * 0.35 + cx.rr(-0.15, 0.15), cx.rr(0.0, TAU), -dir.x * 0.35 + cx.rr(-0.15, 0.15));
            let e = { let mut b = cx.builder(); let root = b.root(p - Vec3::Y * h);
                b.bx(root, Vec3::new(0.32, h * 0.7, 0.32), 0xe8e2d0, Vec3::new(0.0, h * 0.35, 0.0)); b.bx(root, Vec3::new(0.2, h * 0.3, 0.2), 0xf4f0e4, Vec3::new(0.0, h * 0.82, 0.0));
                b.bx(root, Vec3::new(0.1, h * 0.12, 0.1), 0xffffff, Vec3::new(0.0, h * 1.0, 0.0)); b.bx(root, Vec3::new(0.4, 0.12, 0.4), 0xb8b0a0, Vec3::new(0.0, 0.1, 0.0)); root };
            z.ents.push(e);
            spikes.push((e, p, h, tilt, t));
            cx.gfx().debris(p + Vec3::Y * 0.1, &[0x6a4a2c, 0xe8e2d0], 3, 4.0, 0.12, 5.0);
            if k % 3 == 0 { cx.play("bone"); }
            let near = cx.near(p, 1.4);
            for (i, _) in near {
                let id = cx.g.enemies[i].id; if hit.contains(&id) { continue; } hit.push(id);
                cx.damage(i, dmg, Hit { kb: 2.0, up: 8.0, src: Some(p - dir), col: 0xe8e2d0, ..default() });
                cx.status(i, Status { slow: 1.5, ..default() });
                let hp = cx.g.enemies[i].body.pos + Vec3::Y; cx.gfx().slash(hp, 0xe8e2d0, 1.6, 0.14, PI / 2.0);
            }
        }
        // spikes punch up, quiver, then sink back into the earth
        let left = z.life - t;
        for &(e, p, h, rot, born) in spikes.iter() {
            let a = t - born; let rise = 1.0 - (1.0 - (a / 0.08).min(1.0)).powi(3); let sink = (left / 0.35).clamp(0.0, 1.0);
            let k = rise.min(sink); let q = if a < 0.25 { (a * 70.0).sin() * 0.04 } else { 0.0 };
            set_tf(cx, e, |tr| { tr.translation = Vec3::new(p.x + q, p.y - h + h * k, p.z); tr.rotation = rot; });
            if shards && (a - 0.12).abs() < cx.dt * 0.5 {
                for _ in 0..3 { let an = cx.rr(0.0, TAU); let s = shard_mesh(cx, 0xe8e2d0, 1.0); let mut pr = Proj::new(p + Vec3::Y * h * 0.8, Vec3::new(an.cos(), 0.3, an.sin()) * 22.0); pr.e = Some(s); pr.r = 0.25; pr.life = 0.5; pr.grav = 10.0;
                    pr.on_hit = Some(Box::new(|cx, q, i| { if let Some(i) = i { let s = q.pos; let dm = (9.0 * cx.g.flags.get("spear_pw").copied().unwrap_or(1.0)).round(); cx.damage(i, dm, Hit { kb: 1.0, src: Some(s), col: 0xe8e2d0, quiet: true, ..default() }); } })); spawn_proj(cx, pr); }
            }
        }
        // the wall slows anything pushing through it
        if wall { slow_t -= cx.dt; if slow_t <= 0.0 { slow_t = 0.3; for &(_, p, _, _, _) in spikes.iter() { let v = cx.near(p, 1.1); for (i, _) in v { cx.status(i, Status { slow: 0.6, ..default() }); } } } }
    }));
    zone(cx, z);
}

/// every corpse near the aim bursts: a green-fire blast and a shower of gore
fn corpse_explosion(cx: &mut Ctx) -> bool {
    let c = clamp_aim(cx, 30.0);
    let lv = alv(cx, "corpse");
    let n = 4 + 2 * cx.m("corpse.more") as usize + lv as usize;
    cx.g.flags.insert("corpse_pw", pw(cx, "corpse"));
    let mut found: Vec<(f32, usize)> = cx.g.corpses.iter().enumerate().map(|(i, k)| (Vec2::new(k.pos.x - c.x, k.pos.z - c.z).length(), i)).filter(|(d, _)| *d < 10.0).collect();
    found.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let mut pts: Vec<Vec3> = vec![];
    let mut idx: Vec<usize> = found.into_iter().take(n).map(|x| x.1).collect();
    idx.sort_unstable_by(|a, b| b.cmp(a));
    for i in idx { let k = cx.g.corpses.remove(i); pts.push(k.pos); cx.c.entity(k.root).despawn_recursive(); }
    // no bodies? the necromancer makes some: bones heave out of the ground at the aim
    if pts.is_empty() { let mut g = c; g.y = cx.ground(c.x, c.z, c.y + 2.0); pts.push(g); for k in 0..cx.m("corpse.more") + lv / 2 { let a = k as f32 * 2.4; pts.push(g + Vec3::new(a.cos() * 2.5, 0.0, a.sin() * 2.5)); } }
    cx.play("squelch"); cx.p.pose(Pose::Push, 0.4); cx.p.cast_kick = 1.0;
    let depth = if cx.m("corpse.cascade") > 0 { 2 } else { 0 };
    for (k, p) in pts.into_iter().enumerate() { cx.later(0.06 * k as f32, move |cx| gore_blast(cx, p, depth)); }
    true
}
fn gore_blast(cx: &mut Ctx, p: Vec3, depth: u32) {
    let k = cx.g.flags.get("corpse_pw").copied().unwrap_or(1.0);
    let r = 3.2 * (1.0 + 0.3 * cx.m("corpse.wide") as f32 + 0.04 * (k - 1.0) / 0.08) * cx.g.st.area;
    let dmg = (42.0 * (1.0 + cx.g.minutes() * 0.06) * k).round();
    cx.play("boom"); cx.shake(0.25);
    cx.fx.burst(p + Vec3::Y * 0.6, 0x9aff6a, 30, r * 2.2, 0.5, 0.55, 2.0, 0.0);
    cx.fx.burst(p + Vec3::Y * 0.6, 0xff3a3a, 12, r * 1.6, 0.4, 0.3, 3.0, 10.0);
    cx.gfx().debris(p + Vec3::Y * 0.4, &[0x8a2a22, 0x6a1a1a, 0xe8e2d0], 10, 7.0, 0.14, 5.0);
    { let mut g = cx.gfx(); g.flare(p + Vec3::Y, 0x9aff6a, 4.0, 0.25, true); g.ring(p, 0x9aff6a, 0.4, r * 1.3, 0.35); g.decal(p, false, r * 0.7, 5.0, Some(0x5a8a2a)); g.impact(p + Vec3::Y * 0.8, 0x9aff6a, 1.6); }
    cx.fx.smoke(p, 0x4a5a3a, 5, 1.5, 1.0, 1.0);
    let mut killed = vec![];
    cx.aoe(p, r, &mut |cx, i, _| {
        cx.damage(i, dmg, Hit { kb: 7.0, up: 5.0, src: Some(p), col: 0x9aff6a, ..default() });
        cx.status(i, Status { poison: 2.0, ..default() });
        if cx.g.enemies[i].dead { killed.push(cx.g.enemies[i].body.pos); }
    });
    if cx.m("corpse.plague") > 0 {
        let mut z = crate::combat::Zone::new(p, r * 0.8, 3.0); z.every = 0.5;
        z.on_tick = Some(Box::new(|cx, z| { let (c, rr) = (z.pos, z.r); let dm = (5.0 * cx.g.flags.get("corpse_pw").copied().unwrap_or(1.0)).round(); cx.aoe(c, rr, &mut |cx, i, _| { cx.damage(i, dm, Hit { quiet: true, col: 0x9aff6a, ..default() }); cx.status(i, Status { poison: 1.5, slow: 0.6, ..default() }); }); }));
        z.on_frame = Some(Box::new(|cx, z| { if cx.r() < 0.6 { let a = cx.rr(0.0, TAU); let rr = cx.r().sqrt() * z.r; cx.fx.emit(false, z.pos + Vec3::new(a.cos() * rr, 0.3, a.sin() * rr), Vec3::Y * 0.6, 1.2, 0.5, 1.4, hexc(0x6a9a3a), -0.2, 0.5, 0.35); } }));
        zone(cx, z);
    }
    // Cascade: whatever this blast killed goes up too
    if depth > 0 { for (k, q) in killed.into_iter().take(4).enumerate() { cx.later(0.15 + 0.05 * k as f32, move |cx| gore_blast(cx, q, depth - 1)); } }
}

/// green tethers to the nearest enemies, pulling their life into yours
fn soul_drain(cx: &mut Ctx) -> bool {
    let pp = cx.p.body.pos;
    let n = 3 + 2 * cx.m("drain.more") as usize + alv(cx, "drain") as usize;
    let ids: Vec<u32> = cx.foes(pp, 16.0).into_iter().take(n).map(|i| cx.g.enemies[i].id).collect();
    if ids.is_empty() { return false; }
    cx.play("soul"); cx.p.pose(Pose::Push, 0.6); cx.p.cast_kick = 1.0;
    let life = 2.5 + 1.5 * cx.m("drain.long") as f32;
    let heal_k = 0.4 * (1.0 + 0.5 * cx.m("drain.feast") as f32);
    let harvest = cx.m("drain.harvest") > 0;
    let dmg = (8.0 * (1.0 + cx.g.minutes() * 0.06) * pw(cx, "drain")).round();
    let mut z = crate::combat::Zone::new(pp, 0.0, life); z.follow = true; z.every = 0.25;
    let ids2 = ids.clone();
    z.on_tick = Some(Box::new(move |cx, _z| {
        let mut took = 0.0;
        for &id in ids.iter() {
            let Some(i) = cx.find(id) else { continue };
            let before = cx.g.enemies[i].hp;
            cx.damage(i, dmg, Hit { quiet: true, col: 0x6adf9a, ..default() });
            took += (before - cx.g.enemies.get(i).map(|e| e.hp).unwrap_or(0.0)).max(0.0);
            if harvest && cx.g.enemies[i].dead { let at = cx.g.enemies[i].body.pos; cx.later(0.3, move |cx| spawn_minion(cx, at, false)); }
        }
        if took > 0.0 { cx.heal((took * heal_k).min(cx.p.max_hp * 0.05 * pw(cx, "drain").sqrt()), true); }
    }));
    z.on_frame = Some(Box::new(move |cx, _z| {
        let h = cx.hand();
        for &id in ids2.iter() {
            let Some(i) = cx.find(id) else { continue };
            let e = cx.g.enemies[i].body.pos + Vec3::Y * cx.g.enemies[i].body.h * 0.6;
            // souls stream from the victim into the necromancer's hand
            for _ in 0..3 { let t = cx.r(); let p = e.lerp(h, t) + Vec3::new(cx.rr(-0.15, 0.15), cx.rr(-0.15, 0.15), cx.rr(-0.15, 0.15)); cx.fx.emit(true, p, (h - e).normalize() * 9.0, 0.18, 0.28, 0.05, hexc(0x7affb0) * 2.5, 0.0, 0.0, 0.9); }
            if cx.r() < 0.12 { cx.gfx().lightning(e, h, 0x6adf9a, 0.1, 0.06); }
        }
    }));
    zone(cx, z);
    true
}

// ================= GUNSLINGER =================
/// thumb the hammer six times in a heartbeat, sweeping the muzzle across a cone
fn fan(cx: &mut Ctx) -> bool {
    let lv = alv(cx, "fan");
    let n = 6 + 3 * cx.m("fan.more") + 2 * lv;
    cx.g.flags.insert("fan_pw", pw(cx, "fan"));
    cx.p.pose(Pose::FanHammer, 0.45);
    let cone = (1.0 + 0.22 * lv as f32).min(TAU);
    fan_burst(cx, n, cone, cone >= TAU - 0.01);
    if cx.m("fan.hellfire") > 0 { cx.later(0.45, move |cx| { cx.p.pose(Pose::FanHammer, 0.45); fan_burst(cx, n + 6, TAU, true); }); }
    true
}
fn fan_burst(cx: &mut Ctx, n: u32, cone: f32, ring: bool) {
    let fire = cx.m("fan.fire") > 0;
    let dmg = (14.0 * (1.0 + cx.g.minutes() * 0.05) * (1.0 + 0.35 * cx.m("w.revolvers.hollow") as f32) * cx.g.flags.get("fan_pw").copied().unwrap_or(1.0)).round();
    for k in 0..n {
        cx.later(k as f32 * if ring { 0.025 } else { 0.045 } / (1.0 + n as f32 * 0.02), move |cx| {
            if cx.p.dead { return; }
            let dir = { let mut d = (cx.p.aim_pt - cx.hand()).normalize(); if ring { d.y = 0.02; } d.normalize() };
            let a = if ring { k as f32 / n as f32 * cone } else { (k as f32 / (n - 1).max(1) as f32 - 0.5) * cone };
            let d = Quat::from_rotation_y(a) * dir;
            let from = cx.hand();
            cx.play("gun"); muzzle(cx, from, d, false);
            bullet(cx, from, d, dmg, false, 0, false, fire);
            cx.p.cast_kick = 1.0;
        });
    }
    cx.shake(0.15);
}

/// a roll that leaves a choking cloud of smoke behind
fn smoke_roll(cx: &mut Ctx) -> bool {
    let mut d = cx.p.move_dir; if d.length_squared() < 0.01 { d = -cx.p.fwd(); }
    d.y = 0.0; let d = d.normalize();
    let pp = cx.p.body.pos;
    cx.p.dash = Some(Dash { dir: d, speed: 24.0, t: 0.3, kind: 1 });
    cx.p.roll_t = 0.3; cx.p.invuln = 0.45;
    cx.play("dash"); cx.play("whoosh");
    crate::abilities::afterimage(cx, 0xc8c0b0, 0.35);
    let lv = alv(cx, "smoke");
    let r = 4.2 * cx.g.st.area * (1.0 + 0.05 * lv as f32);
    let flash = cx.m("smoke.flash") > 0;
    // a levelled roll leaves little bombs in its wake
    for k in 0..lv / 2 { let a = k as f32 * 2.4; let at = pp + Vec3::new(a.cos(), 0.0, a.sin()) * 1.5; cx.later(0.3 + 0.12 * k as f32, move |cx| { let mut g = at; g.y = cx.ground(g.x, g.z, at.y + 2.0); let k = pw(cx, "smoke"); blast(cx, g, true, k); }); }
    if flash { cx.play("snap"); cx.gfx().flare(pp + Vec3::Y, 0xffffff, 6.0, 0.15, true); cx.aoe(pp, r, &mut |cx, i, _| cx.status(i, Status { stun: 1.2, ..default() })); }
    let mut z = crate::combat::Zone::new(pp, r, 4.0); z.every = 0.3;
    z.on_tick = Some(Box::new(|cx, z| { let (c, rr) = (z.pos, z.r); cx.aoe(c, rr, &mut |cx, i, _| { if !cx.g.enemies[i].boss { cx.status(i, Status { fear: 0.8, slow: 0.6, ..default() }); } }); }));
    z.on_frame = Some(Box::new(|cx, z| {
        let fade = ((z.life - z.t) / 1.0).clamp(0.0, 1.0);
        if cx.r() < 0.9 * fade { let a = cx.rr(0.0, TAU); let rr = cx.r().sqrt() * z.r; let p = z.pos + Vec3::new(a.cos() * rr, cx.rr(0.2, 1.8), a.sin() * rr); cx.fx.emit(false, p, Vec3::new(cx.rr(-0.3, 0.3), cx.rr(0.1, 0.5), cx.rr(-0.3, 0.3)), 1.6, 1.0, 2.2, hexc(0xb8b4ac), -0.05, 0.4, 0.55); }
    }));
    zone(cx, z);
    cx.fx.smoke(pp, 0xc8c0b0, 18, 3.0, 1.6, 1.8);
    if cx.m("smoke.caltrops") > 0 {
        { let mut g = cx.gfx(); g.decal(pp, false, 2.6, 5.0, Some(0x6a6e76)); }
        cx.gfx().debris(pp + Vec3::Y * 0.3, &[0x8a8e96, 0x5a5e66], 14, 5.0, 0.08, 3.0);
        let mut c = crate::combat::Zone::new(pp, 2.8, 5.0); c.every = 0.4;
        c.on_tick = Some(Box::new(|cx, z| { let (p, rr) = (z.pos, z.r); let dm = (5.0 * pw(cx, "smoke")).round(); cx.aoe(p, rr, &mut |cx, i, _| { cx.damage(i, dm, Hit { quiet: true, col: 0xc8c8d0, ..default() }); cx.status(i, Status { slow: 1.0, ..default() }); }); }));
        zone(cx, c);
    }
    // Ghost: vanish, and every shot finds a vital spot while you're hidden
    if cx.m("smoke.ghost") > 0 { cx.p.invuln = cx.p.invuln.max(1.2); cx.g.buffs.insert("focus", 2.0); cx.toast("Ghost", "Unseen: every shot crits", 1.5); }
    true
}

/// a lit bundle of dynamite thrown in an arc; it lands, the fuse fizzes, then it blows
fn dynamite(cx: &mut Ctx) -> bool {
    let mut to = clamp_aim(cx, 24.0); to.y = cx.ground(to.x, to.z, to.y + 2.0);
    let from = cx.hand() + Vec3::Y * 0.3;
    cx.p.pose(Pose::Throw, 0.45); cx.play("whoosh");
    cx.later(0.15, move |cx| throw_bundle(cx, from, to, false));
    let cl = cx.m("dynamite.cluster");
    cx.g.flags.insert("dyn_pw", pw(cx, "dynamite"));
    for k in 0..2 * cl + alv(cx, "dynamite") { let a = k as f32 * 2.4 + cx.rr(-0.4, 0.4); let off = Vec3::new(a.cos(), 0.0, a.sin()) * cx.rr(2.5, 4.0); let t2 = to + off; cx.later(0.2 + 0.05 * k as f32, move |cx| throw_bundle(cx, from, t2, true)); }
    true
}
fn bundle_mesh(cx: &mut Ctx, small: bool) -> Entity {
    let mut b = cx.builder(); b.shadow = false;
    let root = b.root(Vec3::ZERO);
    let s = if small { 0.6 } else { 1.0 };
    for (x, z) in [(-0.07, 0.0), (0.07, 0.0), (0.0, 0.11)] { b.bx(root, Vec3::new(0.12, 0.42, 0.12) * s, 0xc8341e, Vec3::new(x, 0.0, z) * s); }
    b.bx(root, Vec3::new(0.28, 0.06, 0.26) * s, 0x6a4a28, Vec3::new(0.0, 0.08, 0.04) * s);
    b.bx(root, Vec3::new(0.02, 0.16, 0.02) * s, 0xe8e0c8, Vec3::new(0.0, 0.28, 0.0) * s);
    b.glow(root, Vec3::splat(0.06 * s), 0xffd27a, Vec3::new(0.0, 0.37, 0.0) * s);
    root
}
fn throw_bundle(cx: &mut Ctx, from: Vec3, to: Vec3, small: bool) {
    let flight = 0.55;
    let g = 22.0;
    let v = Vec3::new((to.x - from.x) / flight, (to.y + 0.3 - from.y) / flight + 0.5 * g * flight, (to.z - from.z) / flight);
    let e = bundle_mesh(cx, small);
    let mut p = Proj::new(from, v); p.e = Some(e); p.grav = g; p.life = 2.5; p.r = 0.3; p.orient = Orient::Tumble;
    let sticky = cx.m("dynamite.sticky") > 0 && !small;
    p.trail = Some(Box::new(|cx, p, _| { let top = p.pos + Vec3::Y * 0.3; cx.fx.emit(true, top, Vec3::new(cx.rr(-1.0, 1.0), cx.rr(0.5, 2.0), cx.rr(-1.0, 1.0)), 0.2, 0.12, 0.02, hexc(0xffd27a) * 3.0, 8.0, 1.0, 1.0); }));
    p.on_world = Some(Box::new(move |cx, p, _| { let mut g = p.pos; g.y = cx.ground(g.x, g.z, g.y + 1.0); fuse(cx, g, None, small); }));
    p.on_hit = Some(Box::new(move |cx, p, i| { let id = if sticky { i.map(|i| cx.g.enemies[i].id) } else { None }; let mut g = p.pos; g.y = cx.ground(g.x, g.z, g.y + 1.0); fuse(cx, g, id, small); }));
    spawn_proj(cx, p);
}
/// the bundle sits (or clings to a victim) and fizzes for a moment before it goes up
fn fuse(cx: &mut Ctx, at: Vec3, stuck: Option<u32>, small: bool) {
    let e = bundle_mesh(cx, small);
    let t = if small { 0.35 } else { 0.55 };
    let mut z = crate::combat::Zone::new(at, 0.0, t);
    z.ents.push(e);
    let mut pos = at;
    z.on_frame = Some(Box::new(move |cx, z| {
        if let Some(id) = stuck { if let Some(i) = cx.find(id) { pos = cx.g.enemies[i].body.pos + Vec3::Y * cx.g.enemies[i].body.h * 0.6; } }
        z.pos = pos;
        let sh = (z.t * 40.0).sin() * 0.03;
        set_tf(cx, e, |tr| { tr.translation = pos + Vec3::new(sh, 0.0, 0.0); tr.rotation = Quat::from_rotation_z(0.25); });
        cx.fx.emit(true, pos + Vec3::Y * 0.4, Vec3::new(cx.rr(-1.5, 1.5), cx.rr(1.0, 3.0), cx.rr(-1.5, 1.5)), 0.25, 0.14, 0.02, hexc(0xffd27a) * 3.0, 8.0, 1.0, 1.0);
    }));
    z.on_end = Some(Box::new(move |cx, z| { let p = z.pos; let k = cx.g.flags.get("dyn_pw").copied().unwrap_or(1.0); blast(cx, p, small, k); }));
    zone(cx, z);
}
fn blast(cx: &mut Ctx, p: Vec3, small: bool, k: f32) {
    let big = cx.m("dynamite.big") as f32;
    let r = if small { 2.6 } else { 4.6 } * (1.0 + 0.3 * big) * cx.g.st.area;
    let dmg = (if small { 25.0 } else { 62.0 } * (1.0 + 0.3 * big) * (1.0 + cx.g.minutes() * 0.06) * k).round();
    cx.play("bomb"); cx.shake(if small { 0.3 } else { 0.7 }); cx.punch(if small { 0.2 } else { 0.6 });
    cx.fx.burst(p + Vec3::Y * 0.5, 0xff7a22, if small { 20 } else { 50 }, r * 2.4, 0.55, 0.7, 2.0, 0.0);
    cx.fx.burst(p + Vec3::Y * 0.5, 0xffe08a, if small { 8 } else { 20 }, r * 1.4, 0.35, 0.5, 0.0, 0.0);
    cx.fx.smoke(p, 0x3a3430, if small { 5 } else { 14 }, 2.5, 1.6, 1.6);
    cx.gfx().debris(p + Vec3::Y * 0.3, &[0x6a4a2c, 0x8a5b34, 0x7a7e86], if small { 6 } else { 16 }, 9.0, 0.18, 7.0);
    { let mut g = cx.gfx(); g.flare(p + Vec3::Y, 0xffb04a, if small { 3.0 } else { 7.0 }, 0.3, true); g.ring(p, 0xff8a3a, 0.4, r * 1.3, 0.35); g.shock(p, 0xffb04a, 0.4, r * 1.2, 0.35, 1.6); g.decal(p, false, r * 0.7, 8.0, None); if !small { g.impact(p + Vec3::Y, 0xffd27a, 2.4); } }
    cx.aoe(p, r, &mut |cx, i, _| { cx.damage(i, dmg, Hit { kb: 10.0, up: 8.0, src: Some(p), col: 0xff8a3a, ..default() }); });
    if !small && cx.m("dynamite.keg") > 0 {
        fire_pool(cx, p, r * 0.6, 4.0, (6.0 * k).round(), false);
        cx.later(0.8, move |cx| { cx.play("boom"); { let mut g = cx.gfx(); g.ring(p, 0xff8a3a, 0.5, r * 1.5, 0.4); g.shock(p, 0xff8a3a, 0.5, r * 1.4, 0.4, 1.2); } cx.aoe(p, r * 1.4, &mut |cx, i, _| { cx.damage(i, (30.0 * k).round(), Hit { kb: 6.0, up: 5.0, src: Some(p), col: 0xff8a3a, ..default() }); cx.status(i, Status { stun: 0.6, ..default() }); }); });
    }
}

/// time crawls: red marks blossom on the targets, then one shot each, every one a critical
fn deadeye(cx: &mut Ctx) -> bool {
    let pp = cx.p.body.pos; let f = flat_aim(cx);
    let noon = cx.m("deadeye.noon") > 0;
    let n = if noon { 40 } else { 6 + 3 * cx.m("deadeye.more") as usize } + 2 * alv(cx, "deadeye") as usize;
    let mut cand: Vec<(f32, u32)> = cx.foes(pp, if noon { 40.0 } else { 30.0 }).into_iter().filter_map(|i| {
        let e = &cx.g.enemies[i]; let d = Vec3::new(e.body.pos.x - pp.x, 0.0, e.body.pos.z - pp.z);
        let dot = d.normalize_or_zero().dot(f);
        if noon || dot > 0.35 { Some((-dot * 10.0 + d.length() * 0.1, e.id)) } else { None }
    }).collect();
    cand.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let ids: Vec<u32> = cand.into_iter().take(n).map(|c| c.1).collect();
    if ids.is_empty() { return false; }
    cx.g.flags.insert("slowmo", 0.9);
    cx.play("swell"); cx.p.pose(Pose::Point, 0.9);
    cx.toast("DEADEYE", "", 1.0);
    for &id in ids.iter() { if let Some(i) = cx.find(id) { let p = cx.g.enemies[i].body.pos + Vec3::Y * cx.g.enemies[i].body.h * 0.7; cx.gfx().glint(p, 0xff3a3a, 1.2, 0.6); } }
    let exec = cx.m("deadeye.exec") > 0;
    let dmg = (80.0 * (1.0 + cx.g.minutes() * 0.07) * pw(cx, "deadeye")).round();
    for (k, id) in ids.into_iter().enumerate() {
        cx.later(0.25 + k as f32 * (0.06 / (1.0 + k as f32 * 0.01)), move |cx| {
            let Some(i) = cx.find(id) else { return };
            let to = cx.g.enemies[i].body.pos + Vec3::Y * cx.g.enemies[i].body.h * 0.7;
            let from = cx.hand();
            cx.play("rifle"); muzzle(cx, from, (to - from).normalize(), true);
            { let mut g = cx.gfx(); g.lightning(from, to, 0xffd27a, 0.08, 0.04); g.impact(to, 0xff6a3a, 0.9); }
            let had = cx.g.buffs.contains_key("focus");
            cx.g.buffs.insert("focus", 0.05);
            cx.damage(i, dmg, Hit { kb: 4.0, src: Some(from), col: 0xff6a3a, ..default() });
            if !had { cx.g.buffs.remove("focus"); }
            if exec { if let Some(j) = cx.find(id) { let e = &cx.g.enemies[j]; if !e.boss && e.hp / e.max_hp < 0.3 { cx.apply_damage(j, 1.0e6, Hit { quiet: true, ..default() }); } } }
        });
    }
    true
}
