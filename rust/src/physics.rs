//! Axis-separated voxel collision with auto step-up, gravity and swimming.
use crate::world::*;
use bevy::prelude::*;

#[derive(Clone, Debug)]
pub struct Body { pub pos: Vec3, pub vel: Vec3, pub w: f32, pub h: f32, pub on_ground: bool, pub in_water: bool, pub hit_wall: bool, pub fly: bool }
impl Body { pub fn new(pos: Vec3, w: f32, h: f32) -> Self { Body { pos, vel: Vec3::ZERO, w, h, on_ground: false, in_water: false, hit_wall: false, fly: false } } }

pub fn collides(w: &mut Voxels, x: f32, y: f32, z: f32, bw: f32, bh: f32) -> bool {
    let (x0, x1) = ((x - bw).floor() as i32, (x + bw).floor() as i32);
    let (y0, y1) = (y.floor() as i32, (y + bh - 0.01).floor() as i32);
    let (z0, z1) = ((z - bw).floor() as i32, (z + bw).floor() as i32);
    for yy in y0..=y1 { for zz in z0..=z1 { for xx in x0..=x1 { if w.get(xx, yy, zz) != 0 { return true; } } } }
    false
}

pub fn move_body(w: &mut Voxels, e: &mut Body, dt: f32, grav: f32) {
    e.in_water = e.pos.y + 0.5 < WATER_Y;
    if !e.fly {
        e.vel.y -= grav * if e.in_water { 0.35 } else { 1.0 } * dt;
        if e.in_water { e.vel.y = e.vel.y.max(-3.0); e.vel *= 1.0 - dt * 1.5; }
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
