//! Axis-separated voxel collision with auto step-up, gravity and swimming.
use crate::world::*;
use bevy::prelude::*;

#[derive(Clone, Debug)]
pub struct Body { pub pos: Vec3, pub vel: Vec3, pub w: f32, pub h: f32, pub on_ground: bool, pub in_water: bool, pub hit_wall: bool, pub fly: bool, pub depth: f32 }
impl Body { pub fn new(pos: Vec3, w: f32, h: f32) -> Self { Body { pos, vel: Vec3::ZERO, w, h, on_ground: false, in_water: false, hit_wall: false, fly: false, depth: 0.0 } } }

pub fn collides(w: &mut Voxels, x: f32, y: f32, z: f32, bw: f32, bh: f32) -> bool {
    let (x0, x1) = ((x - bw).floor() as i32, (x + bw).floor() as i32);
    let (y0, y1) = (y.floor() as i32, (y + bh - 0.01).floor() as i32);
    let (z0, z1) = ((z - bw).floor() as i32, (z + bw).floor() as i32);
    for yy in y0..=y1 { for zz in z0..=z1 { for xx in x0..=x1 { if w.get(xx, yy, zz) != 0 { return true; } } } }
    false
}

pub fn move_body(w: &mut Voxels, e: &mut Body, dt: f32, grav: f32) {
    // how deep the feet are below the surface; past about half the body's height it swims
    e.depth = (WATER_Y - e.pos.y).max(0.0);
    e.in_water = e.depth > e.h * 0.55;
    let lava = w.kind.lava();
    if !e.fly {
        if e.in_water {
            // buoyancy floats the body with its shoulders at the surface; the liquid damps every motion
            let want = e.h * 0.72;
            e.vel.y += ((e.depth - want).clamp(-1.5, 1.5) * 16.0 - grav * 0.08) * dt;
            e.vel.y *= 1.0 - (dt * 2.6).min(0.9);
            let drag = if lava { 4.0 } else { 1.5 };
            e.vel.x *= 1.0 - (dt * drag).min(0.9); e.vel.z *= 1.0 - (dt * drag).min(0.9);
        } else {
            e.vel.y -= grav * dt;
            if e.depth > 0.25 { let d = if lava { 3.0 } else { 0.8 }; e.vel.x *= 1.0 - (dt * d).min(0.9); e.vel.z *= 1.0 - (dt * d).min(0.9); }
        }
    }
    e.vel.y = e.vel.y.max(-40.0);
    let was_ground = e.on_ground;
    e.on_ground = false;
    e.hit_wall = false;
    let d = e.vel * dt;
    let n = ((d.x.abs().max(d.y.abs()).max(d.z.abs())) / 0.35).ceil().max(1.0) as i32;
    let nf = n as f32;
    for _ in 0..n {
        e.pos.y += d.y / nf;
        if collides(w, e.pos.x, e.pos.y, e.pos.z, e.w, e.h) {
            if d.y < 0.0 { e.pos.y = e.pos.y.floor() + 1.0; e.on_ground = true; } else { e.pos.y = (e.pos.y + e.h).floor() - e.h - 0.001; }
            e.vel.y = 0.0;
        }
        for axis in [0, 2] {
            let step = if axis == 0 { d.x / nf } else { d.z / nf };
            if axis == 0 { e.pos.x += step } else { e.pos.z += step }
            if collides(w, e.pos.x, e.pos.y, e.pos.z, e.w, e.h) {
                if (was_ground || e.on_ground) && !collides(w, e.pos.x, e.pos.y + 1.02, e.pos.z, e.w, e.h) { e.pos.y = e.pos.y.floor() + 1.0; }
                else { if axis == 0 { e.pos.x -= step; e.vel.x = 0.0 } else { e.pos.z -= step; e.vel.z = 0.0 } e.hit_wall = true; }
            }
        }
    }
    // the arena's edge holds everything in, even what flies or gets knocked over the cliffs
    let lim = HALF as f32 - 1.5;
    if e.pos.x.abs() > lim { e.pos.x = e.pos.x.clamp(-lim, lim); e.vel.x = 0.0; e.hit_wall = true; }
    if e.pos.z.abs() > lim { e.pos.z = e.pos.z.clamp(-lim, lim); e.vel.z = 0.0; e.hit_wall = true; }
    if e.pos.y < -10.0 { e.pos.y = w.top_at(e.pos.x.floor() as i32, e.pos.z.floor() as i32) as f32 + 2.0; e.vel = Vec3::ZERO; }
}
