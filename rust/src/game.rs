//! Gameplay core: the simulation state (enemies, projectiles, zones, pickups, delayed actions), the Ctx
//! that bundles everything a gameplay function needs, and damage, status effects, kills, loot and XP.
use crate::enemies::*;
use crate::fx::*;
use crate::models::*;
use crate::player::*;
use crate::world::*;
use bevy::prelude::*;
use std::collections::HashMap;
use std::f32::consts::TAU;

pub type Act = Box<dyn FnOnce(&mut Ctx) + Send + Sync>;
pub struct Timer { pub t: f32, pub f: Act, pub ghost: bool }

#[derive(Clone, Copy, Default)]
pub struct Stats { pub dmg: f32, pub cd: f32, pub speed: f32, pub leech: f32, pub crit: f32, pub armor: f32, pub regen: f32, pub food: f32, pub elite: f32, pub magnet: f32, pub area: f32,
    pub extra_proj: f32, pub charge_mul: f32, pub fire_rad: f32, pub shatter: f32, pub ww_heal: f32, pub radiance: f32, pub atk_spd: f32, pub stam_max: f32, pub stam_regen: f32, pub xp_mul: f32 }
impl Stats { pub fn base() -> Self { Stats { dmg: 1.0, cd: 1.0, speed: 1.0, crit: 0.06, food: 1.0, magnet: 1.0, area: 1.0, charge_mul: 1.0, fire_rad: 1.0, atk_spd: 1.0, stam_max: 100.0, stam_regen: 1.0, xp_mul: 1.0, ..default() } } }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PickKind { Xp, Meat, Berries, Heart, Magnet, Bomb, Gold }
pub struct Pickup { pub kind: PickKind, pub v: f32, pub pos: Vec3, pub vel: Vec3, pub t: f32, pub e: Entity, pub vac: bool, pub ph: f32 }

#[derive(Clone)]
pub struct Popup { pub pos: Vec3, pub text: String, pub kind: u8 }

#[derive(Default, Clone, Copy)]
pub struct Hit { pub kb: f32, pub up: f32, pub src: Option<Vec3>, pub col: u32, pub quiet: bool }
#[derive(Default, Clone, Copy)]
pub struct Status { pub stun: f32, pub slow: f32, pub freeze: f32, pub burn: f32, pub poison: f32, pub fear: f32, pub chill: u8, pub thr: u8, pub push: Option<Vec3> }

#[derive(Resource)]
pub struct Game {
    pub enemies: Vec<Enemy>, pub projs: Vec<crate::combat::Proj>, pub zones: Vec<crate::combat::Zone>, pub traps: Vec<crate::combat::Trap>, pub pickups: Vec<Pickup>,
    pub timers: Vec<Timer>, pub corpses: Vec<Corpse>, pub popups: Vec<Popup>, pub toasts: Vec<(String, String, f32)>,
    pub next_id: u32, pub run_t: f32, pub kills: u32, pub hitstop: f32, pub st: Stats, pub mods: HashMap<String, u32>, pub perks: Vec<String>,
    pub lvl: u32, pub xp: f32, pub cds: [f32; 4], pub atk_t: f32, pub buffs: HashMap<&'static str, f32>, pub food_meat: u32, pub food_berries: u32,
    pub hits: u32, pub perk_queue: u32, pub level_pending: u32, pub regen_acc: f32, pub over: bool, pub flash: f32, pub dir: crate::director::Director,
    pub rel: crate::relics::RelicState, pub flags: HashMap<&'static str, f32>, pub rerolls: u32, pub chest_pending: u32, pub boss: Option<u32>, pub ab: crate::abilities::AbState, pub heal_budget: f32, pub last_hit: f32, pub gold: u32, pub locked: Vec<&'static str>, pub ev: crate::events::EventState, pub prompt: String, pub net: crate::net::Net,
    pub dmg_taken: f32, pub healed: f32,
    /// the run is three stages on three maps: which one we're on, how long we've been here and where we're headed
    pub stage: u32, pub stage_t: f32, pub map: MapKind, pub route: MapKind, pub pending_stage: Option<(u32, MapKind, u32)>,
    pub minions: Vec<crate::abilities2::Minion>,
}
impl Default for Game {
    fn default() -> Self {
        Game { enemies: vec![], projs: vec![], zones: vec![], traps: vec![], pickups: vec![], timers: vec![], corpses: vec![], popups: vec![], toasts: vec![],
            next_id: 1, run_t: 0.0, kills: 0, hitstop: 0.0, st: Stats::base(), mods: HashMap::new(), perks: vec![], lvl: 1, xp: 0.0, cds: [0.0; 4], atk_t: 0.0,
            buffs: HashMap::new(), food_meat: 1, food_berries: 3, hits: 0, perk_queue: 0, level_pending: 0, regen_acc: 0.0, over: false, flash: 0.0,
            dir: Default::default(), rel: Default::default(), flags: HashMap::new(), rerolls: 2, chest_pending: 0, boss: None, ab: Default::default(), heal_budget: 0.0, last_hit: -10.0, gold: 0, locked: vec![], ev: Default::default(), prompt: String::new(), net: Default::default(), dmg_taken: 0.0, healed: 0.0,
            stage: 0, stage_t: 0.0, map: MapKind::Forest, route: MapKind::Swamp, pending_stage: None, minions: vec![] }
    }
}
impl Game {
    pub fn m(&self, k: &str) -> u32 { *self.mods.get(k).unwrap_or(&0) }
    pub fn buff(&self, k: &str) -> f32 { *self.buffs.get(k).unwrap_or(&0.0) }
    /// the run's intensity clock: the three five-minute stages compress what used to be twenty minutes of escalation
    pub fn minutes(&self) -> f32 { self.run_t / 45.0 }
    /// the map a stage is played on
    pub fn stage_map(&self, stage: u32) -> MapKind { match stage { 0 => MapKind::Forest, 1 => self.route, _ => MapKind::Volcanic } }
    /// difficulty multipliers from the Ember level (set when a run starts)
    pub fn mods_hp(&self) -> f32 { *self.flags.get("ember_hp").unwrap_or(&1.0) }
    pub fn mods_dmg(&self) -> f32 { *self.flags.get("ember_dmg").unwrap_or(&1.0) }
    pub fn ember(&self) -> u32 { *self.flags.get("ember").unwrap_or(&0.0) as u32 }
    pub fn xp_need(&self) -> f32 { let l = (self.lvl - 1) as f32; (20.0 + l * 14.0 + l * l * 1.1).round() }
}

/// everything a gameplay function might touch this frame
pub struct Ctx<'a, 'w, 's> {
    pub c: &'a mut Commands<'w, 's>, pub g: &'a mut Game, pub w: &'a mut Voxels, pub p: &'a mut Player, pub pal: &'a mut Palette,
    pub mats: &'a mut Assets<StandardMaterial>, pub meshes: &'a mut Assets<Mesh>, pub fx: &'a mut Fx, pub q: &'a mut Query<'w, 's, &'static mut Transform>,
    pub cam: &'a mut CamRig, pub dc: &'a crate::scene::DayCycle, pub sfx: &'a mut crate::audio::Sfx, pub dt: f32,
}
impl<'a, 'w, 's> Ctx<'a, 'w, 's> {
    pub fn gfx(&mut self) -> Gfx<'_, 'w, 's> { Gfx { c: &mut *self.c, meshes: &mut *self.meshes, mats: &mut *self.mats, pal: &mut *self.pal, fx: &mut *self.fx } }
    pub fn builder(&mut self) -> Builder<'_, 'w, 's> { Builder::new(&mut *self.c, &mut *self.pal, &mut *self.mats) }
    pub fn later(&mut self, t: f32, f: impl FnOnce(&mut Ctx) + Send + Sync + 'static) { let ghost = self.g.net.ghost; self.g.timers.push(Timer { t, f: Box::new(f), ghost }); }
    /// a toast every player sees
    pub fn announce(&mut self, a: &str, b: &str, d: f32) { self.toast(a, b, d); if self.g.net.host() { self.g.net.send(crate::net::Msg::Toast { a: a.into(), b: b.into() }); } }
    /// change a block and tell the other players
    pub fn set_block(&mut self, x: i32, y: i32, z: i32, b: u8) { self.w.set(x, y, z, b); if !self.g.net.ghost { self.g.net.send(crate::net::Msg::Block { x, y, z, b }); } }
    pub fn r(&self) -> f32 { self.fx.r() }
    pub fn rr(&self, a: f32, b: f32) -> f32 { self.fx.rr(a, b) }
    pub fn pick<T: Copy>(&self, v: &[T]) -> T { v[(self.r() * v.len() as f32) as usize % v.len()] }
    pub fn m(&self, k: &str) -> u32 { self.g.m(k) }
    /// a quick lens punch and a white impact frame for the big moments
    pub fn punch(&mut self, s: f32) { self.cam.punch = self.cam.punch.max(s.min(1.0)); self.g.flash = self.g.flash.max(0.05 * s); }
    pub fn shake(&mut self, s: f32) { self.cam.shake = (self.cam.shake + s * self.cam.shake_mul).min(1.0); }
    pub fn play(&mut self, name: &'static str) { self.sfx.play(name); }
    pub fn toast(&mut self, a: &str, b: &str, d: f32) { self.g.toasts.push((a.to_string(), b.to_string(), d)); }
    pub fn pop(&mut self, pos: Vec3, text: String, kind: u8) { if self.g.popups.len() < 60 { self.g.popups.push(Popup { pos, text, kind }); } }
    pub fn ground(&mut self, x: f32, z: f32, y: f32) -> f32 { self.w.ground_at(x, z, y) }
    pub fn ppos(&self) -> Vec3 { self.p.body.pos }
    pub fn hand(&mut self) -> Vec3 {
        if let Some(w) = self.p.weapons.first() { if let Ok(t) = self.q.get(w.tip) { let _ = t; } }
        // world position of the weapon tip from the propagated transforms isn't ready mid-frame; approximate from the rig
        let f = self.p.fwd(); let r = self.p.right();
        self.p.body.pos + Vec3::Y * 1.45 + f * 0.55 + r * if self.p.weapon_id() == "longbow" { -0.25 } else { 0.32 }
    }
    pub fn dmg_mul(&self) -> f32 {
        self.g.st.dmg * if self.g.buff("riposte") > 0.0 { 1.5 } else { 1.0 } * if self.g.buff("cry") > 0.0 { self.g.flags.get("cry_amp").copied().unwrap_or(1.4) } else { 1.0 } * if self.g.buff("shrine") > 0.0 { 1.25 } else { 1.0 } * crate::augments::dmg_mul(self)
    }

    // ---------------- damage ----------------
    pub fn find(&self, id: u32) -> Option<usize> { self.g.enemies.iter().position(|e| e.id == id && !e.dead) }
    /// player damage to an enemy, with crits, lifesteal, knockback, popups and hit sparks
    pub fn damage(&mut self, i: usize, amount: f32, o: Hit) {
        if self.g.net.ghost || self.g.net.no_enemy_dmg { return; }
        if i >= self.g.enemies.len() || self.g.enemies[i].dead || self.g.enemies[i].ai.hidden { return; }
        let crit = self.r() < self.g.st.crit || self.g.buff("focus") > 0.0;
        let (elite, frozen, brute, pos, h, prop) = { let e = &self.g.enemies[i]; (e.elite, e.frozen > 0.0, e.affix == Some(Affix::Brute), e.body.pos, e.body.h, e.t.prop) };
        let mut a = amount * self.dmg_mul() * if elite { 1.0 + self.g.st.elite } else { 1.0 } * if frozen { 1.25 + self.g.st.shatter } else { 1.0 } * if crit { 2.0 } else { 1.0 };
        if brute { a *= 0.8; }
        // shieldbearers turn blows that land on the front of the shield
        if self.g.enemies[i].kind == Kind::Shieldbearer { if let Some(s) = o.src { let e = &self.g.enemies[i]; let f = Vec3::new(e.yaw.sin(), 0.0, e.yaw.cos()); if (s - e.body.pos).with_y(0.0).normalize_or_zero().dot(f) > 0.35 { a *= 0.2; let p = e.body.pos + Vec3::Y * 1.4; self.fx.sparks(p + f * 0.6, 0xbfe8ff, 5, 6.0); } } }
        // enemies grow armour over the run: a flat cut per hit that punishes many tiny hits, never below 30%
        if !prop { let armor = (self.g.minutes() / 4.0).floor().min(5.0); a = (a - armor).max(a * 0.3); }
        a = (a * self.rr(0.92, 1.08)).round().max(1.0);
        let hp = pos + Vec3::Y * h * 0.6;
        if !o.quiet || self.r() < 0.5 { self.pop(pos + Vec3::Y * (h + 0.2), format!("{}", a as i32), if crit { 1 } else { 0 }); }
        if crit {
            self.play("crit");
            if self.g.run_t > *self.g.flags.get("crit_flare").unwrap_or(&0.0) { self.g.flags.insert("crit_flare", self.g.run_t + 0.08); self.gfx().impact(hp, 0xffe8a0, 1.1); }
            crate::augments::on_crit(self, i);
        } else { self.play("hit"); }
        let col = if o.col == 0 { 0xffffff } else { o.col };
        self.fx.emit(true, hp, Vec3::ZERO, 0.12, if crit { 2.2 } else { 1.4 }, 0.2, hexc(col) * 2.5, 0.0, 0.0, 1.0);
        let blood = self.g.enemies[i].t.blood;
        for _ in 0..if crit { 8 } else { 4 } { let v = Vec3::new(self.rr(-4.0, 4.0), self.rr(1.0, 5.0), self.rr(-4.0, 4.0)); let l = self.rr(0.3, 0.6); let bc = hexc(self.pick(blood)); self.fx.emit(false, hp, v, l, 0.18, 0.06, bc, 14.0, 1.0, 1.0); }
        let raw = self.g.st.leech + if self.g.buff("cry") > 0.0 { *self.g.flags.get("cry_leech").unwrap_or(&0.1) } else { 0.0 };
        // lifesteal has diminishing returns and draws from the per-second sustain budget
        let leech = raw / (1.0 + raw * 3.0);
        if leech > 0.0 && !prop { self.sustain(a * leech); }
        self.g.flags.insert("cross_hit", 0.11);
        self.apply_damage(i, a, o);
    }
    pub fn apply_damage(&mut self, i: usize, a: f32, o: Hit) {
        if self.g.net.ghost || self.g.net.no_enemy_dmg || i >= self.g.enemies.len() { return; }
        // clients report their hits to the host, which owns every monster's health
        if self.g.net.client() {
            let e = &mut self.g.enemies[i]; if e.dead || e.ai.hidden { return; } e.flash = 0.1; e.hit_p = 0.14;
            let m = crate::net::Msg::Hit { id: e.id, dmg: a, kb: o.kb, up: o.up, src: o.src.unwrap_or(e.body.pos).to_array(), from: self.g.net.my_id };
            self.g.net.send(m); return;
        }
        let e = &mut self.g.enemies[i];
        if e.dead || e.ai.hidden { return; }
        e.hp -= a; e.flash = 0.1; e.hit_p = 0.14; e.alert = true;
        let goblin = e.kind == Kind::Goblin; let gp = e.body.pos + Vec3::Y;
        if e.t.passive { e.flee = 4.0; }
        if let Some(src) = o.src { if o.kb > 0.0 && !e.elite && !e.t.prop {
            let kr = e.t.kb_res;
            let mut d = Vec3::new(e.body.pos.x - src.x, 0.0, e.body.pos.z - src.z);
            if d.length_squared() < 0.001 { d = Vec3::X; }
            d = d.normalize() * o.kb * (1.0 - kr);
            e.body.vel.x += d.x; e.body.vel.z += d.z;
            e.body.vel.y = e.body.vel.y.max(if o.up > 0.0 { o.up * (1.0 - kr) } else { (o.kb * 0.6).min(5.0) * (1.0 - kr) });
        } }
        let dead = e.hp <= 0.0;
        if goblin && !dead && self.r() < 0.35 { self.drop_pick(gp, PickKind::Gold, 1.0); }
        if dead { kill_enemy(self, i, false, false); }
    }
    pub fn status(&mut self, i: usize, s: Status) {
        if i >= self.g.enemies.len() || self.g.net.ghost || self.g.net.no_enemy_dmg { return; }
        if self.g.net.client() { let id = self.g.enemies[i].id; self.g.net.send(crate::net::Msg::Status { id, stun: s.stun, slow: s.slow, freeze: s.freeze, burn: s.burn, poison: s.poison, fear: s.fear, chill: s.chill, thr: s.thr, push: s.push.map(|v| v.to_array()) }); return; }
        let e = &mut self.g.enemies[i];
        if e.dead || e.t.prop { return; }
        let el = if e.boss { 0.25 } else if e.elite { 0.5 } else { 1.0 };
        if s.stun > 0.0 { e.stun = e.stun.max(s.stun * el); }
        if s.fear > 0.0 { e.fear = e.fear.max(s.fear); }
        if s.slow > 0.0 { e.slow = e.slow.max(s.slow); }
        if s.burn > 0.0 { e.burn = e.burn.max(s.burn); }
        if s.poison > 0.0 { e.poison = e.poison.max(s.poison); }
        let mut freeze = if s.freeze > 0.0 { s.freeze * el } else { 0.0 };
        if s.chill > 0 { e.chill += s.chill as u32; if e.chill >= if s.thr > 0 { s.thr as u32 } else { 5 } { e.chill = 0; freeze = freeze.max(1.6); } }
        if let Some(p) = s.push { if !e.elite { e.body.vel.x += p.x; e.body.vel.y = e.body.vel.y.max(p.y); e.body.vel.z += p.z; } }
        if freeze > 0.0 { let was = e.frozen > 0.0; e.frozen = e.frozen.max(freeze); if !was { freeze_fx(self, i); } }
    }
    /// every live enemy near a point (by index); visit with f(ctx, index, dist)
    pub fn near(&self, c: Vec3, r: f32) -> Vec<(usize, f32)> {
        let mut out = vec![];
        for (i, e) in self.g.enemies.iter().enumerate() { if e.dead || e.ai.hidden { continue; } let d = Vec2::new(e.body.pos.x - c.x, e.body.pos.z - c.z).length(); if d < r + e.body.w && (e.body.pos.y - c.y).abs() < r + 2.0 { out.push((i, d)); } }
        out
    }
    pub fn aoe(&mut self, c: Vec3, r: f32, f: &mut dyn FnMut(&mut Ctx, usize, f32)) -> usize { let v = self.near(c, r); let n = v.len(); for (i, d) in v { if !self.g.enemies[i].dead { f(self, i, d); } } n }
    /// hostile targets sorted by distance
    pub fn foes(&self, pos: Vec3, r: f32) -> Vec<usize> {
        let mut v: Vec<(f32, usize)> = self.g.enemies.iter().enumerate().filter(|(_, e)| !e.dead && !e.ai.hidden && !e.t.passive && !e.t.prop).map(|(i, e)| (Vec2::new(e.body.pos.x - pos.x, e.body.pos.z - pos.z).length(), i)).filter(|(d, _)| *d < r).collect();
        v.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap()); v.into_iter().map(|x| x.1).collect()
    }

    // ---------------- the player ----------------
    pub fn heal(&mut self, a: f32, quiet: bool) {
        if self.p.dead || self.g.net.ghost { return; }
        let a = a * if self.g.ember() >= 4 { 0.75 } else { 1.0 };
        let before = self.p.hp; self.p.hp = (self.p.hp + a).min(self.p.max_hp); self.g.healed += self.p.hp - before;
        let d = (self.p.hp - before).round();
        if d >= 2.0 && !quiet { let pp = self.p.body.pos + Vec3::Y * 2.1; self.pop(pp, format!("+{}", d as i32), 2); }
    }
    /// healing over time and lifesteal: capped by a budget of 4% max health per second, halved for 2s after a hit
    pub fn sustain(&mut self, a: f32) {
        if self.p.dead || self.g.net.ghost { return; }
        let a = (a * if self.g.run_t - self.g.last_hit < 2.0 { 0.5 } else { 1.0 }).min(self.g.heal_budget);
        if a <= 0.0 { return; }
        self.g.heal_budget -= a;
        self.heal(a, true);
    }
    pub fn hurt_player(&mut self, amount: f32, src: Option<Vec3>, kb: f32) {
        if self.p.dead || self.g.over || self.g.net.ghost || !self.p.spawned { return; }
        let pp = self.p.body.pos;
        // a dodge that slips a hit at the last moment is perfect: stamina back and a moment of guaranteed crits
        let since_dodge = self.g.run_t - self.g.flags.get("dodge_at").copied().unwrap_or(-9.0);
        if self.p.invuln > 0.0 && since_dodge < 0.3 && self.g.flags.get("perfect_used").copied().unwrap_or(0.0) <= 0.0 {
            self.g.flags.insert("perfect_used", 1.0);
            self.pop(pp + Vec3::Y * 2.2, "PERFECT".into(), 1); self.play("crit"); self.g.hitstop = 0.12;
            self.p.stam = (self.p.stam + 30.0).min(self.g.st.stam_max); self.g.buffs.insert("focus", 1.5);
            self.gfx().ring(pp, 0xbfe8ff, 0.4, 3.0, 0.3); self.fx.burst(pp + Vec3::Y, 0xbfe8ff, 20, 5.0, 0.4, 0.3, 0.0, 0.0);
            return;
        }
        if self.p.invuln > 0.0 || self.g.buff("shield") > 0.0 { self.pop(pp + Vec3::Y * 2.0, "Blocked".into(), 3); return; }
        // blocking turns frontal hits; a block raised just in time is a parry
        let mut amount = amount;
        if self.p.blocking > 0.0 {
            let front = src.map(|s| (s - pp).with_y(0.0).normalize_or_zero().dot(self.p.fwd()) > 0.1).unwrap_or(true);
            if front {
                let f = self.p.fwd();
                if self.p.blocking < 0.22 {
                    self.g.flags.insert("parried", 1.0); self.g.buffs.insert("riposte", 2.5);
                    self.p.stam = (self.p.stam + 15.0).min(self.g.st.stam_max);
                    self.pop(pp + Vec3::Y * 2.2, "PARRY".into(), 1); self.play("snap"); self.play("crit"); self.shake(0.25); self.g.hitstop = 0.1;
                    self.fx.sparks(pp + Vec3::Y * 1.3 + f * 0.6, 0xfff0c0, 18, 10.0); self.gfx().flare(pp + Vec3::Y * 1.3 + f * 0.6, 0xfff0c0, 3.0, 0.25, true);
                    return;
                }
                amount *= 0.35; self.p.stam -= amount * 1.5;
                self.fx.sparks(pp + Vec3::Y * 1.3 + f * 0.6, 0xd8e8ff, 6, 6.0); self.play("hit");
                if self.p.stam <= 0.0 { self.p.stam = 0.0; self.p.blocking = 0.0; self.pop(pp + Vec3::Y * 2.2, "Guard broken".into(), 4); }
            }
        }
        let mut a = amount * (1.0 - self.g.st.armor);
        if self.p.weapon_id() == "swordshield" { a *= 0.7 - 0.1 * self.m("w.swordshield.bulwark") as f32; }
        a = a.round().max(1.0);
        self.p.hp -= a; self.p.hit_t = 0.35; self.g.last_hit = self.g.run_t; self.g.dmg_taken += a;
        self.shake(0.35 + a / 60.0);
        self.play("hurt");
        self.pop(pp + Vec3::Y * 2.0, format!("-{}", a as i32), 4);
        self.fx.burst(pp + Vec3::Y * 1.1, 0xff4a3a, 10, 4.0, 0.4, 0.25, 0.0, 9.0);
        if let Some(s) = src { let mut d = Vec3::new(pp.x - s.x, 0.0, pp.z - s.z); if d.length_squared() > 1e-4 { d = d.normalize() * if kb > 0.0 { kb } else { 5.0 }; self.p.body.vel.x += d.x; self.p.body.vel.z += d.z; self.p.body.vel.y = self.p.body.vel.y.max(3.0); } }
        crate::combat::on_player_hurt(self);
        if self.p.hp <= 0.0 { self.kill_player(); }
    }
    pub fn kill_player(&mut self) {
        // Second Chance from the Camp: rise once at half health
        if self.g.flags.get("lives").copied().unwrap_or(0.0) >= 1.0 {
            *self.g.flags.get_mut("lives").unwrap() -= 1.0;
            self.p.hp = (self.p.max_hp * 0.5).round(); self.p.invuln = 3.0;
            let c = self.p.body.pos; self.play("relic"); self.g.flash = 0.3; self.toast("Second Chance", "You rise again", 2.5);
            { let mut gx = self.gfx(); gx.pillar(c, 0xffd35a, 2.2, 24.0, 1.4); gx.shock(c, 0xffd35a, 0.5, 9.0, 0.6, 3.0); }
            self.aoe(c, 9.0, &mut |cx, i, _| { cx.damage(i, 80.0, Hit { kb: 14.0, up: 6.0, src: Some(c), col: 0xffd35a, ..default() }); cx.status(i, Status { stun: 1.5, ..default() }); });
            return;
        }
        if crate::augments::phoenix(self) { return; }
        self.p.hp = 0.0; self.p.dead = true; self.p.death_t = 0.0; self.p.charging = false;
        if self.g.net.on() { self.play("die"); self.toast("You are down", "A teammate can revive you", 3.0); return; }
        self.play("die"); self.g.hitstop = 0.25;
        let pp = self.p.body.pos + Vec3::Y;
        self.fx.burst(pp, 0xff4a3a, 40, 6.0, 0.9, 0.4, 0.0, 6.0);
        self.g.over = true;
    }
    pub fn gain_xp(&mut self, n: f32) {
        if self.g.net.ghost { return; }
        self.g.xp += n * self.g.st.xp_mul;
        while self.g.xp >= self.g.xp_need() {
            self.g.xp -= self.g.xp_need(); self.g.lvl += 1; self.g.level_pending += 1;
            level_up_blast(self);
        }
    }

    // ---------------- loot ----------------
    pub fn drop_gems(&mut self, pos: Vec3, xp: f32) { let n = ((xp / 9.0).ceil() as usize).clamp(1, 6); for _ in 0..n { self.drop_pick(pos, PickKind::Xp, xp / n as f32); } }
    pub fn drop_pick(&mut self, pos: Vec3, kind: PickKind, v: f32) {
        if self.g.net.ghost || self.g.net.client() { return; }
        if self.g.pickups.len() >= 260 { if kind == PickKind::Xp { if let Some(o) = self.g.pickups.iter_mut().find(|p| p.kind == PickKind::Xp) { o.v += v; return; } } else { return; } }
        let vel = Vec3::new(self.rr(-3.0, 3.0), self.rr(3.0, 6.0), self.rr(-3.0, 3.0));
        let ph = self.rr(0.0, 6.0);
        let e = pickup_model(self, kind, v, pos);
        if kind != PickKind::Xp { let c = match kind { PickKind::Heart => 0xff4a5a, PickKind::Magnet => 0x7fd4ff, PickKind::Bomb => 0xfff0a0, _ => 0xffffff }; if matches!(kind, PickKind::Heart | PickKind::Magnet | PickKind::Bomb) && kind != PickKind::Gold { self.gfx().pillar(pos - Vec3::Y * 0.4, c, 0.4, 4.0, 1.0); } }
        self.g.pickups.push(Pickup { kind, v, pos, vel, t: 0.0, e, vac: false, ph });
    }
}

pub const GEM: [(f32, u32, f32); 3] = [(0.0, 0x5ab8ff, 0.17), (4.0, 0x5aff8a, 0.22), (12.0, 0xff5a7a, 0.3)];
fn pickup_model(cx: &mut Ctx, kind: PickKind, v: f32, pos: Vec3) -> Entity {
    let mut b = cx.builder(); b.shadow = false;
    let root = b.root(pos);
    match kind {
        PickKind::Xp => { let g = if v >= GEM[2].0 { GEM[2] } else if v >= GEM[1].0 { GEM[1] } else { GEM[0] }; let oct = b.pal.octa.clone(); b.mesh(root, oct, g.1, Mk::Glow, Transform::from_scale(Vec3::new(g.2, g.2 * 1.5, g.2))); }
        PickKind::Meat => { b.bx(root, Vec3::new(0.32, 0.18, 0.24), 0xd06a6a, Vec3::ZERO); b.bx(root, Vec3::new(0.08, 0.08, 0.22), 0xf0e8d8, Vec3::new(0.0, 0.0, 0.2)); }
        PickKind::Berries => { for (x, z) in [(-0.08, 0.0), (0.08, 0.04), (0.0, -0.08)] { b.bx(root, Vec3::splat(0.12), 0xd23a4e, Vec3::new(x, 0.0, z)); } }
        PickKind::Heart => { b.bx(root, Vec3::new(0.22, 0.22, 0.16), 0xe8344e, Vec3::new(-0.1, 0.06, 0.0)); b.bx(root, Vec3::new(0.22, 0.22, 0.16), 0xe8344e, Vec3::new(0.1, 0.06, 0.0));
            let t = b.bx(root, Vec3::new(0.26, 0.26, 0.16), 0xe8344e, Vec3::new(0.0, -0.07, 0.0)); b.rot(t, 0.0, 0.0, std::f32::consts::FRAC_PI_4); b.glow(root, Vec3::new(0.06, 0.06, 0.18), 0xff8a9a, Vec3::new(-0.12, 0.12, 0.0)); }
        PickKind::Magnet => { for x in [-0.16, 0.16] { b.bx(root, Vec3::new(0.12, 0.36, 0.12), 0xd8343a, Vec3::new(x, 0.0, 0.0)); b.bx(root, Vec3::new(0.12, 0.1, 0.13), 0xd8dde4, Vec3::new(x, -0.21, 0.0)); } b.bx(root, Vec3::new(0.44, 0.12, 0.12), 0xd8343a, Vec3::new(0.0, 0.18, 0.0)); }
        PickKind::Gold => { for k in 0..(v.min(5.0) as usize).max(1) { b.bx(root, Vec3::new(0.26, 0.06, 0.26), 0xffc83a, Vec3::new(0.0, k as f32 * 0.07, 0.0)); } b.glow(root, Vec3::new(0.14, 0.02, 0.14), 0xfff0a0, Vec3::new(0.0, (v.min(5.0) as usize).max(1) as f32 * 0.07, 0.0)); }
        PickKind::Bomb => { b.bx(root, Vec3::new(0.1, 0.5, 0.1), 0xe0b84a, Vec3::ZERO); b.bx(root, Vec3::new(0.36, 0.1, 0.1), 0xe0b84a, Vec3::new(0.0, 0.1, 0.0)); let g = b.glow(root, Vec3::splat(0.16), 0xfff8d8, Vec3::new(0.0, 0.1, 0.0)); b.rot(g, 0.6, 0.6, 0.0); }
    }
    root
}

pub fn freeze_fx(cx: &mut Ctx, i: usize) {
    let (pos, h, w) = { let e = &cx.g.enemies[i]; (e.body.pos, e.body.h, e.body.w) };
    cx.fx.burst(pos + Vec3::Y * h * 0.5, 0xbff4ff, 16, 4.0, 0.5, 0.3, 0.0, 0.0);
    cx.gfx().debris(pos + Vec3::Y * 0.3, &[0xbff4ff, 0x8fe6ff, 0xffffff], 5, 3.0, 0.14, 4.0);
    // a real block of ice encases the enemy until it thaws or shatters
    if cx.g.enemies[i].ice.is_none() {
        let m = cx.pal.mat(cx.mats, 0xbff4ff, Mk::Ice);
        let e = cx.c.spawn((Mesh3d(cx.pal.cube.clone()), MeshMaterial3d(m), Transform::from_translation(pos + Vec3::Y * h * 0.5).with_scale(Vec3::new(w * 2.6 + 0.3, h + 0.25, w * 2.6 + 0.3)), bevy::pbr::NotShadowCaster)).id();
        cx.g.enemies[i].ice = Some(e);
    }
}

/// levelling up clears breathing room: a bright wave that hurls back and stuns everything close
pub fn level_up_blast(cx: &mut Ctx) {
    cx.play("level");
    let c = cx.p.body.pos;
    let mut g = cx.gfx();
    g.ring(c, 0x79d0ff, 0.5, 8.0, 0.7); g.shock(c, 0x79d0ff, 0.6, 8.0, 0.5, 2.2); g.circle(c, 0x79d0ff, 5.0, 1.2, "arcane", 3.0, None, None);
    g.pillar(c, 0x79d0ff, 1.2, 16.0, 1.2); g.flare(c + Vec3::Y * 1.2, 0xbfe8ff, 6.0, 0.5, true);
    cx.fx.burst(c + Vec3::Y, 0x79d0ff, 60, 8.0, 0.9, 0.4, 2.0, 0.0);
    cx.aoe(c, 6.0, &mut |cx, i, _| { if cx.g.enemies[i].t.passive || cx.g.enemies[i].t.prop { return; } cx.damage(i, 10.0, Hit { kb: 9.0, up: 4.0, src: Some(c), col: 0x79d0ff, quiet: true }); cx.status(i, Status { stun: 0.4, ..default() }); });
}

/// run delayed actions whose time has come
pub fn run_timers(cx: &mut Ctx) {
    let dt = cx.dt;
    let mut due = vec![];
    let mut i = 0;
    while i < cx.g.timers.len() { cx.g.timers[i].t -= dt; if cx.g.timers[i].t <= 0.0 { due.push(cx.g.timers.swap_remove(i)); } else { i += 1; } }
    for t in due { cx.g.net.ghost = t.ghost; (t.f)(cx); cx.g.net.ghost = false; }
}

pub fn update_pickups(cx: &mut Ctx) {
    let dt = cx.dt;
    let pp = cx.p.body.pos;
    let mut k = 0;
    while k < cx.g.pickups.len() {
        let mag = { let p = &cx.g.pickups[k]; (if p.kind == PickKind::Xp { 5.0 } else if matches!(p.kind, PickKind::Heart | PickKind::Magnet | PickKind::Bomb) { 3.0 } else { 2.4 }) * cx.g.st.magnet };
        let p = &mut cx.g.pickups[k];
        p.t += dt;
        let d = (pp + Vec3::Y * 0.9) - p.pos;
        let dist = d.length();
        let mut grounded = false;
        if ((p.vac && p.t > 0.2) || (p.t > 0.45 && dist < mag)) && !cx.p.dead {
            let sp = if p.vac { 16.0 + p.t * 4.0 } else { 7.0 + p.t * 10.0 }; p.vel = d / dist.max(0.01) * sp; p.pos += p.vel * dt;
        } else {
            p.vel.y -= 18.0 * dt; p.vel.x *= 1.0 - dt * 2.0; p.vel.z *= 1.0 - dt * 2.0; p.pos += p.vel * dt;
            if cx.w.peek(p.pos.x.floor() as i32, (p.pos.y - 0.1).floor() as i32, p.pos.z.floor() as i32) != 0 { p.pos.y = (p.pos.y - 0.1).floor() + 1.1; p.vel.y = p.vel.y.abs() * 0.35; p.vel.x *= 0.5; p.vel.z *= 0.5; grounded = true; }
        }
        let bob = if grounded || p.vel.length_squared() < 1.0 { 0.18 + (cx.g.run_t * 3.0 + p.ph).sin() * 0.08 } else { 0.0 };
        let (e, pos, t, kind, v) = (p.e, p.pos, p.t, p.kind, p.v);
        if let Ok(mut tr) = cx.q.get_mut(e) { tr.translation = pos + Vec3::Y * bob; tr.rotation = Quat::from_rotation_y(t * if kind == PickKind::Xp { 2.0 } else { 3.0 }); }
        if kind == PickKind::Xp && cx.r() < 0.03 { let c = if v >= GEM[2].0 { GEM[2].1 } else if v >= GEM[1].0 { GEM[1].1 } else { GEM[0].1 }; cx.fx.emit(true, pos + Vec3::Y * 0.2, Vec3::Y * 0.5, 0.4, 0.28, 0.02, hexc(c) * 2.0, 0.0, 0.0, 0.8); }
        if dist < 1.0 && t > 0.3 && !cx.p.dead {
            cx.c.entity(e).despawn_recursive();
            cx.g.pickups.swap_remove(k);
            collect(cx, kind, v);
            continue;
        }
        if t > if kind == PickKind::Xp { 150.0 } else { 90.0 } { cx.c.entity(e).despawn_recursive(); cx.g.pickups.swap_remove(k); continue; }
        k += 1;
    }
}
fn collect(cx: &mut Ctx, kind: PickKind, v: f32) {
    let pp = cx.p.body.pos;
    match kind {
        PickKind::Xp => { cx.play("gem"); cx.gain_xp(v); }
        PickKind::Gold => { let n = (v * cx.g.flags.get("greed").copied().unwrap_or(1.0)).round() as u32; cx.g.gold += n; cx.play("gem"); cx.pop(pp + Vec3::Y * 2.0, format!("+{} gold", n), 3); }
        PickKind::Meat => { cx.g.food_meat += 1; cx.play("pick"); cx.pop(pp + Vec3::Y * 2.0, "+ Meat".into(), 2); }
        PickKind::Berries => { cx.g.food_berries += 1; cx.play("pick"); cx.pop(pp + Vec3::Y * 2.0, "+ Berries".into(), 2); }
        PickKind::Heart => { let a = cx.p.max_hp * 0.25; cx.heal(a, false); cx.play("heal"); cx.toast("Heart", "Restored health", 1.6); cx.gfx().pillar(pp, 0x8dff9e, 1.1, 6.0, 0.9); }
        PickKind::Magnet => { cx.play("relic"); for p in cx.g.pickups.iter_mut() { if p.kind == PickKind::Xp { p.vac = true; } } cx.gfx().ring(pp, 0x7fd4ff, 0.5, 14.0, 0.6); cx.toast("Lodestone", "Every gem flies to you", 1.6); }
        PickKind::Bomb => {
            cx.play("bomb"); cx.g.flash = 0.22; cx.shake(1.0); cx.toast("Holy Relic", "Holy light scours the horde", 1.6);
            let mut g = cx.gfx(); g.ring(pp, 0xfff0a0, 0.5, 30.0, 0.9); g.shock(pp, 0xfff0a0, 1.0, 30.0, 0.8, 3.0);
            let dm = 120.0 * (1.0 + cx.g.minutes() * 0.12);
            let foes = cx.foes(pp, 30.0);
            for (n, i) in foes.into_iter().enumerate() {
                let ep = cx.g.enemies[i].body.pos;
                if n < 14 { cx.gfx().pillar(ep, 0xfff0a0, 0.45, 7.0, 0.45); } else { cx.fx.sparks(ep + Vec3::Y, 0xfff0a0, 4, 6.0); }
                cx.damage(i, dm, Hit { kb: 6.0, up: 4.0, src: Some(pp), col: 0xfff0a0, quiet: true });
            }
        }
    }
}
pub fn rare_drop(cx: &mut Ctx, c: Vec3, elite: bool) {
    let r = cx.r();
    let hearts = cx.g.ember() < 8;
    if elite { let g = (4.0 + cx.g.minutes() * 0.5).round(); cx.drop_pick(c, PickKind::Gold, g); if r < 0.5 && hearts { cx.drop_pick(c, PickKind::Heart, 0.0) } else if r < 0.7 { cx.drop_pick(c, PickKind::Magnet, 0.0) } return; }
    if r < 0.004 { cx.drop_pick(c, PickKind::Magnet, 0.0) } else if r < 0.011 && hearts { cx.drop_pick(c, PickKind::Heart, 0.0) } else if r < 0.014 { cx.drop_pick(c, PickKind::Bomb, 0.0) } else if r < 0.04 { cx.drop_pick(c, PickKind::Gold, 1.0) }
}
pub fn urn_loot(cx: &mut Ctx, c: Vec3) {
    let r = cx.r(); let m = cx.g.minutes();
    cx.drop_pick(c, PickKind::Gold, 3.0);
    if r < 0.3 && cx.g.ember() < 8 { cx.drop_pick(c, PickKind::Heart, 0.0) } else if r < 0.45 { cx.drop_pick(c, PickKind::Magnet, 0.0) } else if r < 0.6 { cx.drop_pick(c, PickKind::Bomb, 0.0) } else if r < 0.75 { cx.drop_pick(c, PickKind::Meat, 0.0) } else { cx.drop_gems(c, (5.0 + m * 2.5).round()) }
    if cx.r() < 0.5 { cx.drop_gems(c, (2.0 + m).round()); }
}
pub fn ang(a: f32) -> Vec3 { Vec3::new(a.cos(), 0.0, a.sin()) }
pub fn tau() -> f32 { TAU }
