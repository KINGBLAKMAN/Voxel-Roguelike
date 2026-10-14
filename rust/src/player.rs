//! The player: movement and physics, body animation (walk cycle, idle breathing, capes, cast poses,
//! roll somersault, slam leap) and the over-the-shoulder camera.
use crate::models::*;
use crate::noise::lerp;
use crate::physics::*;
use crate::scene::MainCam;
use crate::world::*;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Pose { Push, Raise, Throw, Shout, Pray, Point, Lunge, BowUp, Kneel, Slam, Stab }

#[derive(Clone)]
pub struct Dash { pub dir: Vec3, pub speed: f32, pub t: f32, pub kind: u8 }
#[derive(Clone)]
pub struct Leap { pub from: Vec3, pub to: Vec3, pub t: f32, pub dur: f32, pub height: f32 }

pub fn class_weapons(c: Class) -> [&'static str; 2] {
    match c { Class::Ranger => ["longbow", "daggers"], Class::Mage => ["firestaff", "frostwand"], Class::Warrior => ["greatsword", "swordshield"], Class::Priest => ["mace", "scepter"] }
}

#[derive(Resource)]
pub struct Player {
    pub cls: Class, pub body: Body, pub yaw: f32, pub pitch: f32, pub vis_y: f32, pub rig: Rig, pub weapons: Vec<Weapon>, pub widx: usize,
    pub hp: f32, pub max_hp: f32, pub ghost_hp: f32, pub stam: f32, pub hunger: f32, pub dead: bool, pub death_t: f32, pub invuln: f32, pub hit_t: f32,
    pub walk_ph: f32, pub idle_ph: f32, pub swing_t: f32, pub swing_dur: f32, pub swing_side: f32, pub swing_kind: u8, pub cast_kick: f32,
    pub charge: f32, pub charging: bool, pub firing: bool, pub anim: Option<(Pose, f32, f32)>, pub roll_t: f32, pub dash: Option<Dash>, pub leap: Option<Leap>,
    pub whirl: f32, pub coyote: f32, pub move_dir: Vec3, pub aim_pt: Vec3, pub aim_dist: f32, pub sprinting: bool, pub spawned: bool, pub blocking: f32, pub dodge_cd: f32,
}
impl Default for Player {
    fn default() -> Self {
        Player { cls: Class::Warrior, body: Body::new(Vec3::new(0.5, 40.0, 0.5), 0.3, 1.8), yaw: 0.0, pitch: -0.15, vis_y: 40.0, rig: Rig::default(), weapons: vec![], widx: 0,
            hp: 100.0, max_hp: 100.0, ghost_hp: 100.0, stam: 100.0, hunger: 100.0, dead: false, death_t: 0.0, invuln: 0.0, hit_t: 0.0,
            walk_ph: 0.0, idle_ph: 0.0, swing_t: 1.0, swing_dur: 0.3, swing_side: 1.0, swing_kind: 0, cast_kick: 0.0, charge: 0.0, charging: false, firing: false,
            anim: None, roll_t: 0.0, dash: None, leap: None, whirl: 0.0, coyote: 0.0, move_dir: Vec3::ZERO, aim_pt: Vec3::ZERO, aim_dist: 60.0, sprinting: false, spawned: false, blocking: 0.0, dodge_cd: 0.0 }
    }
}
impl Player {
    pub fn fwd(&self) -> Vec3 { Vec3::new(self.yaw.sin(), 0.0, self.yaw.cos()) }
    pub fn right(&self) -> Vec3 { Vec3::new(-self.yaw.cos(), 0.0, self.yaw.sin()) }
    pub fn look(&self) -> Vec3 { Vec3::new(self.yaw.sin() * self.pitch.cos(), self.pitch.sin(), self.yaw.cos() * self.pitch.cos()) }
    pub fn weapon_id(&self) -> &'static str { class_weapons(self.cls)[self.widx] }
    pub fn pose(&mut self, p: Pose, d: f32) { self.anim = Some((p, 0.0, d)); }
}

/// spawn (or respawn) the player's model and weapons
pub fn build_player(c: &mut Commands, pal: &mut Palette, mats: &mut Assets<StandardMaterial>, p: &mut Player) {
    if let Some(r) = p.rig.root { c.entity(r).despawn_recursive(); }
    let mut b = Builder::new(c, pal, mats);
    p.rig = player_model(&mut b, p.body.pos, p.cls);
    p.weapons.clear();
    attach_weapon(c, pal, mats, p);
}
pub fn attach_weapon(c: &mut Commands, pal: &mut Palette, mats: &mut Assets<StandardMaterial>, p: &mut Player) {
    for w in p.weapons.drain(..) { c.entity(w.root).despawn_recursive(); }
    let mut b = Builder::new(c, pal, mats);
    let (hl, hr) = (p.rig.hand_l.unwrap(), p.rig.hand_r.unwrap());
    match p.weapon_id() {
        "longbow" => p.weapons.push(weapon_model(&mut b, hl, "longbow")),
        "daggers" => { p.weapons.push(weapon_model(&mut b, hr, "daggers")); p.weapons.push(weapon_model(&mut b, hl, "daggers")); }
        "swordshield" => { p.weapons.push(weapon_model(&mut b, hr, "sword")); p.weapons.push(weapon_model(&mut b, hl, "shield")); }
        id => p.weapons.push(weapon_model(&mut b, hr, id)),
    }
}

#[derive(Resource, Default)]
pub struct Controls { pub locked: bool, pub lock_t: f32, pub want_lock: bool, pub look_scale: f32, pub relock: bool }

pub fn cursor_lock(mut win: Query<&mut Window, With<PrimaryWindow>>, mouse: Res<ButtonInput<MouseButton>>, keys: Res<ButtonInput<KeyCode>>, mut ctl: ResMut<Controls>, time: Res<Time>) {
    let Ok(mut w) = win.get_single_mut() else { return };
    if ctl.want_lock && (mouse.just_pressed(MouseButton::Left) || ctl.relock) && !ctl.locked {
        ctl.relock = false;
        w.cursor_options.grab_mode = CursorGrabMode::Locked; w.cursor_options.visible = false; ctl.locked = true; ctl.lock_t = time.elapsed_secs();
    }
    if (keys.just_pressed(KeyCode::Escape) || !ctl.want_lock) && ctl.locked {
        w.cursor_options.grab_mode = CursorGrabMode::None; w.cursor_options.visible = true; ctl.locked = false;
    }
    // Windows reports the grab as lost when the window loses focus
    if !w.focused && ctl.locked { w.cursor_options.grab_mode = CursorGrabMode::None; w.cursor_options.visible = true; ctl.locked = false; }
}

/// mouse look, ignoring the bogus jumps some systems report right after locking or when the cursor warps
pub fn mouse_look(mm: Res<AccumulatedMouseMotion>, ctl: Res<Controls>, time: Res<Time>, mut p: ResMut<Player>) {
    if !ctl.locked || p.dead || time.elapsed_secs() - ctl.lock_t < 0.12 { return; }
    let d = mm.delta;
    if d.x.abs() > 400.0 || d.y.abs() > 300.0 { return; }
    let k = if ctl.look_scale > 0.0 { ctl.look_scale } else { 1.0 };
    p.yaw -= d.x * 0.0024 * k;
    p.pitch = (p.pitch - d.y * 0.0022 * k).clamp(-1.25, 1.05);
}

/// movement, stamina, jumping, dashes and leaps. `speed_mul` comes from upgrades.
pub fn move_player(w: &mut Voxels, p: &mut Player, keys: &ButtonInput<KeyCode>, dt: f32, speed_mul: f32, class_speed: f32, stam_max: f32) {
    if p.dead { p.death_t += dt; move_body(w, &mut p.body, dt, 26.0); return; }
    let (fwd, right) = (p.fwd(), p.right());
    let mut mv = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) { mv += fwd; } if keys.pressed(KeyCode::KeyS) { mv -= fwd; }
    if keys.pressed(KeyCode::KeyD) { mv += right; } if keys.pressed(KeyCode::KeyA) { mv -= right; }
    let moving = mv.length_squared() > 0.0;
    if moving { mv = mv.normalize(); }
    p.move_dir = mv;
    let sprint = keys.pressed(KeyCode::ShiftLeft) && moving && p.stam > 1.0 && !p.charging;
    p.sprinting = sprint;
    if sprint { p.stam = (p.stam - dt * 22.0).max(0.0); } else if p.blocking <= 0.0 { p.stam = (p.stam + dt * 20.0).min(stam_max); }
    let spd = 6.4 * class_speed * speed_mul * if sprint { 1.55 } else { 1.0 } * if p.charging { 0.55 } else { 1.0 } * if p.body.in_water { 0.6 } else { 1.0 };
    if let Some(d) = p.dash.as_mut() {
        d.t -= dt;
        p.body.vel.x = d.dir.x * d.speed; p.body.vel.z = d.dir.z * d.speed; p.body.vel.y = p.body.vel.y.max(-2.0);
        if d.t <= 0.0 { p.dash = None; }
    } else if let Some(l) = p.leap.as_mut() {
        l.t += dt;
        let k = (l.t / l.dur).min(1.0);
        p.body.pos = l.from.lerp(l.to, k) + Vec3::Y * (k * std::f32::consts::PI).sin() * l.height;
        p.body.vel = Vec3::ZERO;
        if k >= 1.0 { p.leap = None; }
        return;
    } else {
        let acc = if p.body.on_ground { 14.0 } else { 4.0 };
        p.body.vel.x = lerp(p.body.vel.x, mv.x * spd, (acc * dt).min(1.0));
        p.body.vel.z = lerp(p.body.vel.z, mv.z * spd, (acc * dt).min(1.0));
    }
    p.coyote = if p.body.on_ground { 0.12 } else { p.coyote - dt };
    if keys.pressed(KeyCode::Space) {
        if p.body.in_water { p.body.vel.y = p.body.vel.y.max(4.5); }
        else if p.coyote > 0.0 { p.body.vel.y = 9.2; p.coyote = 0.0; p.body.on_ground = false; }
    }
    move_body(w, &mut p.body, dt, 26.0);
    p.vis_y = if (p.vis_y - p.body.pos.y).abs() > 2.0 { p.body.pos.y } else { lerp(p.vis_y, p.body.pos.y, (dt * 16.0).min(1.0)) };
}

pub fn set_rot(q: &mut Query<&mut Transform>, e: Option<Entity>, x: f32, y: f32, z: f32) {
    if let Some(e) = e { if let Ok(mut t) = q.get_mut(e) { t.rotation = Quat::from_euler(EulerRot::XYZ, x, y, z); } }
}
fn get_rot(q: &Query<&mut Transform>, e: Option<Entity>) -> Vec3 {
    e.and_then(|e| q.get(e).ok()).map(|t| { let (x, y, z) = t.rotation.to_euler(EulerRot::XYZ); Vec3::new(x, y, z) }).unwrap_or(Vec3::ZERO)
}
fn blend(q: &mut Query<&mut Transform>, e: Option<Entity>, to: Vec3, w: f32) { let r = get_rot(q, e); let n = r.lerp(to, w); set_rot(q, e, n.x, n.y, n.z); }
fn set_pos(q: &mut Query<&mut Transform>, e: Option<Entity>, p: Vec3) { if let Some(e) = e { if let Ok(mut t) = q.get_mut(e) { t.translation = p; } } }

/// all the player's body animation for one frame
pub fn animate_player(p: &mut Player, q: &mut Query<&mut Transform>, dt: f32) {
    let r = p.rig.clone();
    if let Some(root) = r.root { if let Ok(mut t) = q.get_mut(root) {
        t.translation = Vec3::new(p.body.pos.x, p.vis_y, p.body.pos.z);
        let cur = t.rotation.to_euler(EulerRot::YXZ).0;
        let dy = ((p.yaw - cur) % std::f32::consts::TAU + std::f32::consts::TAU * 1.5) % std::f32::consts::TAU - std::f32::consts::PI;
        t.rotation = Quat::from_rotation_y(cur + dy * (dt * 18.0).min(1.0));
    } }
    if p.dead { let k = (p.death_t * 3.0).min(std::f32::consts::FRAC_PI_2); set_rot(q, r.body, -k, 0.0, 0.0); set_pos(q, r.body, Vec3::new(0.0, (p.death_t).min(0.3), 0.0)); return; }
    let hs = Vec2::new(p.body.vel.x, p.body.vel.z).length();
    p.walk_ph += dt * hs * 1.7;
    p.idle_ph += dt;
    let amt = (hs / 6.0).min(1.0) * if p.body.on_ground { 1.0 } else { 0.3 };
    let s = p.walk_ph.sin();
    let mut legl = s * 0.75 * amt; let mut legr = -s * 0.75 * amt;
    if !p.body.on_ground && p.leap.is_none() && p.dash.is_none() { legl = -0.5; legr = 0.3; }
    set_rot(q, r.leg_l, legl, 0.0, 0.0); set_rot(q, r.leg_r, legr, 0.0, 0.0);
    set_pos(q, r.body, Vec3::new(0.0, s.abs() * 0.06 * amt + (p.idle_ph * 2.2).sin() * 0.012 * (1.0 - amt), 0.0));
    set_rot(q, r.body, 0.0, if p.whirl > 0.0 { p.idle_ph * 22.0 } else { 0.0 }, 0.0);
    if p.whirl > 0.0 { p.whirl -= dt; }
    let mut torso = Vec3::new(amt * 0.08 + if p.sprinting { 0.12 } else { 0.0 }, 0.0, 0.0);
    let mut head = Vec3::ZERO;
    // arms: weapon-specific stance plus swings
    let walk_a = s * 0.6 * amt;
    let mut al = Vec3::new(-walk_a, 0.0, 0.08);
    let mut ar = Vec3::new(walk_a, 0.0, -0.08);
    p.cast_kick = (p.cast_kick - dt * 5.0).max(0.0);
    let id = p.weapon_id();
    let sw = p.swing_t.min(1.0);
    p.swing_t = (p.swing_t + dt / p.swing_dur).min(1.0);
    match id {
        "longbow" => {
            al = Vec3::new(-std::f32::consts::FRAC_PI_2 + 0.05 - p.pitch * 0.6, 0.12, 0.0);
            let c = if p.charging { p.charge } else { 0.0 };
            ar = Vec3::new(-std::f32::consts::FRAC_PI_2 + 0.1 - p.pitch * 0.6, -0.5 - c * 0.35, 0.0);
        }
        "firestaff" | "frostwand" | "scepter" => {
            if p.cast_kick > 0.0 || p.firing { ar = Vec3::new(-std::f32::consts::FRAC_PI_2 - p.pitch * 0.7 + p.cast_kick * 0.5, 0.0, 0.0); } else { ar = Vec3::new(-0.6 + walk_a * 0.3, 0.0, -0.1); }
        }
        _ => {
            if sw < 1.0 {
                let e = 1.0 - (1.0 - sw).powf(2.4);
                match id {
                    "daggers" => { let a = Vec3::new(-std::f32::consts::FRAC_PI_2 + lerp(-0.6, 0.5, e), p.swing_side * lerp(0.9, -0.9, e), 0.0); if p.swing_side > 0.0 { ar = a } else { al = a } }
                    "greatsword" => { ar = Vec3::new(lerp(-2.8, -0.2, e), lerp(0.4, -0.6, e), 0.0); al = Vec3::new(lerp(-2.6, -0.3, e), lerp(0.8, -0.2, e), 0.0); torso.y = lerp(0.5, -0.5, e); }
                    _ => { ar = Vec3::new(-std::f32::consts::FRAC_PI_2 + lerp(-1.1, 0.7, e) * 0.6, p.swing_side * lerp(1.2, -1.0, e), 0.0); torso.y = p.swing_side * lerp(0.35, -0.35, e); }
                }
            } else if id == "greatsword" { ar = Vec3::new(-0.5, 0.0, -0.1); al = Vec3::new(-0.6, 0.5, 0.1); } else { ar.x = -0.45 + walk_a * 0.4; }
            if id == "swordshield" { al = Vec3::new(-0.9, 0.5, 0.0); }
        }
    }
    // guard: forearms crossed in front of the face
    if p.blocking > 0.0 { al = Vec3::new(-1.65, 0.55, 0.35); ar = Vec3::new(-1.55, -0.55, -0.35); torso.x = 0.12; }
    set_rot(q, r.arm_l, al.x, al.y, al.z); set_rot(q, r.arm_r, ar.x, ar.y, ar.z);
    // the bow string draws back with the charge, an arrow nocked
    if let Some(w) = p.weapons.first() { if let Some([s1, s2, arrow]) = w.string {
        let c = if p.charging { p.charge } else { 0.0 };
        let ny = w.tip_y + 0.03 + c * 0.42;
        for (e, sg) in [(s1, 1.0), (s2, -1.0)] {
            let (dy, dz) = (w.tip_y - ny, sg * w.tip_z);
            let len = (dy * dy + dz * dz).sqrt();
            if let Ok(mut t) = q.get_mut(e) { t.scale.z = len; t.translation = Vec3::new(0.0, (w.tip_y + ny) / 2.0, dz / 2.0); t.rotation = Quat::from_rotation_x((-dy).atan2(dz)); }
        }
        if let Ok(mut t) = q.get_mut(arrow) { t.translation.y = ny; t.scale = if c > 0.02 { Vec3::ONE } else { Vec3::ZERO }; }
    } }
    // cast poses blend over everything above
    if let Some((pose, t, d)) = p.anim.as_mut() {
        *t += dt; let k = *t / *d; let w = (k / 0.12).min((1.0 - k) / 0.22).clamp(0.0, 1.0);
        let pose = *pose;
        if k >= 1.0 { p.anim = None; } else { apply_pose(q, &r, pose, k, w, &mut torso, &mut head); }
    }
    set_rot(q, r.torso, torso.x, torso.y, torso.z);
    set_rot(q, r.head, head.x, head.y, head.z);
    if p.roll_t > 0.0 {
        p.roll_t -= dt;
        let k = 1.0 - p.roll_t.max(0.0) / 0.3; let th = k * std::f32::consts::TAU;
        set_rot(q, r.body, th, 0.0, 0.0);
        set_pos(q, r.body, Vec3::new(0.0, 0.85 - 0.85 * th.cos() - 0.3 * (k * std::f32::consts::PI).sin(), -0.85 * th.sin()));
        set_rot(q, r.leg_l, -1.4, 0.0, 0.0); set_rot(q, r.leg_r, -1.4, 0.0, 0.0); set_rot(q, r.arm_l, -1.2, 0.0, 0.0); set_rot(q, r.arm_r, -1.2, 0.0, 0.0);
    }
    if let Some(l) = &p.leap {
        let k = (l.t / l.dur).min(1.0);
        set_rot(q, r.arm_l, -2.9 + k * 1.2, 0.0, 0.3); set_rot(q, r.arm_r, -2.9 + k * 1.2, 0.0, -0.3);
        let lg = -0.9 * (k * std::f32::consts::PI).sin(); set_rot(q, r.leg_l, lg, 0.0, 0.0); set_rot(q, r.leg_r, lg, 0.0, 0.0);
        set_rot(q, r.torso, -0.3 + k * 0.7, 0.0, 0.0);
    }
    if let Some(cape) = r.cape {
        let cur = get_rot(q, Some(cape)).x;
        let target = 0.08 + amt * 0.7 + if p.body.on_ground { 0.0 } else { 0.5 } + (p.idle_ph * (3.0 + amt * 6.0)).sin() * (0.04 + amt * 0.06);
        set_rot(q, Some(cape), lerp(cur, target, (dt * 8.0).min(1.0)), 0.0, 0.0);
    }
}
fn apply_pose(q: &mut Query<&mut Transform>, r: &Rig, pose: Pose, k: f32, w: f32, torso: &mut Vec3, head: &mut Vec3) {
    let v = Vec3::new;
    let lt = |a: &mut Vec3, b: Vec3| *a = a.lerp(b, w);
    match pose {
        Pose::Push => { let e = (k / 0.3).min(1.0); blend(q, r.arm_l, v(-1.2 - 0.4 * e, 0.25, 0.25), w); blend(q, r.arm_r, v(-1.2 - 0.4 * e, -0.25, -0.25), w); lt(torso, v(0.2 * e - 0.1, 0.0, 0.0)); }
        Pose::Raise => { blend(q, r.arm_l, v(-2.95, 0.0, 0.4), w); blend(q, r.arm_r, v(-2.95, 0.0, -0.4), w); lt(head, v(-0.4, 0.0, 0.0)); lt(torso, v(-0.2, 0.0, 0.0)); }
        Pose::Throw => { let f = if k < 0.35 { 0.0 } else { ((k - 0.35) / 0.25).min(1.0) }; blend(q, r.arm_r, v(lerp(-2.9, -0.8, f), 0.0, -0.2), w); blend(q, r.arm_l, v(-0.6, 0.0, 0.5), w); lt(torso, v(0.0, lerp(0.45, -0.35, f), 0.0)); }
        Pose::Shout => { blend(q, r.arm_l, v(-0.5, 0.0, 1.35), w); blend(q, r.arm_r, v(-0.5, 0.0, -1.35), w); lt(head, v(-0.5, 0.0, 0.0)); lt(torso, v(-0.3, 0.0, 0.0)); }
        Pose::Pray => { blend(q, r.arm_l, v(-2.0, 0.62, 0.0), w); blend(q, r.arm_r, v(-2.0, -0.62, 0.0), w); lt(head, v(-0.35 + 0.6 * (k * 2.0).min(1.0), 0.0, 0.0)); }
        Pose::Point => { let f = if k < 0.4 { 0.0 } else { ((k - 0.4) / 0.15).min(1.0) }; blend(q, r.arm_r, v(lerp(-3.0, -1.3, f), 0.0, -0.15), w); blend(q, r.arm_l, v(-0.3, 0.0, 0.3), w); lt(torso, v(-0.15 + 0.35 * f, 0.0, 0.0)); }
        Pose::Lunge => { lt(torso, v(0.55, 0.0, 0.0)); blend(q, r.arm_r, v(-1.5, 0.0, -0.1), w); blend(q, r.arm_l, v(-0.2, 0.0, 0.5), w); blend(q, r.leg_l, v(-0.9, 0.0, 0.0), w); blend(q, r.leg_r, v(0.7, 0.0, 0.0), w); }
        Pose::BowUp => { blend(q, r.arm_l, v(-2.7, 0.0, 0.2), w); blend(q, r.arm_r, v(-2.4, -0.4, 0.0), w); lt(head, v(-0.55, 0.0, 0.0)); lt(torso, v(-0.25, 0.0, 0.0)); }
        Pose::Kneel => {
            let f = if k < 0.35 { 0.0 } else { 1.0 };
            if let Some(b) = r.body { if let Ok(mut t) = q.get_mut(b) { t.translation.y -= 0.35 * w * (1.0 - f * 0.5); } }
            blend(q, r.arm_l, v(if f > 0.0 { -1.0 } else { -0.2 }, 0.0, if f > 0.0 { 1.4 } else { 0.9 }), w); blend(q, r.arm_r, v(if f > 0.0 { -1.0 } else { -0.2 }, 0.0, if f > 0.0 { -1.4 } else { -0.9 }), w);
            blend(q, r.leg_l, v(-1.0, 0.0, 0.0), w * (1.0 - f * 0.5)); blend(q, r.leg_r, v(0.6, 0.0, 0.0), w * (1.0 - f * 0.5)); lt(torso, v(0.3 - f * 0.4, 0.0, 0.0));
        }
        Pose::Slam => { let f = if k < 0.4 { 0.0 } else { ((k - 0.4) / 0.12).min(1.0) }; let a = lerp(-3.0, -0.6, f); blend(q, r.arm_l, v(a, 0.0, 0.25), w); blend(q, r.arm_r, v(a, 0.0, -0.25), w); lt(torso, v(lerp(-0.3, 0.5, f), 0.0, 0.0)); }
        Pose::Stab => { let f = ((k - 0.2) / 0.2).clamp(0.0, 1.0); blend(q, r.arm_r, v(lerp(-1.0, -1.7, f), 0.0, 0.0), w); lt(torso, v(lerp(-0.1, 0.3, f), lerp(0.4, -0.2, f), 0.0)); }
    }
}

#[derive(Resource)]
pub struct CamRig { pub d: f32, pub sh: f32, pub shake: f32, pub fov: f32, pub kick: Vec3 }
impl Default for CamRig { fn default() -> Self { CamRig { d: 6.6, sh: 0.75, shake: 0.0, fov: 72.0, kick: Vec3::ZERO } } }

/// over-the-shoulder camera: pulls in fast when blocked, eases back out gently so turning past trees never pops
pub fn update_camera(w: &Voxels, p: &Player, cr: &mut CamRig, cam: &mut Transform, proj: &mut Projection, dt: f32, rng: &mut impl FnMut() -> f32) {
    let f = p.look();
    let right = p.right();
    let mut tgt = Vec3::new(p.body.pos.x, p.vis_y + 1.78, p.body.pos.z);
    let mut sh = 0.75;
    for s in [0.25f32, 0.5, 0.75] { let q = tgt + right * s; if w.peek(q.x.floor() as i32, q.y.floor() as i32, q.z.floor() as i32) != 0 { sh = (s - 0.3).max(0.0); break; } }
    cr.sh = if sh < cr.sh { lerp(cr.sh, sh, (dt * 20.0).min(1.0)) } else { lerp(cr.sh, sh, (dt * 4.0).min(1.0)) };
    tgt += right * cr.sh;
    let back = -f;
    let mut dist: f32 = 6.6;
    for oy in [0.0, 0.35, -0.35] { if let Some((_, t, _)) = voxel_ray(w, tgt + Vec3::Y * oy, back, dist) { dist = (t - 0.3).max(0.5); } }
    let hard = dist;
    cr.d = if dt <= 0.0 { dist } else if dist < cr.d { lerp(cr.d, dist, (dt * 22.0).min(1.0)) } else { lerp(cr.d, dist, (dt * 3.5).min(1.0)) };
    cr.d = cr.d.min(hard + 0.6);
    let mut pos = tgt + back * cr.d;
    if cr.shake > 0.0 { let s = cr.shake * cr.shake * 0.35; pos += Vec3::new(rng() - 0.5, rng() - 0.5, rng() - 0.5) * 2.0 * s; cr.shake = (cr.shake - dt * 2.2).max(0.0); }
    *cam = Transform::from_translation(pos).looking_to(f, Vec3::Y);
    let target_fov = if p.charging { 64.0 - p.charge * 10.0 } else if p.dash.is_some() { 80.0 } else { 72.0 };
    cr.fov = lerp(cr.fov, target_fov, (dt * 8.0).min(1.0));
    if let Projection::Perspective(pp) = proj { pp.fov = cr.fov.to_radians(); }
}

pub fn _unused(_: Query<&Transform, With<MainCam>>) {}
