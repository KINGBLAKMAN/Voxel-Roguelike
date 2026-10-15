//! Co-op for up to four players over TCP (port 7777). The host is the authority for the horde: it runs the
//! director, enemy AI, bosses and events, and streams compact enemy snapshots. Every machine runs its own player,
//! abilities and relics; clients send the hits they land to the host. Anything that can hurt a player is replayed
//! on every machine and each player takes their own damage, so blocking, parrying and dodging feel the same
//! online. Teammates' casts are replayed as real visuals with their damage switched off.
use crate::enemies::*;
use crate::game::*;
use crate::models::*;
use crate::player::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub const PORT: u16 = 7777;

#[derive(Serialize, Deserialize, Clone, Copy, Default)]
pub struct PState { pub id: u32, pub cls: u8, pub pos: [f32; 3], pub yaw: f32, pub pitch: f32, pub hp: f32, pub max_hp: f32, pub dead: bool, pub in_run: bool,
    pub blocking: bool, pub charging: bool, pub charge: f32, pub widx: u8, pub roll: f32, pub whirl: f32 }
#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct NEnemy { pub id: u32, pub kind: u8, pub affix: u8, pub boss: bool, pub size: f32, pub color: u32, pub pos: [f32; 3], pub yaw: f32, pub hp: f32, pub frozen: bool, pub sink: f32, pub wind: bool }

#[derive(Serialize, Deserialize, Clone)]
pub enum Msg {
    Hello,
    Welcome { id: u32, seed: u32, map: u8, stage: u32 },
    State(PState),
    Leave { id: u32 },
    Snap { t: f32, day: f32, ens: Vec<NEnemy> },
    Die { id: u32, killer: u32, xp: f32 },
    Hit { id: u32, dmg: f32, kb: f32, up: f32, src: [f32; 3], from: u32 },
    Status { id: u32, stun: f32, slow: f32, freeze: f32, burn: f32, poison: f32, fear: f32, chill: u8, thr: u8, push: Option<[f32; 3]> },
    Hurt { to: u32, dmg: f32, src: [f32; 3], kb: f32 },
    /// an enemy attack effect to replay everywhere: k selects which, a/b are positions, f are numbers
    Fx { k: u8, a: [f32; 3], b: [f32; 3], f: [f32; 4] },
    Cast { id: u32, ab: String, aim: [f32; 3] },
    Fire { id: u32, charge: f32, aim: [f32; 3] },
    Block { x: i32, y: i32, z: i32, b: u8 },
    Toast { a: String, b: String },
    Revive { to: u32 },
    Victory,
    /// a stage boss fell: the portal to the next map opened here
    Portal { a: [f32; 3], col: u32 },
    /// everyone moves to the next map
    Stage { stage: u32, kind: u8, seed: u32 },
    GameOver,
}

/// the socket side: background threads move bytes, the game reads `inbox` and writes through `send`
#[derive(Resource, Clone)]
pub struct Link { pub inbox: Arc<Mutex<Vec<(u32, Msg)>>>, peers: Arc<Mutex<Vec<(u32, Sender<Arc<Vec<u8>>>)>>>, next: Arc<Mutex<u32>>, pub host: bool, pub status: Arc<Mutex<String>> }
fn frame(m: &Msg) -> Arc<Vec<u8>> { let body = bincode::serialize(m).unwrap_or_default(); let mut v = (body.len() as u32).to_le_bytes().to_vec(); v.extend(body); Arc::new(v) }
fn spawn_io(stream: TcpStream, id: u32, inbox: Arc<Mutex<Vec<(u32, Msg)>>>, peers: Arc<Mutex<Vec<(u32, Sender<Arc<Vec<u8>>>)>>>) {
    let _ = stream.set_nodelay(true);
    let (tx, rx) = channel::<Arc<Vec<u8>>>();
    peers.lock().unwrap().push((id, tx));
    let mut w = match stream.try_clone() { Ok(s) => s, Err(_) => return };
    std::thread::spawn(move || { while let Ok(b) = rx.recv() { if w.write_all(&b).is_err() { break; } } });
    let mut r = stream;
    std::thread::spawn(move || {
        let mut len = [0u8; 4];
        loop {
            if r.read_exact(&mut len).is_err() { break; }
            let n = u32::from_le_bytes(len) as usize; if n > 8 << 20 { break; }
            let mut buf = vec![0u8; n]; if r.read_exact(&mut buf).is_err() { break; }
            if let Ok(m) = bincode::deserialize::<Msg>(&buf) { inbox.lock().unwrap().push((id, m)); }
        }
        inbox.lock().unwrap().push((id, Msg::Leave { id }));
        peers.lock().unwrap().retain(|p| p.0 != id);
    });
}
impl Link {
    pub fn host() -> Result<Link, String> {
        let l = TcpListener::bind(("0.0.0.0", PORT)).map_err(|e| e.to_string())?;
        let link = Link { inbox: Arc::default(), peers: Arc::default(), next: Arc::new(Mutex::new(1)), host: true, status: Arc::new(Mutex::new(format!("Hosting on {}:{}", local_ip(), PORT))) };
        let (inbox, peers, next) = (link.inbox.clone(), link.peers.clone(), link.next.clone());
        std::thread::spawn(move || {
            for s in l.incoming().flatten() {
                if peers.lock().unwrap().len() >= 3 { continue; }
                let id = { let mut n = next.lock().unwrap(); *n += 1; *n };
                spawn_io(s, id, inbox.clone(), peers.clone());
                inbox.lock().unwrap().push((id, Msg::Hello));
            }
        });
        Ok(link)
    }
    pub fn join(addr: &str) -> Result<Link, String> {
        let a = if addr.contains(':') { addr.to_string() } else { format!("{}:{}", addr, PORT) };
        let sa = a.to_socket_addrs().map_err(|e| e.to_string())?.next().ok_or("bad address")?;
        let s = TcpStream::connect_timeout(&sa, Duration::from_secs(5)).map_err(|e| e.to_string())?;
        let link = Link { inbox: Arc::default(), peers: Arc::default(), next: Arc::new(Mutex::new(0)), host: false, status: Arc::new(Mutex::new(format!("Connected to {}", a))) };
        spawn_io(s, 0, link.inbox.clone(), link.peers.clone());
        Ok(link)
    }
    pub fn send(&self, to: Option<u32>, m: &Msg) { let f = frame(m); for (id, tx) in self.peers.lock().unwrap().iter() { if to.map(|t| t == *id).unwrap_or(true) { let _ = tx.send(f.clone()); } } }
    pub fn send_except(&self, skip: u32, m: &Msg) { let f = frame(m); for (id, tx) in self.peers.lock().unwrap().iter() { if *id != skip { let _ = tx.send(f.clone()); } } }
    pub fn peer_count(&self) -> usize { self.peers.lock().unwrap().len() }
    pub fn peer_ids(&self) -> Vec<u32> { self.peers.lock().unwrap().iter().map(|p| p.0).collect() }
}
pub fn local_ip() -> String {
    std::net::UdpSocket::bind("0.0.0.0:0").and_then(|s| { s.connect("8.8.8.8:80")?; s.local_addr() }).map(|a| a.ip().to_string()).unwrap_or_else(|_| "your-ip".into())
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum Mode { #[default] Off, Host, Client }
pub struct Remote { pub id: u32, pub p: Player, pub target: Vec3, pub state: PState, pub revive: f32 }
#[derive(Default)]
pub struct Net {
    pub mode: Mode, pub my_id: u32, pub out: Vec<(Option<u32>, Msg)>, pub remotes: Vec<Remote>, pub ghost: bool, pub no_enemy_dmg: bool,
    pub snap_t: f32, pub state_t: f32, pub last_hitter: std::collections::HashMap<u32, u32>, pub day: Option<f32>, pub seed: Option<u32>, pub join_map: Option<(crate::world::MapKind, u32)>, pub revive_t: f32,
}
impl Net {
    pub fn on(&self) -> bool { self.mode != Mode::Off }
    pub fn host(&self) -> bool { self.mode == Mode::Host }
    pub fn client(&self) -> bool { self.mode == Mode::Client }
    pub fn send(&mut self, m: Msg) { if self.on() { self.out.push((None, m)); } }
}
fn v3(a: [f32; 3]) -> Vec3 { Vec3::from_array(a) }
pub fn cls_u8(c: Class) -> u8 { match c { Class::Ranger => 0, Class::Mage => 1, Class::Warrior => 2, Class::Priest => 3 } }
pub fn u8_cls(c: u8) -> Class { [Class::Ranger, Class::Mage, Class::Warrior, Class::Priest][c as usize % 4] }
pub fn kind_u8(k: Kind) -> u8 { ALL_KINDS.iter().position(|q| *q == k).unwrap_or(0) as u8 }

/// alive teammates as (id, position)
pub fn targets(g: &Game) -> Vec<(u32, Vec3)> { g.net.remotes.iter().filter(|r| r.state.in_run && !r.state.dead).map(|r| (r.id, r.p.body.pos)).collect() }

// ---------------- effects replayed on every machine ----------------
/// a telegraph circle
pub fn tele(cx: &mut Ctx, pos: Vec3, col: u32, r: f32, life: f32, style: u8) {
    let st = ["fire", "war", "frost", "nature", "blood", "holy", "arcane"][style as usize % 7];
    cx.gfx().circle(pos, col, r, life, st, 3.0, None, None);
    if cx.g.net.host() { cx.g.net.send(Msg::Fx { k: 0, a: pos.to_array(), b: [0.0; 3], f: [r, life, col as f32, style as f32] }); }
}
/// a ground slam that hurts whoever stands in it
pub fn slam(cx: &mut Ctx, c: Vec3, r: f32, dmg: f32, kb: f32) {
    slam_fx(cx, c, r);
    if !cx.p.dead && Vec2::new(cx.p.body.pos.x - c.x, cx.p.body.pos.z - c.z).length() < r && (cx.p.body.pos.y - c.y).abs() < 2.6 { cx.hurt_player(dmg, Some(c), kb); }
    if cx.g.net.host() { cx.g.net.send(Msg::Fx { k: 1, a: c.to_array(), b: [0.0; 3], f: [r, dmg, kb, 0.0] }); }
}
pub fn fx_streak(cx: &mut Ctx, a: Vec3, b: Vec3, col: u32, life: f32, w: f32) {
    cx.gfx().streak(a, b, col, life, w);
    if cx.g.net.host() { cx.g.net.send(Msg::Fx { k: 2, a: a.to_array(), b: b.to_array(), f: [col as f32, life, w, 0.0] }); }
}
/// a sweeping blow in front of an attacker: hurts anyone inside the arc
pub fn cleave(cx: &mut Ctx, c: Vec3, dir: Vec3, r: f32, dmg: f32, kb: f32) {
    let yaw = dir.x.atan2(dir.z); cx.gfx().swoosh(c + Vec3::Y * 2.0, yaw, r, 2.4, 0xff7a2a, 0.3, 0.0, 1.0); cx.play("heavy");
    let to = (cx.p.body.pos - c).with_y(0.0);
    if !cx.p.dead && to.length() < r && to.normalize_or_zero().dot(dir) > 0.3 { cx.hurt_player(dmg, Some(c), kb); }
    if cx.g.net.host() { cx.g.net.send(Msg::Fx { k: 3, a: c.to_array(), b: dir.to_array(), f: [r, dmg, kb, 0.0] }); }
}
fn replay_fx(cx: &mut Ctx, k: u8, a: Vec3, b: Vec3, f: [f32; 4]) {
    cx.g.net.no_enemy_dmg = true;
    match k {
        0 => tele(cx, a, f[2] as u32, f[0], f[1], f[3] as u8),
        1 => slam(cx, a, f[0], f[1], f[2]),
        2 => fx_streak(cx, a, b, f[0] as u32, f[1], f[2]),
        3 => cleave(cx, a, b, f[0], f[1], f[2]),
        4 => crate::director::eruption(cx, a, f[0], f[1]),
        5 => crate::combat::fire_enemy_proj(cx, [crate::combat::EProj::Arrow, crate::combat::EProj::Dark, crate::combat::EProj::Rock][f[1] as usize % 3], a, b, f[0]),
        6 => crate::horde::enemy_ball(cx, a, b, f[0], f[1] as u32, f[2], f[3] > 0.0),
        7 => crate::horde::bomb(cx, a, b, f[0]),
        8 => crate::horde::hot_ground(cx, a, f[0], f[1], f[2]),
        9 => crate::horde::spikes_fx(cx, a, f[0]),
        _ => {}
    }
    cx.g.net.no_enemy_dmg = false;
}

// ---------------- per-frame plumbing ----------------
/// apply everything that arrived since last frame
pub fn process(cx: &mut Ctx, link: &Link) {
    let msgs: Vec<(u32, Msg)> = std::mem::take(&mut *link.inbox.lock().unwrap());
    for (from, m) in msgs {
        // a player's state is whoever's connection it came in on
        let m = match m { Msg::State(mut st) if link.host => { st.id = from; Msg::State(st) } m => m };
        // the host relays player-to-player traffic
        if link.host { match &m { Msg::State(_) | Msg::Cast { .. } | Msg::Fire { .. } | Msg::Block { .. } => link.send_except(from, &m), Msg::Revive { to } => link.send(Some(*to), &m), _ => {} } }
        match m {
            Msg::Hello => { if link.host { let seed = cx.w.seed; link.send(Some(from), &Msg::Welcome { id: from, seed, map: cx.w.kind.id(), stage: cx.g.stage }); cx.toast("A friend joined", "", 2.0); } }
            Msg::Welcome { id, seed, map, stage } => { cx.g.net.my_id = id; cx.g.net.seed = Some(seed); cx.g.net.join_map = Some((crate::world::MapKind::from_id(map), stage)); }
            Msg::Portal { a, col } => if cx.g.net.client() { crate::director::open_portal(cx, v3(a), col); },
            Msg::Stage { stage, kind, seed } => if cx.g.net.client() { cx.g.pending_stage = Some((stage, crate::world::MapKind::from_id(kind), seed)); },
            Msg::Leave { id } => { if let Some(k) = cx.g.net.remotes.iter().position(|r| r.id == id) { let r = cx.g.net.remotes.remove(k); if let Some(e) = r.p.rig.root { cx.c.entity(e).despawn_recursive(); } cx.toast("A friend left", "", 2.0); }
                if !link.host { *link.status.lock().unwrap() = "Disconnected from host".into(); } }
            Msg::State(s) => state_in(cx, s),
            Msg::Snap { t, day, ens } => if cx.g.net.client() { cx.g.run_t = t; cx.g.net.day = Some(day); snap_in(cx, &ens); },
            Msg::Die { id, killer, xp } => if cx.g.net.client() {
                if xp > 0.0 && !cx.p.dead { cx.gain_xp(xp); }
                if let Some(i) = cx.g.enemies.iter().position(|e| e.id == id && !e.dead) { let mine = killer == cx.g.net.my_id; let c = cx.g.enemies[i].body.pos; kill_enemy(cx, i, true, false); if mine { crate::augments::on_kill(cx, c); } }
            },
            Msg::Hit { id, dmg, kb, up, src, from: who } => if link.host { if let Some(i) = cx.find(id) { cx.g.net.last_hitter.insert(id, who); cx.apply_damage(i, dmg, Hit { kb, up, src: Some(v3(src)), ..default() }); } },
            Msg::Status { id, stun, slow, freeze, burn, poison, fear, chill, thr, push } => if link.host { if let Some(i) = cx.find(id) { cx.status(i, Status { stun, slow, freeze, burn, poison, fear, chill, thr, push: push.map(v3) }); } },
            Msg::Hurt { to, dmg, src, kb } => if to == cx.g.net.my_id { cx.hurt_player(dmg, Some(v3(src)), kb); },
            Msg::Fx { k, a, b, f } => if cx.g.net.client() && cx.p.spawned { replay_fx(cx, k, v3(a), v3(b), f); },
            Msg::Cast { id, ab, aim } => replay(cx, id, |cx| { cx.p.aim_pt = v3(aim); crate::abilities::cast_remote(cx, &ab); }),
            Msg::Fire { id, charge, aim } => replay(cx, id, |cx| { cx.p.aim_pt = v3(aim); crate::combat::replay_primary(cx, charge); }),
            Msg::Block { x, y, z, b } => { cx.w.set(x, y, z, b); }
            Msg::Toast { a, b } => cx.toast(&a, &b, 3.0),
            Msg::Revive { to } => if to == cx.g.net.my_id && cx.p.dead { cx.p.dead = false; cx.p.hp = (cx.p.max_hp * 0.4).round(); cx.p.invuln = 2.0; cx.toast("Revived", "Back on your feet", 2.0); cx.play("heal"); },
            Msg::Victory => { cx.g.flags.insert("victory", cx.g.run_t); }
            Msg::GameOver => { cx.g.over = true; }
        }
    }
}
/// play a teammate's cast or attack: the real effects, with their damage switched off
fn replay(cx: &mut Ctx, id: u32, f: impl FnOnce(&mut Ctx)) {
    let Some(k) = cx.g.net.remotes.iter().position(|r| r.id == id) else { return };
    if !cx.p.spawned { return; }
    let mut rp = std::mem::take(&mut cx.g.net.remotes[k].p);
    std::mem::swap(cx.p, &mut rp);
    cx.g.net.ghost = true;
    f(cx);
    cx.g.net.ghost = false;
    std::mem::swap(cx.p, &mut rp);
    cx.g.net.remotes[k].p = rp;
    // ability state machines belong to the local player
    if let Some(ch) = cx.g.ab.charge.take() { cx.c.entity(ch.wedge()).despawn_recursive(); }
    cx.g.ab.roll = None; cx.g.ab.slam = None;
}
fn state_in(cx: &mut Ctx, s: PState) {
    if s.id == cx.g.net.my_id { return; }
    let k = match cx.g.net.remotes.iter().position(|r| r.id == s.id) { Some(k) => k, None => { cx.g.net.remotes.push(Remote { id: s.id, p: Player::default(), target: v3(s.pos), state: s, revive: 0.0 }); cx.g.net.remotes.len() - 1 } };
    let cls = u8_cls(s.cls);
    let need_build = s.in_run && (cx.g.net.remotes[k].p.rig.root.is_none() || cx.g.net.remotes[k].p.cls != cls);
    if !s.in_run { if let Some(e) = cx.g.net.remotes[k].p.rig.root.take() { cx.c.entity(e).despawn_recursive(); } }
    let r = &mut cx.g.net.remotes[k];
    r.state = s; r.target = v3(s.pos);
    if need_build {
        let mut p = std::mem::take(&mut r.p);
        if let Some(e) = p.rig.root.take() { cx.c.entity(e).despawn_recursive(); }
        p.cls = cls; p.body.pos = v3(s.pos); p.vis_y = s.pos[1]; p.widx = s.widx as usize; p.spawned = true;
        build_player(cx.c, cx.pal, cx.mats, &mut p);
        cx.g.net.remotes[k].p = p;
    }
    let r = &mut cx.g.net.remotes[k];
    if r.p.widx != s.widx as usize && r.p.rig.root.is_some() { r.p.widx = s.widx as usize; let mut p = std::mem::take(&mut r.p); attach_weapon(cx.c, cx.pal, cx.mats, &mut p); cx.g.net.remotes[k].p = p; }
}
fn snap_in(cx: &mut Ctx, ens: &[NEnemy]) {
    // drop puppets the host no longer has (despawned rather than killed)
    let ids: std::collections::HashSet<u32> = ens.iter().map(|e| e.id).collect();
    let mut i = 0;
    while i < cx.g.enemies.len() { if !ids.contains(&cx.g.enemies[i].id) && !cx.g.enemies[i].dead { let e = cx.g.enemies.swap_remove(i); if let Some(r) = e.rig.root { cx.c.entity(r).despawn_recursive(); } if let Some(ic) = e.ice { cx.c.entity(ic).despawn_recursive(); } } else { i += 1; } }
    cx.g.boss = None;
    for n in ens {
        let pos = v3(n.pos);
        let i = match cx.g.enemies.iter().position(|e| e.id == n.id) {
            Some(i) => i,
            None => {
                let k = ALL_KINDS[n.kind as usize % ALL_KINDS.len()];
                let affix = if n.affix > 0 { Some(AFFIXES[(n.affix - 1) as usize % AFFIXES.len()]) } else { None };
                match spawn_enemy(cx, k, pos, SpawnOpts { id: Some(n.id), boss: n.boss, affix, size: n.size, color: n.color, no_rise: true, force: true, ..default() }) { Some(i) => i, None => continue }
            }
        };
        let (was_frozen, max) = (cx.g.enemies[i].frozen > 0.0, cx.g.enemies[i].max_hp);
        { let e = &mut cx.g.enemies[i]; e.wdir = pos; e.yaw = n.yaw; e.hp = n.hp * max; e.frozen = if n.frozen { 0.5 } else { 0.0 }; e.ai.sink = n.sink; e.ai.hidden = n.sink > 0.5; e.wind = if n.wind { 0.3 } else { 0.0 }; }
        if n.boss { cx.g.boss = Some(n.id); }
        if n.frozen && !was_frozen { freeze_fx(cx, i); }
    }
    if cx.g.boss.is_some() { cx.g.dir.next_boss = 1; }
}
/// clients move puppets toward the host's positions and animate them
pub fn puppets(cx: &mut Ctx) {
    let dt = cx.dt;
    for i in 0..cx.g.enemies.len() {
        if cx.g.enemies[i].dead { continue; }
        let (cur, tgt) = (cx.g.enemies[i].body.pos, cx.g.enemies[i].wdir);
        let d = tgt - cur;
        let np = if d.length() > 8.0 { tgt } else { cur + d * (dt * 12.0).min(1.0) };
        let moving = d.with_y(0.0).length() > 0.05;
        { let e = &mut cx.g.enemies[i]; e.body.pos = np; e.flash -= dt; if e.hit_p > 0.0 { e.hit_p = (e.hit_p - dt).max(0.0); } }
        visual(cx, i, moving);
    }
    let mut k = 0;
    while k < cx.g.enemies.len() { if cx.g.enemies[k].dead { let e = cx.g.enemies.swap_remove(k); if let Some(r) = e.rig.root { cx.c.entity(r).despawn_recursive(); } } else { k += 1; } }
}
/// teammates: move toward their reported positions and animate them like the local player
pub fn remotes(cx: &mut Ctx) {
    let dt = cx.dt;
    for k in 0..cx.g.net.remotes.len() {
        if cx.g.net.remotes[k].p.rig.root.is_none() { continue; }
        let mut p = std::mem::take(&mut cx.g.net.remotes[k].p);
        let (s, tgt) = (cx.g.net.remotes[k].state, cx.g.net.remotes[k].target);
        let d = tgt - p.body.pos;
        let prev = p.body.pos;
        p.body.pos = if d.length() > 6.0 { tgt } else { p.body.pos + d * (dt * 14.0).min(1.0) };
        p.body.vel = (p.body.pos - prev) / dt.max(1e-3); p.body.on_ground = true;
        p.vis_y = p.body.pos.y; p.yaw = s.yaw; p.pitch = s.pitch; p.dead = s.dead; p.hp = s.hp; p.max_hp = s.max_hp;
        p.blocking = if s.blocking { 0.5 } else { 0.0 }; p.charging = s.charging; p.charge = s.charge;
        if s.roll > 0.0 && p.roll_t <= 0.0 { p.roll_t = 0.3; }
        if s.whirl > 0.0 { p.whirl = p.whirl.max(0.1); }
        if s.dead && !p.dead { p.death_t = 0.0; }
        animate_player(&mut p, cx.q, dt);
        cx.g.net.remotes[k].p = p;
    }
}
/// send our player state and, on the host, enemy snapshots; flush the outbox
pub fn flush(cx: &mut Ctx, link: &Link, raw_dt: f32) {
    cx.g.net.state_t -= raw_dt;
    if cx.g.net.state_t <= 0.0 && (link.host || cx.g.net.my_id != 0) {
        cx.g.net.state_t = 0.05;
        let p = &cx.p;
        let s = PState { id: cx.g.net.my_id, cls: cls_u8(p.cls), pos: p.body.pos.to_array(), yaw: p.yaw, pitch: p.pitch, hp: p.hp, max_hp: p.max_hp, dead: p.dead, in_run: p.spawned,
            blocking: p.blocking > 0.0, charging: p.charging, charge: p.charge, widx: p.widx as u8, roll: p.roll_t, whirl: p.whirl };
        link.send(None, &Msg::State(s));
    }
    if link.host {
        cx.g.net.snap_t -= raw_dt;
        if cx.g.net.snap_t <= 0.0 && link.peer_count() > 0 {
            cx.g.net.snap_t = 1.0 / 12.0;
            let ens: Vec<NEnemy> = cx.g.enemies.iter().filter(|e| !e.dead).map(|e| NEnemy { id: e.id, kind: kind_u8(e.kind), affix: e.affix.map(|a| AFFIXES.iter().position(|q| *q == a).unwrap_or(0) as u8 + 1).unwrap_or(0),
                boss: e.boss, size: e.size, color: e.color, pos: e.body.pos.to_array(), yaw: e.yaw, hp: (e.hp / e.max_hp).max(0.0), frozen: e.frozen > 0.0, sink: e.ai.sink, wind: e.wind > 0.0 || e.ai.st == 1 }).collect();
            let day = cx.g.net.day.unwrap_or(0.0);
            link.send(None, &Msg::Snap { t: cx.g.run_t, day, ens });
        }
        // the run ends when everyone is down
        if cx.p.spawned && cx.p.dead && !cx.g.over && targets(cx.g).is_empty() { cx.g.over = true; link.send(None, &Msg::GameOver); }
    }
    for (to, m) in std::mem::take(&mut cx.g.net.out) { link.send(to, &m); }
}
/// hold E next to a downed teammate to bring them back
pub fn revive_prompt(cx: &mut Ctx, e_held: bool) -> Option<String> {
    if cx.p.dead { return Some("You are down. A teammate can revive you".into()); }
    let pp = cx.p.body.pos;
    let k = cx.g.net.remotes.iter().position(|r| r.state.in_run && r.state.dead && r.p.body.pos.distance(pp) < 3.0)?;
    if e_held {
        cx.g.net.revive_t += cx.dt;
        let p = cx.g.net.remotes[k].p.body.pos;
        if cx.r() < 0.5 { cx.fx.emit(true, p + Vec3::new(cx.rr(-0.5, 0.5), cx.rr(0.0, 1.0), cx.rr(-0.5, 0.5)), Vec3::Y * 2.0, 0.6, 0.35, 0.05, crate::fx::hexc(0x8dff9e) * 2.5, 0.0, 0.0, 1.0); }
        if cx.g.net.revive_t >= 2.0 { cx.g.net.revive_t = 0.0; let to = cx.g.net.remotes[k].id; cx.g.net.out.push((None, Msg::Revive { to })); cx.play("heal"); cx.gfx().pillar(p, 0x8dff9e, 1.1, 6.0, 0.9); }
    } else { cx.g.net.revive_t = 0.0; }
    Some(format!("Hold E to revive your teammate  {:.0}%", (cx.g.net.revive_t / 2.0 * 100.0).min(100.0)))
}
