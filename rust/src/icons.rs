//! Procedural icon glyphs for the character panel: about thirty simple silhouettes (sword, flame, snowflake,
//! bolt, heart...) rasterised once at startup from signed distance functions into white alpha masks. The UI tints
//! them per category, so one mask serves every colour.
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use std::collections::HashMap;
use std::f32::consts::{PI, TAU};

const S: usize = 64;

#[derive(Resource, Default)]
pub struct Icons { pub glyphs: HashMap<&'static str, Handle<Image>> }

// ---- signed distance shapes, in a -1..1 square with y up ----
fn circle(p: Vec2, c: Vec2, r: f32) -> f32 { (p - c).length() - r }
fn ring(p: Vec2, c: Vec2, r: f32, w: f32) -> f32 { ((p - c).length() - r).abs() - w * 0.5 }
fn seg(p: Vec2, a: Vec2, b: Vec2, w: f32) -> f32 { let pa = p - a; let ba = b - a; let h = (pa.dot(ba) / ba.length_squared()).clamp(0.0, 1.0); (pa - ba * h).length() - w * 0.5 }
fn poly(p: Vec2, v: &[Vec2]) -> f32 {
    let mut d = (p - v[0]).length_squared(); let mut s = 1.0;
    let n = v.len(); let mut j = n - 1;
    for i in 0..n {
        let e = v[j] - v[i]; let w = p - v[i];
        let b = w - e * (w.dot(e) / e.length_squared()).clamp(0.0, 1.0);
        d = d.min(b.length_squared());
        let c = [p.y >= v[i].y, p.y < v[j].y, e.x * w.y > e.y * w.x];
        if (c[0] && c[1] && c[2]) || (!c[0] && !c[1] && !c[2]) { s = -s; }
        j = i;
    }
    s * d.sqrt()
}
fn v(x: f32, y: f32) -> Vec2 { Vec2::new(x, y) }
fn rot(p: Vec2, a: f32) -> Vec2 { let (s, c) = a.sin_cos(); Vec2::new(p.x * c - p.y * s, p.x * s + p.y * c) }
fn star(n: usize, r0: f32, r1: f32, a0: f32) -> Vec<Vec2> { (0..n * 2).map(|i| { let a = a0 + i as f32 / (n * 2) as f32 * TAU; let r = if i % 2 == 0 { r0 } else { r1 }; v(a.sin() * r, a.cos() * r) }).collect() }

/// the glyph's distance at p: negative inside
fn glyph(name: &str, p: Vec2) -> f32 {
    let u = |a: f32, b: f32| a.min(b);
    let cut = |a: f32, b: f32| a.max(-b);
    match name {
        "sword" => { let q = rot(p, PI / 4.0); u(u(poly(q, &[v(0.0, 0.95), v(0.13, 0.7), v(0.13, -0.3), v(-0.13, -0.3), v(-0.13, 0.7)]), seg(q, v(-0.4, -0.36), v(0.4, -0.36), 0.15)), u(seg(q, v(0.0, -0.4), v(0.0, -0.75), 0.13), circle(q, v(0.0, -0.84), 0.11))) }
        "daggers" => { let a = |q: Vec2| u(poly(q, &[v(0.0, 0.8), v(0.1, 0.55), v(0.1, -0.15), v(-0.1, -0.15), v(-0.1, 0.55)]), u(seg(q, v(-0.25, -0.2), v(0.25, -0.2), 0.1), seg(q, v(0.0, -0.22), v(0.0, -0.55), 0.1))); u(a(rot(p, 0.6) + v(0.0, 0.1)), a(rot(p, -0.6) + v(0.0, 0.1))) }
        "shield" => cut(poly(p, &[v(-0.68, 0.75), v(0.68, 0.75), v(0.68, 0.12), v(0.0, -0.88), v(-0.68, 0.12)]), poly(p, &[v(-0.5, 0.58), v(0.5, 0.58), v(0.5, 0.15), v(0.0, -0.62), v(-0.5, 0.15)]).max(-seg(p, v(0.0, 0.55), v(0.0, -0.55), 0.18)).max(-seg(p, v(-0.45, 0.2), v(0.45, 0.2), 0.18))),
        "heart" => u(u(circle(p, v(-0.34, 0.28), 0.4), circle(p, v(0.34, 0.28), 0.4)), poly(p, &[v(-0.72, 0.16), v(0.72, 0.16), v(0.0, -0.82)])),
        "flame" => cut(u(circle(p, v(0.0, -0.32), 0.52), poly(p, &[v(-0.5, -0.2), v(0.5, -0.2), v(0.12, 0.55), v(0.2, 0.95), v(-0.15, 0.5)])), u(circle(p, v(0.02, -0.4), 0.22), poly(p, &[v(-0.2, -0.35), v(0.22, -0.35), v(0.05, 0.12)]))),
        "snow" => { let mut d = 9.0f32; for k in 0..3 { let a = k as f32 * PI / 3.0; let dir = v(a.sin(), a.cos()); d = d.min(seg(p, -dir * 0.85, dir * 0.85, 0.13));
            for sg in [-1.0f32, 1.0] { let tip = dir * 0.55 * sg; let side = v(dir.y, -dir.x); d = d.min(seg(p, tip, tip + (dir * sg + side) * 0.22, 0.1)).min(seg(p, tip, tip + (dir * sg - side) * 0.22, 0.1)); } } d }
        "bolt" => poly(p, &[v(0.2, 0.95), v(-0.45, 0.02), v(-0.02, 0.02), v(-0.25, -0.95), v(0.48, 0.12), v(0.05, 0.12), v(0.38, 0.95)]),
        "arrow" => { let q = rot(p, -PI / 4.0); u(u(seg(q, v(0.0, -0.8), v(0.0, 0.5), 0.12), poly(q, &[v(0.0, 0.95), v(0.24, 0.5), v(-0.24, 0.5)])), u(poly(q, &[v(0.0, -0.55), v(0.22, -0.82), v(0.22, -0.95), v(0.0, -0.7)]), poly(q, &[v(0.0, -0.55), v(-0.22, -0.82), v(-0.22, -0.95), v(0.0, -0.7)]))) }
        "bow" => u(cut(ring(p, v(0.35, 0.0), 0.85, 0.16), circle(p, v(0.9, 0.0), 0.62)), seg(p, v(-0.18, 0.66), v(-0.18, -0.66), 0.06)),
        "orb" => u(u(circle(p, Vec2::ZERO, 0.32), ring(p, Vec2::ZERO, 0.7, 0.11)), ring(rot(p, 0.6) * v(1.0, 2.6), Vec2::ZERO, 0.78, 0.22)),
        "star" => poly(p, &star(5, 0.95, 0.4, 0.0)),
        "burst" => poly(p, &star(8, 0.95, 0.42, 0.2)),
        "sparkle" => u(poly(p, &star(4, 0.95, 0.18, 0.0)), poly(p - v(0.55, 0.55), &star(4, 0.32, 0.07, 0.0))),
        "skull" => cut(u(circle(p, v(0.0, 0.18), 0.62), poly(p, &[v(-0.36, -0.2), v(0.36, -0.2), v(0.28, -0.72), v(-0.28, -0.72)])), u(u(circle(p, v(-0.24, 0.12), 0.16), circle(p, v(0.24, 0.12), 0.16)), u(seg(p, v(-0.12, -0.5), v(-0.12, -0.72), 0.07), seg(p, v(0.12, -0.5), v(0.12, -0.72), 0.07)))),
        "eye" => cut(cut(poly(p, &[v(-0.95, 0.0), v(-0.5, 0.42), v(0.0, 0.55), v(0.5, 0.42), v(0.95, 0.0), v(0.5, -0.42), v(0.0, -0.55), v(-0.5, -0.42)]), circle(p, Vec2::ZERO, 0.36)).min(circle(p, Vec2::ZERO, 0.2)), circle(p, v(0.08, 0.08), 0.06)),
        "wing" => { let mut d = 9.0f32; for k in 0..4 { let y = 0.55 - k as f32 * 0.32; let x0 = -0.75 + k as f32 * 0.12; d = d.min(seg(p, v(x0, y), v(0.85 - k as f32 * 0.2, y + 0.18), 0.17 - k as f32 * 0.02)); } d.min(seg(p, v(-0.75, 0.55), v(-0.45, -0.5), 0.15)) }
        "clock" => u(ring(p, Vec2::ZERO, 0.75, 0.14), u(seg(p, Vec2::ZERO, v(0.0, 0.5), 0.12), seg(p, Vec2::ZERO, v(0.38, -0.12), 0.12))),
        "drop" => u(circle(p, v(0.0, -0.28), 0.55), poly(p, &[v(-0.48, -0.06), v(0.48, -0.06), v(0.0, 0.95)])),
        "magnet" => { let o = ring(p, v(0.0, -0.05), 0.55, 0.32).max(p.y + 0.05); u(o, u(poly(p, &[v(-0.71, -0.05), v(-0.39, -0.05), v(-0.39, 0.8), v(-0.71, 0.8)]), poly(p, &[v(0.39, -0.05), v(0.71, -0.05), v(0.71, 0.8), v(0.39, 0.8)]))) }
        "crown" => poly(p, &[v(-0.8, -0.55), v(0.8, -0.55), v(0.85, 0.5), v(0.42, 0.05), v(0.0, 0.7), v(-0.42, 0.05), v(-0.85, 0.5)]),
        "leaf" => { let q = rot(p, -0.7); cut(circle(q, v(-0.45, 0.0), 0.85).max(circle(q, v(0.45, 0.0), 0.85)), seg(q, v(0.0, -0.75), v(0.0, 0.75), 0.07)) }
        "cross" => u(seg(p, v(0.0, -0.85), v(0.0, 0.85), 0.3), seg(p, v(-0.55, 0.3), v(0.55, 0.3), 0.3)),
        "spiral" => { let mut d = 9.0f32; let mut prev = v(0.0, 0.0); for i in 1..60 { let t = i as f32 / 59.0; let a = t * TAU * 2.2; let q = v(a.cos(), a.sin()) * t * 0.9; d = d.min(seg(p, prev, q, 0.06 + 0.12 * t)); prev = q; } d }
        "mountain" => u(poly(p, &[v(-0.95, -0.7), v(-0.25, 0.65), v(0.3, -0.7)]), poly(p, &[v(-0.1, -0.7), v(0.42, 0.3), v(0.95, -0.7)])),
        "banner" => u(seg(p, v(-0.55, -0.9), v(-0.55, 0.9), 0.12), poly(p, &[v(-0.5, 0.85), v(0.75, 0.85), v(0.45, 0.45), v(0.75, 0.05), v(-0.5, 0.05)])),
        "crosshair" => u(u(ring(p, Vec2::ZERO, 0.6, 0.12), circle(p, Vec2::ZERO, 0.12)), u(u(seg(p, v(0.0, 0.45), v(0.0, 0.95), 0.12), seg(p, v(0.0, -0.45), v(0.0, -0.95), 0.12)), u(seg(p, v(0.45, 0.0), v(0.95, 0.0), 0.12), seg(p, v(-0.45, 0.0), v(-0.95, 0.0), 0.12)))),
        "coin" => u(ring(p, Vec2::ZERO, 0.75, 0.16), poly(p, &star(4, 0.45, 0.16, 0.0))),
        "trap" => { let teeth = |y0: f32, dir: f32| { let mut pts = vec![v(-0.85, y0)]; for i in 0..7 { let x = -0.85 + (i as f32 + 0.5) / 7.0 * 1.7; pts.push(v(x, y0 + dir * 0.4)); pts.push(v(x + 0.12, y0)); } pts.push(v(0.85, y0)); pts.push(v(0.85, y0 - dir * 0.18)); pts.push(v(-0.85, y0 - dir * 0.18)); pts }; u(poly(p, &teeth(0.25, -1.0)), poly(p, &teeth(-0.25, 1.0))) }
        "meteor" => u(circle(p, v(-0.3, -0.3), 0.45), u(seg(p, v(0.0, 0.05), v(0.85, 0.85), 0.16), u(seg(p, v(-0.25, 0.2), v(0.35, 0.9), 0.1), seg(p, v(0.05, -0.25), v(0.9, 0.3), 0.1)))),
        "cloud" => u(u(circle(p, v(-0.4, -0.05), 0.4), circle(p, v(0.1, 0.18), 0.5)), u(circle(p, v(0.55, -0.1), 0.35), poly(p, &[v(-0.8, -0.45), v(0.9, -0.45), v(0.9, -0.1), v(-0.8, -0.1)]))),
        "fist" => u(poly(p, &[v(-0.6, -0.85), v(0.45, -0.85), v(0.6, -0.1), v(0.6, 0.55), v(-0.6, 0.55)]), seg(p, v(-0.6, 0.0), v(-0.9, 0.3), 0.3)),
        "mace" => u(u(circle(p, v(0.3, 0.3), 0.38), poly(p, &star(6, 0.62, 0.3, 0.0).iter().map(|q| *q + v(0.3, 0.3)).collect::<Vec<_>>())), seg(p, v(0.1, 0.1), v(-0.75, -0.75), 0.15)),
        "staff" => u(seg(p, v(-0.7, -0.85), v(0.35, 0.4), 0.13), u(ring(p, v(0.5, 0.58), 0.28, 0.1), circle(p, v(0.5, 0.58), 0.13))),
        "boot" => u(poly(p, &[v(-0.45, 0.85), v(0.1, 0.85), v(0.1, -0.2), v(0.85, -0.45), v(0.85, -0.8), v(-0.45, -0.8)]), u(seg(p, v(-0.95, 0.4), v(-0.65, 0.4), 0.09), seg(p, v(-0.95, 0.05), v(-0.6, 0.05), 0.09))),
        "dash" => u(poly(p, &[v(-0.2, 0.7), v(0.85, 0.0), v(-0.2, -0.7), v(0.15, 0.0)]), u(seg(p, v(-0.95, 0.35), v(-0.35, 0.35), 0.1), seg(p, v(-0.95, -0.35), v(-0.35, -0.35), 0.1))),
        _ => poly(p, &star(5, 0.95, 0.4, 0.0)),
    }
}

pub const GLYPHS: [&str; 35] = ["dash", "sword", "daggers", "shield", "heart", "flame", "snow", "bolt", "arrow", "bow", "orb", "star", "burst", "sparkle", "skull", "eye", "wing", "clock", "drop", "magnet", "crown",
    "leaf", "cross", "spiral", "mountain", "banner", "crosshair", "coin", "trap", "meteor", "cloud", "fist", "mace", "staff", "boot"];

/// pick a glyph for an upgrade from its key, name and description
pub fn glyph_for(key: &str, name: &str, desc: &str) -> &'static str {
    let s = format!("{} {} {}", key, name, desc).to_lowercase();
    let has = |w: &[&str]| w.iter().any(|x| s.contains(x));
    if has(&["meteor", "armageddon", "cataclysm"]) { "meteor" }
    else if has(&["fire", "flame", "burn", "inferno", "phoenix", "pyro", "ember", "sun"]) { "flame" }
    else if has(&["frost", "ice", "freeze", "nova", "shatter", "winter", "glacial", "brittle", "chill"]) { "snow" }
    else if has(&["lightning", "thunder", "storm throne", "bolt", "chain"]) { "bolt" }
    else if has(&["vamp", "blood", "lifesteal", "leech"]) { "drop" }
    else if has(&["heal", "mend", "vitality", "regen", "second wind", "life", "renew", "juggernaut", "health"]) { "heart" }
    else if has(&["shield", "aegis", "ironhide", "armor", "armour", "bulwark", "guard", "less damage"]) { "shield" }
    else if has(&["crit", "keen", "mark", "giant slayer", "elite"]) { "crosshair" }
    else if has(&["trap", "snare", "jaws", "mine"]) { "trap" }
    else if has(&["whirl", "cyclone", "vortex", "spin", "tornado", "bladestorm"]) { "spiral" }
    else if has(&["slam", "quake", "earth", "fissure", "aftershock"]) { "mountain" }
    else if has(&["cry", "rally", "rage", "warlord", "terrify", "banner"]) { "banner" }
    else if has(&["smite", "judg", "holy", "consecrat", "sanct", "radiance", "prayer", "hallow", "purge", "choir", "spirit"]) { "cross" }
    else if has(&["blink", "mirror", "image", "flicker", "far step", "wake"]) { "sparkle" }
    else if has(&["orb", "arcane", "singularity", "echo", "wisp"]) { "orb" }
    else if has(&["arrow", "volley", "multishot", "splinter", "hail", "quickdraw", "hunter"]) { "arrow" }
    else if has(&["roll", "dance", "charge", "momentum", "dash", "relentless"]) { "dash" }
    else if has(&["swift", "fleet", "speed", "faster move", "movement"]) { "boot" }
    else if has(&["haste", "recharge", "cooldown", "fury", "flurry", "faster"]) { "clock" }
    else if has(&["magnet", "lodestone", "pickup", "reach"]) { "magnet" }
    else if has(&["wisdom", "experience", " xp"]) { "star" }
    else if has(&["fortune", "luck", "gold", "reroll", "greed"]) { "coin" }
    else if has(&["forage", "food", "meat"]) { "leaf" }
    else if has(&["skull", "death", "curse", "glass", "chaos", "doom"]) { "skull" }
    else if has(&["blade", "sword", "dagger", "edge", "venom", "shadow"]) { "sword" }
    else if has(&["might", "damage", "power", "wrath"]) { "fist" }
    else if has(&["area", "amplify", "wide", "radius", "explo", "blast", "bomb", "burst"]) { "burst" }
    else if has(&["axe", "ricochet"]) { "daggers" }
    else if has(&["stamina", "endurance"]) { "wing" }
    else { "star" }
}
/// glyph for an ability or weapon id
pub fn glyph_for_id(id: &str) -> &'static str {
    match id {
        "volley" => "arrow", "roll" => "dash", "snare" => "trap", "storm" => "cloud", "blink" => "sparkle", "nova" => "snow", "orb" => "orb", "meteor" => "meteor",
        "charge" => "dash", "whirl" => "spiral", "slam" => "mountain", "cry" => "banner", "mend" => "heart", "smite" => "sword", "shield" => "shield", "consecrate" => "cross",
        "longbow" => "bow", "daggers" => "daggers", "firestaff" => "staff", "frostwand" => "snow", "greatsword" => "sword", "swordshield" => "shield", "mace" => "mace", "scepter" => "cross",
        _ => "star",
    }
}

pub fn build_icons(mut images: ResMut<Assets<Image>>, mut icons: ResMut<Icons>) {
    for name in GLYPHS {
        let mut d = vec![0u8; S * S * 4];
        let px = 2.0 / S as f32;
        for y in 0..S { for x in 0..S {
            // 2x2 supersampling for clean edges
            let mut a = 0.0;
            for (ox, oy) in [(0.25, 0.25), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)] {
                let p = v((x as f32 + ox) / S as f32 * 2.0 - 1.0, 1.0 - (y as f32 + oy) / S as f32 * 2.0) * 1.08;
                a += (0.5 - glyph(name, p) / px).clamp(0.0, 1.0);
            }
            let i = (y * S + x) * 4; d[i] = 255; d[i + 1] = 255; d[i + 2] = 255; d[i + 3] = (a / 4.0 * 255.0) as u8;
        } }
        icons.glyphs.insert(name, images.add(Image::new(Extent3d { width: S as u32, height: S as u32, depth_or_array_layers: 1 }, TextureDimension::D2, d, TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::RENDER_WORLD)));
    }
}
