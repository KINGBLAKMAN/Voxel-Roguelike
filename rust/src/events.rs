//! Things that happen during a run besides the horde: timed challenges, cursed shrines that trade danger for
//! loot, totems to defend, treasure goblins to chase down, and a wandering merchant who takes run gold.
use crate::director::ring_spot;
use crate::enemies::*;
use crate::fx::hexc;
use crate::game::*;
use crate::models::*;
use crate::player::Player;
use bevy::prelude::*;
use std::f32::consts::TAU;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EvKind { Cursed, Totem, Merchant }
pub struct Ev { pub kind: EvKind, pub pos: Vec3, pub root: Entity, pub life: f32, pub active: f32, pub used: bool, pub hp: f32 }
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ChKind { Slay, Untouched }
pub struct Challenge { pub kind: ChKind, pub target: u32, pub base: u32, pub t: f32, pub start: f32 }
#[derive(Clone)]
pub struct Offer { pub name: &'static str, pub desc: &'static str, pub cost: u32, pub sold: bool }

pub struct EventState { pub list: Vec<Ev>, pub next: f32, pub idx: u32, pub challenge: Option<Challenge>, pub merchant_t: f32, pub shop: Vec<Offer>, pub goblin: Option<(u32, f32)> }
impl Default for EventState { fn default() -> Self { EventState { list: vec![], next: 150.0, idx: 0, challenge: None, merchant_t: 210.0, shop: vec![], goblin: None } } }

fn model(cx: &mut Ctx, kind: EvKind, pos: Vec3) -> Entity {
    let mut b = cx.builder();
    let root = b.root(pos);
    let v = Vec3::new;
    match kind {
        EvKind::Cursed => {
            b.bx(root, v(1.4, 0.3, 1.4), 0x1a1420, v(0.0, 0.15, 0.0));
            b.bx(root, v(0.8, 2.8, 0.8), 0x14101a, v(0.0, 1.7, 0.0));
            for y in [0.9, 1.6, 2.3] { b.glow(root, v(0.84, 0.1, 0.3), 0xb04aff, v(0.0, y, 0.0)); b.glow(root, v(0.3, 0.1, 0.84), 0xb04aff, v(0.0, y + 0.3, 0.0)); }
            b.bx(root, v(0.5, 0.45, 0.5), 0xe8e4d8, v(0.0, 3.35, 0.0));
            b.glow(root, v(0.1, 0.08, 0.02), 0xd07aff, v(-0.1, 3.4, 0.26)); b.glow(root, v(0.1, 0.08, 0.02), 0xd07aff, v(0.1, 3.4, 0.26));
        }
        EvKind::Totem => {
            b.bx(root, v(1.2, 0.3, 1.2), 0x5a3a22, v(0.0, 0.15, 0.0));
            for (k, c) in [0x8a5b34, 0x7a4e2a, 0x9a6a3a].iter().enumerate() {
                let y = 0.8 + k as f32 * 0.95;
                b.bx(root, v(0.75, 0.9, 0.75), *c, v(0.0, y, 0.0));
                b.glow(root, v(0.14, 0.1, 0.02), 0xffd35a, v(-0.16, y + 0.15, 0.38)); b.glow(root, v(0.14, 0.1, 0.02), 0xffd35a, v(0.16, y + 0.15, 0.38));
                b.bx(root, v(0.4, 0.1, 0.06), 0x2a1a10, v(0.0, y - 0.2, 0.39));
            }
            for s in [-1.0f32, 1.0] { b.bx(root, v(0.9, 0.18, 0.2), 0x9a6a3a, v(s * 0.7, 3.2, 0.0)); }
            b.glow(root, v(0.3, 0.3, 0.3), 0xffd35a, v(0.0, 3.45, 0.0));
        }
        EvKind::Merchant => {
            // a covered cart, its keeper and a lantern
            b.bx(root, v(1.6, 0.7, 2.4), 0x7a4e2a, v(0.0, 0.85, 0.0));
            for (x, z) in [(-0.85, 0.7), (0.85, 0.7), (-0.85, -0.7), (0.85, -0.7)] { b.bx(root, v(0.12, 0.7, 0.7), 0x3a2618, v(x, 0.35, z)); }
            b.bx(root, v(1.7, 0.9, 2.2), 0xe8dcc0, v(0.0, 1.7, 0.0)); b.bx(root, v(1.75, 0.1, 2.25), 0xb03a2a, v(0.0, 2.1, 0.0));
            b.bx(root, v(0.1, 0.1, 1.2), 0x5a3a22, v(0.0, 0.6, 1.8));
            for (x, c) in [(-0.4, 0xffd35a), (0.0, 0x7fd4ff), (0.4, 0xff6a3a)] { b.glow(root, v(0.18, 0.22, 0.18), c, v(x, 1.32, 1.05)); }
            let npc = humanoid(&mut b, pos + v(1.5, 0.0, 0.6), &Look { skin: 0xd9a47a, shirt: 0x6a4a8a, pants: 0x3a2a4a, robe: Some(0x5a3a7a), belt: Some(0xe0b84a), ..default() });
            let h = npc.head.unwrap();
            b.bx(h, v(0.7, 0.08, 0.7), 0x3a2a4a, v(0.0, 0.46, 0.0)); b.bx(h, v(0.4, 0.3, 0.4), 0x3a2a4a, v(0.0, 0.62, 0.0));
            b.bx(h, v(0.3, 0.16, 0.06), 0xd8d0c0, v(0.0, 0.08, 0.24));
            let lan = b.pivot(npc.hand_l.unwrap(), v(0.0, -0.1, 0.1));
            b.bx(lan, v(0.2, 0.28, 0.2), 0x3a2a1a, v(0.0, -0.18, 0.0)); b.glow(lan, v(0.14, 0.18, 0.14), 0xffc060, v(0.0, -0.18, 0.0));
            if let Some(r) = npc.root { b.c.entity(root).add_child(r); b.c.entity(r).insert(Transform::from_xyz(1.5, 0.0, 0.6)); }
        }
    }
    root
}

fn spawn_event(cx: &mut Ctx, kind: EvKind) {
    let (pp, yaw) = (cx.p.body.pos, cx.p.yaw);
    let Some(s) = ring_spot(cx, pp, yaw, 10.0, 16.0) else { return };
    let root = model(cx, kind, s);
    cx.gfx().pillar(s, match kind { EvKind::Cursed => 0xb04aff, EvKind::Totem => 0xffd35a, EvKind::Merchant => 0xffc060 }, 0.9, 14.0, 1.4);
    let (a, b) = match kind {
        EvKind::Cursed => ("A cursed shrine appears", "Accept its curse for a relic"),
        EvKind::Totem => ("A war totem rises", "Defend it for 40 seconds"),
        EvKind::Merchant => ("A wandering merchant", "Spend your gold before he moves on"),
    };
    cx.announce(a, b, 3.0); cx.play("horn");
    if kind == EvKind::Merchant { roll_shop(cx); }
    cx.g.ev.list.push(Ev { kind, pos: s, root, life: if kind == EvKind::Merchant { 70.0 } else { 60.0 }, active: 0.0, used: false, hp: 0.0 });
}
fn roll_shop(cx: &mut Ctx) {
    let m = cx.g.minutes();
    let mut s = vec![
        Offer { name: "Relic cache", desc: "Choose a relic or mastery", cost: (40.0 + m * 6.0) as u32, sold: false },
        Offer { name: "Healing draught", desc: "Restore half your health", cost: (15.0 + m * 2.0) as u32, sold: false },
        Offer { name: "Lucky charm", desc: "+2 rerolls", cost: 25, sold: false },
        Offer { name: "Tome of insight", desc: "Gain a level", cost: (30.0 + m * 4.0) as u32, sold: false },
        Offer { name: "Whetstone", desc: "+8% damage for the rest of the run", cost: (35.0 + m * 5.0) as u32, sold: false },
    ];
    // four of the five, so every visit is a little different
    let k = (cx.r() * 5.0) as usize % 5; s.remove(k);
    cx.g.ev.shop = s;
}
pub fn buy(g: &mut Game, p: &mut Player, i: usize) -> bool {
    let Some(o) = g.ev.shop.get(i).cloned() else { return false };
    if o.sold || g.gold < o.cost { return false; }
    g.gold -= o.cost; g.ev.shop[i].sold = true;
    match o.name {
        "Relic cache" => { g.chest_pending += 1; g.flags.insert("relic_chest", 1.0); }
        "Healing draught" => { p.hp = (p.hp + p.max_hp * 0.5).min(p.max_hp); }
        "Lucky charm" => { g.rerolls += 2; }
        "Tome of insight" => { g.lvl += 1; g.level_pending += 1; }
        _ => { g.st.dmg += 0.08; }
    }
    true
}
pub fn near_merchant(g: &Game, pos: Vec3) -> bool { g.ev.list.iter().any(|e| e.kind == EvKind::Merchant && e.pos.distance(pos) < 4.5) }

/// the prompt for the nearest event the player can use
pub fn interact_prompt(g: &Game, pos: Vec3) -> Option<(usize, String)> {
    for (i, e) in g.ev.list.iter().enumerate() {
        if e.used && e.kind != EvKind::Merchant { continue; }
        if e.pos.distance(pos) > 4.0 { continue; }
        return Some((i, match e.kind { EvKind::Cursed => "[E]  Accept the curse: a minute of fury for a relic".into(), EvKind::Totem => "[E]  Raise the totem: defend it for 40 seconds".into(), EvKind::Merchant => "[E]  Trade with the merchant".into() }));
    }
    None
}
/// returns true when the merchant's shop should open
pub fn use_event(cx: &mut Ctx, i: usize) -> bool {
    let (kind, pos, root) = { let e = &cx.g.ev.list[i]; (e.kind, e.pos, e.root) };
    match kind {
        EvKind::Merchant => return true,
        EvKind::Cursed => {
            let e = &mut cx.g.ev.list[i]; e.used = true; e.active = 60.0;
            cx.g.flags.insert("curse", 1.0);
            cx.announce("The curse takes hold", "Survive a minute of fury", 2.5); cx.play("bomb"); cx.shake(0.5);
            let mut g = cx.gfx(); g.circle(pos, 0xb04aff, 9.0, 60.0, "blood", 0.6, None, None); g.shock(pos, 0xb04aff, 0.5, 12.0, 0.6, 3.0);
        }
        EvKind::Totem => {
            let m = cx.g.minutes();
            let hp = 500.0 + m * 60.0;
            let e = &mut cx.g.ev.list[i]; e.used = true; e.active = 40.0; e.hp = hp;
            cx.g.rel.decoys.push(crate::relics::Decoy { pos, t: 40.5, hp, e: root });
            cx.announce("Defend the totem", "The horde will come for it", 2.5); cx.play("horn");
            cx.gfx().circle(pos, 0xffd35a, 6.0, 40.0, "war", 0.5, None, None);
        }
    }
    false
}

/// one line per running event, for the HUD
pub fn status(g: &Game) -> Vec<String> {
    let mut v = vec![];
    if let Some(c) = &g.ev.challenge {
        v.push(match c.kind { ChKind::Slay => format!("Challenge: slay {}  ({}/{})  {:.0}s", c.target, g.kills.saturating_sub(c.base).min(c.target), c.target, c.t.max(0.0)), ChKind::Untouched => format!("Challenge: take no damage  {:.0}s", c.t.max(0.0)) });
    }
    for e in g.ev.list.iter() {
        if e.active <= 0.0 { continue; }
        match e.kind {
            EvKind::Cursed => v.push(format!("Cursed  {:.0}s", e.active)),
            EvKind::Totem => { let hp = g.rel.decoys.iter().find(|d| d.e == e.root).map(|d| d.hp).unwrap_or(0.0); v.push(format!("Totem  {:.0} hp  {:.0}s", hp.max(0.0), e.active)); }
            _ => {}
        }
    }
    if let Some((_, t)) = g.ev.goblin { v.push(format!("Treasure goblin escapes in {:.0}s", t.max(0.0))); }
    v
}

fn reward(cx: &mut Ctx, at: Vec3, gold: f32, chest: bool) {
    for _ in 0..4 { cx.drop_pick(at, PickKind::Gold, (gold / 4.0).round()); }
    if chest { crate::director::drop_chest(cx, at + Vec3::new(0.0, 0.0, 2.0), true); }
    let xp = (10.0 + cx.g.minutes() * 4.0).round(); cx.drop_gems(at, xp);
    cx.play("chest");
}

pub fn update(cx: &mut Ctx) {
    if cx.p.dead { return; }
    let dt = cx.dt; let m = cx.g.minutes(); let pp = cx.p.body.pos;
    // the schedule
    cx.g.ev.next -= dt;
    if cx.g.ev.next <= 0.0 {
        cx.g.ev.next = cx.rr(70.0, 100.0);
        let i = cx.g.ev.idx; cx.g.ev.idx += 1;
        match i % 4 {
            0 => start_challenge(cx),
            1 => spawn_event(cx, EvKind::Cursed),
            2 => spawn_goblin(cx),
            _ => spawn_event(cx, EvKind::Totem),
        }
    }
    cx.g.ev.merchant_t -= dt;
    if cx.g.ev.merchant_t <= 0.0 { cx.g.ev.merchant_t = 270.0; spawn_event(cx, EvKind::Merchant); }
    // challenges
    if let Some(c) = cx.g.ev.challenge.as_mut() {
        c.t -= dt;
        let (kind, target, base, t, start) = (c.kind, c.target, c.base, c.t, c.start);
        let done = match kind { ChKind::Slay => cx.g.kills.saturating_sub(base) >= target, ChKind::Untouched => t <= 0.0 && cx.g.last_hit < start };
        let failed = match kind { ChKind::Slay => t <= 0.0, ChKind::Untouched => cx.g.last_hit >= start };
        if done { cx.g.ev.challenge = None; cx.announce("Challenge complete", "Your reward falls from the sky", 2.5); reward(cx, pp + cx.p.fwd() * 3.0, 20.0 + m * 3.0, false); }
        else if failed { cx.g.ev.challenge = None; cx.announce("Challenge failed", "", 1.8); cx.play("hurt"); }
    }
    // goblin escape timer
    if let Some((id, t)) = cx.g.ev.goblin.as_mut() {
        *t -= dt;
        let (id, t) = (*id, *t);
        match cx.find(id) {
            None => cx.g.ev.goblin = None,
            Some(i) if t <= 0.0 => {
                let p = cx.g.enemies[i].body.pos; cx.fx.smoke(p, 0xd8d0c0, 10, 2.0, 1.0, 1.0); cx.gfx().flare(p + Vec3::Y, 0xffd35a, 3.0, 0.4, true);
                kill_enemy(cx, i, true, false); cx.announce("The goblin escaped", "", 1.8); cx.g.ev.goblin = None;
            }
            _ => {}
        }
    }
    // the event props
    let mut k = 0;
    while k < cx.g.ev.list.len() {
        let (kind, pos, root, active, used) = { let e = &mut cx.g.ev.list[k]; if e.active > 0.0 { e.active -= dt; } else { e.life -= dt; } (e.kind, e.pos, e.root, e.active, e.used) };
        let life = cx.g.ev.list[k].life;
        let col = match kind { EvKind::Cursed => 0xb04aff, EvKind::Totem => 0xffd35a, EvKind::Merchant => 0xffc060 };
        if cx.r() < 0.4 { let a = cx.rr(0.0, TAU); cx.fx.emit(true, pos + Vec3::new(a.cos() * 0.7, cx.rr(0.5, 3.2), a.sin() * 0.7), Vec3::Y * cx.rr(0.5, 1.5), 0.9, 0.3, 0.05, hexc(col) * 2.5, 0.0, 0.0, 1.0); }
        let mut remove = false;
        match kind {
            EvKind::Cursed if used && active <= 0.0 => {
                cx.g.flags.remove("curse");
                cx.announce("The curse lifts", "The shrine yields its relic", 2.5);
                reward(cx, pos, 40.0 + m * 4.0, true);
                cx.gfx().debris(pos + Vec3::Y * 1.5, &[0x14101a, 0x1a1420, 0xb04aff], 18, 5.0, 0.35, 6.0);
                remove = true;
            }
            EvKind::Totem if used => {
                let alive = cx.g.rel.decoys.iter().any(|d| d.e == root);
                if !alive {
                    if active > 0.6 { cx.announce("The totem fell", "", 2.0); cx.play("hurt"); } else { cx.announce("The totem stands", "Its spirits reward you", 2.5); reward(cx, pos, 30.0 + m * 4.0, true); }
                    cx.g.ev.list.swap_remove(k); continue; // the decoy system already removed the model
                }
            }
            EvKind::Merchant if life <= 0.0 => { cx.announce("The merchant moves on", "", 1.6); remove = true; }
            _ if !used && life <= 0.0 => { remove = true; }
            _ => {}
        }
        if remove { let e = cx.g.ev.list.swap_remove(k); cx.c.entity(e.root).despawn_recursive(); cx.fx.smoke(e.pos, 0xd8d0c0, 6, 1.5, 0.8, 0.9); continue; }
        if kind == EvKind::Merchant { if let Ok(mut t) = cx.q.get_mut(root) { t.rotation = Quat::from_rotation_y((pp - pos).x.atan2((pp - pos).z) * 0.0); } }
        k += 1;
    }
}

pub fn debug_start(cx: &mut Ctx, k: u32) { match k { 0 => start_challenge(cx), 1 => spawn_event(cx, EvKind::Cursed), 2 => spawn_goblin(cx), 3 => spawn_event(cx, EvKind::Totem), _ => spawn_event(cx, EvKind::Merchant) } }
fn start_challenge(cx: &mut Ctx) {
    let m = cx.g.minutes();
    let c = if cx.r() < 0.6 {
        let n = (30.0 + m * 4.0) as u32; cx.announce("Challenge", &format!("Slay {} monsters in 45 seconds", n), 3.0);
        Challenge { kind: ChKind::Slay, target: n, base: cx.g.kills, t: 45.0, start: cx.g.run_t }
    } else {
        cx.announce("Challenge", "Take no damage for 30 seconds", 3.0);
        Challenge { kind: ChKind::Untouched, target: 0, base: 0, t: 30.0, start: cx.g.run_t }
    };
    cx.play("horn");
    cx.g.ev.challenge = Some(c);
}
fn spawn_goblin(cx: &mut Ctx) {
    let (pp, yaw) = (cx.p.body.pos, cx.p.yaw);
    let Some(s) = ring_spot(cx, pp, yaw, 12.0, 18.0) else { return };
    if let Some(i) = spawn_enemy(cx, Kind::Goblin, s, SpawnOpts { force: true, no_rise: true, ..default() }) {
        cx.g.enemies[i].flee = 999.0;
        let id = cx.g.enemies[i].id; cx.g.ev.goblin = Some((id, 25.0));
        cx.announce("A treasure goblin!", "Catch it before it gets away", 2.5); cx.play("relic");
        cx.gfx().flare(s + Vec3::Y, 0xffd35a, 4.0, 0.5, true);
    }
}
