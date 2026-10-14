//! Deterministic hashing and 2D simplex noise, ported from the original game so seeds produce the same worlds.

pub fn mulberry32(mut a: u32) -> impl FnMut() -> f32 {
    move || {
        a = a.wrapping_add(0x6D2B79F5);
        let mut t = (a ^ (a >> 15)).wrapping_mul(1 | a);
        t = (t.wrapping_add((t ^ (t >> 7)).wrapping_mul(61 | t))) ^ t;
        ((t ^ (t >> 14)) as f64 / 4294967296.0) as f32
    }
}

#[inline]
pub fn hash2(x: i32, z: i32, s: u32) -> f32 {
    let mut h = x.wrapping_mul(374761393).wrapping_add(z.wrapping_mul(668265263)).wrapping_add((s as i32).wrapping_mul(1013904223)) as u32;
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^= h >> 16;
    h = h.wrapping_mul(0x45d9f3b);
    h ^= h >> 15;
    (h as f64 / 4294967296.0) as f32
}
#[inline]
pub fn hash3(x: i32, y: i32, z: i32, s: u32) -> f32 { hash2(x, z.wrapping_add(y.wrapping_mul(198491317)), s) }
#[inline]
pub fn hash_int(x: i32, z: i32, s: u32) -> u32 { (hash2(x, z, s) as f64 * 4294967296.0) as u32 }

#[derive(Clone)]
pub struct Simplex { perm: [u8; 512] }
const GX: [f32; 8] = [1., -1., 1., -1., 1., -1., 0., 0.];
const GY: [f32; 8] = [1., 1., -1., -1., 0., 0., 1., -1.];

impl Simplex {
    pub fn new(seed: u32) -> Self {
        let mut rng = mulberry32(seed);
        let mut p = [0u8; 256];
        for (i, v) in p.iter_mut().enumerate() { *v = i as u8; }
        for i in (1..256).rev() { let j = (rng() * (i as f32 + 1.0)) as usize; p.swap(i, j.min(i)); }
        let mut perm = [0u8; 512];
        for i in 0..512 { perm[i] = p[i & 255]; }
        Simplex { perm }
    }
    pub fn get(&self, xin: f32, yin: f32) -> f32 {
        let f2 = 0.5 * (3f32.sqrt() - 1.0);
        let g2 = (3.0 - 3f32.sqrt()) / 6.0;
        let s = (xin + yin) * f2;
        let i = (xin + s).floor();
        let j = (yin + s).floor();
        let t = (i + j) * g2;
        let x0 = xin - (i - t);
        let y0 = yin - (j - t);
        let (i1, j1) = if x0 > y0 { (1, 0) } else { (0, 1) };
        let x1 = x0 - i1 as f32 + g2;
        let y1 = y0 - j1 as f32 + g2;
        let x2 = x0 - 1.0 + 2.0 * g2;
        let y2 = y0 - 1.0 + 2.0 * g2;
        let ii = (i as i32 & 255) as usize;
        let jj = (j as i32 & 255) as usize;
        let p = &self.perm;
        let mut n = 0.0;
        let mut tt = 0.5 - x0 * x0 - y0 * y0;
        if tt > 0.0 { let g = (p[ii + p[jj] as usize] & 7) as usize; tt *= tt; n += tt * tt * (GX[g] * x0 + GY[g] * y0); }
        tt = 0.5 - x1 * x1 - y1 * y1;
        if tt > 0.0 { let g = (p[ii + i1 + p[jj + j1] as usize] & 7) as usize; tt *= tt; n += tt * tt * (GX[g] * x1 + GY[g] * y1); }
        tt = 0.5 - x2 * x2 - y2 * y2;
        if tt > 0.0 { let g = (p[ii + 1 + p[jj + 1] as usize] & 7) as usize; tt *= tt; n += tt * tt * (GX[g] * x2 + GY[g] * y2); }
        70.0 * n
    }
    pub fn fbm(&self, x: f32, z: f32, oct: u32) -> f32 {
        let (mut a, mut f, mut s, mut t) = (1.0, 1.0, 0.0, 0.0);
        for _ in 0..oct { s += self.get(x * f, z * f) * a; t += a; a *= 0.5; f *= 2.03; }
        s / t
    }
}

pub fn smooth(a: f32, b: f32, x: f32) -> f32 { let t = ((x - a) / (b - a)).clamp(0.0, 1.0); t * t * (3.0 - 2.0 * t) }
pub fn lerp(a: f32, b: f32, t: f32) -> f32 { a + (b - a) * t }
