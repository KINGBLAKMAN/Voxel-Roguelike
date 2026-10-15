//! Level-up choices. Every pick draws three cards from one weighted pool: ability and weapon upgrades (with a
//! mastery per ability once it has two ranks), relics (new or ranked up), chaos augments and general boons.
use crate::abilities::{ab_info, ab_level, capped, class_abilities, growth, unbound, unbound_count, LV_CAP, MODS, WMODS};
use crate::augments::{AUGS, AUG_COLOR};
use crate::combat::weapon_info;
use crate::game::*;
use crate::models::Class;
use crate::player::{class_weapons, Player};
use crate::relics::{relic_count, rk, RELICS, RMAX, RSLOTS};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CardKind { Upgrade, Mastery, Relic, Chaos, Boon, Evolution, Empower }
#[derive(Clone)]
pub struct Card { pub kind: CardKind, pub key: String, pub name: String, pub desc: String, pub tag: String, pub color: u32, pub w: f32 }

/// (id, name, description, class restriction, once)
pub const PERKS: [(&str, &str, &str, Option<Class>, bool); 26] = [
    ("vit", "Vitality", "+25 max health and heal fully.", None, false),
    ("might", "Might", "+15% damage with everything.", None, false),
    ("haste", "Haste", "Abilities recharge 12% faster.", None, false),
    ("swift", "Fleetfoot", "+10% movement speed.", None, false),
    ("vamp", "Vampirism", "Heal for 4% of damage you deal.", None, false),
    ("crit", "Keen Edge", "+8% critical chance. Crits deal double.", None, false),
    ("skin", "Ironhide", "Take 10% less damage.", None, false),
    ("regen", "Second Wind", "Regenerate 0.6 health per second.", None, false),
    ("forage", "Forager", "Food restores 50% more. Animals drop extra meat.", None, false),
    ("hunter", "Giant Slayer", "+35% damage to elites and bosses.", None, false),
    ("magnet", "Lodestone", "Collect experience and loot from twice as far.", None, false),
    ("aoe", "Amplify", "All area abilities are 20% larger.", None, false),
    ("stam", "Endurance", "+40 stamina, recovers 30% faster.", None, false),
    ("fury", "Fury", "Weapon attacks are 15% faster.", None, false),
    ("m1", "Pyromancer", "Fire burns 35% larger and longer.", Some(Class::Mage), false),
    ("m2", "Shatter", "Frozen enemies take 50% more damage.", Some(Class::Mage), false),
    ("w1", "Bloodlust", "Whirlwind heals 1 health per enemy hit (up to 6 per spin).", Some(Class::Warrior), false),
    ("w2", "Juggernaut", "+40 max health and take 8% less damage.", Some(Class::Warrior), false),
    ("p1", "Radiance", "Mend also blasts nearby enemies for 40.", Some(Class::Priest), true),
    ("r2", "Quickdraw", "Draw your bow 35% faster.", Some(Class::Ranger), false),
    ("n1", "Grave Pact", "Your skeletons hit 30% harder and last 4s longer.", Some(Class::Necromancer), false),
    ("n2", "Soul Siphon", "Every kill heals 1 health.", Some(Class::Necromancer), false),
    ("g1", "Steady Hands", "+15% critical chance with guns.", Some(Class::Gunslinger), false),
    ("g2", "Speed Loader", "Reload 30% faster.", Some(Class::Gunslinger), false),
    ("wis", "Wisdom", "Gain 20% more experience.", None, false),
    ("luck", "Fortune", "+2 rerolls on level-ups.", None, false),
];
pub fn class_color(c: Class) -> u32 { match c { Class::Ranger => 0x74c454, Class::Mage => 0xa183ff, Class::Warrior => 0xe25b45, Class::Priest => 0xf2cf63, Class::Necromancer => 0x6adf9a, Class::Gunslinger => 0xe0a85a } }
pub fn class_name(c: Class) -> &'static str { match c { Class::Ranger => "Ranger", Class::Mage => "Mage", Class::Warrior => "Warrior", Class::Priest => "Priest", Class::Necromancer => "Necromancer", Class::Gunslinger => "Gunslinger" } }

fn options(g: &Game, cls: Class) -> Vec<Card> {
    let mut out = vec![];
    for ab in class_abilities(cls) {
        let info = ab_info(ab);
        if capped(g, cls, ab) { continue; }
        let lv = ab_level(g, ab); let ub = unbound(g, ab);
        let tag = if ub { format!("UNBOUND {}  -  level {}", info.name, lv) } else if lv + 1 >= LV_CAP && unbound_count(g, cls) < 2 { format!("{}  -  level 10 makes it UNBOUND", info.name) } else if lv + 1 >= LV_CAP { format!("{}  -  final level (2 abilities already Unbound)", info.name) } else { format!("{}  -  level {} of 10", info.name, lv) };
        out.push(Card { kind: CardKind::Empower, key: format!("lv.{}", ab), name: format!("Empower {}", info.name), desc: format!("Level {} to {}: {}.", lv, lv + 1, growth(ab)), tag: tag.clone(), color: info.color, w: if ub { 1.7 } else { 1.25 } });
        if g.lvl >= 4 && (ub || lv + 2 <= LV_CAP) { out.push(Card { kind: CardKind::Empower, key: format!("lv2.{}", ab), name: format!("Surge: {} +2", info.name), desc: format!("Two levels at once ({} to {}): {}.", lv, lv + 2, growth(ab)), tag, color: info.color, w: 0.15 }); }
        let inv: u32 = MODS.iter().filter(|m| m.0 == ab && !m.5).map(|m| g.m(&format!("{}.{}", ab, m.1))).sum();
        for m in MODS.iter().filter(|m| m.0 == ab) {
            let key = format!("{}.{}", ab, m.1); let r = g.m(&key);
            if r >= m.4 || (m.5 && inv < 2) { continue; }
            out.push(Card { kind: if m.5 { CardKind::Mastery } else { CardKind::Upgrade }, key, name: m.2.into(), desc: m.3.into(),
                tag: if m.5 { format!("{} mastery  (+1 level)", info.name) } else { format!("{} upgrade - rank {}/{}  (+1 level)", info.name, r + 1, m.4) }, color: info.color, w: if m.5 { 2.2 } else { 1.0 } });
        }
    }
    for w in class_weapons(cls) {
        for m in WMODS.iter().filter(|m| m.0 == w) {
            let key = format!("w.{}.{}", w, m.1); let r = g.m(&key);
            if r < m.4 { out.push(Card { kind: CardKind::Upgrade, key, name: m.2.into(), desc: m.3.into(), tag: format!("{} upgrade - rank {}/{}", weapon_info(w).0, r + 1, m.4), color: class_color(cls), w: 0.8 }); }
        }
    }
    // evolutions: the most exciting card in the game, so they come up often once unlocked
    for e in crate::relics::EVOS.iter() {
        if rk(g, e.0) >= RMAX && !crate::relics::ev(g, e.0) && g.perks.iter().any(|q| q == e.2) {
            out.push(Card { kind: CardKind::Evolution, key: format!("e.{}", e.0), name: e.3.into(), desc: e.4.into(), tag: "Evolution".into(), color: 0xffd35a, w: 8.0 });
        }
    }
    let owned = relic_count(g);
    for rl in RELICS.iter() {
        let r = rk(g, rl.id);
        if r >= RMAX || (r == 0 && owned >= RSLOTS) || g.locked.contains(&rl.id) { continue; }
        let evo_hint = if r + 1 == RMAX { crate::relics::EVOS.iter().find(|e| e.0 == rl.id).map(|e| format!(" Max rank: evolves with {}.", e.2)).unwrap_or_default() } else { String::new() };
        out.push(Card { kind: CardKind::Relic, key: format!("r.{}", rl.id), name: rl.name.into(), desc: format!("{}{}", rl.ranks[r as usize], evo_hint), tag: if r > 0 { format!("Relic - rank {}/{}", r + 1, RMAX) } else { "New relic".into() }, color: rl.color, w: if r > 0 { 1.7 } else if owned < 2 { 2.4 } else { 1.3 } });
    }
    // two chaos augments per run, so they're a choice
    let chaos_owned = AUGS.iter().filter(|a| g.m(&format!("x.{}", a.id)) > 0).count();
    for a in AUGS.iter() {
        let r = g.m(&format!("x.{}", a.id));
        if r as usize >= a.ranks.len() || (r == 0 && chaos_owned >= 2) { continue; }
        out.push(Card { kind: CardKind::Chaos, key: format!("x.{}", a.id), name: a.name.into(), desc: a.ranks[r as usize].into(), tag: if r > 0 { format!("Chaos augment - rank {}/{}", r + 1, a.ranks.len()) } else { "Chaos augment".into() }, color: AUG_COLOR, w: if g.lvl < 3 { 0.4 } else if r > 0 { 1.1 } else { 1.2 } });
    }
    // the rare card that feeds every ability at once
    if g.lvl >= 5 && class_abilities(cls).iter().filter(|a| !capped(g, cls, a)).count() >= 2 {
        out.push(Card { kind: CardKind::Empower, key: "lvall".into(), name: "Ascendance".into(), desc: "+1 level to every ability that can still grow.".into(), tag: "Rare".into(), color: 0xffd35a, w: 0.25 });
    }
    // each boon can be taken at most 5 times
    let pool: Vec<_> = PERKS.iter().filter(|p| p.3.map(|c| c == cls).unwrap_or(true) && !(p.4 && g.perks.iter().any(|q| q == p.1)) && g.perks.iter().filter(|q| *q == p.1).count() < 5).collect();
    let n = pool.len() as f32;
    for p in pool { out.push(Card { kind: CardKind::Boon, key: p.0.into(), name: p.1.into(), desc: p.2.into(), tag: match p.3 { Some(c) => format!("{} boon", class_name(c)), None => "Boon".into() }, color: p.3.map(class_color).unwrap_or(0xff9a3d), w: 5.0 / n }); }
    out
}
/// draw three distinct cards; relic chests offer only relics and masteries
pub fn roll(g: &Game, cls: Class, relic_only: bool, rnd: &mut impl FnMut() -> f32) -> Vec<Card> {
    let mut pool = options(g, cls);
    if relic_only { let r: Vec<Card> = pool.iter().filter(|c| matches!(c.kind, CardKind::Relic | CardKind::Mastery | CardKind::Evolution)).cloned().collect(); if !r.is_empty() { pool = r; } }
    let mut out = vec![];
    while out.len() < 3 && !pool.is_empty() {
        let tot: f32 = pool.iter().map(|c| c.w).sum();
        let mut r = rnd() * tot; let mut k = 0;
        while k < pool.len() - 1 { r -= pool[k].w; if r <= 0.0 { break; } k += 1; }
        out.push(pool.remove(k));
    }
    out
}
pub fn apply(g: &mut Game, p: &mut Player, c: &Card) {
    match c.kind {
        CardKind::Boon => {
            let s = &mut g.st;
            match c.key.as_str() {
                "vit" => { p.max_hp += 25.0; p.hp = p.max_hp; }
                "might" => s.dmg += 0.15, "haste" => s.cd *= 0.88, "swift" => s.speed += 0.1, "vamp" => s.leech += 0.04, "crit" => s.crit += 0.08,
                "skin" => s.armor = 1.0 - (1.0 - s.armor) * 0.9, "regen" => s.regen += 0.6, "forage" => s.food += 0.5, "hunter" => s.elite += 0.35,
                "magnet" => s.magnet *= 1.8, "aoe" => s.area += 0.2, "stam" => { s.stam_max += 40.0; s.stam_regen += 0.3; }, "fury" => s.atk_spd += 0.15,
                "m1" => s.fire_rad += 0.35, "m2" => s.shatter += 0.5, "w1" => s.ww_heal += 1.0,
                "w2" => { p.max_hp += 40.0; p.hp += 40.0; s.armor = 1.0 - (1.0 - s.armor) * 0.92; }
                "p1" => s.radiance = 1.0, "g1" => s.crit += 0.15, "r2" => s.charge_mul += 0.35, "wis" => s.xp_mul += 0.2, "luck" => g.rerolls += 2,
                _ => {}
            }
        }
        CardKind::Empower => {
            let cls = p.cls;
            let bump = |g: &mut Game, id: &str, n: u32| { let k = format!("lv.{}", id); *g.mods.entry(k).or_insert(0) += n; };
            if c.key == "lvall" { for a in class_abilities(cls) { if !capped(g, cls, a) { bump(g, a, 1); } } }
            else if let Some(id) = c.key.strip_prefix("lv2.") { bump(g, id, 2); }
            else if let Some(id) = c.key.strip_prefix("lv.") { bump(g, id, 1); }
        }
        _ => {
            let was = g.m(&c.key);
            g.mods.insert(c.key.clone(), was + 1);
            if c.key == "x.storm" && was == 0 { g.st.crit += 0.05; }
            if c.key == "x.bounce" { g.flags.insert("bounce", (was + 1) as f32); }
        }
    }
    g.perks.push(c.name.clone());
    // the first two abilities to reach level 10 break their limit
    while let Some(id) = crate::abilities::check_unbound(g, p.cls) {
        g.toasts.push((format!("{} is UNBOUND", ab_info(id).name), "It keeps growing forever; the other abilities stop at level 10 once two are Unbound".into(), 4.0));
        g.flash = 0.3;
    }
}
