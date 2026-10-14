//! Procedural block textures, painted at startup: a 32px pixel-art albedo map and a normal + roughness map for
//! every block face, stored as two texture arrays the terrain shader samples by layer. Everything tiles, so the
//! mesher can use world coordinates as UVs.
use crate::noise::{hash2, smooth};
use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureViewDescriptor, TextureViewDimension};

pub const N: usize = 32;
const NF: f32 = N as f32;

/// texture layers
pub mod l {
    pub const GRASS: u32 = 0; pub const GRASS_SIDE: u32 = 1; pub const DIRT: u32 = 2; pub const STONE: u32 = 3; pub const SAND: u32 = 4;
    pub const SNOW: u32 = 5; pub const SNOW_SIDE: u32 = 6; pub const BARK: u32 = 7; pub const RINGS: u32 = 8; pub const LEAVES: u32 = 9;
    pub const PINE: u32 = 10; pub const CACTUS: u32 = 11; pub const CACTUS_TOP: u32 = 12; pub const BRICK: u32 = 13; pub const MOSS: u32 = 14;
    pub const GRAVEL: u32 = 15; pub const FGRASS: u32 = 16; pub const FGRASS_SIDE: u32 = 17; pub const BIRCH: u32 = 18; pub const AUTUMN: u32 = 19;
    pub const CRYSTAL: u32 = 20; pub const PLANK: u32 = 21; pub const CLOTH: u32 = 22; pub const GOLD: u32 = 23; pub const EMBER: u32 = 24;
    pub const DSTONE: u32 = 25; pub const RUNE: u32 = 26; pub const BERRY: u32 = 27; pub const BIRCH_TOP: u32 = 28;
    pub const COUNT: u32 = 29;
}

/// which layer a block shows on a face (0 top, 1 side, 2 bottom); `capped` is a side face under open air
pub fn face_layer(bl: u8, face: u8, capped: bool) -> u32 {
    use crate::world::b;
    match bl {
        b::GRASS => match face { 0 => l::GRASS, 1 if capped => l::GRASS_SIDE, _ => l::DIRT },
        b::FGRASS => match face { 0 => l::FGRASS, 1 if capped => l::FGRASS_SIDE, _ => l::DIRT },
        b::SNOW => match face { 0 => l::SNOW, 1 if capped => l::SNOW_SIDE, _ => l::SNOW },
        b::DIRT => l::DIRT, b::STONE => l::STONE, b::SAND => l::SAND,
        b::WOOD => if face == 1 { l::BARK } else { l::RINGS },
        b::BIRCH => if face == 1 { l::BIRCH } else { l::BIRCH_TOP },
        b::LEAVES | b::BUSH => l::LEAVES, b::BERRY => l::BERRY, b::AUTUMN => l::AUTUMN,
        b::PINE => l::PINE, b::SPINE => if face == 0 { l::SNOW } else { l::PINE },
        b::CACTUS => if face == 1 { l::CACTUS } else { l::CACTUS_TOP },
        b::BRICK => l::BRICK, b::MOSS => l::MOSS, b::GRAVEL => l::GRAVEL, b::CRYSTAL => l::CRYSTAL, b::PLANK => l::PLANK,
        b::CLOTH => l::CLOTH, b::GOLD => l::GOLD, b::EMBER => l::EMBER, b::DSTONE => l::DSTONE, b::RUNE => l::RUNE,
        _ => l::STONE,
    }
}

// ---------------- painting helpers ----------------
fn hx(hex: u32) -> Vec3 { Vec3::new(((hex >> 16) & 255) as f32 / 255.0, ((hex >> 8) & 255) as f32 / 255.0, (hex & 255) as f32 / 255.0) }
fn h(x: i32, y: i32, s: u32) -> f32 { hash2(x.rem_euclid(N as i32), y.rem_euclid(N as i32), s) }
/// tileable value noise with `per` cells across the texture (per x and y may differ)
fn vn(x: f32, y: f32, px: i32, py: i32, s: u32) -> f32 {
    let (fx, fy) = (x / NF * px as f32, y / NF * py as f32);
    let (ix, iy) = (fx.floor() as i32, fy.floor() as i32);
    let (tx, ty) = (fx - ix as f32, fy - iy as f32);
    let (ux, uy) = (tx * tx * (3.0 - 2.0 * tx), ty * ty * (3.0 - 2.0 * ty));
    let g = |a: i32, b: i32| hash2(a.rem_euclid(px), b.rem_euclid(py), s);
    let a = g(ix, iy) + (g(ix + 1, iy) - g(ix, iy)) * ux;
    let b = g(ix, iy + 1) + (g(ix + 1, iy + 1) - g(ix, iy + 1)) * ux;
    a + (b - a) * uy
}
fn fbm(x: f32, y: f32, px: i32, py: i32, s: u32) -> f32 {
    let (mut v, mut a, mut t) = (0.0, 0.5, 0.0);
    for o in 0..4 { let k = 1 << o; v += a * vn(x, y, px * k, py * k, s + o as u32 * 17); t += a; a *= 0.5; }
    v / t
}
/// tileable Voronoi: distance to the nearest and second-nearest feature point, and the nearest point's index
struct Cells { pts: Vec<Vec2> }
impl Cells {
    fn new(n: usize, s: u32) -> Self { Cells { pts: (0..n).map(|i| Vec2::new(hash2(i as i32, 1, s) * NF, hash2(i as i32, 2, s) * NF)).collect() } }
    fn at(&self, x: f32, y: f32) -> (f32, f32, usize) {
        let (mut f1, mut f2, mut id) = (1e9f32, 1e9f32, 0);
        for (i, p) in self.pts.iter().enumerate() {
            let mut d = (Vec2::new(x, y) - *p).abs(); d = Vec2::new(d.x.min(NF - d.x), d.y.min(NF - d.y));
            let dd = d.length();
            if dd < f1 { f2 = f1; f1 = dd; id = i; } else if dd < f2 { f2 = dd; }
        }
        (f1, f2, id)
    }
}
struct Tex { c: Vec<Vec3>, h: Vec<f32>, rough: f32, bump: f32 }
impl Tex {
    fn new(rough: f32, bump: f32) -> Self { Tex { c: vec![Vec3::ONE; N * N], h: vec![0.5; N * N], rough, bump } }
    fn each(&mut self, mut f: impl FnMut(i32, i32, &mut Vec3, &mut f32)) {
        for y in 0..N { for x in 0..N { let i = y * N + x; let (mut c, mut hh) = (self.c[i], self.h[i]); f(x as i32, y as i32, &mut c, &mut hh); self.c[i] = c; self.h[i] = hh; } }
    }
    fn px(&mut self, x: i32, y: i32, c: Vec3, hh: f32) { let i = y.rem_euclid(N as i32) as usize * N + x.rem_euclid(N as i32) as usize; self.c[i] = c; self.h[i] = hh; }
}
fn pick(pal: &[u32], t: f32) -> Vec3 {
    let t = t.clamp(0.0, 0.999) * (pal.len() - 1) as f32;
    let i = t.floor() as usize; hx(pal[i]).lerp(hx(pal[(i + 1).min(pal.len() - 1)]), t - i as f32)
}

// ---------------- the textures ----------------
fn grass_top(pal: &[u32], litter: bool, s: u32) -> Tex {
    let mut t = Tex::new(0.9, 2.2);
    t.each(|x, y, c, hh| {
        let n = fbm(x as f32, y as f32, 4, 4, s) * 0.75 + h(x, y, s + 1) * 0.25;
        *c = pick(pal, 0.15 + n * 0.7); *hh = n;
        let r = h(x, y, s + 2);
        if r < 0.05 { *c = hx(pal[pal.len() - 1]); *hh = 0.9; } else if r < 0.09 { *c = hx(pal[0]) * 0.9; *hh = 0.25; }
        if litter && h(x / 2, y / 2, s + 3) < 0.05 { *c = pick(&[0x6a4a22, 0x9a6a2a, 0xb88a3a], h(x, y, s + 4)); *hh = 0.6; }
    });
    t
}
fn dirt(s: u32) -> Tex {
    let mut t = Tex::new(0.95, 2.5);
    let cells = Cells::new(14, s + 9);
    t.each(|x, y, c, hh| {
        let n = fbm(x as f32, y as f32, 4, 4, s) * 0.7 + h(x, y, s + 1) * 0.3;
        *c = pick(&[0x5e3c22, 0x7a4f2e, 0x8a5b34, 0x9a6a40], n); *hh = n * 0.5;
        let (f1, _, id) = cells.at(x as f32 + 0.5, y as f32 + 0.5);
        let rad = 0.9 + hash2(id as i32, 5, s) * 1.2;
        if f1 < rad { let k = hash2(id as i32, 6, s); *c = if k < 0.5 { hx(0x9a8a78) * (0.9 + 0.2 * (1.0 - f1 / rad)) } else { hx(0x4a3020) }; *hh = 0.7 + 0.3 * (1.0 - f1 / rad); }
    });
    t
}
fn with_cap(mut t: Tex, pal: &[u32], depth: f32, s: u32) -> Tex {
    // a grass or snow fringe hanging over the top of a side face, with ragged drips
    for x in 0..N as i32 {
        let d = depth + (vn(x as f32, 0.0, 8, 1, s) - 0.5) * 3.0 + if h(x, 0, s + 1) < 0.25 { 2.0 + h(x, 1, s + 2) * 3.0 } else { 0.0 };
        for y in 0..N as i32 {
            if (y as f32) < d {
                let n = h(x, y, s + 3) * 0.6 + 0.4 * (1.0 - y as f32 / d.max(1.0));
                t.px(x, y, pick(pal, n) * if (y as f32) > d - 1.0 { 0.82 } else { 1.0 }, 0.6 + 0.4 * n);
            }
        }
    }
    t
}
fn stone(pal: &[u32], s: u32, cracks: f32) -> Tex {
    let mut t = Tex::new(0.85, 3.0);
    let cells = Cells::new(5, s + 4);
    t.each(|x, y, c, hh| {
        let n = fbm(x as f32, y as f32, 3, 3, s) * 0.8 + h(x, y, s + 1) * 0.2;
        *c = pick(pal, n); *hh = 0.5 + n * 0.4;
        let (f1, f2, _) = cells.at(x as f32 + 0.5, y as f32 + 0.5);
        let show = vn(x as f32, y as f32, 2, 2, s + 9) > 0.42;
        if show && f2 - f1 < cracks * 0.6 { *c *= 0.7; *hh = 0.15; } else if show && f2 - f1 < cracks * 0.6 + 1.0 { *c *= 1.04; *hh = 0.7; }
    });
    t
}
fn sand(s: u32) -> Tex {
    let mut t = Tex::new(0.97, 1.6);
    t.each(|x, y, c, hh| {
        let rip = ((y as f32 + fbm(x as f32, y as f32, 2, 2, s) * 8.0) * std::f32::consts::TAU / 8.0).sin() * 0.5 + 0.5;
        let n = h(x, y, s + 1);
        *c = pick(&[0xcdb478, 0xe0cd8c, 0xeadba0], rip * 0.5 + n * 0.5); *hh = rip;
        if n < 0.04 { *c = hx(0xa08a5a); } else if n > 0.97 { *c = hx(0xfaf0c8); }
    });
    t
}
fn snow(s: u32) -> Tex {
    let mut t = Tex::new(0.55, 1.4);
    t.each(|x, y, c, hh| {
        let n = fbm(x as f32, y as f32, 3, 3, s);
        *c = pick(&[0xcfdaea, 0xe8eef6, 0xf7fafd], n); *hh = n;
        if h(x, y, s + 1) < 0.03 { *c = Vec3::ONE; *hh = 1.0; }
    });
    t
}
fn bark(pal: &[u32], s: u32) -> Tex {
    let mut t = Tex::new(0.92, 3.2);
    t.each(|x, y, c, hh| {
        let streak = fbm(x as f32, y as f32, 8, 2, s);
        let groove = vn(x as f32, y as f32, 6, 1, s + 3);
        *c = pick(pal, streak * 0.7 + h(x, y, s + 1) * 0.3); *hh = streak;
        if groove < 0.3 { *c *= 0.6; *hh = 0.05; }
    });
    t
}
fn rings(wood: &[u32], barkc: u32, s: u32) -> Tex {
    let mut t = Tex::new(0.9, 2.0);
    t.each(|x, y, c, hh| {
        let d = Vec2::new(x as f32 + 0.5 - NF / 2.0, y as f32 + 0.5 - NF / 2.0).length();
        let r = (d * 0.42 + fbm(x as f32, y as f32, 3, 3, s) * 1.2).fract();
        *c = pick(wood, if r < 0.3 { 0.15 } else { 0.7 + h(x, y, s) * 0.3 }); *hh = if r < 0.3 { 0.3 } else { 0.6 };
        if d > NF / 2.0 - 2.5 { *c = hx(barkc) * (0.8 + 0.3 * h(x, y, s + 1)); *hh = 0.8; }
    });
    t
}
fn leaves(pal: &[u32], s: u32, berries: bool) -> Tex {
    let mut t = Tex::new(0.8, 3.5);
    t.each(|x, y, c, hh| { *c = hx(pal[0]) * (0.7 + 0.3 * h(x, y, s)); *hh = 0.0; });
    let mut rng = crate::noise::mulberry32(s);
    for _ in 0..70 {
        let (cx, cy, a) = (rng() * NF, rng() * NF, rng() * std::f32::consts::PI);
        let (rx, ry) = (2.0 + rng() * 1.4, 1.1 + rng() * 0.7);
        let shade = rng();
        let (ca, sa) = (a.cos(), a.sin());
        for dy in -4..=4 { for dx in -4..=4 {
            let (px, py) = (cx.floor() + dx as f32, cy.floor() + dy as f32);
            let (ox, oy) = (px + 0.5 - cx, py + 0.5 - cy);
            let (u, v) = ((ox * ca + oy * sa) / rx, (-ox * sa + oy * ca) / ry);
            let d = u * u + v * v;
            if d > 1.0 { continue; }
            let lit = (0.5 - (ox + oy) * 0.12).clamp(0.0, 1.0);
            let col = pick(pal, 0.25 + shade * 0.5 + lit * 0.35) * if v.abs() < 0.18 { 0.85 } else { 1.0 };
            let i = (py as i32).rem_euclid(N as i32) as usize * N + (px as i32).rem_euclid(N as i32) as usize;
            let hh = 0.3 + 0.7 * (1.0 - d).sqrt();
            if hh > t.h[i] { t.c[i] = col; t.h[i] = hh; }
        } }
    }
    if berries {
        for _ in 0..9 {
            let (cx, cy) = ((rng() * NF) as i32, (rng() * NF) as i32);
            for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] { t.px(cx + dx, cy + dy, hx(if dx == 0 && dy == 0 { 0xff8a9a } else { 0xc8203a }), 1.0); }
        }
    }
    t
}
fn needles(pal: &[u32], s: u32) -> Tex {
    let mut t = Tex::new(0.85, 3.0);
    t.each(|x, y, c, hh| { *c = hx(pal[0]) * (0.75 + 0.25 * h(x, y, s)); *hh = 0.1; });
    let mut rng = crate::noise::mulberry32(s + 5);
    for k in 0..150 {
        let (mut x, mut y) = (rng() * NF, rng() * NF); let dir = if k % 2 == 0 { 1.0 } else { -1.0 }; let col = pick(pal, 0.3 + rng() * 0.7);
        for i in 0..5 { t.px(x as i32, y as i32, col * (0.85 + 0.04 * i as f32), 0.5 + 0.1 * i as f32); x += dir * 0.7; y += 0.7; }
    }
    t
}
fn cactus(s: u32, top: bool) -> Tex {
    let mut t = Tex::new(0.7, 3.0);
    t.each(|x, y, c, hh| {
        let n = h(x, y, s) * 0.3 + fbm(x as f32, y as f32, 2, 4, s) * 0.7;
        *c = pick(&[0x2f6a2a, 0x4c8b37, 0x6aa848], n); *hh = 0.5;
        if top {
            let d = Vec2::new(x as f32 + 0.5 - NF / 2.0, y as f32 + 0.5 - NF / 2.0);
            let a = d.y.atan2(d.x); let rib = (a * 8.0 / std::f32::consts::TAU * 2.0).fract();
            if rib < 0.2 { *c *= 0.7; *hh = 0.2; }
            if d.length() < 3.0 { *c = hx(0xb8d070); }
        } else {
            let k = x % 8;
            if k == 0 { *c *= 0.62; *hh = 0.1; } else if k == 3 || k == 4 { *c *= 1.15; *hh = 0.8; }
            if k == 4 && y % 6 == 2 { *c = hx(0xf0ecc8); *hh = 1.0; }
        }
    });
    t
}
fn bricks(pal: &[u32], s: u32, rows: i32, mortar: u32) -> Tex {
    let mut t = Tex::new(0.88, 3.4);
    let rh = N as i32 / rows;
    t.each(|x, y, c, hh| {
        let row = y / rh; let off = if row % 2 == 1 { rh } else { 0 };
        let col = (x + off) / (rh * 2);
        let (lx, ly) = ((x + off) % (rh * 2), y % rh);
        let id = hash2(row, col, s);
        let n = fbm(x as f32, y as f32, 4, 4, s + 1) * 0.5 + h(x, y, s + 2) * 0.2 + id * 0.3;
        *c = pick(pal, n); *hh = 0.6 + 0.2 * n;
        if lx == 0 || ly == 0 { *c = hx(mortar) * (0.8 + 0.2 * h(x, y, s + 3)); *hh = 0.0; }
        else if lx == 1 || ly == 1 { *c *= 1.12; *hh = 0.75; }
        else if lx == rh * 2 - 1 || ly == rh - 1 { *c *= 0.82; *hh = 0.45; }
    });
    t
}
fn moss(s: u32) -> Tex {
    let mut t = stone(&[0x6e7278, 0x8c9097, 0xa0a4aa], s, 0.8);
    t.rough = 0.9;
    t.each(|x, y, c, hh| { let m = fbm(x as f32, y as f32, 3, 3, s + 50) + h(x, y, s + 51) * 0.15; if m > 0.5 { *c = pick(&[0x4a6a2a, 0x66804e, 0x7a9a4a], h(x, y, s + 52)); *hh = 0.7 + m * 0.2; } });
    t
}
fn gravel(s: u32) -> Tex {
    let mut t = Tex::new(0.95, 3.0);
    let cells = Cells::new(26, s);
    t.each(|x, y, c, hh| {
        let (f1, f2, id) = cells.at(x as f32 + 0.5, y as f32 + 0.5);
        *c = pick(&[0x5a524a, 0x837b70, 0x9a9288, 0x6a6058], hash2(id as i32, 3, s)) * (0.9 + 0.15 * (1.0 - f1 / 4.0).max(0.0));
        *hh = (1.0 - f1 / 4.0).max(0.0);
        if f2 - f1 < 0.7 { *c *= 0.55; *hh = 0.0; }
    });
    t
}
fn birch(s: u32) -> Tex {
    let mut t = Tex::new(0.8, 2.0);
    t.each(|x, y, c, hh| { let n = fbm(x as f32, y as f32, 6, 2, s); *c = pick(&[0xcfc8b8, 0xe6e2d6, 0xf4f1ea], n); *hh = 0.6; });
    let mut rng = crate::noise::mulberry32(s + 1);
    for _ in 0..14 {
        let (x0, y0, len) = ((rng() * NF) as i32, (rng() * NF) as i32, 2 + (rng() * 5.0) as i32);
        for i in 0..len { t.px(x0 + i, y0, hx(0x2a2622), 0.2); if rng() < 0.3 { t.px(x0 + i, y0 + 1, hx(0x4a443a), 0.3); } }
    }
    t
}
fn crystal(s: u32) -> Tex {
    let mut t = Tex::new(0.15, 2.5);
    t.each(|x, y, c, hh| {
        let band = ((x + y) as f32 / 6.0 + fbm(x as f32, y as f32, 2, 2, s) * 1.5).fract();
        let edge = ((x - y).rem_euclid(11)) == 0;
        *c = pick(&[0x7a4ad8, 0xb98aff, 0xe8d0ff], band); *hh = band;
        if edge { *c = hx(0xf8f0ff); *hh = 1.0; }
    });
    t
}
fn planks(s: u32) -> Tex {
    let mut t = Tex::new(0.85, 2.6);
    t.each(|x, y, c, hh| {
        let board = y / 8; let joint = (hash2(board, 0, s) * NF) as i32;
        let grain = fbm(x as f32, y as f32, 2, 16, s + board as u32);
        *c = pick(&[0x7a5430, 0xa77845, 0xc0925a], grain * 0.7 + hash2(board, 1, s) * 0.3); *hh = 0.6;
        if y % 8 == 0 { *c *= 0.5; *hh = 0.0; } else if y % 8 == 7 { *c *= 0.85; }
        if x == joint { *c *= 0.55; *hh = 0.1; }
        if (x == (joint + 3).rem_euclid(N as i32) || x == (joint - 3).rem_euclid(N as i32)) && (y % 8 == 2 || y % 8 == 5) { *c = hx(0x5a5a5a); *hh = 0.9; }
    });
    t
}
fn cloth(s: u32) -> Tex {
    let mut t = Tex::new(0.95, 1.5);
    t.each(|x, y, c, hh| {
        let w = (x + y) % 2 == 0;
        *c = hx(0xb04a3a) * if w { 1.0 } else { 0.85 } * (0.9 + 0.1 * h(x, y, s)); *hh = if w { 0.6 } else { 0.4 };
        if y % 16 == 3 || y % 16 == 4 { *c = hx(0xe0b84a) * if w { 1.0 } else { 0.85 }; }
    });
    t
}
fn gold(s: u32) -> Tex {
    let mut t = Tex::new(0.3, 2.5);
    t.each(|x, y, c, hh| {
        let n = fbm(x as f32, y as f32, 4, 4, s);
        *c = pick(&[0xc8902a, 0xf0c04a, 0xffe08a], n * 0.7 + 0.15); *hh = 0.5;
        let e = x.min(y).min(N as i32 - 1 - x).min(N as i32 - 1 - y);
        if e < 2 { *c = if x < 2 || y < 2 { hx(0xfff0b0) } else { hx(0x9a6a1a) }; *hh = 0.3; }
    });
    t
}
fn ember(s: u32) -> Tex {
    let mut t = Tex::new(0.9, 3.0);
    let cells = Cells::new(9, s);
    t.each(|x, y, c, hh| {
        let (f1, f2, _) = cells.at(x as f32 + 0.5, y as f32 + 0.5);
        let crack = smooth(1.6, 0.3, f2 - f1);
        *c = hx(0x2a1410).lerp(hx(0xffd27a), crack) * (0.85 + 0.3 * h(x, y, s)) + Vec3::splat(0.0) * f1; *hh = 1.0 - crack;
    });
    t
}
fn rune(s: u32) -> Tex {
    let mut t = stone(&[0x1a1418, 0x2a2228, 0x3a3038], s, 0.6);
    let mut rng = crate::noise::mulberry32(s + 7);
    for _ in 0..5 {
        let (mut x, mut y) = (4.0 + rng() * 24.0, 4.0 + rng() * 24.0);
        let mut dir = Vec2::new(rng() - 0.5, rng() - 0.5).normalize_or_zero();
        for i in 0..10 { t.px(x as i32, y as i32, hx(0xff8a9a), 0.3); x += dir.x; y += dir.y; if i % 3 == 2 { dir = Vec2::new(-dir.y, dir.x); } }
    }
    t
}

/// paint every layer and build the albedo and normal/roughness texture arrays
pub fn build(images: &mut Assets<Image>) -> (Handle<Image>, Handle<Image>) {
    let grass = [0x3a6a24, 0x4c8630, 0x5c9a38, 0x72ae48];
    let fgrass = [0x2e5e20, 0x46802e, 0x569634, 0x6aa848];
    let mut layers: Vec<Tex> = Vec::new();
    for i in 0..l::COUNT {
        let s = 1000 + i * 97;
        layers.push(match i {
            l::GRASS => grass_top(&grass, false, s), l::GRASS_SIDE => with_cap(dirt(s), &grass, 5.0, s + 3), l::DIRT => dirt(s),
            l::STONE => stone(&[0x6e7278, 0x8c9097, 0xa4a8ae], s, 0.9), l::SAND => sand(s), l::SNOW => snow(s), l::SNOW_SIDE => with_cap(dirt(s), &[0xcfdaea, 0xf2f6fa, 0xffffff], 6.0, s + 3),
            l::BARK => bark(&[0x3e2716, 0x5a3a22, 0x6a4528, 0x7a5432], s), l::RINGS => rings(&[0x7a5432, 0xa77845, 0xc89a62], 0x5a3a22, s),
            l::LEAVES => leaves(&[0x1e4a1a, 0x2e6a26, 0x3f8a2f, 0x5aa83c, 0x7cc050], s, false), l::BERRY => leaves(&[0x1e4a1a, 0x2e6a26, 0x3a7a2c, 0x52983a, 0x6ab048], s, true),
            l::AUTUMN => leaves(&[0x6a2a12, 0xa84a1a, 0xd4772a, 0xe89a3a, 0xf4c25a], s, false),
            l::PINE => needles(&[0x14321e, 0x1f4a2a, 0x2b5d36, 0x3f7a48], s), l::CACTUS => cactus(s, false), l::CACTUS_TOP => cactus(s, true),
            l::BRICK => bricks(&[0x7e786c, 0x9f988b, 0xb4ad9e], s, 4, 0x4a463e), l::MOSS => moss(s), l::GRAVEL => gravel(s),
            l::FGRASS => grass_top(&fgrass, true, s), l::FGRASS_SIDE => with_cap(dirt(s), &fgrass, 5.0, s + 3), l::BIRCH => birch(s),
            l::BIRCH_TOP => rings(&[0xb89a6a, 0xd8c08a, 0xe8d8a8], 0xe6e2d6, s), l::CRYSTAL => crystal(s), l::PLANK => planks(s), l::CLOTH => cloth(s),
            l::GOLD => gold(s), l::EMBER => ember(s), l::DSTONE => bricks(&[0x4a4e56, 0x5a5e66, 0x6a6e76], s, 2, 0x2a2c30), l::RUNE => rune(s),
            _ => stone(&[0x808080, 0x909090], s, 1.0),
        });
    }
    let mips = 6;
    let (mut alb, mut nrm) = (vec![], vec![]);
    for t in &layers {
        // level 0
        let mut col: Vec<Vec3> = t.c.clone();
        let mut nr: Vec<Vec4> = (0..N * N).map(|i| {
            let (x, y) = (i % N, i / N);
            let g = |dx: i32, dy: i32| t.h[((y as i32 + dy).rem_euclid(N as i32) as usize) * N + (x as i32 + dx).rem_euclid(N as i32) as usize];
            let (du, dv) = ((g(1, 0) - g(-1, 0)) * 0.5, (g(0, 1) - g(0, -1)) * 0.5);
            Vec3::new(-du * t.bump, -dv * t.bump, 1.0).normalize().extend(t.rough)
        }).collect();
        let mut size = N;
        for m in 0..mips {
            for c in &col { alb.extend_from_slice(&[(c.x.clamp(0.0, 1.0) * 255.0) as u8, (c.y.clamp(0.0, 1.0) * 255.0) as u8, (c.z.clamp(0.0, 1.0) * 255.0) as u8, 255]); }
            for n in &nr { let v = n.truncate().normalize_or_zero(); nrm.extend_from_slice(&[((v.x * 0.5 + 0.5) * 255.0) as u8, ((v.y * 0.5 + 0.5) * 255.0) as u8, ((v.z * 0.5 + 0.5) * 255.0) as u8, (n.w.clamp(0.0, 1.0) * 255.0) as u8]); }
            if m + 1 == mips { break; }
            let half = size / 2;
            let down3 = |v: &Vec<Vec3>| (0..half * half).map(|i| { let (x, y) = (i % half * 2, i / half * 2); (v[y * size + x] + v[y * size + x + 1] + v[(y + 1) * size + x] + v[(y + 1) * size + x + 1]) * 0.25 }).collect::<Vec<_>>();
            let down4 = |v: &Vec<Vec4>| (0..half * half).map(|i| { let (x, y) = (i % half * 2, i / half * 2); (v[y * size + x] + v[y * size + x + 1] + v[(y + 1) * size + x] + v[(y + 1) * size + x + 1]) * 0.25 }).collect::<Vec<_>>();
            col = down3(&col); nr = down4(&nr); size = half;
        }
    }
    let mk = |data: Vec<u8>, fmt: TextureFormat| {
        let mut img = Image::new(Extent3d { width: N as u32, height: N as u32, depth_or_array_layers: l::COUNT }, TextureDimension::D2, vec![0; N * N * 4 * l::COUNT as usize], fmt, RenderAssetUsages::RENDER_WORLD);
        img.data = data;
        img.texture_descriptor.mip_level_count = mips as u32;
        img.texture_view_descriptor = Some(TextureViewDescriptor { dimension: Some(TextureViewDimension::D2Array), ..default() });
        img.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor { address_mode_u: ImageAddressMode::Repeat, address_mode_v: ImageAddressMode::Repeat,
            mag_filter: ImageFilterMode::Nearest, min_filter: ImageFilterMode::Linear, mipmap_filter: ImageFilterMode::Linear, ..default() });
        img
    };
    (images.add(mk(alb, TextureFormat::Rgba8UnormSrgb)), images.add(mk(nrm, TextureFormat::Rgba8Unorm)))
}
