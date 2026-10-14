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
    pub land: f32, pub air_t: f32, pub prev_yaw: f32, pub lean: f32, pub hip: f32,
}
impl Default for Player {
    fn default() -> Self {
        Player { cls: Class::Warrior, body: Body::new(Vec3::new(0.5, 40.0, 0.5), 0.3, 1.8), yaw: 0.0, pitch: -0.15, vis_y: 40.0, rig: Rig::default(), weapons: vec![], widx: 0,
            hp: 100.0, max_hp: 100.0, ghost_hp: 100.0, stam: 100.0, hunger: 100.0, dead: false, death_t: 0.0, invuln: 0.0, hit_t: 0.0,
            walk_ph: 0.0, idle_ph: 0.0, swing_t: 1.0, swing_dur: 0.3, swing_side: 1.0, swing_kind: 0, cast_kick: 0.0, charge: 0.0, charging: false, firing: false,
            anim: None, roll_t: 0.0, dash: None, leap: None, whirl: 0.0, coyote: 0.0, move_dir: Vec3::ZERO, aim_pt: Vec3::ZERO, aim_dist: 60.0, sprinting: false, spawned: false, blocking: 0.0, dodge_cd: 0.0, land: 0.0, air_t: 0.0, prev_yaw: 0.0, lean: 0.0, hip: 0.0 }
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
pub struct Controls { pub locked: bool, pub lock_t: f32, pub want_lock: bool, pub look_scale: f32, pub relock: bool, pub invert_y: bool, pub pad: bool }

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
    p.pitch = (p.pitch - d.y * 0.0022 * k * if ctl.invert_y { -1.0 } else { 1.0 }).clamp(-1.25, 1.05);
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

/// test hook: freeze attack/draw progress at a value (f32 bits; NaN = off) so stills can be captured mid-swing
pub static FREEZE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0x7fc0_0000);
pub fn frozen() -> Option<f32> { let v = f32::from_bits(FREEZE.load(std::sync::atomic::Ordering::Relaxed)); if v.is_nan() { None } else { Some(v) } }

/// joint targets for one frame, blended onto the rig at the end
#[derive(Clone, Copy, Default)]
struct Tg { body: Vec3, body_pos: Vec3, torso: Vec3, head: Vec3, arm_l: Vec3, arm_r: Vec3, el_l: f32, el_r: f32, hand_l: Vec3, hand_r: Vec3, leg_l: f32, leg_r: f32, kn_l: f32, kn_r: f32 }
fn drive(q: &mut Query<&mut Transform>, e: Option<Entity>, v: Vec3, k: f32) {
    if let Some(e) = e { if let Ok(mut t) = q.get_mut(e) { let target = Quat::from_euler(EulerRot::XYZ, v.x, v.y, v.z); t.rotation = if k >= 1.0 { target } else { t.rotation.slerp(target, k) }; } }
}
pub fn is_melee(id: &str) -> bool { matches!(id, "greatsword" | "swordshield" | "daggers" | "mace") }

/// all the player's body animation for one frame
pub fn animate_player(p: &mut Player, q: &mut Query<&mut Transform>, dt: f32) {
    let r = p.rig.clone();
    let freeze = frozen();
    if let Some(root) = r.root { if let Ok(mut t) = q.get_mut(root) {
        t.translation = Vec3::new(p.body.pos.x, p.vis_y, p.body.pos.z);
        let cur = t.rotation.to_euler(EulerRot::YXZ).0;
        let dy = ((p.yaw - cur) % std::f32::consts::TAU + std::f32::consts::TAU * 1.5) % std::f32::consts::TAU - std::f32::consts::PI;
        t.rotation = Quat::from_rotation_y(cur + dy * if freeze.is_some() { 1.0 } else { (dt * 18.0).min(1.0) });
    } }
    if p.dead { let k = (p.death_t * 3.0).min(std::f32::consts::FRAC_PI_2); set_rot(q, r.body, -k, 0.0, 0.0); set_pos(q, r.body, Vec3::new(0.0, (p.death_t).min(0.3), 0.0)); return; }
    // the run cycle: phase follows distance covered, so feet never skate
    let vel = Vec2::new(p.body.vel.x, p.body.vel.z);
    let hs = vel.length();
    let (fw, rt) = (p.fwd(), p.right());
    let (vf, vr) = (vel.dot(Vec2::new(fw.x, fw.z)), vel.dot(Vec2::new(rt.x, rt.z)));
    let back = vf < -0.3 * hs && hs > 0.5;
    let stride = if p.sprinting { 2.15 } else { 1.75 };
    p.walk_ph += dt * hs * stride * if back { -1.0 } else { 1.0 };
    p.idle_ph += dt;
    let amt = (hs / 6.0).min(1.0) * if p.body.on_ground { 1.0 } else { 0.3 };
    let run = ((hs - 2.5) / 3.5).clamp(0.0, 1.0);
    let spr = if p.sprinting { 1.0 } else { 0.0 };
    let (s, c) = (p.walk_ph.sin(), p.walk_ph.cos());
    // landing: a squash that the knees absorb
    if !p.body.on_ground { p.air_t += dt; } else { if p.air_t > 0.25 { p.land = (p.air_t * 0.5).min(0.35); } p.air_t = 0.0; }
    p.land = (p.land - dt * 1.6).max(0.0);
    // hips turn towards the direction of travel (shoulders keep facing the aim); bank into turns
    let move_a = if hs > 0.8 && !back { vr.atan2(vf) } else { 0.0 };
    p.hip += (-(move_a.clamp(-0.85, 0.85)) * amt - p.hip) * (dt * 10.0).min(1.0);
    let yaw_rate = { let d = ((p.yaw - p.prev_yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)) - std::f32::consts::PI; d / dt.max(1e-4) };
    p.prev_yaw = p.yaw;
    let bank = (-yaw_rate * 0.035 + vr / 6.4 * 0.08).clamp(-0.22, 0.22) * amt;
    p.lean += (bank - p.lean) * (dt * 6.0).min(1.0);
    let mut t = Tg::default();
    // legs: a longer reach forward than back, knees lift on the swing and soften on contact
    let reach = 0.55 + 0.35 * run + 0.25 * spr;
    t.leg_l = (s * reach - s.max(0.0) * 0.12) * amt; t.leg_r = (-s * reach + (-s).max(0.0) * 0.12) * amt;
    let lift = 0.7 + 0.8 * run + 0.4 * spr;
    t.kn_l = ((-c).max(0.0) * lift + (c.max(0.0) * 0.18)) * amt + 0.06; t.kn_r = (c.max(0.0) * lift + ((-c).max(0.0) * 0.18)) * amt + 0.06;
    if !p.body.on_ground && p.leap.is_none() && p.dash.is_none() { t.leg_l = -0.65; t.leg_r = 0.3; t.kn_l = 1.1; t.kn_r = 0.65; }
    // highest in mid-stride, lowest as a foot passes under the body; landing dips
    let bob = s.abs() * (0.05 + 0.05 * run) - 0.04 * run;
    t.body_pos = Vec3::new(0.0, bob * amt + (p.idle_ph * 2.2).sin() * 0.012 * (1.0 - amt) - p.land * 0.5, 0.0);
    t.kn_l += p.land * 2.2; t.kn_r += p.land * 2.2; t.leg_l -= p.land * 1.1; t.leg_r -= p.land * 1.1;
    t.body = Vec3::new(0.0, p.hip + s * 0.1 * amt, p.lean);
    if p.whirl > 0.0 { t.body.y = p.idle_ph * 22.0; p.whirl -= dt; }
    // lean into the run; the shoulders counter the hips so the chest keeps facing the aim
    t.torso = Vec3::new(amt * (0.1 + 0.12 * run) + spr * 0.18 + p.land * 0.6 - p.pitch * 0.12, -(p.hip + s * 0.1 * amt) - s * 0.12 * amt, -p.lean * 0.4);
    t.head = Vec3::new(-p.pitch * 0.45 - t.torso.x * 0.7, s * 0.05 * amt, p.lean * 0.3);
    // arms: the weapon stance with a walk swing layered on
    let id = p.weapon_id();
    let st = crate::anim::stance(id);
    let mut pose = st;
    p.cast_kick = (p.cast_kick - dt * 5.0).max(0.0);
    p.swing_t = (p.swing_t + dt / p.swing_dur.max(0.05)).min(1.0);
    let mut sw = p.swing_t;
    if let Some(f) = freeze { sw = f.min(1.0); if id == "longbow" && p.swing_kind == 0 { p.charging = true; p.charge = f.min(1.0); } }
    let attacking = sw < 1.0;
    if id == "longbow" {
        if p.charging { pose = crate::anim::bow_draw(p.charge, p.pitch); } else if attacking { pose = crate::anim::bow_release(sw, p.pitch); }
    } else if attacking {
        pose = crate::anim::attack(id, p.swing_kind, sw);
    }
    let busy = attacking || p.charging;
    // arms: free arms pump against the legs with bent elbows; arms holding a weapon keep the stance and bounce with it
    let pump = 0.45 + 0.45 * run + 0.25 * spr;
    let (pump_r, pump_l) = (-s * pump * amt, s * pump * amt);
    let pump_el = -(0.35 + 0.95 * run * amt + 0.3 * spr);
    let two_hand = matches!(id, "greatsword" | "swordshield" | "longbow");
    t.arm_r = pose.ar; t.arm_l = pose.al;
    if !busy {
        if id == "daggers" && p.sprinting && amt > 0.5 {
            // the swept-back sprint
            t.arm_r = Vec3::new(1.05, 0.0, 0.18); t.arm_l = Vec3::new(1.05, 0.0, -0.18); pose.er = -0.15; pose.el = -0.15; pose.hr = Vec3::new(-0.6, 0.0, 0.0); pose.hl = Vec3::new(-0.6, 0.0, 0.0);
        } else if two_hand {
            t.arm_r.x += pump_r * 0.25; t.arm_l.x += pump_l * 0.25; t.arm_r.x -= bob * amt * 1.5; t.arm_l.x -= bob * amt * 1.5;
        } else {
            t.arm_r.x += pump_r * 0.45;
            t.arm_l = Vec3::new(st.al.x * (1.0 - amt) + pump_l, 0.0, st.al.z - 0.06 * amt); pose.el = st.el * (1.0 - amt) + pump_el * amt;
        }
    }
    t.el_r = pose.er; t.el_l = pose.el; t.hand_r = pose.hr; t.hand_l = pose.hl;
    t.torso += pose.torso;
    t.body_pos.z += pose.step;
    // casters raise the focus at the target while holding the attack
    if !attacking && p.firing && !is_melee(id) && id != "longbow" { t.arm_r = Vec3::new(-1.45 - p.pitch * 0.7, 0.0, 0.05); t.el_r = -0.25; }
    // guard: forearms crossed in front of the face
    if p.blocking > 0.0 { t.arm_l = Vec3::new(-1.3, 0.0, -0.45); t.el_l = -1.2; t.arm_r = Vec3::new(-1.25, 0.0, 0.45); t.el_r = -1.25; t.torso.x = 0.12; t.hand_r = Vec3::new(0.3, 0.0, 0.0); }
    // the bow string draws back with the charge, an arrow nocked
    if let Some(w) = p.weapons.first() { if let Some([s1, s2, arrow]) = w.string {
        let c = if p.charging { p.charge } else { 0.0 };
        let ny = w.tip_y + 0.03 + c * 0.42;
        for (e, sg) in [(s1, 1.0), (s2, -1.0)] {
            let (dy, dz) = (w.tip_y - ny, sg * w.tip_z);
            let len = (dy * dy + dz * dz).sqrt();
            if let Ok(mut tr) = q.get_mut(e) { tr.scale.z = len; tr.translation = Vec3::new(0.0, (w.tip_y + ny) / 2.0, dz / 2.0); tr.rotation = Quat::from_rotation_x((-dy).atan2(dz)); }
        }
        if let Ok(mut tr) = q.get_mut(arrow) { tr.translation.y = ny; tr.scale = if c > 0.02 { Vec3::ONE } else { Vec3::ZERO }; }
    } }
    // ability poses blend over everything above
    if let Some((pose, tt, d)) = p.anim.as_mut() {
        *tt += dt; let k = *tt / *d; let w = (k / 0.12).min((1.0 - k) / 0.22).clamp(0.0, 1.0);
        let pose = *pose;
        if k >= 1.0 { p.anim = None; } else { apply_pose(&mut t, pose, k, w); }
    }
    // blend every joint towards its target: quick enough to stay responsive, smooth enough never to pop
    let k = if freeze.is_some() { 1.0 } else { 1.0 - (-dt * if attacking { 38.0 } else { 20.0 }).exp() };
    let kl = if freeze.is_some() { 1.0 } else { 1.0 - (-dt * 26.0).exp() };
    drive(q, r.leg_l, Vec3::new(t.leg_l, 0.0, 0.0), kl); drive(q, r.leg_r, Vec3::new(t.leg_r, 0.0, 0.0), kl);
    drive(q, r.knee_l, Vec3::new(t.kn_l, 0.0, 0.0), kl); drive(q, r.knee_r, Vec3::new(t.kn_r, 0.0, 0.0), kl);
    drive(q, r.arm_l, t.arm_l, k); drive(q, r.arm_r, t.arm_r, k);
    drive(q, r.elbow_l, Vec3::new(t.el_l, 0.0, 0.0), k); drive(q, r.elbow_r, Vec3::new(t.el_r, 0.0, 0.0), k);
    for (hand, v) in [(r.hand_l, t.hand_l), (r.hand_r, t.hand_r)] { drive(q, hand, v, k); }
    drive(q, r.torso, t.torso, k); drive(q, r.head, t.head, k);
    drive(q, r.body, t.body, if p.whirl > 0.0 { 1.0 } else { k });
    if let Some(b) = r.body { if let Ok(mut tr) = q.get_mut(b) { tr.translation = tr.translation.lerp(t.body_pos, if freeze.is_some() { 1.0 } else { k }); } }
    if p.roll_t > 0.0 {
        p.roll_t -= dt;
        let kk = 1.0 - p.roll_t.max(0.0) / 0.3; let th = kk * std::f32::consts::TAU;
        set_rot(q, r.body, th, 0.0, 0.0);
        set_pos(q, r.body, Vec3::new(0.0, 0.85 - 0.85 * th.cos() - 0.3 * (kk * std::f32::consts::PI).sin(), -0.85 * th.sin()));
        set_rot(q, r.leg_l, -1.4, 0.0, 0.0); set_rot(q, r.leg_r, -1.4, 0.0, 0.0); set_rot(q, r.knee_l, 1.6, 0.0, 0.0); set_rot(q, r.knee_r, 1.6, 0.0, 0.0);
        set_rot(q, r.arm_l, -1.2, 0.0, 0.0); set_rot(q, r.arm_r, -1.2, 0.0, 0.0); set_rot(q, r.elbow_l, -1.2, 0.0, 0.0); set_rot(q, r.elbow_r, -1.2, 0.0, 0.0);
    }
    if let Some(l) = &p.leap {
        let kk = (l.t / l.dur).min(1.0);
        set_rot(q, r.arm_l, -2.9 + kk * 1.2, 0.0, 0.3); set_rot(q, r.arm_r, -2.9 + kk * 1.2, 0.0, -0.3);
        let lg = -0.9 * (kk * std::f32::consts::PI).sin(); set_rot(q, r.leg_l, lg, 0.0, 0.0); set_rot(q, r.leg_r, lg, 0.0, 0.0);
        set_rot(q, r.knee_l, -lg * 1.2, 0.0, 0.0); set_rot(q, r.knee_r, -lg * 1.2, 0.0, 0.0);
        set_rot(q, r.torso, -0.3 + kk * 0.7, 0.0, 0.0);
    }
    if let Some(cape) = r.cape {
        let cur = get_rot(q, Some(cape)).x;
        let target = 0.08 + amt * 0.7 + if p.body.on_ground { 0.0 } else { 0.5 } + (p.idle_ph * (3.0 + amt * 6.0)).sin() * (0.04 + amt * 0.06) + if attacking { 0.15 } else { 0.0 };
        set_rot(q, Some(cape), lerp(cur, target, (dt * 8.0).min(1.0)), 0.0, 0.0);
    }
}
fn apply_pose(t: &mut Tg, pose: Pose, k: f32, w: f32) {
    let v = Vec3::new;
    let lt = |a: &mut Vec3, b: Vec3| *a = a.lerp(b, w);
    let lf = |a: &mut f32, b: f32| *a += (b - *a) * w;
    match pose {
        Pose::Push => { let e = (k / 0.3).min(1.0); lt(&mut t.arm_l, v(-1.2 - 0.4 * e, 0.0, -0.25)); lt(&mut t.arm_r, v(-1.2 - 0.4 * e, 0.0, 0.25)); lf(&mut t.el_l, -0.2); lf(&mut t.el_r, -0.2); lt(&mut t.torso, v(0.2 * e - 0.1, 0.0, 0.0)); }
        Pose::Raise => { lt(&mut t.arm_l, v(-2.95, 0.0, 0.4)); lt(&mut t.arm_r, v(-2.95, 0.0, -0.4)); lf(&mut t.el_l, -0.3); lf(&mut t.el_r, -0.3); lt(&mut t.head, v(-0.4, 0.0, 0.0)); lt(&mut t.torso, v(-0.2, 0.0, 0.0)); }
        Pose::Throw => { let f = if k < 0.35 { 0.0 } else { ((k - 0.35) / 0.25).min(1.0) }; lt(&mut t.arm_r, v(lerp(-2.9, -0.8, f), 0.0, -0.2)); lf(&mut t.el_r, lerp(-1.4, -0.1, f)); lt(&mut t.arm_l, v(-0.6, 0.0, 0.5)); lt(&mut t.torso, v(0.0, lerp(0.45, -0.35, f), 0.0)); }
        Pose::Shout => { lt(&mut t.arm_l, v(-0.5, 0.0, 1.35)); lt(&mut t.arm_r, v(-0.5, 0.0, -1.35)); lf(&mut t.el_l, -1.3); lf(&mut t.el_r, -1.3); lt(&mut t.head, v(-0.5, 0.0, 0.0)); lt(&mut t.torso, v(-0.3, 0.0, 0.0)); }
        Pose::Pray => { lt(&mut t.arm_l, v(-1.2, 0.0, -0.3)); lt(&mut t.arm_r, v(-1.2, 0.0, 0.3)); lf(&mut t.el_l, -1.3); lf(&mut t.el_r, -1.3); lt(&mut t.head, v(-0.35 + 0.6 * (k * 2.0).min(1.0), 0.0, 0.0)); }
        Pose::Point => { let f = if k < 0.4 { 0.0 } else { ((k - 0.4) / 0.15).min(1.0) }; lt(&mut t.arm_r, v(lerp(-3.0, -1.4, f), 0.0, -0.15)); lf(&mut t.el_r, lerp(-0.8, 0.0, f)); lt(&mut t.arm_l, v(-0.3, 0.0, 0.3)); lt(&mut t.torso, v(-0.15 + 0.35 * f, 0.0, 0.0)); }
        Pose::Lunge => { lt(&mut t.torso, v(0.55, 0.0, 0.0)); lt(&mut t.arm_r, v(-1.5, 0.0, -0.1)); lf(&mut t.el_r, -0.1); lt(&mut t.arm_l, v(-0.2, 0.0, 0.5)); lf(&mut t.leg_l, -0.9); lf(&mut t.leg_r, 0.7); lf(&mut t.kn_l, 0.9); lf(&mut t.kn_r, 0.3); }
        Pose::BowUp => { lt(&mut t.arm_l, v(-2.7, 0.0, 0.2)); lf(&mut t.el_l, 0.0); lt(&mut t.hand_l, v(0.0, 0.0, 0.0)); lt(&mut t.arm_r, v(-2.4, 0.0, 0.3)); lf(&mut t.el_r, -1.6); lt(&mut t.head, v(-0.55, 0.0, 0.0)); lt(&mut t.torso, v(-0.25, 0.0, 0.0)); }
        Pose::Kneel => {
            let f = if k < 0.35 { 0.0 } else { 1.0 };
            t.body_pos.y -= 0.35 * w * (1.0 - f * 0.5);
            lt(&mut t.arm_l, v(if f > 0.0 { -1.0 } else { -0.2 }, 0.0, if f > 0.0 { 1.4 } else { 0.9 })); lt(&mut t.arm_r, v(if f > 0.0 { -1.0 } else { -0.2 }, 0.0, if f > 0.0 { -1.4 } else { -0.9 }));
            let w2 = w * (1.0 - f * 0.5); t.leg_l += (-1.0 - t.leg_l) * w2; t.leg_r += (0.6 - t.leg_r) * w2; t.kn_l += (1.6 - t.kn_l) * w2; t.kn_r += (0.6 - t.kn_r) * w2;
            lt(&mut t.torso, v(0.3 - f * 0.4, 0.0, 0.0));
        }
        Pose::Slam => { let f = if k < 0.4 { 0.0 } else { ((k - 0.4) / 0.12).min(1.0) }; let a = lerp(-3.0, -0.6, f); lt(&mut t.arm_l, v(a, 0.0, 0.25)); lt(&mut t.arm_r, v(a, 0.0, -0.25)); lf(&mut t.el_l, lerp(-1.0, -0.1, f)); lf(&mut t.el_r, lerp(-1.0, -0.1, f)); lt(&mut t.torso, v(lerp(-0.3, 0.5, f), 0.0, 0.0)); }
        Pose::Stab => { let f = ((k - 0.2) / 0.2).clamp(0.0, 1.0); lt(&mut t.arm_r, v(lerp(-1.0, -1.7, f), 0.0, 0.0)); lf(&mut t.el_r, lerp(-1.6, 0.0, f)); lt(&mut t.torso, v(lerp(-0.1, 0.3, f), lerp(0.4, -0.2, f), 0.0)); }
    }
}

#[derive(Resource)]
pub struct CamRig { pub d: f32, pub sh: f32, pub shake: f32, pub fov: f32, pub kick: Vec3, pub base_fov: f32, pub shake_mul: f32, pub orbit: f32, pub max_d: f32 }
impl Default for CamRig { fn default() -> Self { CamRig { d: 6.6, sh: 0.75, shake: 0.0, fov: 72.0, kick: Vec3::ZERO, base_fov: 72.0, shake_mul: 1.0, orbit: 0.0, max_d: 6.6 } } }

/// over-the-shoulder camera: pulls in fast when blocked, eases back out gently so turning past trees never pops
pub fn update_camera(w: &Voxels, p: &Player, cr: &mut CamRig, cam: &mut Transform, proj: &mut Projection, dt: f32, rng: &mut impl FnMut() -> f32) {
    let f = Quat::from_rotation_y(cr.orbit) * p.look();
    let right = p.right();
    let mut tgt = Vec3::new(p.body.pos.x, p.vis_y + 1.78, p.body.pos.z);
    let mut sh = 0.75;
    for s in [0.25f32, 0.5, 0.75] { let q = tgt + right * s; if w.peek(q.x.floor() as i32, q.y.floor() as i32, q.z.floor() as i32) != 0 { sh = (s - 0.3).max(0.0); break; } }
    cr.sh = if sh < cr.sh { lerp(cr.sh, sh, (dt * 20.0).min(1.0)) } else { lerp(cr.sh, sh, (dt * 4.0).min(1.0)) };
    tgt += right * cr.sh;
    let back = -f;
    let mut dist: f32 = cr.max_d;
    for oy in [0.0, 0.35, -0.35] { if let Some((_, t, _)) = voxel_ray(w, tgt + Vec3::Y * oy, back, dist) { dist = (t - 0.3).max(0.5); } }
    let hard = dist;
    cr.d = if dt <= 0.0 { dist } else if dist < cr.d { lerp(cr.d, dist, (dt * 22.0).min(1.0)) } else { lerp(cr.d, dist, (dt * 3.5).min(1.0)) };
    cr.d = cr.d.min(hard + 0.6);
    let mut pos = tgt + back * cr.d;
    if cr.shake > 0.0 { let s = cr.shake * cr.shake * 0.35; pos += Vec3::new(rng() - 0.5, rng() - 0.5, rng() - 0.5) * 2.0 * s; cr.shake = (cr.shake - dt * 2.2).max(0.0); }
    *cam = Transform::from_translation(pos).looking_to(f, Vec3::Y);
    let b = cr.base_fov;
    let target_fov = if p.charging { b - 8.0 - p.charge * 10.0 } else if p.dash.is_some() { b + 8.0 } else { b };
    cr.fov = lerp(cr.fov, target_fov, (dt * 8.0).min(1.0));
    if let Projection::Perspective(pp) = proj { pp.fov = cr.fov.to_radians(); }
}

pub fn _unused(_: Query<&Transform, With<MainCam>>) {}

/// ribbon trails swept by the blade (or a caster's focus) during the strike of an attack
#[derive(Default)]
pub struct TrailState { e: Option<Entity>, mesh: Option<Handle<Mesh>>, pts: Vec<(Vec3, Vec3, f32, usize)> }
pub fn weapon_trails(mut st: Local<TrailState>, p: Res<Player>, gt: Query<&GlobalTransform>, mut meshes: ResMut<Assets<Mesh>>, mut mats: ResMut<Assets<StandardMaterial>>, mut c: Commands, time: Res<Time>) {
    use bevy::render::mesh::{Indices, PrimitiveTopology};
    const LIFE: f32 = 0.17;
    let st = &mut *st;
    if st.mesh.is_none() {
        let m = meshes.add(Mesh::new(PrimitiveTopology::TriangleList, bevy::render::render_asset::RenderAssetUsages::default()));
        let mat = mats.add(StandardMaterial { base_color: Color::WHITE, unlit: true, alpha_mode: AlphaMode::Add, cull_mode: None, double_sided: true, fog_enabled: false, ..default() });
        st.e = Some(c.spawn((Mesh3d(m.clone()), MeshMaterial3d(mat), Transform::default(), bevy::render::view::NoFrustumCulling, bevy::pbr::NotShadowCaster)).id());
        st.mesh = Some(m);
    }
    let dt = time.delta_secs().min(0.05);
    for q in st.pts.iter_mut() { q.2 += dt; }
    st.pts.retain(|q| q.2 < LIFE);
    let id = p.weapon_id();
    let sw = frozen().unwrap_or(p.swing_t);
    let active = p.spawned && !p.dead && sw > 0.1 && sw < 0.66 && id != "longbow";
    let (col, from) = match id { "greatsword" => (Vec3::new(0.85, 0.92, 1.0) * 1.8, 0.3), "swordshield" => (Vec3::new(1.0, 0.96, 0.9) * 1.6, 0.3), "daggers" => (Vec3::new(0.75, 0.55, 1.0) * 2.0, 0.25),
        "mace" => (Vec3::new(1.0, 0.82, 0.4) * 1.8, 0.7), "firestaff" => (Vec3::new(1.0, 0.45, 0.12) * 3.0, 0.88), "frostwand" => (Vec3::new(0.5, 0.85, 1.0) * 3.0, 0.8), _ => (Vec3::new(1.0, 0.9, 0.5) * 3.0, 0.85) };
    if active {
        for (wi, w) in p.weapons.iter().enumerate() {
            if id == "swordshield" && wi > 0 { break; }
            if id == "daggers" && ((p.swing_kind == 0 && wi == 1) || (p.swing_kind == 1 && wi == 0)) { continue; }
            if let (Ok(a), Ok(b)) = (gt.get(w.root), gt.get(w.tip)) { let (a, b) = (a.translation(), b.translation()); st.pts.push((a.lerp(b, from), b, 0.0, wi)); }
        }
    }
    let (mut pos, mut cols, mut nrm, mut idx) = (vec![], vec![], vec![], vec![]);
    for wi in 0..2 {
        let mut pts: Vec<&(Vec3, Vec3, f32, usize)> = st.pts.iter().filter(|q| q.3 == wi).collect();
        pts.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
        if pts.len() < 2 { continue; }
        let base = pos.len() as u32;
        for q in &pts {
            let a = (1.0 - q.2 / LIFE).max(0.0).powf(1.5);
            pos.push(q.0.to_array()); pos.push(q.1.to_array());
            cols.push([col.x, col.y, col.z, a * 0.1]); cols.push([col.x, col.y, col.z, a * 0.85]);
            nrm.push([0.0, 1.0, 0.0]); nrm.push([0.0, 1.0, 0.0]);
        }
        for i in 0..pts.len() as u32 - 1 { let b = base + i * 2; idx.extend_from_slice(&[b, b + 1, b + 3, b, b + 3, b + 2]); }
    }
    if idx.is_empty() { pos = vec![[0.0; 3]; 3]; cols = vec![[0.0; 4]; 3]; nrm = vec![[0.0, 1.0, 0.0]; 3]; idx = vec![0, 1, 2]; }
    if let Some(m) = st.mesh.as_ref().and_then(|h| meshes.get_mut(h)) {
        m.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos); m.insert_attribute(Mesh::ATTRIBUTE_COLOR, cols); m.insert_attribute(Mesh::ATTRIBUTE_NORMAL, nrm); m.insert_indices(Indices::U32(idx));
    }
}
