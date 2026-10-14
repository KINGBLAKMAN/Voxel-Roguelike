//! Chunk meshing: visible faces with baked ambient occlusion, a separate HDR mesh for glowing blocks
//! (they feed the bloom), and a foliage mesh of grass tufts, flowers, reeds, ferns, mushrooms and lily pads.
use crate::noise::hash2;
use crate::noise::hash3;
use crate::world::*;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

#[derive(Default)]
pub struct MeshData { pub p: Vec<[f32; 3]>, pub n: Vec<[f32; 3]>, pub c: Vec<[f32; 4]>, pub i: Vec<u32>, pub uv: Vec<[f32; 2]>, pub uv2: Vec<[f32; 2]> }
impl MeshData {
    pub fn is_empty(&self) -> bool { self.p.is_empty() }
    pub fn into_mesh(self) -> Mesh {
        let n = self.p.len();
        let mut m = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD)
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.p)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.n)
            .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.c)
            .with_inserted_indices(Indices::U32(self.i));
        if self.uv.len() == n { m.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uv); }
        if self.uv2.len() == n { m.insert_attribute(Mesh::ATTRIBUTE_UV_1, self.uv2); }
        m
    }
}

struct Face { d: [i32; 3], c: [[i32; 3]; 4], s: f32 }
const FACES: [Face; 6] = [
    Face { d: [1, 0, 0], c: [[1, 0, 0], [1, 1, 0], [1, 1, 1], [1, 0, 1]], s: 0.92 },
    Face { d: [-1, 0, 0], c: [[0, 0, 1], [0, 1, 1], [0, 1, 0], [0, 0, 0]], s: 0.92 },
    Face { d: [0, 1, 0], c: [[0, 1, 1], [1, 1, 1], [1, 1, 0], [0, 1, 0]], s: 1.0 },
    Face { d: [0, -1, 0], c: [[0, 0, 0], [1, 0, 0], [1, 0, 1], [0, 0, 1]], s: 0.72 },
    Face { d: [0, 0, 1], c: [[1, 0, 1], [1, 1, 1], [0, 1, 1], [0, 0, 1]], s: 0.86 },
    Face { d: [0, 0, -1], c: [[0, 0, 0], [0, 1, 0], [1, 1, 0], [1, 0, 0]], s: 0.86 },
];
const AOF: [f32; 4] = [0.42, 0.6, 0.8, 1.0];

pub struct ChunkMeshes { pub solid: MeshData, pub glow: MeshData, pub foliage: MeshData }

pub fn mesh_chunk(w: &Voxels, cx: i32, cz: i32) -> ChunkMeshes {
    let ch = &w.chunks[&(cx, cz)];
    let (x0, z0) = (cx * CS, cz * CS);
    let max_y = ch.top.iter().copied().max().unwrap_or(0) as i32;
    let mut solid = MeshData::default();
    let mut glow = MeshData::default();
    let gl = |x: i32, y: i32, z: i32| -> bool {
        if y < 0 { return true; }
        if y >= CH { return false; }
        if (0..CS).contains(&x) && (0..CS).contains(&z) { ch.d[Chunk::idx(x, y, z)] != 0 } else { w.peek(x0 + x, y, z0 + z) != 0 }
    };
    // ambient occlusion corner offsets per face, precomputed like the original
    let mut ao_off = [[[[0i32; 3]; 3]; 4]; 6];
    for (fi, f) in FACES.iter().enumerate() {
        let a = if f.d[0] != 0 { 0 } else if f.d[1] != 0 { 1 } else { 2 };
        let (u, v) = ((a + 1) % 3, (a + 2) % 3);
        for k in 0..4 {
            let c = f.c[k];
            let su = if c[u] != 0 { 1 } else { -1 };
            let sv = if c[v] != 0 { 1 } else { -1 };
            let mut o1 = f.d; let mut o2 = f.d; let mut o3 = f.d;
            o1[u] += su; o2[v] += sv; o3[u] += su; o3[v] += sv;
            ao_off[fi][k] = [o1, o2, o3];
        }
    }
    for y in 0..=max_y { for z in 0..CS { for x in 0..CS {
        let bl = ch.d[Chunk::idx(x, y, z)];
        if bl == 0 { continue; }
        let def = block_def(bl);
        let (wx, wz) = (x0 + x, z0 + z);
        let jit = 1.0 + (hash3(wx, y, wz, 5) - 0.5) * def.j;
        let tint = if def.tint { 1.0 + w.n_v.get(wx as f32 * 0.05, wz as f32 * 0.05) * 0.14 } else { 1.0 };
        let glows = def.glow > 0.0;
        let capped = def.side.is_some() && !gl(x, y + 1, z);
        let out = if glows { &mut glow } else { &mut solid };
        for (fi, f) in FACES.iter().enumerate() {
            if gl(x + f.d[0], y + f.d[1], z + f.d[2]) { continue; }
            let side = fi != 2 && fi != 3;
            let layer = crate::textures::face_layer(bl, if fi == 2 { 0 } else if fi == 3 { 2 } else { 1 }, capped && side);
            let mut ao = [3usize; 4];
            if !glows { for k in 0..4 {
                let o = ao_off[fi][k];
                let s1 = gl(x + o[0][0], y + o[0][1], z + o[0][2]) as usize;
                let s2 = gl(x + o[1][0], y + o[1][1], z + o[1][2]) as usize;
                let s3 = gl(x + o[2][0], y + o[2][1], z + o[2][2]) as usize;
                ao[k] = if s1 == 1 && s2 == 1 { 0 } else { 3 - (s1 + s2 + s3) };
            } }
            let base = out.p.len() as u32;
            for k in 0..4 {
                let c = f.c[k];
                let (px, py, pz) = ((wx + c[0]) as f32, (y + c[1]) as f32, (wz + c[2]) as f32);
                out.p.push([px, py, pz]);
                out.n.push([f.d[0] as f32, f.d[1] as f32, f.d[2] as f32]);
                // world-space UVs; see the tangent frame in terrain.wgsl
                out.uv.push(if f.d[0] != 0 { [pz, -py] } else if f.d[1] != 0 { [px, pz] } else { [px, -py] });
                out.uv2.push([layer as f32, 0.0]);
                let m = if glows { def.glow * (0.85 + 0.15 * f.s) * jit } else { f.s * AOF[ao[k]] * jit };
                out.c.push([m * (2.0 - tint).max(0.8), m * tint, m, 1.0]);
            }
            if ao[0] + ao[2] < ao[1] + ao[3] { out.i.extend_from_slice(&[base + 1, base + 2, base + 3, base + 1, base + 3, base]); }
            else { out.i.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]); }
        }
    } } }
    ChunkMeshes { solid, glow, foliage: foliage(w, ch) }
}

fn foliage(w: &Voxels, ch: &Chunk) -> MeshData {
    let mut m = MeshData::default();
    let s = w.seed;
    let (x0, z0) = (ch.cx * CS, ch.cz * CS);
    // a crossed pair of double-sided quads; alpha carries the sway weight (0 at the root, 1 at the tip)
    let mut quad = |m: &mut MeshData, x: f32, y: f32, z: f32, ax: f32, az: f32, h: f32, c0: [f32; 3], c1: [f32; 3]| {
        let n = [0.0, 1.0, 0.0];
        let v = [[x - ax, y, z - az], [x + ax, y, z + az], [x + ax, y + h, z + az], [x - ax, y + h, z - az]];
        let cols = [c0, c0, c1, c1];
        let base = m.p.len() as u32;
        for k in 0..4 { m.p.push(v[k]); m.n.push(n); let c = cols[k]; m.c.push([c[0], c[1], c[2], if k >= 2 { 1.0 } else { 0.0 }]); }
        m.i.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    };
    let flat = |m: &mut MeshData, x: f32, y: f32, z: f32, r: f32, c: [f32; 3]| {
        let base = m.p.len() as u32;
        for p in [[x - r, y, z - r], [x + r, y, z - r], [x + r, y, z + r], [x - r, y, z + r]] { m.p.push(p); m.n.push([0.0, 1.0, 0.0]); m.c.push([c[0], c[1], c[2], 0.0]); }
        m.i.extend_from_slice(&[base, base + 2, base + 1, base, base + 3, base + 2]);
    };
    let sc = |c: [f32; 3], k: f32| [c[0] * k, c[1] * k, c[2] * k];
    for lz in 0..CS { for lx in 0..CS {
        let i = (lx + lz * CS) as usize;
        let (tb, ty) = (ch.top_b[i], ch.top[i] as i32);
        let (wx, wz) = (x0 + lx, z0 + lz);
        let r = hash2(wx, wz, s + 700);
        if ty < SEA && ty >= SEA - 2 && r < 0.09 && w.n_v.get(wx as f32 * 0.04, wz as f32 * 0.04) > -0.1 {
            let (x, z, y) = (wx as f32 + 0.5, wz as f32 + 0.5, WATER_Y + 0.02);
            flat(&mut m, x, y, z, 0.34, lin(0x3f8a34));
            if hash2(wx, wz, s + 708) < 0.3 { flat(&mut m, x + 0.05, y + 0.05, z, 0.13, lin(0xf6c6dc)); }
            continue;
        }
        if ty <= SEA || ty >= CH - 2 { continue; }
        let is_g = tb == b::GRASS || tb == b::FGRASS;
        let shore = ty <= SEA + 1 && (tb == b::SAND || is_g);
        let meadow = tb == b::GRASS && w.n_v.get(wx as f32 * 0.03 + 5.0, wz as f32 * 0.03 - 9.0) > 0.22;
        let dens = if shore { 0.4 } else if tb == b::GRASS { if meadow { 0.55 } else { 0.38 } } else if tb == b::FGRASS { 0.34 }
            else if tb == b::SAND && ch.bio[i] == Bio::Desert { 0.03 } else if tb == b::SNOW { 0.04 } else { 0.0 };
        if r > dens { continue; }
        let (x, z, y) = (wx as f32 + hash2(wx, wz, s + 701) * 0.5 + 0.25, wz as f32 + hash2(wx, wz, s + 702) * 0.5 + 0.25, (ty + 1) as f32);
        let q = hash2(wx, wz, s + 703);
        if shore {
            let hh = 0.9 + hash2(wx, wz, s + 709) * 0.7;
            quad(&mut m, x, y, z, 0.1, 0.22, hh, lin(0x5a6a30), lin(0x9ab05a)); quad(&mut m, x, y, z, 0.22, -0.1, hh, lin(0x5a6a30), lin(0x9ab05a));
            if q < 0.4 { quad(&mut m, x + 0.05, y + hh - 0.25, z, 0.05, 0.05, 0.28, lin(0x6a4428), lin(0x5a3820)); }
            continue;
        }
        if is_g && q < if meadow { 0.34 } else { 0.12 } {
            let pc = [0xe64545, 0xf2d14a, 0xf4f1ea, 0x6a8cff, 0xc070f0, 0xff9a3d][(hash2(wx, wz, s + 704) * 6.0) as usize % 6];
            quad(&mut m, x, y, z, 0.18, 0.18, 0.42, lin(0x3c7a2a), lin(0x4f9a35)); quad(&mut m, x, y, z, 0.18, -0.18, 0.42, lin(0x3c7a2a), lin(0x4f9a35));
            quad(&mut m, x, y + 0.36, z, 0.16, 0.16, 0.2, sc(lin(pc), 0.9), sc(lin(pc), 1.1)); quad(&mut m, x, y + 0.36, z, 0.16, -0.16, 0.2, sc(lin(pc), 0.9), sc(lin(pc), 1.1));
        } else if tb == b::FGRASS && q < 0.2 {
            if q < 0.06 {
                quad(&mut m, x, y, z, 0.05, 0.05, 0.22, lin(0xe8e0d0), lin(0xf4ece0)); quad(&mut m, x, y, z, 0.05, -0.05, 0.22, lin(0xe8e0d0), lin(0xf4ece0));
                let cap = if hash2(wx, wz, s + 710) < 0.6 { 0xd8343a } else { 0xc89a5a };
                quad(&mut m, x, y + 0.18, z, 0.17, 0.17, 0.12, sc(lin(cap), 0.7), lin(cap)); quad(&mut m, x, y + 0.18, z, 0.17, -0.17, 0.12, sc(lin(cap), 0.7), lin(cap));
            } else {
                for k in 0..3 { let an = k as f32 * 1.05 + q * 9.0; quad(&mut m, x, y, z, an.cos() * 0.5, an.sin() * 0.5, 0.55, lin(0x2a5a22), lin(0x5a9a3a)); }
            }
        } else {
            let hh = (0.28 + hash2(wx, wz, s + 705) * 0.4) * if meadow && q > 0.85 { 1.6 } else { 1.0 };
            let (c0, c1) = if is_g { if tb == b::GRASS { (lin(0x3f7a28), lin(0x7cbc4a)) } else { (lin(0x346622), lin(0x66a63c)) } }
                else if tb == b::SAND { (lin(0x7a5a33), lin(0xa98a55)) } else { (lin(0x8fa1a8), lin(0xd8e2e6)) };
            quad(&mut m, x, y, z, 0.28, 0.28, hh, c0, c1); quad(&mut m, x, y, z, 0.28, -0.28, hh, c0, c1);
        }
    } }
    m
}
