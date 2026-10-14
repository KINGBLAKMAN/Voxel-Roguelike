//! Effects engine. Particles are two big camera-facing quad meshes rebuilt each frame (additive glow and
//! soft smoke), debris is a pool of tumbling physical cubes, and everything else is a "tween": a mesh
//! entity whose position, scale, spin, colour and opacity animate over its life. Textures (glow, smoke,
//! star flares, rune circles, cracks, scorch marks) are drawn procedurally at startup.
use crate::models::{srgb, Mk, Palette};
use crate::world::Voxels;
use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::render::view::NoFrustumCulling;
use std::collections::HashMap;
use std::f32::consts::{PI, TAU};

pub fn hexc(hex: u32) -> Vec3 { let c = srgb(hex).to_linear(); Vec3::new(c.red, c.green, c.blue) }

#[derive(Clone, Copy)]
pub struct Particle { pub pos: Vec3, pub vel: Vec3, pub life: f32, pub max: f32, pub s0: f32, pub s1: f32, pub col: Vec3, pub grav: f32, pub drag: f32, pub a0: f32 }

#[derive(Clone, Copy, PartialEq)]
pub enum Ease { Lin, Out, Pop, InOut }
fn ease(e: Ease, k: f32) -> f32 {
    match e { Ease::Lin => k, Ease::Out => 1.0 - (1.0 - k).powi(3), Ease::Pop => { let s = 1.70158 * 1.2; let x = k - 1.0; 1.0 + x * x * ((s + 1.0) * x + s) }, Ease::InOut => k * k * (3.0 - 2.0 * k) }
}

/// an animated mesh effect. Opacity follows fade-in, hold, fade-out; emissive scales with opacity so it glows into bloom.
pub struct Tween {
    pub e: Entity, pub mat: Option<Handle<StandardMaterial>>, pub t: f32, pub life: f32, pub delay: f32,
    pub pos: Vec3, pub vel: Vec3, pub grav: f32, pub s0: Vec3, pub s1: Vec3, pub ease: Ease, pub fade_in: f32, pub fade_out: f32,
    pub col: LinearRgba, pub alpha: f32, pub rot: Quat, pub spin: Vec3, pub follow: Option<Entity>, pub flicker: f32, pub ground: bool,
}
impl Tween {
    pub fn new(e: Entity, life: f32, pos: Vec3) -> Self {
        Tween { e, mat: None, t: 0.0, life, delay: 0.0, pos, vel: Vec3::ZERO, grav: 0.0, s0: Vec3::ONE, s1: Vec3::ONE, ease: Ease::Out, fade_in: 0.08, fade_out: 0.35,
            col: LinearRgba::WHITE, alpha: 1.0, rot: Quat::IDENTITY, spin: Vec3::ZERO, follow: None, flicker: 0.0, ground: false }
    }
}
pub struct Debris { pub e: Entity, pub pos: Vec3, pub vel: Vec3, pub rot: Vec3, pub rv: Vec3, pub life: f32, pub max: f32, pub s: f32 }

#[derive(Clone)]
pub struct Tex { pub glow: Handle<Image>, pub smoke: Handle<Image>, pub star: Handle<Image>, pub ring: Handle<Image>, pub crack: Handle<Image>, pub scorch: Handle<Image>, pub circles: HashMap<&'static str, Handle<Image>> }

#[derive(Resource)]
pub struct Fx {
    pub add: Vec<Particle>, pub blend: Vec<Particle>, pub add_mesh: Handle<Mesh>, pub blend_mesh: Handle<Mesh>,
    pub tweens: Vec<Tween>, pub debris: Vec<Debris>, pub debris_free: Vec<Entity>, pub tex: Tex, pub quality: f32,
    pub quad: Handle<Mesh>, pub ring: Handle<Mesh>, pub disk: Handle<Mesh>, pub cyl: Handle<Mesh>, pub cone: Handle<Mesh>, pub arcs: HashMap<(i32, i32), Handle<Mesh>>,
    pub rng: u32,
}
impl Fx {
    pub fn r(&mut self) -> f32 { self.rng ^= self.rng << 13; self.rng ^= self.rng >> 17; self.rng ^= self.rng << 5; (self.rng % 1_000_000) as f32 / 1_000_000.0 }
    pub fn rr(&mut self, a: f32, b: f32) -> f32 { a + self.r() * (b - a) }
    pub fn emit(&mut self, add: bool, pos: Vec3, vel: Vec3, life: f32, s0: f32, s1: f32, col: Vec3, grav: f32, drag: f32, a0: f32) {
        if self.quality < 1.0 && self.r() > self.quality { return; }
        let v = if add { &mut self.add } else { &mut self.blend };
        if v.len() >= if add { 9000 } else { 3000 } { v.swap_remove(0); }
        v.push(Particle { pos, vel, life, max: life, s0, s1, col, grav, drag, a0 });
    }
    /// glowing spray in all directions
    pub fn burst(&mut self, pos: Vec3, hex: u32, n: usize, speed: f32, life: f32, size: f32, up: f32, grav: f32) {
        let c = hexc(hex) * 2.2;
        for _ in 0..n { let u = self.rr(-1.0, 1.0); let a = self.rr(0.0, TAU); let r = (1.0 - u * u).sqrt(); let sp = speed * self.rr(0.3, 1.0);
            let l = life * self.rr(0.6, 1.2);
            self.emit(true, pos, Vec3::new(a.cos() * r * sp, up + u * sp, a.sin() * r * sp), l, size, size * 0.15, c, grav, 2.5, 1.0); }
    }
    pub fn sparks(&mut self, pos: Vec3, hex: u32, n: usize, speed: f32) {
        let c = hexc(hex) * 3.0;
        for _ in 0..n { let u = self.rr(-0.3, 1.0); let a = self.rr(0.0, TAU); let r = (1.0 - u * u).max(0.0).sqrt(); let sp = speed * self.rr(0.5, 1.0);
            let l = self.rr(0.18, 0.32);
            self.emit(true, pos, Vec3::new(a.cos() * r * sp, u * sp, a.sin() * r * sp), l, 0.16, 0.02, c, 14.0, 3.0, 1.0); }
    }
    pub fn smoke(&mut self, pos: Vec3, hex: u32, n: usize, speed: f32, life: f32, size: f32) {
        let c = hexc(hex);
        for _ in 0..n { let p = pos + Vec3::new(self.rr(-0.4, 0.4), self.rr(0.0, 0.5), self.rr(-0.4, 0.4)); let v = Vec3::new(self.rr(-1.0, 1.0), self.rr(0.4, 1.4), self.rr(-1.0, 1.0)) * speed;
            let l = life * self.rr(0.7, 1.2);
            self.emit(false, p, v, l, size * 0.5, size * 1.7, c, -0.6, 1.5, 0.6); }
    }
}

// ---------- procedural textures ----------
fn img(images: &mut Assets<Image>, s: usize, f: impl Fn(f32, f32) -> f32) -> Handle<Image> {
    let mut d = vec![0u8; s * s * 4];
    for y in 0..s { for x in 0..s {
        let a = f((x as f32 + 0.5) / s as f32 * 2.0 - 1.0, (y as f32 + 0.5) / s as f32 * 2.0 - 1.0).clamp(0.0, 1.0);
        let i = (y * s + x) * 4; d[i] = 255; d[i + 1] = 255; d[i + 2] = 255; d[i + 3] = (a * 255.0) as u8;
    } }
    images.add(Image::new(Extent3d { width: s as u32, height: s as u32, depth_or_array_layers: 1 }, TextureDimension::D2, d, TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::RENDER_WORLD))
}
/// tiny rasteriser for rune circles and cracks: draw shapes as distance fields into an alpha buffer
struct Canvas { s: usize, a: Vec<f32> }
impl Canvas {
    fn new(s: usize) -> Self { Canvas { s, a: vec![0.0; s * s] } }
    fn plot(&mut self, f: impl Fn(Vec2) -> f32, bb: (Vec2, Vec2)) {
        let s = self.s as f32;
        let (x0, y0) = (((bb.0.x * 0.5 + 0.5) * s).floor().max(0.0) as usize, ((bb.0.y * 0.5 + 0.5) * s).floor().max(0.0) as usize);
        let (x1, y1) = (((bb.1.x * 0.5 + 0.5) * s).ceil().min(s - 1.0) as usize, ((bb.1.y * 0.5 + 0.5) * s).ceil().min(s - 1.0) as usize);
        for y in y0..=y1 { for x in x0..=x1 { let p = Vec2::new((x as f32 + 0.5) / s * 2.0 - 1.0, (y as f32 + 0.5) / s * 2.0 - 1.0); let v = f(p); let i = y * self.s + x; self.a[i] = self.a[i].max(v); } }
    }
    fn ring(&mut self, r: f32, w: f32) { let px = 2.0 / self.s as f32; self.plot(|p| 1.0 - ((p.length() - r).abs() - w * 0.5).max(0.0) / px, (Vec2::splat(-r - w), Vec2::splat(r + w))); }
    fn line(&mut self, a: Vec2, b: Vec2, w: f32) {
        let px = 2.0 / self.s as f32;
        self.plot(|p| { let pa = p - a; let ba = b - a; let h = (pa.dot(ba) / ba.length_squared()).clamp(0.0, 1.0); 1.0 - ((pa - ba * h).length() - w * 0.5).max(0.0) / px },
            (a.min(b) - Vec2::splat(w), a.max(b) + Vec2::splat(w)));
    }
    fn dot(&mut self, c: Vec2, r: f32) { let px = 2.0 / self.s as f32; self.plot(|p| 1.0 - ((p - c).length() - r).max(0.0) / px, (c - Vec2::splat(r + 0.02), c + Vec2::splat(r + 0.02))); }
    fn finish(self, images: &mut Assets<Image>, glow: f32) -> Handle<Image> {
        let s = self.s;
        // a soft halo around every stroke so the circles bloom nicely
        let mut out = self.a.clone();
        if glow > 0.0 { let r = 3i32; for y in 0..s as i32 { for x in 0..s as i32 { let mut m = 0.0f32;
            for dy in -r..=r { for dx in -r..=r { let (xx, yy) = (x + dx, y + dy); if xx < 0 || yy < 0 || xx >= s as i32 || yy >= s as i32 { continue; }
                let d = ((dx * dx + dy * dy) as f32).sqrt(); m = m.max(self.a[(yy as usize) * s + xx as usize] * (1.0 - d / (r as f32 + 1.0))); } }
            let i = y as usize * s + x as usize; out[i] = out[i].max(m * glow); } } }
        let mut d = vec![0u8; s * s * 4];
        for i in 0..s * s { d[i * 4] = 255; d[i * 4 + 1] = 255; d[i * 4 + 2] = 255; d[i * 4 + 3] = (out[i].clamp(0.0, 1.0) * 255.0) as u8; }
        images.add(Image::new(Extent3d { width: s as u32, height: s as u32, depth_or_array_layers: 1 }, TextureDimension::D2, d, TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::RENDER_WORLD))
    }
}
fn rune_circle(images: &mut Assets<Image>, pts: usize, runes: usize, seed: u32) -> Handle<Image> {
    let mut rng = crate::noise::mulberry32(seed);
    let mut c = Canvas::new(256);
    c.ring(0.94, 0.03); c.ring(0.87, 0.012); c.ring(0.59, 0.018); c.ring(0.53, 0.008);
    for i in 0..72 { let a = i as f32 / 72.0 * TAU; let l = if i % 6 == 0 { 0.07 } else { 0.03 }; let d = Vec2::new(a.cos(), a.sin()); c.line(d * 0.87, d * (0.87 - l), if i % 6 == 0 { 0.016 } else { 0.008 }); }
    for i in 0..runes {
        let a = i as f32 / runes as f32 * TAU; let ctr = Vec2::new(a.cos(), a.sin()) * 0.73;
        let pts3: Vec<Vec2> = (0..3).map(|_| ctr + Vec2::new(rng() - 0.5, rng() - 0.5) * 0.09).collect();
        c.line(pts3[0], pts3[1], 0.012); c.line(pts3[1], pts3[2], 0.012); c.dot(ctr, 0.012);
    }
    let step = if pts >= 5 { 2 } else { 1 };
    let vert = |i: usize| { let a = (i * step) as f32 / pts as f32 * TAU - PI / 2.0; Vec2::new(a.cos(), a.sin()) * 0.59 };
    for i in 0..pts { c.line(vert(i), vert(i + 1), 0.018); }
    if pts == 6 { for i in 0..3 { let a = |k: usize| { let an = (k * 2 + 1) as f32 / 6.0 * TAU - PI / 2.0; Vec2::new(an.cos(), an.sin()) * 0.59 }; c.line(a(i), a(i + 1), 0.018); } }
    for i in 0..pts { let a = i as f32 / pts as f32 * TAU - PI / 2.0; c.dot(Vec2::new(a.cos(), a.sin()) * 0.59, 0.035); }
    c.ring(0.21, 0.016); c.ring(0.08, 0.024);
    c.finish(images, 0.7)
}
fn crack_tex(images: &mut Assets<Image>) -> Handle<Image> {
    let mut rng = crate::noise::mulberry32(91);
    let mut c = Canvas::new(256);
    for i in 0..11 {
        let mut a = i as f32 / 11.0 * TAU + rng() * 0.4; let mut r = 0.08; let mut p = Vec2::new(a.cos(), a.sin()) * r;
        while r < 0.92 { r += 0.08 + rng() * 0.1; a += (rng() - 0.5) * 0.5; let n = Vec2::new(a.cos(), a.sin()) * r; c.line(p, n, (0.05 - r * 0.04).max(0.012));
            if rng() < 0.25 { let b2 = a + (rng() - 0.5) * 1.4; c.line(n, n + Vec2::new(b2.cos(), b2.sin()) * 0.16, 0.012); } p = n; }
    }
    c.dot(Vec2::ZERO, 0.12);
    c.finish(images, 0.5)
}

pub struct FxPlugin;
impl Plugin for FxPlugin {
    fn build(&self, app: &mut App) { app.add_systems(Startup, setup_fx).add_systems(PostUpdate, (update_fx, face_camera, render_particles).chain().before(bevy::transform::TransformSystem::TransformPropagate)); }
}

fn open_cyl(seg: usize) -> Mesh {
    // an open tube whose vertex alpha fades from 1 at the base to 0 at the top: light pillars and shock walls
    let (mut p, mut n, mut c, mut uv, mut i) = (vec![], vec![], vec![], vec![], vec![]);
    for k in 0..=seg { let a = k as f32 / seg as f32 * TAU; for (y, al) in [(0.0, 1.0), (1.0, 0.0)] { p.push([a.cos(), y, a.sin()]); n.push([a.cos(), 0.0, a.sin()]); c.push([1.0, 1.0, 1.0, al]); uv.push([k as f32 / seg as f32, y]); } }
    for k in 0..seg as u32 { let b = k * 2; i.extend_from_slice(&[b, b + 1, b + 2, b + 1, b + 3, b + 2]); }
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD).with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, p).with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, n)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, c).with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv).with_inserted_indices(Indices::U32(i))
}
pub fn annulus(r0: f32, seg: usize, a0: f32, len: f32, grad: bool) -> Mesh {
    // flat ring (or arc) in the XZ plane. With `grad` the alpha rises along the arc: a swoosh with a bright leading edge.
    let (mut p, mut n, mut c, mut uv, mut i) = (vec![], vec![], vec![], vec![], vec![]);
    for k in 0..=seg { let t = k as f32 / seg as f32; let a = a0 + t * len; for (j, r) in [(0.0, r0), (1.0, 1.0)] {
        p.push([a.sin() * r, 0.0, a.cos() * r]); n.push([0.0, 1.0, 0.0]);
        let al = if grad { t * t * if j > 0.5 { 1.0 } else { 0.25 } } else { 1.0 }; c.push([1.0, 1.0, 1.0, al]); uv.push([t, j]); } }
    for k in 0..seg as u32 { let b = k * 2; i.extend_from_slice(&[b, b + 1, b + 2, b + 1, b + 3, b + 2]); }
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD).with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, p).with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, n)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, c).with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv).with_inserted_indices(Indices::U32(i))
}
fn dyn_mesh() -> Mesh {
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD)
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0f32; 3]; 3]).with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0f32, 1.0, 0.0]; 3])
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0f32; 2]; 3]).with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.0f32; 4]; 3]).with_inserted_indices(Indices::U32(vec![0, 1, 2]))
}

fn setup_fx(mut c: Commands, mut meshes: ResMut<Assets<Mesh>>, mut mats: ResMut<Assets<StandardMaterial>>, mut images: ResMut<Assets<Image>>, mut pal: ResMut<Palette>) {
    let glow = img(&mut images, 64, |x, y| { let d = (x * x + y * y).sqrt(); ((1.0 - d).max(0.0)).powf(2.2) });
    let smoke = img(&mut images, 64, |x, y| { let d = (x * x + y * y).sqrt(); let n = ((x * 7.0).sin() * (y * 6.0).cos()) * 0.12; ((1.0 - d + n).max(0.0) * 1.3).min(1.0).powf(1.5) });
    let star = img(&mut images, 128, |x, y| { let d = (x * x + y * y).sqrt(); let core = (1.0 - d * 3.0).max(0.0).powf(2.0); let a = (1.0 - y.abs() * 30.0).max(0.0) * (1.0 - x.abs()).max(0.0); let b = (1.0 - x.abs() * 30.0).max(0.0) * (1.0 - y.abs()).max(0.0); core + a + b + (1.0 - d).max(0.0).powf(6.0) * 0.6 });
    let ring = img(&mut images, 128, |x, y| { let d = (x * x + y * y).sqrt(); (1.0 - ((d - 0.85).abs() * 9.0)).max(0.0).powf(1.5) });
    let scorch = img(&mut images, 128, |x, y| { let d = (x * x + y * y).sqrt(); let n = (x * 9.1).sin() * (y * 7.3).cos() * 0.15 + (x * 21.0 + y * 13.0).sin() * 0.06; ((1.0 - d + n) * 1.4).clamp(0.0, 1.0).powf(1.2) * 0.85 });
    let crack = crack_tex(&mut images);
    let mut circles = HashMap::new();
    for (k, pts, runes, seed) in [("arcane", 6, 12, 1u32), ("holy", 8, 16, 2), ("nature", 5, 10, 3), ("frost", 6, 18, 4), ("fire", 3, 9, 5), ("war", 4, 8, 6), ("blood", 5, 14, 7)] { circles.insert(k, rune_circle(&mut images, pts, runes, seed)); }
    let tex = Tex { glow: glow.clone(), smoke: smoke.clone(), star, ring, crack, scorch, circles };
    let add_mesh = meshes.add(dyn_mesh());
    let blend_mesh = meshes.add(dyn_mesh());
    let add_mat = mats.add(StandardMaterial { base_color_texture: Some(glow), unlit: true, alpha_mode: AlphaMode::Add, fog_enabled: false, ..default() });
    let blend_mat = mats.add(StandardMaterial { base_color_texture: Some(smoke), unlit: true, alpha_mode: AlphaMode::Blend, ..default() });
    c.spawn((Mesh3d(add_mesh.clone()), MeshMaterial3d(add_mat), Transform::default(), NoFrustumCulling, NotShadowCaster));
    c.spawn((Mesh3d(blend_mesh.clone()), MeshMaterial3d(blend_mat), Transform::default(), NoFrustumCulling, NotShadowCaster));
    // debris pool: small physical cubes that tumble and bounce
    let mut free = vec![];
    let dm = pal.mat(&mut mats, 0x8a5b34, Mk::Lit);
    for _ in 0..500 { free.push(c.spawn((Mesh3d(pal.cube.clone()), MeshMaterial3d(dm.clone()), Transform::from_scale(Vec3::ZERO), Visibility::Hidden, NotShadowCaster)).id()); }
    let quad = meshes.add(Plane3d::default().mesh().size(2.0, 2.0));
    let ring_m = meshes.add(annulus(0.88, 64, 0.0, TAU, false));
    let disk = meshes.add(Circle::new(1.0).mesh().resolution(48).build().rotated_by(Quat::from_rotation_x(-PI / 2.0)));
    let cyl = meshes.add(open_cyl(32));
    let cone = meshes.add(Cone { radius: 0.5, height: 1.0 }.mesh().resolution(6).build());
    c.insert_resource(Fx { add: vec![], blend: vec![], add_mesh, blend_mesh, tweens: vec![], debris: vec![], debris_free: free, tex, quality: 1.0, quad, ring: ring_m, disk, cyl, cone, arcs: HashMap::new(), rng: 0x9e3779b9 });
}

fn update_fx(time: Res<Time>, mut fx: ResMut<Fx>, mut q: Query<&mut Transform>, mut mats: ResMut<Assets<StandardMaterial>>, mut c: Commands, w: Option<Res<Voxels>>, paused: Option<Res<crate::Paused>>) {
    let dt = if paused.map(|p| p.0).unwrap_or(false) { 0.0 } else { time.delta_secs().min(0.05) };
    let fx = &mut *fx;
    for list in [&mut fx.add, &mut fx.blend] {
        list.retain_mut(|p| { p.life -= dt; if p.life <= 0.0 { return false; }
            let k = (1.0 - p.drag * dt).max(0.0); p.vel.x *= k; p.vel.y = p.vel.y * k - p.grav * dt; p.vel.z *= k; p.pos += p.vel * dt; true });
    }
    let mut dead = vec![];
    for (i, t) in fx.tweens.iter_mut().enumerate() {
        if t.delay > 0.0 { t.delay -= dt; continue; }
        t.t += dt;
        let k = (t.t / t.life).min(1.0);
        t.vel.y -= t.grav * dt; t.pos += t.vel * dt;
        if t.ground { if let Some(w) = &w { if w.peek(t.pos.x.floor() as i32, (t.pos.y - 0.05).floor() as i32, t.pos.z.floor() as i32) != 0 && t.vel.y < 0.0 { t.vel = Vec3::ZERO; t.grav = 0.0; } } }
        let follow = t.follow.and_then(|f| q.get(f).ok().map(|tr| tr.translation));
        if let Some(fp) = follow { t.pos = fp; }
        let s = t.s0.lerp(t.s1, ease(t.ease, k));
        let a = t.alpha * (k / t.fade_in.max(1e-3)).min(1.0) * if k > 1.0 - t.fade_out { ((1.0 - k) / t.fade_out.max(1e-3)).max(0.0) } else { 1.0 } * if t.flicker > 0.0 { 1.0 - t.flicker * ((t.t * 37.0).sin() * 0.5 + 0.5) } else { 1.0 };
        let sp = t.spin * t.t;
        if let Ok(mut tr) = q.get_mut(t.e) { tr.translation = t.pos; tr.scale = s; tr.rotation = t.rot * Quat::from_euler(EulerRot::YXZ, sp.y, sp.x, sp.z); }
        if let Some(m) = t.mat.as_ref().and_then(|h| mats.get_mut(h)) { m.base_color.set_alpha(a); m.emissive = LinearRgba::rgb(t.col.red * a, t.col.green * a, t.col.blue * a); }
        if t.t >= t.life { dead.push(i); }
    }
    for i in dead.into_iter().rev() { let t = fx.tweens.swap_remove(i); c.entity(t.e).despawn_recursive(); if let Some(m) = t.mat { mats.remove(&m); } }
    let mut done = vec![];
    for (i, d) in fx.debris.iter_mut().enumerate() {
        d.life -= dt;
        if d.life <= 0.0 { done.push(i); continue; }
        d.vel.y -= 24.0 * dt; d.pos += d.vel * dt;
        if let Some(w) = &w { if w.peek(d.pos.x.floor() as i32, (d.pos.y - d.s * 0.5).floor() as i32, d.pos.z.floor() as i32) != 0 && d.vel.y < 0.0 {
            d.pos.y = (d.pos.y - d.s * 0.5).floor() + 1.0 + d.s * 0.5; d.vel.y *= -0.32; d.vel.x *= 0.6; d.vel.z *= 0.6; d.rv *= 0.6; } }
        d.rot += d.rv * dt;
        let sc = d.s * (d.life / d.max * 3.0).min(1.0);
        if let Ok(mut tr) = q.get_mut(d.e) { tr.translation = d.pos; tr.scale = Vec3::splat(sc); tr.rotation = Quat::from_euler(EulerRot::XYZ, d.rot.x, d.rot.y, d.rot.z); }
    }
    for i in done.into_iter().rev() { let d = fx.debris.swap_remove(i); c.entity(d.e).insert(Visibility::Hidden); fx.debris_free.push(d.e); }
}

fn render_particles(fx: Res<Fx>, cam: Query<&Transform, With<crate::scene::MainCam>>, mut meshes: ResMut<Assets<Mesh>>) {
    let Ok(ct) = cam.get_single() else { return };
    let (right, up) = (ct.right().as_vec3(), ct.up().as_vec3());
    for (list, h) in [(&fx.add, &fx.add_mesh), (&fx.blend, &fx.blend_mesh)] {
        let n = list.len().max(1);
        let (mut p, mut c, mut uv, mut nn, mut idx) = (Vec::with_capacity(n * 4), Vec::with_capacity(n * 4), Vec::with_capacity(n * 4), Vec::with_capacity(n * 4), Vec::with_capacity(n * 6));
        for (i, q) in list.iter().enumerate() {
            let t = 1.0 - q.life / q.max;
            let s = q.s0 + (q.s1 - q.s0) * t;
            let a = q.a0 * (t * 12.0).min(1.0) * ((1.0 - t) * 2.2).min(1.0);
            for (dx, dy, u, v) in [(-1.0, -1.0, 0.0, 1.0), (1.0, -1.0, 1.0, 1.0), (1.0, 1.0, 1.0, 0.0), (-1.0, 1.0, 0.0, 0.0)] {
                p.push((q.pos + right * dx * s + up * dy * s).to_array()); c.push([q.col.x, q.col.y, q.col.z, a]); uv.push([u, v]); nn.push([0.0, 1.0, 0.0]);
            }
            let b = (i * 4) as u32; idx.extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);
        }
        if list.is_empty() { p = vec![[0.0; 3]; 3]; c = vec![[0.0; 4]; 3]; uv = vec![[0.0; 2]; 3]; nn = vec![[0.0, 1.0, 0.0]; 3]; idx = vec![0, 1, 2]; }
        if let Some(m) = meshes.get_mut(h) {
            m.insert_attribute(Mesh::ATTRIBUTE_POSITION, p); m.insert_attribute(Mesh::ATTRIBUTE_COLOR, c); m.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv); m.insert_attribute(Mesh::ATTRIBUTE_NORMAL, nn);
            m.insert_indices(Indices::U32(idx));
        }
    }
}

// ---------- spawning effects (needs Commands and assets) ----------
pub struct Gfx<'a, 'w, 's> { pub c: &'a mut Commands<'w, 's>, pub meshes: &'a mut Assets<Mesh>, pub mats: &'a mut Assets<StandardMaterial>, pub pal: &'a mut Palette, pub fx: &'a mut Fx }

pub enum Blend { Add, Alpha }
impl<'a, 'w, 's> Gfx<'a, 'w, 's> {
    /// a unique fading material for one effect
    pub fn fmat(&mut self, hex: u32, glow: f32, tex: Option<Handle<Image>>, blend: Blend, unlit: bool) -> (Handle<StandardMaterial>, LinearRgba) {
        let c = srgb(hex);
        let em = crate::models::lrgb(hex, glow);
        let h = self.mats.add(StandardMaterial { base_color: c, base_color_texture: tex, emissive: em, unlit, alpha_mode: match blend { Blend::Add => AlphaMode::Add, Blend::Alpha => AlphaMode::Blend },
            double_sided: true, cull_mode: None, fog_enabled: !matches!(blend, Blend::Add), ..default() });
        (h, em)
    }
    pub fn spawn_tween(&mut self, mesh: Handle<Mesh>, mat: (Handle<StandardMaterial>, LinearRgba), life: f32, pos: Vec3, f: impl FnOnce(&mut Tween)) -> Entity {
        let e = self.c.spawn((Mesh3d(mesh), MeshMaterial3d(mat.0.clone()), Transform::from_translation(pos).with_scale(Vec3::ZERO), NotShadowCaster)).id();
        let mut t = Tween::new(e, life, pos); t.mat = Some(mat.0); t.col = mat.1; f(&mut t);
        self.fx.tweens.push(t);
        e
    }
    /// expanding flat ring on the ground
    pub fn ring(&mut self, pos: Vec3, hex: u32, r0: f32, r1: f32, life: f32) {
        let m = self.fmat(hex, 3.0, None, Blend::Add, true); let mesh = self.fx.ring.clone();
        self.spawn_tween(mesh, m, life, pos + Vec3::Y * 0.12, |t| { t.s0 = Vec3::splat(r0); t.s1 = Vec3::splat(r1); t.fade_in = 0.02; t.fade_out = 0.8; });
    }
    /// a rising translucent wall that races outward: the visible shove of a shockwave
    pub fn shock(&mut self, pos: Vec3, hex: u32, r0: f32, r1: f32, life: f32, h: f32) {
        let m = self.fmat(hex, 3.0, None, Blend::Add, true); let mesh = self.fx.cyl.clone();
        self.spawn_tween(mesh, m, life, pos, |t| { t.s0 = Vec3::new(r0, h, r0); t.s1 = Vec3::new(r1, h * 0.4, r1); t.fade_in = 0.02; t.fade_out = 0.9; });
    }
    /// column of light
    pub fn pillar(&mut self, pos: Vec3, hex: u32, r: f32, h: f32, life: f32) {
        let m = self.fmat(hex, 4.0, None, Blend::Add, true); let mesh = self.fx.cyl.clone();
        self.spawn_tween(mesh, m, life, pos, |t| { t.s0 = Vec3::new(r * 0.6, h, r * 0.6); t.s1 = Vec3::new(r * 1.3, h, r * 1.3); t.fade_in = 0.15; t.fade_out = 0.7; t.spin = Vec3::new(0.0, 2.0, 0.0); });
    }
    pub fn disk(&mut self, pos: Vec3, hex: u32, r: f32, life: f32, op: f32) {
        let m = self.fmat(hex, 1.5, None, Blend::Add, true); let mesh = self.fx.disk.clone();
        self.spawn_tween(mesh, m, life, pos + Vec3::Y * 0.08, |t| { t.s0 = Vec3::splat(r); t.s1 = Vec3::splat(r); t.alpha = op; t.fade_in = 0.05; t.fade_out = 0.3; });
    }
    /// camera-facing burst of light; star-shaped for crits and impacts
    pub fn flare(&mut self, pos: Vec3, hex: u32, size: f32, life: f32, star: bool) {
        let tex = if star { self.fx.tex.star.clone() } else { self.fx.tex.glow.clone() };
        let m = self.fmat(hex, 4.0, Some(tex), Blend::Add, true); let mesh = self.fx.quad.clone();
        let e = self.spawn_tween(mesh, m, life, pos, |t| { t.s0 = Vec3::splat(size * 0.4); t.s1 = Vec3::splat(size); t.fade_in = 0.05; t.fade_out = 0.8; });
        self.c.entity(e).insert(Billboard);
    }
    /// a rotating rune circle on the ground, or standing up facing `dir`
    pub fn circle(&mut self, pos: Vec3, hex: u32, r: f32, life: f32, style: &str, spin: f32, dir: Option<Vec3>, follow: Option<Entity>) {
        let tex = self.fx.tex.circles.get(style).cloned().unwrap_or(self.fx.tex.glow.clone());
        let m = self.fmat(hex, 5.0, Some(tex), Blend::Add, true); let mesh = self.fx.quad.clone();
        let rot = match dir { Some(d) => Quat::from_rotation_arc(Vec3::Y, d.normalize()), None => Quat::IDENTITY };
        self.spawn_tween(mesh, m, life, pos + if dir.is_none() { Vec3::Y * 0.07 } else { Vec3::ZERO }, |t| { t.s0 = Vec3::splat(r * 0.15); t.s1 = Vec3::splat(r); t.ease = Ease::Pop; t.rot = rot; t.spin = Vec3::new(0.0, spin, 0.0); t.fade_in = 0.06; t.fade_out = 0.2; t.alpha = 0.85; t.follow = follow; });
    }
    /// ground decals: dark scorch marks and cracks that linger, optionally with a glowing core
    pub fn decal(&mut self, pos: Vec3, crack: bool, r: f32, life: f32, glow_hex: Option<u32>) {
        let tex = if crack { self.fx.tex.crack.clone() } else { self.fx.tex.scorch.clone() };
        let m = self.fmat(0x0a0806, 0.0, Some(tex.clone()), Blend::Alpha, true); let mesh = self.fx.quad.clone();
        let rot = Quat::from_rotation_y(self.fx.rr(0.0, TAU));
        self.spawn_tween(mesh.clone(), m, life, pos + Vec3::Y * 0.03, |t| { t.s0 = Vec3::splat(r); t.s1 = Vec3::splat(r); t.rot = rot; t.alpha = 0.8; t.fade_in = 0.01; t.fade_out = 0.3; });
        if let Some(g) = glow_hex { let m2 = self.fmat(g, 6.0, Some(tex), Blend::Add, true); self.spawn_tween(mesh, m2, life * 0.35, pos + Vec3::Y * 0.05, |t| { t.s0 = Vec3::splat(r); t.s1 = Vec3::splat(r); t.rot = rot; t.fade_in = 0.01; t.fade_out = 0.9; }); }
    }
    /// an expanding glowing shell
    pub fn sphere(&mut self, pos: Vec3, hex: u32, r0: f32, r1: f32, life: f32) {
        let m = self.fmat(hex, 2.0, Some(self.fx.tex.ring.clone()), Blend::Add, true); let mesh = self.pal.sphere.clone();
        self.spawn_tween(mesh, m, life, pos, |t| { t.s0 = Vec3::splat(r0); t.s1 = Vec3::splat(r1); t.alpha = 0.45; t.fade_in = 0.02; t.fade_out = 0.9; });
    }
    /// a sweeping weapon trail: an arc whose leading edge is brightest
    pub fn swoosh(&mut self, pos: Vec3, yaw: f32, r: f32, len: f32, hex: u32, life: f32, tilt: f32, dir: f32) {
        let key = ((r * 10.0) as i32, (len * 100.0) as i32 * dir.signum() as i32);
        let mesh = if let Some(m) = self.fx.arcs.get(&key) { m.clone() } else { let m = self.meshes.add(annulus(0.3, 28, -len / 2.0 * dir, len * dir, true)); self.fx.arcs.insert(key, m.clone()); m };
        let m = self.fmat(hex, 5.0, None, Blend::Add, true);
        let rot = Quat::from_rotation_y(yaw) * Quat::from_rotation_z(tilt);
        self.spawn_tween(mesh, m, life, pos, |t| { t.s0 = Vec3::splat(r * 0.85); t.s1 = Vec3::splat(r * 1.08); t.rot = rot; t.spin = Vec3::new(0.0, dir * 1.6, 0.0); t.fade_in = 0.02; t.fade_out = 0.75; });
    }
    /// a jagged lightning bolt from a to b with a white-hot core and branches
    pub fn lightning(&mut self, a: Vec3, b: Vec3, hex: u32, life: f32, w: f32) {
        let root = self.c.spawn((Transform::default(), Visibility::default())).id();
        let m = self.fmat(hex, 12.0, None, Blend::Add, true);
        let core = self.pal.mat(self.mats, 0xffffff, Mk::Glow);
        let n = ((a.distance(b) / 2.2).ceil() as usize).clamp(4, 12);
        let mut prev = a;
        for i in 1..=n {
            let t = i as f32 / n as f32;
            let mut q = a.lerp(b, t);
            if i < n { let j = (a.distance(b) * 0.08).min(1.1); q += Vec3::new(self.fx.rr(-j, j), self.fx.rr(-j, j) * 0.5, self.fx.rr(-j, j)); }
            for (mm, s) in [(m.0.clone(), w), (core.clone(), w * 0.3)] {
                let len = prev.distance(q) + 0.05;
                let e = self.c.spawn((Mesh3d(self.pal.cube.clone()), MeshMaterial3d(mm), Transform::from_translation(prev.lerp(q, 0.5)).looking_at(q, Vec3::Y).with_scale(Vec3::new(s, s, len)), NotShadowCaster)).id();
                self.c.entity(root).add_child(e);
            }
            if i < n && self.fx.r() < 0.35 {
                let br = q + Vec3::new(self.fx.rr(-2.0, 2.0), self.fx.rr(-1.5, 0.5), self.fx.rr(-2.0, 2.0));
                let e = self.c.spawn((Mesh3d(self.pal.cube.clone()), MeshMaterial3d(m.0.clone()), Transform::from_translation(q.lerp(br, 0.5)).looking_at(br, Vec3::Y).with_scale(Vec3::new(w * 0.45, w * 0.45, q.distance(br))), NotShadowCaster)).id();
                self.c.entity(root).add_child(e);
            }
            prev = q;
        }
        let mut t = Tween::new(root, life, Vec3::ZERO); t.mat = Some(m.0); t.col = m.1; t.fade_in = 0.01; t.fade_out = 0.8; t.flicker = 0.6;
        self.fx.tweens.push(t);
    }
    /// bright streak between two points (speed lines, beams)
    pub fn streak(&mut self, a: Vec3, b: Vec3, hex: u32, life: f32, w: f32) {
        let m = self.fmat(hex, 6.0, None, Blend::Add, true);
        let len = a.distance(b).max(0.01);
        let mesh = self.pal.cube.clone();
        let rot = Transform::from_translation(a).looking_at(b, if (b - a).normalize().y.abs() > 0.99 { Vec3::X } else { Vec3::Y }).rotation;
        self.spawn_tween(mesh, m, life, a.lerp(b, 0.5), |t| { t.s0 = Vec3::new(w, w, len); t.s1 = Vec3::new(w * 0.2, w * 0.2, len); t.rot = rot; t.fade_in = 0.02; t.fade_out = 0.8; t.ease = Ease::Lin; });
    }
    /// crystals or rock shards punching up out of the ground in a ring, then sinking back
    pub fn spikes(&mut self, pos: Vec3, hex: u32, n: usize, r: f32, life: f32, h: f32, ice: bool) {
        let mat = if ice { self.pal.mat(self.mats, hex, Mk::Ice) } else { self.pal.mat(self.mats, hex, Mk::Lit) };
        for i in 0..n {
            let a = i as f32 / n as f32 * TAU + self.fx.rr(-0.15, 0.15);
            let rr = r * self.fx.rr(0.75, 1.05);
            let hh = h * self.fx.rr(0.6, 1.25);
            let p = pos + Vec3::new(a.cos() * rr, 0.0, a.sin() * rr);
            let tilt = Quat::from_rotation_arc(Vec3::Y, (Vec3::new(a.cos() * 0.35, 1.0, a.sin() * 0.35) + Vec3::new(self.fx.rr(-0.2, 0.2), 0.0, self.fx.rr(-0.2, 0.2))).normalize());
            let mesh = if ice { self.pal.octa.clone() } else { self.fx.cone.clone() };
            let w = self.fx.rr(0.3, 0.5) * h.min(2.0);
            let e = self.c.spawn((Mesh3d(mesh), MeshMaterial3d(mat.clone()), Transform::from_translation(p).with_scale(Vec3::ZERO), NotShadowCaster)).id();
            let mut t = Tween::new(e, life, p - Vec3::Y * 0.15); t.rot = tilt; t.s0 = Vec3::new(w * 0.2, 0.05, w * 0.2); t.s1 = Vec3::new(w, hh, w); t.ease = Ease::Pop; t.delay = i as f32 * 0.008;
            self.fx.tweens.push(t);
        }
    }
    /// tumbling physical cubes
    pub fn debris(&mut self, pos: Vec3, colors: &[u32], n: usize, speed: f32, size: f32, up: f32) {
        for _ in 0..n {
            let Some(e) = self.fx.debris_free.pop() else { return };
            let col = colors[(self.fx.r() * colors.len() as f32) as usize % colors.len()];
            let m = self.pal.mat(self.mats, col, Mk::Lit);
            self.c.entity(e).insert((MeshMaterial3d(m), Visibility::Inherited));
            let a = self.fx.rr(0.0, TAU); let sp = self.fx.rr(0.3, 1.0) * speed;
            let d = Debris { e, pos: pos + Vec3::new(self.fx.rr(-0.3, 0.3), self.fx.rr(0.0, 0.6), self.fx.rr(-0.3, 0.3)), vel: Vec3::new(a.cos() * sp, self.fx.rr(0.4, 1.0) * up, a.sin() * sp),
                rot: Vec3::new(self.fx.rr(0.0, 6.0), self.fx.rr(0.0, 6.0), self.fx.rr(0.0, 6.0)), rv: Vec3::new(self.fx.rr(-9.0, 9.0), self.fx.rr(-9.0, 9.0), self.fx.rr(-9.0, 9.0)),
                life: self.fx.rr(0.8, 1.6), max: 1.6, s: size * self.fx.rr(0.6, 1.3) };
            self.fx.debris.push(d);
        }
    }
    pub fn debris_one(&mut self, pos: Vec3, vel: Vec3, col: u32, size: f32, life: f32) {
        let Some(e) = self.fx.debris_free.pop() else { return };
        let m = self.pal.mat(self.mats, col, Mk::Lit);
        self.c.entity(e).insert((MeshMaterial3d(m), Visibility::Inherited));
        let rv = Vec3::new(self.fx.rr(-9.0, 9.0), self.fx.rr(-9.0, 9.0), self.fx.rr(-9.0, 9.0));
        self.fx.debris.push(Debris { e, pos, vel, rot: Vec3::ZERO, rv, life, max: life, s: size });
    }
}

#[derive(Component)]
pub struct Billboard;
pub fn face_camera(cam: Query<&Transform, (With<crate::scene::MainCam>, Without<Billboard>)>, mut q: Query<&mut Transform, With<Billboard>>) {
    let Ok(ct) = cam.get_single() else { return };
    // quads are XZ planes: tip their normal (+Y) toward the camera
    let r = Quat::from_rotation_arc(Vec3::Y, -ct.forward().as_vec3());
    for mut t in q.iter_mut() { t.rotation = r; }
}
