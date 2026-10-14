//! Progress that outlives a run: gold, permanent upgrades bought at the Camp, unlockable classes and relics, the
//! Ember difficulty ladder, settings and a run history. Saved as JSON next to the player's other app data.
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
}

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

fn dir() -> PathBuf {
    let base = std::env::var("APPDATA").map(PathBuf::from).or_else(|_| std::env::var("XDG_DATA_HOME").map(PathBuf::from)).or_else(|_| std::env::var("HOME").map(|h| PathBuf::from(h).join(".local/share"))).unwrap_or_else(|_| PathBuf::from("."));
    base.join("Emberwild")
}
impl Save {
    pub fn load() -> Self {
        if std::env::var("EW_TEST").is_ok() { return Save::default(); }
        std::fs::read_to_string(dir().join("save.json")).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
    }
    pub fn write(&self) {
        if std::env::var("EW_TEST").is_ok() { return; }
        let d = dir(); let _ = std::fs::create_dir_all(&d);
        if let Ok(s) = serde_json::to_string_pretty(self) { let tmp = d.join("save.json.tmp"); if std::fs::write(&tmp, s).is_ok() { let _ = std::fs::rename(tmp, d.join("save.json")); } }
    }
    pub fn up(&self, id: &str) -> u32 { *self.upgrades.get(id).unwrap_or(&0) }
    pub fn has(&self, id: &str) -> bool { self.unlocks.iter().any(|u| u == id) }
    pub fn cost(&self, i: usize) -> u32 { let u = UPGRADES[i]; u.4 * (self.up(u.0) + 1) }
    pub fn class_open(&self, c: Class) -> bool { match c { Class::Mage => self.has("class.mage"), Class::Priest => self.has("class.priest"), _ => true } }
    pub fn buy(&mut self, i: usize) -> bool {
        let u = UPGRADES[i]; let c = self.cost(i);
        if self.up(u.0) >= u.3 || self.gold < c { return false; }
        self.gold -= c; *self.upgrades.entry(u.0.to_string()).or_insert(0) += 1; self.write(); true
    }
    pub fn unlock(&mut self, i: usize) -> bool {
        let u = UNLOCKS[i];
        if self.has(u.0) || self.gold < u.3 { return false; }
        self.gold -= u.3; self.unlocks.push(u.0.to_string()); self.write(); true
    }
}

/// apply permanent upgrades and the chosen Ember level when a run begins
pub fn apply_start(s: &Save, g: &mut Game, p: &mut Player) {
    let st = &mut g.st;
    let vit = s.up("vit") as f32 * 10.0; p.max_hp += vit; p.hp = p.max_hp;
    st.dmg += 0.05 * s.up("might") as f32;
    st.speed += 0.04 * s.up("swift") as f32;
    st.cd *= 1.0 - 0.04 * s.up("haste") as f32;
    st.armor = 1.0 - (1.0 - st.armor) * (1.0 - 0.03 * s.up("armor") as f32);
    st.xp_mul += 0.06 * s.up("insight") as f32;
    st.magnet *= 1.0 + 0.1 * s.up("magnet") as f32;
    g.rerolls += s.up("reroll");
    g.flags.insert("lives", s.up("revive") as f32);
    g.flags.insert("greed", 1.0 + 0.1 * s.up("greed") as f32);
    for r in ["axe", "flask", "frost"] { if !s.has(&format!("relic.{}", r)) { g.locked.push(r); } }
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
