//! Voxel world: blocks, biomes, terrain generation, points of interest and features (trees, boulders, crystals).
use crate::noise::*;
use bevy::prelude::*;
use std::collections::HashMap;

pub const CS: i32 = 16;
pub const CH: i32 = 64;
pub const CS2: i32 = CS * CS;
pub const SEA: i32 = 15;
pub const WATER_Y: f32 = SEA as f32 + 0.88;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Bio { Beach, Plains, Forest, Desert, Snow, Peak }

pub mod b {
    pub const AIR: u8 = 0; pub const GRASS: u8 = 1; pub const DIRT: u8 = 2; pub const STONE: u8 = 3; pub const SAND: u8 = 4; pub const SNOW: u8 = 5;
    pub const WOOD: u8 = 6; pub const LEAVES: u8 = 7; pub const PINE: u8 = 8; pub const CACTUS: u8 = 9; pub const BRICK: u8 = 10; pub const MOSS: u8 = 11;
    pub const GRAVEL: u8 = 12; pub const FGRASS: u8 = 13; pub const BIRCH: u8 = 14; pub const AUTUMN: u8 = 15; pub const CRYSTAL: u8 = 16; pub const BUSH: u8 = 17;
    pub const BERRY: u8 = 18; pub const SPINE: u8 = 19; pub const PLANK: u8 = 20; pub const CLOTH: u8 = 21; pub const GOLD: u8 = 22; pub const EMBER: u8 = 23;
    pub const DSTONE: u8 = 24; pub const RUNE: u8 = 25;
}

/// Block look: base colour, optional top colour, optional side colour for "wrapped" grass/snow, jitter, tinting and glow.
#[derive(Clone, Copy)]
pub struct BlockDef { pub c: [f32; 3], pub top: Option<[f32; 3]>, pub side: Option<[f32; 3]>, pub j: f32, pub tint: bool, pub glow: f32 }

/// sRGB hex to linear RGB, so colours match what the eye expects under Bevy's colour-managed renderer.
pub fn lin(hex: u32) -> [f32; 3] {
    let c = Color::srgb_u8((hex >> 16) as u8, (hex >> 8) as u8, hex as u8).to_linear();
    [c.red, c.green, c.blue]
}

pub fn block_def(id: u8) -> BlockDef {
    let d = |c: u32| BlockDef { c: lin(c), top: None, side: None, j: 0.07, tint: false, glow: 0.0 };
    match id {
        b::GRASS => BlockDef { top: Some(lin(0x6cb243)), side: Some(lin(0x8a5b34)), tint: true, j: 0.05, ..d(0x5a9638) },
        b::DIRT => BlockDef { j: 0.08, ..d(0x8a5b34) },
        b::STONE => BlockDef { j: 0.09, ..d(0x8c9097) },
        b::SAND => BlockDef { j: 0.05, ..d(0xe0cd8c) },
        b::SNOW => BlockDef { top: Some(lin(0xf5f8fb)), side: Some(lin(0x8a6a4a)), j: 0.03, ..d(0xe4ebf2) },
        b::WOOD => BlockDef { j: 0.08, ..d(0x6a4528) },
        b::LEAVES => BlockDef { tint: true, j: 0.13, ..d(0x3f8a2f) },
        b::PINE => BlockDef { j: 0.1, ..d(0x2b5d36) },
        b::CACTUS => BlockDef { j: 0.08, ..d(0x4c8b37) },
        b::BRICK => BlockDef { j: 0.12, ..d(0x9f988b) },
        b::MOSS => BlockDef { j: 0.12, ..d(0x66804e) },
        b::GRAVEL => BlockDef { j: 0.15, ..d(0x837b70) },
        b::FGRASS => BlockDef { top: Some(lin(0x569634)), side: Some(lin(0x6e4a2c)), tint: true, j: 0.05, ..d(0x46802e) },
        b::BIRCH => BlockDef { j: 0.1, ..d(0xe6e2d6) },
        b::AUTUMN => BlockDef { j: 0.15, ..d(0xd4772a) },
        b::CRYSTAL => BlockDef { j: 0.15, glow: 2.2, ..d(0xb98aff) },
        b::BUSH => BlockDef { j: 0.12, ..d(0x3a7a2c) },
        b::BERRY => BlockDef { j: 0.1, ..d(0x3a7a2c) },
        b::SPINE => BlockDef { top: Some(lin(0xeef3f6)), j: 0.08, ..d(0x2b5d36) },
        b::PLANK => BlockDef { j: 0.08, ..d(0xa77845) },
        b::CLOTH => BlockDef { j: 0.05, ..d(0xb04a3a) },
        b::GOLD => BlockDef { j: 0.08, glow: 1.4, ..d(0xf0c04a) },
        b::EMBER => BlockDef { j: 0.25, glow: 6.0, ..d(0xff8a2a) },
        b::DSTONE => BlockDef { j: 0.1, ..d(0x62666e) },
        b::RUNE => BlockDef { j: 0.18, glow: 5.0, ..d(0xe8344e) },
        _ => d(0xff00ff),
    }
}
pub fn is_foliage_block(id: u8) -> bool { matches!(id, b::LEAVES | b::PINE | b::SPINE | b::AUTUMN | b::CACTUS) }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PoiKind { Ruin, Camp, Shrine, Altar }
#[derive(Clone, Debug)]
pub struct Poi { pub kind: PoiKind, pub x: i32, pub z: i32, pub y: i32, pub r: i32, pub gx: i32, pub gz: i32 }

pub struct Chunk {
    pub cx: i32, pub cz: i32, pub d: Vec<u8>, pub top: Vec<u8>, pub top_b: Vec<u8>, pub bio: Vec<Bio>,
    pub dirty: bool, pub entity: Option<Entity>, pub fol_entity: Option<Entity>,
}
impl Chunk {
    #[inline] pub fn idx(lx: i32, y: i32, lz: i32) -> usize { (lx + lz * CS + y * CS2) as usize }
    fn recalc_top(&mut self, lx: i32, lz: i32) {
        let i = (lx + lz * CS) as usize;
        for y in (0..CH).rev() { let v = self.d[Chunk::idx(lx, y, lz)]; if v != 0 { self.top[i] = y as u8; self.top_b[i] = v; return; } }
        self.top[i] = 0; self.top_b[i] = 0;
    }
}
pub struct Column { pub h: i32, pub b: Bio }

#[derive(Resource)]
pub struct Voxels {
    pub seed: u32,
    n_c: Simplex, n_h: Simplex, n_r: Simplex, n_t: Simplex, n_m: Simplex, pub n_v: Simplex, n_l: Simplex,
    pub chunks: HashMap<(i32, i32), Chunk>,
    poi_cache: std::sync::Mutex<HashMap<(i32, i32), Option<Poi>>>,
}

type Setter<'a> = dyn FnMut(i32, i32, i32, u8, bool) + 'a;

impl Voxels {
    pub fn new(seed: u32) -> Self {
        let s = seed;
        Voxels { seed, n_c: Simplex::new(s), n_h: Simplex::new(s + 1), n_r: Simplex::new(s + 2), n_t: Simplex::new(s + 3), n_m: Simplex::new(s + 4), n_v: Simplex::new(s + 5), n_l: Simplex::new(s + 6),
            chunks: HashMap::new(), poi_cache: std::sync::Mutex::new(HashMap::new()) }
    }
    /// broad, gently rolling land with lakes in the hollows and real mountains only in rare ranges
    pub fn column(&self, x: i32, z: i32) -> Column {
        let (x, z) = (x as f32, z as f32);
        let t = self.n_t.fbm(x * 0.0021, z * 0.0021, 2);
        let m = self.n_m.fbm(x * 0.0024 + 31.7, z * 0.0024 - 12.3, 2);
        let c = self.n_c.fbm(x * 0.0026, z * 0.0026, 3);
        let hl = self.n_h.fbm(x * 0.011, z * 0.011, 2);
        let rr = 1.0 - self.n_r.get(x * 0.0052, z * 0.0052).abs();
        let mm = smooth(0.32, 0.75, self.n_c.get(x * 0.0011 + 77.0, z * 0.0011 - 41.0));
        let lk = smooth(0.3, 0.58, self.n_l.fbm(x * 0.005, z * 0.005, 2));
        let mut h = 22.5 + c * 5.0 + hl * 1.3 + mm * (4.0 + rr * rr * 21.0) - lk * 9.5 * (1.0 - mm);
        let des = ((t - 0.24) * 5.0).clamp(0.0, 1.0) * ((0.1 - m) * 5.0).clamp(0.0, 1.0);
        h = lerp(h, 20.5 + c * 2.5 + hl * 0.9, des * 0.8);
        let h = h.clamp(3.0, (CH - 12) as f32).floor() as i32;
        let b = if h <= SEA + 1 { Bio::Beach } else if h >= 39 { Bio::Peak } else if t < -0.24 { Bio::Snow } else if des > 0.5 { Bio::Desert } else if m > 0.14 { Bio::Forest } else { Bio::Plains };
        Column { h, b }
    }
    pub fn poi(&self, gx: i32, gz: i32) -> Option<Poi> {
        if let Some(p) = self.poi_cache.lock().unwrap().get(&(gx, gz)) { return p.clone(); }
        let s = self.seed;
        let mut p = None;
        if hash2(gx, gz, s + 101) < 0.62 && !(gx == 0 && gz == 0) {
            let x = gx * 64 + 16 + (hash2(gx, gz, s + 102) * 32.0) as i32;
            let z = gz * 64 + 16 + (hash2(gx, gz, s + 103) * 32.0) as i32;
            let col = self.column(x, z);
            if col.h > SEA + 2 && col.h < 38 {
                let tr = hash2(gx, gz, s + 104);
                let kind = if tr < 0.34 { PoiKind::Ruin } else if tr < 0.6 { PoiKind::Camp } else if tr < 0.8 { PoiKind::Shrine } else { PoiKind::Altar };
                p = Some(Poi { kind, x, z, y: col.h, r: if kind == PoiKind::Shrine { 6 } else { 7 }, gx, gz });
            }
        }
        self.poi_cache.lock().unwrap().insert((gx, gz), p.clone());
        p
    }
    pub fn poi_near(&self, x: f32, z: f32, m: f32) -> Option<Poi> {
        let p = self.poi((x / 64.0).floor() as i32, (z / 64.0).floor() as i32)?;
        let (dx, dz) = (x - p.x as f32, z - p.z as f32);
        if dx * dx + dz * dz < (p.r as f32 + m).powi(2) { Some(p) } else { None }
    }

    // ---- block access ----
    pub fn ensure(&mut self, cx: i32, cz: i32) { if !self.chunks.contains_key(&(cx, cz)) { let c = self.gen(cx, cz); self.chunks.insert((cx, cz), c); } }
    pub fn get(&mut self, x: i32, y: i32, z: i32) -> u8 {
        if y < 0 { return b::STONE; }
        if y >= CH { return 0; }
        let (cx, cz) = (x.div_euclid(CS), z.div_euclid(CS));
        self.ensure(cx, cz);
        self.chunks[&(cx, cz)].d[Chunk::idx(x - cx * CS, y, z - cz * CS)]
    }
    /// read-only lookup for chunks already generated (anything else reads as air)
    pub fn peek(&self, x: i32, y: i32, z: i32) -> u8 {
        if y < 0 { return b::STONE; }
        if y >= CH { return 0; }
        let (cx, cz) = (x.div_euclid(CS), z.div_euclid(CS));
        match self.chunks.get(&(cx, cz)) { Some(c) => c.d[Chunk::idx(x - cx * CS, y, z - cz * CS)], None => 0 }
    }
    pub fn solid(&mut self, x: f32, y: f32, z: f32) -> bool { self.get(x.floor() as i32, y.floor() as i32, z.floor() as i32) != 0 }
    pub fn set(&mut self, x: i32, y: i32, z: i32, v: u8) {
        if y < 1 || y >= CH { return; }
        let (cx, cz) = (x.div_euclid(CS), z.div_euclid(CS));
        self.ensure(cx, cz);
        let (lx, lz) = (x - cx * CS, z - cz * CS);
        { let c = self.chunks.get_mut(&(cx, cz)).unwrap(); c.d[Chunk::idx(lx, y, lz)] = v; c.dirty = true; c.recalc_top(lx, lz); }
        let mut mark = |a: i32, b2: i32| { if let Some(c) = self.chunks.get_mut(&(a, b2)) { c.dirty = true; } };
        if lx == 0 { mark(cx - 1, cz); } if lx == CS - 1 { mark(cx + 1, cz); } if lz == 0 { mark(cx, cz - 1); } if lz == CS - 1 { mark(cx, cz + 1); }
    }
    pub fn top_at(&mut self, x: i32, z: i32) -> i32 {
        let (cx, cz) = (x.div_euclid(CS), z.div_euclid(CS));
        self.ensure(cx, cz);
        self.chunks[&(cx, cz)].top[((x - cx * CS) + (z - cz * CS) * CS) as usize] as i32
    }
    pub fn top_block(&mut self, x: i32, z: i32) -> u8 {
        let (cx, cz) = (x.div_euclid(CS), z.div_euclid(CS));
        self.ensure(cx, cz);
        self.chunks[&(cx, cz)].top_b[((x - cx * CS) + (z - cz * CS) * CS) as usize]
    }
    /// first standable surface at or below yHint+3
    pub fn ground_at(&mut self, x: f32, z: f32, y_hint: f32) -> f32 {
        let (fx, fz) = (x.floor() as i32, z.floor() as i32);
        let mut y = (CH - 2).min(y_hint.floor() as i32 + 3);
        while y > 0 { if self.get(fx, y, fz) != 0 && self.get(fx, y + 1, fz) == 0 { return (y + 1) as f32; } y -= 1; }
        self.top_at(fx, fz) as f32 + 1.0
    }
    /// an open surface tile (not water, not inside a tree) for spawning things
    pub fn surface_spot(&mut self, x: f32, z: f32) -> Option<Vec3> {
        let (fx, fz) = (x.floor() as i32, z.floor() as i32);
        let t = self.top_at(fx, fz);
        if t <= SEA || t >= CH - 3 { return None; }
        let bl = self.get(fx, t, fz);
        if is_foliage_block(bl) { return None; }
        if self.get(fx, t + 1, fz) != 0 || self.get(fx, t + 2, fz) != 0 { return None; }
        Some(Vec3::new(fx as f32 + 0.5, (t + 1) as f32, fz as f32 + 0.5))
    }
    pub fn spawn_point(&mut self) -> Vec3 {
        let mut r = 0;
        while r < 400 {
            let mut a = 0.0f32;
            while a < std::f32::consts::TAU {
                let (x, z) = ((a.cos() * r as f32).round() as i32, (a.sin() * r as f32).round() as i32);
                let c = self.column(x, z);
                if matches!(c.b, Bio::Plains | Bio::Forest) && c.h > SEA + 2 && self.poi_near(x as f32, z as f32, 4.0).is_none() {
                    let t = self.top_at(x, z);
                    if t > SEA + 1 && self.get(x, t + 1, z) == 0 && self.get(x, t + 2, z) == 0 && !is_foliage_block(self.get(x, t, z)) {
                        return Vec3::new(x as f32 + 0.5, (t + 1) as f32, z as f32 + 0.5);
                    }
                }
                a += 0.5;
            }
            r += 3;
        }
        Vec3::new(0.5, 40.0, 0.5)
    }

    // ---- generation ----
    fn fill_col(&self, d: &mut [u8], lx: i32, lz: i32, h: i32, bio: Bio, wx: i32, wz: i32) {
        let s = self.seed;
        let (mut top, mut sub) = match bio {
            Bio::Beach | Bio::Desert => (b::SAND, b::SAND),
            Bio::Snow => (b::SNOW, b::DIRT),
            Bio::Forest => (b::FGRASS, b::DIRT),
            Bio::Peak => (if h > 50 { b::SNOW } else { b::STONE }, b::STONE),
            Bio::Plains => (b::GRASS, b::DIRT),
        };
        if h < SEA { top = if hash2(wx, wz, s + 7) < 0.28 { b::GRAVEL } else { b::SAND }; sub = b::SAND; }
        let off = (hash2(wx >> 2, wz >> 2, s + 8) * 4.0) as i32;
        for y in 0..CH {
            let i = Chunk::idx(lx, y, lz);
            d[i] = if y > h { 0 } else if y == h { top } else if y > h - 4 { sub } else if (y + off) % 7 == 0 { b::DSTONE } else { b::STONE };
        }
    }
    pub fn gen(&self, cx: i32, cz: i32) -> Chunk {
        let mut d = vec![0u8; (CS2 * CH) as usize];
        let mut bio = vec![Bio::Plains; CS2 as usize];
        let mut hs = vec![0i32; CS2 as usize];
        let (x0, z0) = (cx * CS, cz * CS);
        for lz in 0..CS { for lx in 0..CS {
            let (wx, wz) = (x0 + lx, z0 + lz);
            let col = self.column(wx, wz);
            let i = (lx + lz * CS) as usize;
            bio[i] = col.b; hs[i] = col.h;
            self.fill_col(&mut d, lx, lz, col.h, col.b, wx, wz);
        } }
        {
            let dd = std::cell::RefCell::new(&mut d);
            let mut set = |x: i32, y: i32, z: i32, v: u8, soft: bool| {
                let (lx, lz) = (x - x0, z - z0);
                if lx < 0 || lz < 0 || lx >= CS || lz >= CS || y < 1 || y >= CH { return; }
                let i = Chunk::idx(lx, y, lz);
                let mut d = dd.borrow_mut();
                if soft && d[i] != 0 { return; }
                d[i] = v;
            };
            // points of interest: flatten the ground around them, then build
            let mut done = vec![];
            for gz in (z0 - 14).div_euclid(64)..=(z0 + CS + 14).div_euclid(64) { for gx in (x0 - 14).div_euclid(64)..=(x0 + CS + 14).div_euclid(64) {
                let Some(p) = self.poi(gx, gz) else { continue };
                if done.contains(&(gx, gz)) { continue; }
                done.push((gx, gz));
                let (r, rr) = (p.r as f32, p.r as f32 + 4.0);
                for lz in 0..CS { for lx in 0..CS {
                    let (wx, wz) = (x0 + lx, z0 + lz);
                    let ddist = (((wx - p.x).pow(2) + (wz - p.z).pow(2)) as f32).sqrt();
                    if ddist > rr { continue; }
                    let i = (lx + lz * CS) as usize;
                    let target = if ddist <= r { p.y } else { lerp(p.y as f32, hs[i] as f32, (ddist - r) / 4.0).round() as i32 };
                    hs[i] = target;
                    let bb = if bio[i] == Bio::Beach { Bio::Plains } else { bio[i] };
                    self.fill_col(&mut dd.borrow_mut(), lx, lz, target, bb, wx, wz);
                } }
                self.build_poi(&p, &mut set);
            } }
            self.features(x0, z0, &mut set);
        }
        let mut ch = Chunk { cx, cz, d, top: vec![0; CS2 as usize], top_b: vec![0; CS2 as usize], bio, dirty: true, entity: None, fol_entity: None };
        for lz in 0..CS { for lx in 0..CS { ch.recalc_top(lx, lz); } }
        ch
    }
    fn build_poi(&self, p: &Poi, set: &mut Setter) {
        let s = self.seed;
        let (cx, cz, h0) = (p.x, p.z, p.y);
        match p.kind {
            PoiKind::Ruin => {
                for dz in -6..=6 { for dx in -6..=6 {
                    let (x, z) = (cx + dx, cz + dz);
                    let r = hash2(x, z, s + 201);
                    if dx.abs() <= 5 && dz.abs() <= 5 {
                        if r > 0.12 { set(x, h0, z, if r < 0.42 { b::MOSS } else { b::BRICK }, false); }
                        let edge = dx.abs() == 5 || dz.abs() == 5;
                        if edge && !(dx == 0 || dz == 0) {
                            let hh = 1 + (hash2(x, z, s + 202) * 4.2) as i32;
                            if hash2(x, z, s + 203) > 0.16 { for y in 1..=hh { set(x, h0 + y, z, if hash3(x, y, z, s + 204) < 0.35 { b::MOSS } else { b::BRICK }, false); } }
                        }
                    } else if r < 0.08 { set(x, h0 + 1, z, if r < 0.03 { b::MOSS } else { b::BRICK }, false); }
                } }
                for (px, pz) in [(-3, -3), (3, -3), (-3, 3), (3, 3)] {
                    let (x, z) = (cx + px, cz + pz);
                    let hh = 2 + (hash2(x, z, s + 205) * 5.0) as i32;
                    for y in 1..=hh { set(x, h0 + y, z, if y == hh && hh > 4 { b::DSTONE } else { b::BRICK }, false); }
                }
                set(cx, h0, cz, b::GOLD, false);
            }
            PoiKind::Camp => {
                for dz in -4..=4 { for dx in -4..=4 { if dx * dx + dz * dz <= 13 && hash2(cx + dx, cz + dz, s + 211) < 0.7 { set(cx + dx, h0, cz + dz, b::GRAVEL, false); } } }
                set(cx, h0 + 1, cz, b::EMBER, false);
                for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] { set(cx + dx, h0 + 1, cz + dz, b::WOOD, false); }
                let flip = hash2(p.gx, p.gz, s + 212) < 0.5;
                for side in [-1, 1] { for l in -2..=1 { for yy in 0..3 {
                    let sgs: &[i32] = if yy == 2 { &[0] } else { &[-1, 1] };
                    for &sg in sgs {
                        let a = side * 5 + sg * (2 - yy);
                        let bb = l + if side > 0 { 2 } else { -2 };
                        let (x, z) = if flip { (cx + bb, cz + a) } else { (cx + a, cz + bb) };
                        set(x, h0 + 1 + yy, z, b::CLOTH, false);
                    }
                } } }
                set(cx + 3, h0 + 1, cz - 3, b::PLANK, false); set(cx + 3, h0 + 2, cz - 3, b::PLANK, false); set(cx + 4, h0 + 1, cz - 3, b::PLANK, false); set(cx - 3, h0 + 1, cz + 3, b::PLANK, false);
            }
            PoiKind::Altar => {
                for dz in -5i32..=5 { for dx in -5i32..=5 {
                    let dd = dx.abs().max(dz.abs());
                    if dd <= 4 { set(cx + dx, h0, cz + dz, if (dx + dz).rem_euclid(2) == 1 { b::DSTONE } else { b::BRICK }, false); }
                    if dd <= 2 { set(cx + dx, h0 + 1, cz + dz, b::DSTONE, false); }
                } }
                for (dx, dz) in [(-4, -4), (4, -4), (-4, 4), (4, 4)] {
                    let hh = 3 + (hash2(cx + dx, cz + dz, s + 231) * 2.0) as i32;
                    for y in 1..=hh { set(cx + dx, h0 + y, cz + dz, if y == hh { b::RUNE } else { b::DSTONE }, false); }
                }
                set(cx, h0 + 2, cz, b::DSTONE, false); set(cx, h0 + 3, cz, b::RUNE, false);
                for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] { set(cx + dx, h0 + 2, cz + dz, b::BRICK, false); }
            }
            PoiKind::Shrine => {
                for dz in -1..=1 { for dx in -1..=1 { set(cx + dx, h0, cz + dz, b::BRICK, false); } }
                set(cx, h0 + 1, cz, b::GOLD, false);
                for i in 0..7 {
                    let a = i as f32 / 7.0 * std::f32::consts::TAU + hash2(p.gx, p.gz, s + 221);
                    let (x, z) = (cx + (a.cos() * 5.0).round() as i32, cz + (a.sin() * 5.0).round() as i32);
                    let hh = 3 + (hash2(x, z, s + 222) * 2.0) as i32;
                    for y in 1..=hh { set(x, h0 + y, z, if hash3(x, y, z, s + 223) < 0.3 { b::MOSS } else { b::STONE }, false); }
                    if i % 2 == 0 { set(x, h0 + hh + 1, z, b::CRYSTAL, false); }
                }
            }
        }
    }
    fn features(&self, x0: i32, z0: i32, set: &mut Setter) {
        let s = self.seed;
        let mut grid = |g: i32, m: i32, salt: u32, f: &mut dyn FnMut(i32, i32, &Column, f32, &mut dyn FnMut() -> f32, &mut Setter)| {
            for gz in (z0 - m).div_euclid(g)..=(z0 + CS + m).div_euclid(g) { for gx in (x0 - m).div_euclid(g)..=(x0 + CS + m).div_euclid(g) {
                let tx = gx * g + (hash2(gx, gz, s + salt + 1) * g as f32) as i32;
                let tz = gz * g + (hash2(gx, gz, s + salt + 2) * g as f32) as i32;
                if self.poi_near(tx as f32, tz as f32, 3.0).is_some() { continue; }
                let col = self.column(tx, tz);
                if col.h <= SEA { continue; }
                let mut rng = mulberry32(hash_int(gx, gz, s + salt + 3));
                f(tx, tz, &col, hash2(gx, gz, s + salt), &mut rng, set);
            } }
        };
        // boulders
        grid(13, 4, 300, &mut |x, z, col, r, rng, set| {
            let dens = match col.b { Bio::Peak => 0.5, Bio::Desert => 0.2, _ => 0.14 };
            if r > dens { return; }
            let rad = 1.1 + rng() * 1.5; let mossy = matches!(col.b, Bio::Forest | Bio::Plains); let y0 = col.h;
            for dy in -2..=3 { for dz in -3..=3 { for dx in -3..=3 {
                let dd = (dx * dx) as f32 + (dy * dy) as f32 * 1.4 + (dz * dz) as f32;
                if dd <= rad * rad + hash3(x + dx, y0 + dy, z + dz, s + 301) * 1.2 {
                    let v = if dy > 0 && mossy && hash3(x + dx, y0 + dy, z + dz, s + 302) < 0.45 { b::MOSS } else if hash3(x + dx, y0 + dy, z + dz, s + 303) < 0.25 { b::DSTONE } else { b::STONE };
                    set(x + dx, y0 + dy, z + dz, v, false);
                }
            } } }
        });
        // trees: forests open into glades so there is always room to fight
        grid(5, 7, 400, &mut |x, z, col, r, rng, set| {
            let mut dens = [0.03, 0.05, 0.46, 0.07, 0.24, 0.05][col.b as usize];
            if matches!(col.b, Bio::Forest | Bio::Plains) { dens *= smooth(-0.35, 0.25, self.n_v.get(x as f32 * 0.021 + 11.0, z as f32 * 0.021 - 7.0)); }
            if r > dens { return; }
            let y = col.h + 1;
            match col.b {
                Bio::Desert => { self.cactus(set, x, y, z, rng); return; }
                Bio::Snow | Bio::Peak => { self.pine(set, x, y, z, rng, col.b == Bio::Snow || col.h > 38); return; }
                Bio::Beach => return,
                _ => {}
            }
            let autumn = self.n_v.get(x as f32 * 0.012, z as f32 * 0.012) > 0.3;
            let q = rng();
            if col.b == Bio::Forest && q < 0.1 { self.big_tree(set, x, y, z, rng, if autumn { b::AUTUMN } else { b::LEAVES }); }
            else if q < 0.38 { self.birch(set, x, y, z, rng); }
            else { let lf = if autumn && rng() < 0.7 { b::AUTUMN } else { b::LEAVES }; self.oak(set, x, y, z, rng, lf); }
        });
        // bushes
        grid(4, 2, 500, &mut |x, z, col, r, rng, set| {
            let dens = [0.0, 0.2, 0.28, 0.04, 0.05, 0.0][col.b as usize];
            if r > dens { return; }
            let y = col.h + 1;
            let bb = if col.b == Bio::Desert { b::WOOD } else if rng() < 0.5 { b::BERRY } else { b::BUSH };
            set(x, y, z, bb, true);
            if rng() < 0.5 { let ox = if rng() < 0.5 { 1 } else { -1 }; set(x + ox, y, z, if bb == b::WOOD { b::WOOD } else { b::BUSH }, true); }
        });
        // crystals
        grid(14, 3, 600, &mut |x, z, col, r, rng, set| {
            if !matches!(col.b, Bio::Peak | Bio::Snow) || r > 0.28 { return; }
            let y = col.h + 1;
            for _ in 0..4 {
                let (dx, dz, hh) = ((rng() * 3.0) as i32 - 1, (rng() * 3.0) as i32 - 1, 1 + (rng() * 4.0) as i32);
                for k in 0..hh { set(x + dx, y + k, z + dz, b::CRYSTAL, true); }
            }
        });
    }
    fn leaf_blob(&self, set: &mut Setter, x: i32, y: i32, z: i32, rx: f32, ry: f32, leaf: u8, salt: u32) {
        let (ix, iy) = (rx.ceil() as i32, ry.ceil() as i32);
        for dy in -iy..=iy { for dz in -ix..=ix { for dx in -ix..=ix {
            let dd = ((dx * dx + dz * dz) as f32) / (rx * rx) + ((dy * dy) as f32) / (ry * ry);
            if dd > 1.05 { continue; }
            if dd > 0.6 && hash3(x + dx, y + dy, z + dz, self.seed + salt) < 0.22 { continue; }
            set(x + dx, y + dy, z + dz, leaf, true);
        } } }
    }
    fn oak(&self, set: &mut Setter, x: i32, y: i32, z: i32, rng: &mut dyn FnMut() -> f32, leaf: u8) {
        let h = 4 + (rng() * 3.0) as i32; self.leaf_blob(set, x, y + h - 1, z, 2.6, 2.1, leaf, 401); for i in 0..h { set(x, y + i, z, b::WOOD, false); }
    }
    fn birch(&self, set: &mut Setter, x: i32, y: i32, z: i32, rng: &mut dyn FnMut() -> f32) {
        let h = 6 + (rng() * 3.0) as i32; self.leaf_blob(set, x, y + h - 1, z, 1.9, 2.6, b::LEAVES, 402); for i in 0..h { set(x, y + i, z, b::BIRCH, false); }
    }
    fn big_tree(&self, set: &mut Setter, x: i32, y: i32, z: i32, rng: &mut dyn FnMut() -> f32, leaf: u8) {
        let h = 8 + (rng() * 3.0) as i32;
        self.leaf_blob(set, x, y + h, z, 4.2, 3.0, leaf, 403); self.leaf_blob(set, x + 2, y + h - 3, z - 1, 2.2, 1.6, leaf, 404);
        for i in 0..h + 1 { for (a, c) in [(0, 0), (1, 0), (0, 1), (1, 1)] { set(x + a, y + i, z + c, b::WOOD, false); } }
        set(x - 1, y, z, b::WOOD, false); set(x + 2, y, z + 1, b::WOOD, false); set(x, y, z + 2, b::WOOD, false);
    }
    fn pine(&self, set: &mut Setter, x: i32, y: i32, z: i32, rng: &mut dyn FnMut() -> f32, snowy: bool) {
        let h = 6 + (rng() * 4.0) as i32; let l = if snowy { b::SPINE } else { b::PINE };
        for k in 0..h - 1 {
            let yy = y + h - k;
            let r = (if k % 3 == 0 { 0.6 } else { 1.0 } + k as f32 * 0.42).min(3.0);
            if k < 1 { set(x, yy, z, l, true); continue; }
            for dz in -3..=3 { for dx in -3..=3 { if ((dx * dx + dz * dz) as f32) <= r * r { set(x + dx, yy, z + dz, l, true); } } }
        }
        set(x, y + h + 1, z, l, true);
        for i in 0..h { set(x, y + i, z, b::WOOD, false); }
    }
    fn cactus(&self, set: &mut Setter, x: i32, y: i32, z: i32, rng: &mut dyn FnMut() -> f32) {
        let h = 2 + (rng() * 3.0) as i32; for i in 0..h { set(x, y + i, z, b::CACTUS, false); }
        if h > 2 && rng() < 0.6 { let sx = if rng() < 0.5 { 1 } else { -1 }; set(x + sx, y + 1, z, b::CACTUS, false); set(x + sx, y + 2, z, b::CACTUS, false); }
    }
}

/// voxel DDA ray: first solid block within max distance
pub fn voxel_ray(w: &Voxels, o: Vec3, dir: Vec3, max_d: f32) -> Option<(IVec3, f32, u8)> {
    let (mut x, mut y, mut z) = (o.x.floor() as i32, o.y.floor() as i32, o.z.floor() as i32);
    let (sx, sy, sz) = (dir.x.signum() as i32 * (dir.x != 0.0) as i32, dir.y.signum() as i32 * (dir.y != 0.0) as i32, dir.z.signum() as i32 * (dir.z != 0.0) as i32);
    let td = |d: f32| if d != 0.0 { (1.0 / d).abs() } else { f32::INFINITY };
    let (tdx, tdy, tdz) = (td(dir.x), td(dir.y), td(dir.z));
    let first = |s: i32, p: f32, c: i32, t: f32| if s > 0 { (c as f32 + 1.0 - p) * t } else if s < 0 { (p - c as f32) * t } else { f32::INFINITY };
    let (mut tx, mut ty, mut tz) = (first(sx, o.x, x, tdx), first(sy, o.y, y, tdy), first(sz, o.z, z, tdz));
    let mut t = 0.0;
    for _ in 0..400 {
        if (0..CH).contains(&y) { let v = w.peek(x, y, z); if v != 0 { return Some((IVec3::new(x, y, z), t, v)); } }
        if tx < ty && tx < tz { x += sx; t = tx; tx += tdx; } else if ty < tz { y += sy; t = ty; ty += tdy; } else { z += sz; t = tz; tz += tdz; }
        if t > max_d { break; }
    }
    None
}
