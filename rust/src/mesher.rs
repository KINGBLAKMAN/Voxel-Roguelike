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
        let v = if (0..CS).contains(&x) && (0..CS).contains(&z) { ch.d[Chunk::idx(x, y, z)] } else { w.peek(x0 + x, y, z0 + z) };
        v != 0 && !is_bush(v)
    };
    let mut fol = MeshData::default();
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
        let (wx, wz) = (x0 + x, z0 + z);
        if is_bush(bl) { bush(&mut fol, wx, y, wz, bl == b::BERRY, w.seed); continue; }
        let def = block_def(bl);
        let leafy = matches!(bl, b::LEAVES | b::AUTUMN | b::PINE | b::SPINE);
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
            if leafy && hash3(wx * 6 + fi as i32, y, wz, w.seed + 77) < if fi == 3 { 0.4 } else { 0.8 } { fringe(&mut fol, wx, y, wz, f.d, bl, w.seed + fi as u32); }
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
    foliage(w, ch, &mut fol);
    ChunkMeshes { solid, glow, foliage: fol }
}

pub fn is_bush(id: u8) -> bool { id == b::BUSH || id == b::BERRY }
use crate::textures::fl;
/// one textured card: centre, half-extent vectors, texture layer, colour, and sway weight at the bottom and top edge
fn card(m: &mut MeshData, c: Vec3, r: Vec3, u: Vec3, layer: u32, col: [f32; 3], s0: f32, s1: f32) {
    let n = r.cross(u).normalize_or_zero();
    let base = m.p.len() as u32;
    for (k, (pp, uv)) in [(c - r - u, [0.0, 1.0]), (c + r - u, [1.0, 1.0]), (c + r + u, [1.0, 0.0]), (c - r + u, [0.0, 0.0])].into_iter().enumerate() {
        m.p.push(pp.to_array()); m.n.push(n.to_array()); m.uv.push(uv); m.uv2.push([layer as f32, 0.0]);
        m.c.push([col[0], col[1], col[2], if k >= 2 { s1 } else { s0 }]);
    }
    m.i.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}
/// a round bush: a dome of leaf-cluster cards around the block, berries on some
fn bush(m: &mut MeshData, x: i32, y: i32, z: i32, berry: bool, seed: u32) {
    let ctr = Vec3::new(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5);
    let tint = 0.9 + hash3(x, y, z, seed + 31) * 0.2;
    let n = 11;
    for k in 0..n {
        let r = |q: u32| hash3(x * 13 + k, y, z, seed + 40 + q);
        let yaw = k as f32 / n as f32 * std::f32::consts::TAU + r(1) * 0.8;
        let up_t = r(2) * 1.2 - 0.25;
        let dir = Vec3::new(yaw.cos() * up_t.cos(), up_t.sin(), yaw.sin() * up_t.cos());
        let p = ctr + dir * (0.28 + r(3) * 0.12) + Vec3::Y * 0.05;
        let size = 0.55 + r(4) * 0.18;
        let right = Vec3::new(-yaw.sin(), 0.0, yaw.cos()) * size;
        let tilt = r(5) * 0.9 - 0.2;
        let up = (Vec3::Y * tilt.cos() + dir * tilt.sin()).normalize() * size;
        let layer = if berry && k % 2 == 0 { fl::BERRIES } else { fl::LEAVES };
        let sw = (p.y - y as f32) * 0.5;
        card(m, p, right, up, layer, [tint, tint, tint], sw * 0.6, sw);
    }
    // a flat cap so the bush looks full from above
    card(m, ctr + Vec3::Y * 0.42, Vec3::X * 0.6, Vec3::Z * 0.6, if berry { fl::BERRIES } else { fl::LEAVES }, [tint, tint, tint], 0.3, 0.3);
}
/// a leafy card just outside a canopy face, so tree crowns have ragged leafy edges instead of cube outlines
fn fringe(m: &mut MeshData, x: i32, y: i32, z: i32, d: [i32; 3], bl: u8, seed: u32) {
    let n = Vec3::new(d[0] as f32, d[1] as f32, d[2] as f32);
    let r = |q: u32| hash3(x * 7 + q as i32, y * 3, z, seed + 90 + q);
    let ctr = Vec3::new(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5) + n * (0.5 + 0.06 + r(1) * 0.12);
    let (t0, t1) = if d[0] != 0 { (Vec3::Z, Vec3::Y) } else if d[1] != 0 { (Vec3::X, Vec3::Z) } else { (Vec3::X, Vec3::Y) };
    let a = r(2) * std::f32::consts::TAU;
    let (ta, tb) = (t0 * a.cos() + t1 * a.sin(), -t0 * a.sin() + t1 * a.cos());
    let tilt = (r(3) - 0.5) * 0.9;
    let size = 0.72 + r(4) * 0.2;
    let (layer, col) = match bl { b::AUTUMN => (fl::AUTUMN, [1.0, 1.0, 1.0]), b::PINE => (fl::NEEDLES, [1.0, 1.0, 1.0]), b::SPINE => (fl::NEEDLES, [1.25, 1.3, 1.35]), _ => (fl::LEAVES, [1.0, 1.0, 1.0]) };
    let k = 0.85 + r(5) * 0.25;
    card(m, ctr, ta * size, (tb + n * tilt).normalize() * size, layer, [col[0] * k, col[1] * k, col[2] * k], 0.35, 0.45);
}

fn foliage(w: &Voxels, ch: &Chunk, m: &mut MeshData) {
    let s = w.seed;
    let (x0, z0) = (ch.cx * CS, ch.cz * CS);
    // a crossed pair of textured cards rooted at (x, y, z)
    let cross = |m: &mut MeshData, x: f32, y: f32, z: f32, wd: f32, h: f32, a: f32, layer: u32, c0: [f32; 3], c1: [f32; 3]| {
        for k in 0..2 {
            let an = a + k as f32 * std::f32::consts::FRAC_PI_2;
            let r = Vec3::new(an.cos(), 0.0, an.sin()) * wd * 0.5;
            let base = m.p.len() as u32;
            for (i, (pp, uv)) in [(Vec3::new(x, y, z) - r, [0.0, 1.0]), (Vec3::new(x, y, z) + r, [1.0, 1.0]), (Vec3::new(x, y + h, z) + r, [1.0, 0.0]), (Vec3::new(x, y + h, z) - r, [0.0, 0.0])].into_iter().enumerate() {
                let c = if i >= 2 { c1 } else { c0 };
                m.p.push(pp.to_array()); m.n.push([0.0, 1.0, 0.0]); m.uv.push(uv); m.uv2.push([layer as f32, 0.0]); m.c.push([c[0], c[1], c[2], if i >= 2 { 1.0 } else { 0.0 }]);
            }
            m.i.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    };
    let flat = |m: &mut MeshData, x: f32, y: f32, z: f32, r: f32, a: f32, layer: u32, c: [f32; 3]| {
        let (u, v) = (Vec3::new(a.cos(), 0.0, a.sin()) * r, Vec3::new(-a.sin(), 0.0, a.cos()) * r);
        let base = m.p.len() as u32;
        let ctr = Vec3::new(x, y, z);
        for (pp, uv) in [(ctr - u - v, [0.0, 1.0]), (ctr + u - v, [1.0, 1.0]), (ctr + u + v, [1.0, 0.0]), (ctr - u + v, [0.0, 0.0])] { m.p.push(pp.to_array()); m.n.push([0.0, 1.0, 0.0]); m.uv.push(uv); m.uv2.push([layer as f32, 0.0]); m.c.push([c[0], c[1], c[2], 0.0]); }
        m.i.extend_from_slice(&[base, base + 2, base + 1, base, base + 3, base + 2]);
    };
    let sc = |c: [f32; 3], k: f32| [c[0] * k, c[1] * k, c[2] * k];
    for lz in 0..CS { for lx in 0..CS {
        let i = (lx + lz * CS) as usize;
        let (tb, ty) = (ch.top_b[i], ch.top[i] as i32);
        let (wx, wz) = (x0 + lx, z0 + lz);
        let r = hash2(wx, wz, s + 700);
        let ang = hash2(wx, wz, s + 711) * std::f32::consts::PI;
        if ty < SEA && ty >= SEA - 2 && r < 0.09 && w.n_v.get(wx as f32 * 0.04, wz as f32 * 0.04) > -0.1 {
            let (x, z, y) = (wx as f32 + 0.5, wz as f32 + 0.5, WATER_Y + 0.02);
            flat(m, x, y, z, 0.42, ang * 2.0, fl::LILY, [1.0, 1.0, 1.0]);
            if hash2(wx, wz, s + 708) < 0.3 { cross(m, x + 0.08, y, z, 0.32, 0.22, ang, fl::PETALS, lin(0xf6c6dc), lin(0xffe0ee)); }
            continue;
        }
        if ty <= SEA || ty >= CH - 2 { continue; }
        // nothing grows under a bush or a block
        if crate::world::is_foliage_block(tb) || is_bush(tb) { continue; }
        let is_g = tb == b::GRASS || tb == b::FGRASS;
        let shore = ty <= SEA + 1 && (tb == b::SAND || is_g);
        let meadow = tb == b::GRASS && w.n_v.get(wx as f32 * 0.03 + 5.0, wz as f32 * 0.03 - 9.0) > 0.22;
        let dens = if shore { 0.45 } else if tb == b::GRASS { if meadow { 0.85 } else { 0.7 } } else if tb == b::FGRASS { 0.6 }
            else if tb == b::SAND && ch.bio[i] == Bio::Desert { 0.04 } else if tb == b::SNOW { 0.06 } else { 0.0 };
        if r > dens { continue; }
        let (x, z, y) = (wx as f32 + hash2(wx, wz, s + 701) * 0.5 + 0.25, wz as f32 + hash2(wx, wz, s + 702) * 0.5 + 0.25, (ty + 1) as f32);
        let q = hash2(wx, wz, s + 703);
        if shore {
            let hh = 1.0 + hash2(wx, wz, s + 709) * 0.8;
            cross(m, x, y, z, 0.7, hh, ang, fl::REEDS, lin(0x4a5a28), lin(0xa8b868));
            if q < 0.4 { cross(m, x + 0.03, y + hh - 0.3, z, 0.12, 0.32, ang, fl::SOLID, lin(0x5a3820), lin(0x6a4428)); }
            continue;
        }
        let (g0, g1) = if tb == b::GRASS { (lin(0x5e9a36), lin(0xb4e472)) } else if tb == b::FGRASS { (lin(0x4a7a2c), lin(0x94c858)) } else if tb == b::SAND { (lin(0x7a5a33), lin(0xb89a62)) } else { (lin(0x8fa1a8), lin(0xe8eef2)) };
        let tint = 0.9 + w.n_v.get(wx as f32 * 0.05, wz as f32 * 0.05) * 0.12;
        if is_g && q < if meadow { 0.3 } else { 0.1 } {
            let pc = [0xe64545, 0xf2d14a, 0xf4f1ea, 0x6a8cff, 0xc070f0, 0xff9a3d][(hash2(wx, wz, s + 704) * 6.0) as usize % 6];
            let hh = 0.42 + q * 0.4;
            cross(m, x, y, z, 0.42, hh, ang, fl::GRASS, g0, sc(g1, 0.9));
            cross(m, x, y + hh - 0.12, z, 0.34, 0.3, ang + 0.4, fl::PETALS, sc(lin(pc), 0.85), lin(pc));
        } else if tb == b::FGRASS && q < 0.24 {
            if q < 0.06 {
                cross(m, x, y, z, 0.12, 0.22, ang, fl::SOLID, lin(0xe8e0d0), lin(0xf4ece0));
                let cap = if hash2(wx, wz, s + 710) < 0.6 { 0xd8343a } else { 0xc89a5a };
                flat(m, x, y + 0.22, z, 0.17, ang, fl::SOLID, lin(cap));
                cross(m, x, y + 0.14, z, 0.34, 0.1, ang, fl::SOLID, sc(lin(cap), 0.7), lin(cap));
            } else {
                // ferns: fronds arching out of the forest floor
                for k in 0..4 {
                    let an = ang + k as f32 * std::f32::consts::FRAC_PI_2 + q * 3.0;
                    let out = Vec3::new(an.cos(), 0.0, an.sin());
                    let c = Vec3::new(x, y, z) + out * 0.32 + Vec3::Y * 0.32;
                    card(m, c, Vec3::new(-an.sin(), 0.0, an.cos()) * 0.22, (Vec3::Y * 0.8 + out * 0.6).normalize() * 0.4, fl::FERN, [tint, tint, tint], 0.0, 0.8);
                }
            }
        } else {
            // grass tufts: one or two per block, taller in meadows
            let hh = (0.34 + hash2(wx, wz, s + 705) * 0.28) * if meadow { 1.25 } else { 1.0 } * if is_g { 1.0 } else { 0.6 };
            cross(m, x, y, z, 0.8, hh, ang, fl::GRASS, sc(g0, tint), sc(g1, tint));
            if is_g && q > 0.5 { let (x2, z2) = (wx as f32 + 1.0 - (x - wx as f32), wz as f32 + 1.0 - (z - wz as f32)); cross(m, x2, y, z2, 0.7, hh * 0.8, ang + 0.7, fl::GRASS, sc(g0, tint), sc(g1, tint * 0.95)); }
        }
    } }
}
