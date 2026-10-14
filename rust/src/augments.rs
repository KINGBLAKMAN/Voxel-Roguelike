//! Chaos augments: level-up picks that bend the rules of the whole run instead of making one ability stronger:
//! kills that chain-explode, abilities that recast themselves, ricochets, black holes, lightning on crits.
//! They stack and snowball, which is the point. Ranks live in the mods map as "x.<id>".
use crate::combat::*;
use crate::fx::hexc;
use crate::game::*;
use crate::models::*;
use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;
use std::f32::consts::TAU;

pub struct AugInfo { pub id: &'static str, pub name: &'static str, pub ranks: &'static [&'static str] }
pub const AUGS: [AugInfo; 10] = [
    AugInfo { id: "chain", name: "Chain Reaction", ranks: &["Slain enemies have a 15% chance to explode, and explosions can kill more.", "+10% chance and bigger blasts.", "+10% chance, and blasts set the horde ablaze."] },
    AugInfo { id: "echo", name: "Echo", ranks: &["Abilities have a 20% chance to cast themselves again for free.", "+15% chance to echo."] },
    AugInfo { id: "surge", name: "Arcane Surge", ranks: &["Every ability you cast also fires 4 seeking bolts.", "+2 bolts.", "+2 bolts and +50% bolt damage."] },
    AugInfo { id: "storm", name: "Stormcaller", ranks: &["+5% critical chance, and critical hits call down lightning.", "Lightning strikes twice as often and hits 50% harder."] },
    AugInfo { id: "frenzy", name: "Frenzy", ranks: &["Every kill takes 0.15s off all your ability cooldowns.", "0.3s per kill."] },
    AugInfo { id: "well", name: "Gravity Well", ranks: &["Every 9s a black hole tears open in the thickest crowd, drags enemies in and detonates.", "Every 6s, wider and twice as deadly."] },
    AugInfo { id: "overload", name: "Overload", ranks: &["Every 5th ability you cast unleashes a massive elemental nova."] },
    AugInfo { id: "bounce", name: "Ricochet", ranks: &["Your projectiles bounce to another enemy after a hit.", "They bounce twice."] },
    AugInfo { id: "phoenix", name: "Phoenix Heart", ranks: &["The first time you would die, you explode in flame and rise with half your health."] },
    AugInfo { id: "berserk", name: "Berserker", ranks: &["Below half health, deal 40% more damage and attack 25% faster.", "80% more damage and 50% faster attacks."] },
];
pub const AUG_COLOR: u32 = 0xff4ad8;
fn ak(g: &Game, k: &str) -> u32 { g.m(&format!("x.{}", k)) }

pub fn dmg_mul(cx: &Ctx) -> f32 { let b = ak(cx.g, "berserk"); if b > 0 && cx.p.hp < cx.p.max_hp * 0.5 { 1.0 + 0.4 * b as f32 } else { 1.0 } }
pub fn atk_mul(cx: &Ctx) -> f32 { let b = ak(cx.g, "berserk"); if b > 0 && cx.p.hp < cx.p.max_hp * 0.5 { 1.0 + 0.25 * b as f32 } else { 1.0 } }

pub fn on_kill(cx: &mut Ctx, c: Vec3) {
    if cx.p.dead { return; }
    let ch = ak(cx.g, "chain");
    if ch > 0 && cx.g.flags.get("chaining").copied().unwrap_or(0.0) <= 0.0 && cx.r() < 0.05 + 0.1 * ch as f32 {
        let r = 2.4 + ch as f32 * 0.6;
        cx.later(0.1, move |cx| {
            cx.play("boom");
            { let mut g = cx.gfx(); g.flare(c, 0xff4ad8, 4.0 + ch as f32, 0.3, true); g.sphere(c, 0xff4ad8, 0.3, r, 0.3); }
            cx.fx.burst(c, 0xff7ae8, 24, r * 2.4, 0.45, 0.45, 0.0, 0.0); cx.fx.sparks(c, 0xffffff, 8, 10.0);
            let dm = (24.0 * (1.0 + cx.g.minutes() * 0.1) * (1.0 + 0.3 * (ch - 1) as f32)).round();
            cx.g.flags.insert("chaining", 1.0);
            cx.aoe(c, r, &mut |cx, i, _| { cx.damage(i, dm, Hit { kb: 7.0, up: 4.0, src: Some(c), col: 0xff4ad8, quiet: true }); if ch >= 3 { cx.status(i, Status { burn: 2.0, ..default() }); } });
            cx.g.flags.insert("chaining", 0.0);
        });
    }
    let fr = ak(cx.g, "frenzy");
    if fr > 0 { for cd in cx.g.cds.iter_mut() { *cd = (*cd - 0.15 * fr as f32).max(0.0); } }
}

/// called whenever an ability is cast; returns true if the ability should echo (recast) shortly
pub fn on_cast(cx: &mut Ctx, echoing: bool) -> bool {
    let su = ak(cx.g, "surge");
    if su > 0 {
        let n = 2 + 2 * su; let dmg = (10.0 * if su >= 3 { 1.5 } else { 1.0 } * (1.0 + cx.g.minutes() * 0.05)).round();
        let from = cx.hand(); cx.play("arcane");
        for i in 0..n { let an = i as f32 / n as f32 * TAU; crate::relics::seek_bolt(cx, from, Vec3::new(an.cos(), 0.5, an.sin()), dmg, 0xff7ae8, false, 0.15, 22.0, 6.0); }
    }
    if ak(cx.g, "overload") > 0 {
        let k = cx.g.flags.entry("cast_n").or_insert(0.0); *k += 1.0;
        if (*k as i32) % 5 == 0 {
            let c = cx.p.body.pos; let col = match cx.p.cls { Class::Ranger => 0x9be07a, Class::Mage => 0xb46aff, Class::Warrior => 0xff6a4a, Class::Priest => 0xffd35a };
            let r = 9.0 * cx.g.st.area;
            cx.play("bomb"); cx.shake(1.0);
            { let mut g = cx.gfx(); g.sphere(c + Vec3::Y, col, 0.5, r, 0.5); g.circle(c, col, r, 1.0, "arcane", 5.0, None, None); g.shock(c, col, 0.5, r, 0.5, 3.0); g.flare(c + Vec3::Y * 1.5, col, 12.0, 0.5, true); }
            cx.pop(c + Vec3::Y * 2.6, "OVERLOAD".into(), 1);
            let dm = (80.0 * (1.0 + cx.g.minutes() * 0.1)).round();
            cx.aoe(c, r, &mut |cx, i, _| { cx.damage(i, dm, Hit { kb: 12.0, up: 7.0, src: Some(c), col, ..default() }); cx.status(i, Status { stun: 0.8, ..default() }); });
        }
    }
    let ec = ak(cx.g, "echo");
    !echoing && ec > 0 && cx.r() < 0.05 + 0.15 * ec as f32
}

pub fn on_crit(cx: &mut Ctx, i: usize) {
    let s = ak(cx.g, "storm");
    if s == 0 || cx.g.run_t < *cx.g.flags.get("storm_t").unwrap_or(&0.0) { return; }
    let t = cx.g.run_t + if s >= 2 { 0.18 } else { 0.36 }; cx.g.flags.insert("storm_t", t);
    let p = cx.g.enemies[i].body.pos;
    let dm = (20.0 * if s >= 2 { 1.5 } else { 1.0 } * (1.0 + cx.g.minutes() * 0.06)).round();
    cx.later(0.0, move |cx| { lightning_strike(cx, p, dm, 1.7, 0xd8a8ff); });
}

pub fn phoenix(cx: &mut Ctx) -> bool {
    if ak(cx.g, "phoenix") == 0 || cx.g.flags.contains_key("phoenix_used") { return false; }
    cx.g.flags.insert("phoenix_used", 1.0);
    cx.p.hp = (cx.p.max_hp * 0.5).round(); cx.p.invuln = 2.5;
    let c = cx.p.body.pos;
    cx.play("bomb"); cx.shake(1.0); cx.g.flash = 0.25; cx.toast("Phoenix Heart", "You rise from the ashes", 2.5);
    { let mut g = cx.gfx(); g.pillar(c, 0xff7a2a, 2.5, 26.0, 1.4); g.sphere(c + Vec3::Y, 0xff8a3a, 0.5, 10.0, 0.6); g.circle(c, 0xff6a1a, 8.0, 1.6, "fire", 3.0, None, None); g.decal(c, false, 8.0, 14.0, Some(0xff6a1a)); }
    cx.fx.burst(c + Vec3::Y, 0xffb03a, 90, 12.0, 1.0, 0.7, 4.0, 0.0);
    let dm = (120.0 * (1.0 + cx.g.minutes() * 0.1)).round();
    cx.aoe(c, 9.0, &mut |cx, i, _| { cx.damage(i, dm, Hit { kb: 14.0, up: 8.0, src: Some(c), col: 0xff8a3a, ..default() }); cx.status(i, Status { burn: 4.0, ..default() }); });
    true
}

/// Gravity Well: a real black sphere tears open in the thickest crowd, drags enemies in and detonates
pub fn update_augments(cx: &mut Ctx) {
    let w = ak(cx.g, "well");
    if w == 0 || cx.p.dead { return; }
    let dt = cx.dt;
    let t = cx.g.flags.entry("well_t").or_insert(3.0); *t -= dt; if *t > 0.0 { return; }
    *t = if w >= 2 { 6.0 } else { 9.0 };
    let pp = cx.p.body.pos;
    let fs = cx.foes(pp, 20.0);
    if fs.is_empty() { return; }
    let mut best = fs[0]; let mut bn = -1;
    for &e in fs.iter().take(12) { let ep = cx.g.enemies[e].body.pos; let n = fs.iter().filter(|&&q| Vec2::new(cx.g.enemies[q].body.pos.x - ep.x, cx.g.enemies[q].body.pos.z - ep.z).length() < 4.0).count() as i32; if n > bn { bn = n; best = e; } }
    let mut c = cx.g.enemies[best].body.pos; c.y = cx.ground(c.x, c.z, c.y + 1.0);
    let rad = if w >= 2 { 8.5 } else { 7.0 } * cx.g.st.area;
    // the singularity: a pitch-black core with a bright accretion ring spinning around it
    let core_m = cx.mats.add(StandardMaterial { base_color: Color::BLACK, unlit: true, ..default() });
    let core = cx.c.spawn((Mesh3d(cx.pal.sphere.clone()), MeshMaterial3d(core_m.clone()), Transform::from_translation(c + Vec3::Y * 1.4).with_scale(Vec3::ZERO), NotShadowCaster)).id();
    let ring_m = cx.pal.mat(cx.mats, 0xd08aff, Mk::Glow);
    let ring = cx.c.spawn((Mesh3d(cx.pal.torus.clone()), MeshMaterial3d(ring_m), Transform::from_scale(Vec3::new(1.6, 0.3, 1.6)), NotShadowCaster)).id();
    cx.c.entity(core).add_child(ring);
    let mut tw = crate::fx::Tween::new(core, 1.7, c + Vec3::Y * 1.4); tw.s0 = Vec3::splat(0.2); tw.s1 = Vec3::splat(1.4); tw.spin = Vec3::new(0.4, 9.0, 0.0); tw.fade_in = 0.01; tw.fade_out = 0.01;
    cx.fx.tweens.push(tw);
    cx.gfx().circle(c, 0xb04aff, rad, 1.7, "arcane", -6.0, None, None);
    cx.play("arcane");
    let mut z = Zone::new(c, rad, 1.6); z.every = 0.2;
    z.on_frame = Some(Box::new(move |cx, z| {
        let c = z.pos; let r = z.r;
        for _ in 0..4 { let an = cx.rr(0.0, TAU); let rr = r * cx.rr(0.5, 1.0); let v = Vec3::new(-an.cos() * rr * 1.8 + an.sin() * 5.0, 0.0, -an.sin() * rr * 1.8 - an.cos() * 5.0);
            let col = hexc(cx.pick(&[0xb04aff, 0xff4ad8, 0xffffff])) * 2.5; let h = cx.rr(0.3, 2.4); cx.fx.emit(true, c + Vec3::new(an.cos() * rr, h, an.sin() * rr), v, 0.5, 0.35, 0.05, col, 0.0, 0.0, 1.0); }
    }));
    z.on_tick = Some(Box::new(|cx, z| {
        let c = z.pos; let r = z.r; let mut n = 0;
        for i in 0..cx.g.enemies.len() {
            let e = &cx.g.enemies[i]; if e.dead || e.t.passive || e.t.prop || n >= 14 { continue; }
            let (dx, dz) = (c.x - e.body.pos.x, c.z - e.body.pos.z); let d = (dx * dx + dz * dz).sqrt();
            if d < r && d > 0.6 { n += 1; cx.status(i, Status { push: Some(Vec3::new(dx / d * 9.0, 2.0, dz / d * 9.0)), slow: 0.5, ..default() }); }
        }
    }));
    z.on_end = Some(Box::new(move |cx, z| {
        let c = z.pos; let r = z.r; let p = c + Vec3::Y * 1.2;
        cx.play("bomb"); cx.shake(0.6);
        { let mut g = cx.gfx(); g.flare(p, 0xd08aff, 10.0, 0.45, true); g.sphere(p, 0xb04aff, 0.5, r * 0.8, 0.45); g.shock(c, 0xff4ad8, 0.5, r * 0.8, 0.4, 2.5); }
        cx.fx.burst(p, 0xff7ae8, 60, r * 2.5, 0.6, 0.5, 0.0, 0.0);
        let dm = (55.0 * if w >= 2 { 2.0 } else { 1.0 } * (1.0 + cx.g.minutes() * 0.1)).round();
        cx.aoe(c, r * 0.75, &mut |cx, i, _| cx.damage(i, dm, Hit { kb: 10.0, up: 6.0, src: Some(c), col: 0xd08aff, ..default() }));
    }));
    zone(cx, z);
}
