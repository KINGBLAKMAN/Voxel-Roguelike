//! Progress that outlives a run. It is purely cosmetic: gold buys outfits, headgear, capes, weapon finishes and
//! auras at the Camp; every class and relic is open from the start; the only thing that climbs besides your
//! wardrobe is the Ember difficulty ladder. Also settings and a run history. Saved as JSON in the app data folder.
use crate::game::Game;
use crate::models::Class;
use crate::player::Player;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Settings { pub sens: f32, pub volume: f32, pub music: f32, pub view: i32, pub shadows: bool, pub fullscreen: bool, pub fov: f32, pub shake: f32, pub damage_numbers: bool, pub invert_y: bool, pub quality: u8 }
impl Default for Settings { fn default() -> Self { Settings { sens: 1.0, volume: 0.7, music: 0.6, view: 8, shadows: true, fullscreen: true, fov: 72.0, shake: 1.0, damage_numbers: true, invert_y: false, quality: 2 } } }

#[derive(Serialize, Deserialize, Clone)]
pub struct RunRecord { pub class: String, pub time: f32, pub kills: u32, pub level: u32, pub won: bool, pub ember: u32, pub gold: u32, pub bosses: u32 }

#[derive(Resource, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Save {
    pub gold: u32, pub upgrades: HashMap<String, u32>, pub unlocks: Vec<String>, pub max_ember: u32, pub ember: u32,
    pub settings: Settings, pub history: Vec<RunRecord>, pub runs: u32, pub wins: u32, pub best_time: f32, pub best_kills: u32, pub total_kills: u32,
    /// cosmetics bought, what each class wears, and gold handed back when the old power upgrades were retired
    pub owned: Vec<String>, pub wear: HashMap<String, crate::cosmetics::Cos>, pub refunded: u32,
}

/// retired: the permanent power upgrades of earlier versions, kept only to refund what was spent on them
/// (id, name, description per rank, max rank, base cost)
pub const UPGRADES: [(&str, &str, &str, u32, u32); 10] = [
    ("vit", "Vitality", "+10 max health", 5, 40),
    ("might", "Might", "+5% damage", 5, 50),
    ("swift", "Swiftness", "+4% move speed", 3, 40),
    ("haste", "Focus", "Abilities recharge 4% faster", 5, 50),
    ("armor", "Hide", "Take 3% less damage", 5, 50),
    ("insight", "Insight", "+6% experience", 5, 35),
    ("magnet", "Reach", "+10% pickup range", 3, 30),
    ("greed", "Greed", "+10% gold", 5, 45),
    ("reroll", "Fortune", "+1 reroll each run", 3, 60),
    ("revive", "Second Chance", "Rise once per run at half health", 1, 400),
];
/// (id, name, description, cost)
pub const UNLOCKS: [(&str, &str, &str, u32); 5] = [
    ("class.mage", "The Mage", "Glass cannon of fire and frost", 60),
    ("class.priest", "The Priest", "Holy bruiser who heals through anything", 60),
    ("relic.axe", "Ricochet Axe", "Adds the relic to level-up offers", 100),
    ("relic.flask", "Firebomb Flask", "Adds the relic to level-up offers", 100),
    ("relic.frost", "Frost Halo", "Adds the relic to level-up offers", 100),
];
/// the Ember ladder: each level adds one twist on top of the ones below
pub const EMBERS: [&str; 11] = [
    "Ember 0: the wild as it is",
    "Ember 1: monsters have 20% more health",
    "Ember 2: monsters hit 20% harder",
    "Ember 3: elites appear twice as often",
    "Ember 4: healing is 25% weaker",
    "Ember 5: bosses have 40% more health",
    "Ember 6: the horde is 25% larger",
    "Ember 7: eruptions start at minute 3 and come faster",
    "Ember 8: one fewer reroll and no hearts",
    "Ember 9: monsters are 15% faster",
    "Ember 10: everything above, and the Ember King has a fourth phase of fury",
];

pub fn data_dir() -> PathBuf { dir() }
fn file() -> &'static str { if cfg!(feature = "dev") { "save-dev.json" } else { "save.json" } }
fn dir() -> PathBuf {
    let base = std::env::var("APPDATA").map(PathBuf::from).or_else(|_| std::env::var("XDG_DATA_HOME").map(PathBuf::from)).or_else(|_| std::env::var("HOME").map(|h| PathBuf::from(h).join(".local/share"))).unwrap_or_else(|_| PathBuf::from("."));
    base.join("Emberwild")
}
impl Save {
    pub fn load() -> Self {
        if crate::testkit::active() || crate::bot::active() { return Save::default(); }
        let mut s: Save = std::fs::read_to_string(dir().join(file())).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
        // meta progression is cosmetic now: give back every coin spent on power upgrades and unlocks
        let mut back = 0;
        for u in UPGRADES.iter() { let r = s.up(u.0); back += (1..=r).map(|k| u.4 * k).sum::<u32>(); }
        if !cfg!(feature = "dev") { for u in UNLOCKS.iter() { if s.has(u.0) { back += u.3; } } }
        if back > 0 || !s.upgrades.is_empty() || !s.unlocks.is_empty() { s.gold += back; s.refunded += back; s.upgrades.clear(); s.unlocks.clear(); s.write(); }
        // the dev build keeps its own save with every cosmetic owned and every Ember level open
        if cfg!(feature = "dev") {
            for c in crate::cosmetics::COSMETICS.iter() { if !s.owns(c.id) { s.owned.push(c.id.to_string()); } }
            s.max_ember = 10;
        }
        s
    }
    pub fn write(&self) {
        if crate::testkit::active() || crate::bot::active() { return; }
        let d = dir(); let _ = std::fs::create_dir_all(&d);
        if let Ok(s) = serde_json::to_string_pretty(self) { let tmp = d.join(format!("{}.tmp", file())); if std::fs::write(&tmp, s).is_ok() { let _ = std::fs::rename(tmp, d.join(file())); } }
    }
    pub fn up(&self, id: &str) -> u32 { *self.upgrades.get(id).unwrap_or(&0) }
    pub fn has(&self, id: &str) -> bool { self.unlocks.iter().any(|u| u == id) }
    pub fn cost(&self, i: usize) -> u32 { let u = UPGRADES[i]; u.4 * (self.up(u.0) + 1) }
    pub fn class_open(&self, _c: Class) -> bool { true }
    pub fn owns(&self, id: &str) -> bool { self.owned.iter().any(|o| o == id) }
    /// what a class wears; anything no longer owned falls back to the default
    pub fn wear(&self, c: Class) -> crate::cosmetics::Cos { self.wear.get(crate::progression::class_name(c)).copied().unwrap_or_default() }
    /// buy (if needed) and put on cosmetic `idx` (1-based) for class `c`, or take that slot off if it's already worn
    pub fn wardrobe(&mut self, c: Class, idx: u8) -> bool {
        let Some(item) = crate::cosmetics::COSMETICS.get(idx as usize - 1) else { return false };
        if !self.owns(item.id) { if self.gold < item.cost { return false; } self.gold -= item.cost; self.owned.push(item.id.to_string()); }
        let mut w = self.wear(c);
        w.set(item.kind, if w.get(item.kind) == idx { 0 } else { idx });
        self.wear.insert(crate::progression::class_name(c).to_string(), w);
        self.write(); true
    }
}

/// apply permanent upgrades and the chosen Ember level when a run begins
/// a run starts with nothing but the chosen Ember level: all power is earned inside the run
pub fn apply_start(s: &Save, g: &mut Game, _p: &mut Player) {
    // the Ember ladder
    let e = s.ember;
    g.flags.insert("ember", e as f32);
    g.flags.insert("ember_hp", if e >= 1 { 1.2 } else { 1.0 });
    g.flags.insert("ember_dmg", if e >= 2 { 1.2 } else { 1.0 });
    if e >= 8 { g.rerolls = g.rerolls.saturating_sub(1); }
}
pub fn ember(g: &Game) -> u32 { *g.flags.get("ember").unwrap_or(&0.0) as u32 }

/// gold earned for a run: what you picked up plus a payout for time, kills, bosses and victory
pub fn payout(g: &Game) -> u32 {
    let won = g.flags.contains_key("victory");
    let base = g.gold as f32 + g.kills as f32 / 12.0 + g.minutes() * 4.0 + g.flags.get("bosses").copied().unwrap_or(0.0) * 30.0 + if won { 150.0 } else { 0.0 };
    (base * g.flags.get("greed").copied().unwrap_or(1.0) * (1.0 + 0.1 * ember(g) as f32)).round() as u32
}
pub fn record_run(s: &mut Save, g: &Game, cls: Class, gold: u32) {
    let won = g.flags.contains_key("victory");
    s.gold += gold; s.runs += 1; s.total_kills += g.kills;
    if won { s.wins += 1; s.max_ember = s.max_ember.max((s.ember + 1).min(10)); }
    s.best_time = s.best_time.max(g.run_t); s.best_kills = s.best_kills.max(g.kills);
    s.history.insert(0, RunRecord { class: crate::progression::class_name(cls).into(), time: g.run_t, kills: g.kills, level: g.lvl, won, ember: ember(g), gold, bosses: g.flags.get("bosses").copied().unwrap_or(0.0) as u32 });
    s.history.truncate(30);
    s.write();
}
