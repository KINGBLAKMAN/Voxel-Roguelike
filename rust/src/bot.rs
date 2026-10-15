//! The balance bot: plays complete runs on its own and logs how they went, so balance changes can be judged on
//! numbers instead of feel. Start it with `Emberwild.exe --bot` (or the EW_BOT environment variable) plus an
//! optional settings string, e.g. `--bot "runs=5;classes=warrior,mage;minutes=20;turbo=6;ember=0"`.
//!
//! The simulation runs several fixed steps per rendered frame (`turbo`), so runs go many times faster than real
//! time. The bot aims at the nearest monster, keeps its weapon's preferred range while circling, draws and
//! releases the bow, casts its abilities, dodges, eats, opens chests and takes level-up cards. Each run is
//! appended to `balance-<time>.csv` and a per-class summary goes to `balance-<time>-summary.txt`. The player's
//! own save file is never touched.
use crate::hud::{Screen, UiState};
use crate::models::Class;
use crate::player::{Controls, Player};
use bevy::prelude::*;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};

static ACTIVE: AtomicBool = AtomicBool::new(false);
pub fn active() -> bool { ACTIVE.load(Ordering::Relaxed) }

#[derive(Resource, Clone)]
pub struct BotCfg { pub runs: u32, pub classes: Vec<Class>, pub minutes: f32, pub turbo: u32, pub ember: u32, pub seed: u32, pub out: String }

/// look for `--bot [settings]` on the command line or EW_BOT in the environment
pub fn from_env() -> Option<BotCfg> {
    if !cfg!(feature = "dev") { return None; }
    let args: Vec<String> = std::env::args().collect();
    let spec = if let Some(i) = args.iter().position(|a| a == "--bot") { Some(args.get(i + 1).filter(|a| !a.starts_with("--")).cloned().unwrap_or_default()) } else { std::env::var("EW_BOT").ok() }?;
    let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let mut c = BotCfg { runs: 3, classes: crate::models::CLASSES.to_vec(), minutes: 20.0, turbo: 6, ember: 0, seed: (stamp % 100_000) as u32, out: format!("balance-{}", stamp) };
    for kv in spec.split(';') {
        let Some((k, v)) = kv.split_once('=') else { continue };
        match k.trim() {
            "runs" => c.runs = v.parse().unwrap_or(3).max(1),
            "minutes" => c.minutes = v.parse().unwrap_or(20.0),
            "turbo" => c.turbo = v.parse().unwrap_or(6).clamp(1, 30),
            "ember" => c.ember = v.parse().unwrap_or(0).min(10),
            "seed" => c.seed = v.parse().unwrap_or(c.seed),
            "out" => c.out = v.to_string(),
            "classes" => { let l: Vec<Class> = v.split(',').filter_map(|n| match n.trim() { "ranger" => Some(Class::Ranger), "mage" => Some(Class::Mage), "warrior" => Some(Class::Warrior), "priest" => Some(Class::Priest), "necromancer" => Some(Class::Necromancer), "gunslinger" => Some(Class::Gunslinger), _ => None }).collect(); if !l.is_empty() { c.classes = l; } }
            _ => {}
        }
    }
    ACTIVE.store(true, Ordering::Relaxed);
    Some(c)
}

#[derive(Clone)]
struct Row { class: Class, seed: u32, time: f32, won: bool, died: bool, level: u32, kills: u32, taken: f32, healed: f32, gold: u32, bosses: u32, relics: u32 }
#[derive(Resource, Default)]
pub struct BotState {
    run: u32, in_run: bool, rows: Vec<Row>, held: Vec<KeyCode>, lmb: bool, strafe: f32, strafe_t: f32, draw_t: f32, ab_t: f32, e_t: f32, stuck_p: Vec3, stuck_t: f32, rng: u32, settings_done: bool, seeded: Option<u32>,
}
impl BotState { fn r(&mut self) -> f32 { let mut x = self.rng.max(1); x ^= x << 13; x ^= x >> 17; x ^= x << 5; self.rng = x; (x % 1_000_000) as f32 / 1_000_000.0 } }

fn class_name(c: Class) -> &'static str { crate::progression::class_name(c) }
fn preferred_range(id: &str) -> f32 { match id { "longbow" => 13.0, "firestaff" | "frostwand" | "scepter" | "skullstaff" => 9.0, "revolvers" | "rifle" => 11.0, "daggers" => 1.6, _ => 2.2 } }

/// drive the menus and the player; runs before the game tick each frame
#[allow(clippy::too_many_arguments)]
pub fn bot_drive(cfg: Res<BotCfg>, mut st: ResMut<BotState>, mut ui: ResMut<UiState>, mut g: ResMut<crate::game::Game>, mut p: ResMut<Player>, mut keys: ResMut<ButtonInput<KeyCode>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>, mut ctl: ResMut<Controls>, mut save: ResMut<crate::meta::Save>, mut exit: EventWriter<AppExit>, time: Res<Time>) {
    let st = &mut *st;
    for k in st.held.drain(..) { keys.release(k); }
    if !st.lmb { mouse.release(MouseButton::Left); }
    ctl.pad = true;
    if !st.settings_done {
        st.settings_done = true; st.rng = cfg.seed.wrapping_mul(2654435761) | 1;
        save.settings.quality = 0; save.settings.view = 5; save.settings.shadows = false; save.settings.volume = 0.0; save.settings.music = 0.0; save.settings.damage_numbers = false; save.settings.fullscreen = false;
        save.ember = cfg.ember; save.max_ember = cfg.ember;
        for u in ["class.mage", "class.priest", "relic.axe", "relic.flask", "relic.frost"] { save.unlocks.push(u.into()); }
    }
    let total = cfg.runs * cfg.classes.len() as u32;
    let press = |k: KeyCode, st: &mut BotState, keys: &mut ButtonInput<KeyCode>| { keys.press(k); st.held.push(k); };
    match ui.screen {
        Screen::Menu => {
            if st.run >= total { finish(&cfg, st, &mut exit); return; }
            if ui.start.is_none() && !st.in_run {
                let cls = cfg.classes[(st.run % cfg.classes.len() as u32) as usize];
                // a fixed seed per run so a balance change can be compared on the same worlds
                if st.seeded != Some(st.run) { g.net.seed = Some(cfg.seed + st.run * 7919); st.seeded = Some(st.run); return; }
                ui.start = Some(cls); ui.paid = 0; ui.recorded = false; st.in_run = true;
            }
        }
        Screen::LevelUp => { let k = [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3][(st.r() * 3.0) as usize % 3]; press(k, st, &mut keys); }
        Screen::Shop | Screen::Paused | Screen::Settings | Screen::Camp | Screen::Join | Screen::Character => { press(KeyCode::Escape, st, &mut keys); }
        Screen::Dead | Screen::Victory => { if st.in_run { record(&cfg, st, &g, &p); } ui.restart = true; }
        Screen::Playing => {
            if st.in_run && g.run_t >= cfg.minutes * 60.0 { record(&cfg, st, &g, &p); ui.restart = true; return; }
            if p.dead || !p.spawned { st.lmb = false; return; }
            play(cfg.turbo as f32 * time.delta_secs().min(0.05).max(1.0 / 60.0), st, &mut g, &mut p, &mut keys, &mut mouse);
        }
    }
}

fn play(dt: f32, st: &mut BotState, g: &mut crate::game::Game, p: &mut Player, keys: &mut ButtonInput<KeyCode>, mouse: &mut ButtonInput<MouseButton>) {
    let press = |k: KeyCode, st: &mut BotState, keys: &mut ButtonInput<KeyCode>| { keys.press(k); st.held.push(k); };
    let pp = p.body.pos;
    let target = g.enemies.iter().filter(|e| !e.dead && !e.t.passive).map(|e| (e.body.pos.distance(pp), e.body.pos + Vec3::Y * e.body.h * 0.5)).min_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let id = p.weapon_id();
    let want = preferred_range(id);
    st.strafe_t -= dt; if st.strafe_t <= 0.0 { st.strafe_t = 2.0 + st.r() * 3.0; st.strafe = if st.r() < 0.5 { -1.0 } else { 1.0 }; }
    st.lmb = false;
    let low = p.hp < p.max_hp * 0.35;
    // with nothing close, sweep up experience gems (and step into an open portal)
    let gem = g.pickups.iter().filter(|k| k.kind == crate::game::PickKind::Xp).map(|k| (k.pos.distance(pp), k.pos)).filter(|x| x.0 < 26.0).min_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let goal = g.dir.portal.as_ref().map(|pt| pt.pos).or(gem.map(|x| x.1));
    if let (Some(to), true) = (goal, target.map(|t| t.0 > 8.0).unwrap_or(true) || g.dir.portal.is_some()) {
        let d = to - pp; p.yaw = d.x.atan2(d.z); p.pitch = -0.1;
        press(KeyCode::KeyW, st, keys);
        st.stuck_t += dt;
        if st.stuck_t > 2.5 { if st.stuck_p.distance(pp) < 1.0 { press(KeyCode::Space, st, keys); } st.stuck_t = 0.0; st.stuck_p = pp; }
        mouse.release(MouseButton::Left);
        return;
    }
    if let Some((d, tp)) = target {
        let to = tp - (pp + Vec3::Y * 1.4);
        p.yaw = to.x.atan2(to.z);
        p.pitch = (to.y.atan2(Vec2::new(to.x, to.z).length())).clamp(-0.6, 0.6);
        let melee = crate::player::is_melee(id);
        if low && !melee || d < want - 1.5 { press(KeyCode::KeyS, st, keys); } else if d > want + 1.0 { press(KeyCode::KeyW, st, keys); }
        if d < want + 6.0 { press(if st.strafe > 0.0 { KeyCode::KeyD } else { KeyCode::KeyA }, st, keys); }
        let in_range = d < if melee { 3.6 } else { 24.0 };
        if id == "longbow" {
            // hold to draw, let go at about two-thirds charge
            if in_range { st.draw_t += dt; if st.draw_t < 0.55 { st.lmb = true; } else if st.draw_t > 0.65 { st.draw_t = 0.0; } } else { st.draw_t = 0.0; }
        } else { st.lmb = in_range; }
        st.ab_t -= dt;
        if st.ab_t <= 0.0 && d < 12.0 { st.ab_t = 1.2 + st.r() * 1.5; let k = [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4][(st.r() * 4.0) as usize % 4]; press(k, st, keys); }
        if d < 1.4 && st.r() < 0.04 { press(KeyCode::ControlLeft, st, keys); }
    } else {
        // nothing nearby: wander so the director keeps spawning
        press(KeyCode::KeyW, st, keys); p.yaw += dt * 0.2 * st.strafe;
    }
    if p.hunger < 40.0 || (low && (g.food_meat > 0 || g.food_berries > 0)) { press(KeyCode::KeyF, st, keys); }
    st.e_t -= dt; if st.e_t <= 0.0 { st.e_t = 1.0; press(KeyCode::KeyE, st, keys); }
    st.stuck_t += dt;
    if st.stuck_t > 2.5 { if st.stuck_p.distance(pp) < 1.0 { press(KeyCode::Space, st, keys); st.strafe = -st.strafe; } st.stuck_t = 0.0; st.stuck_p = pp; }
    if st.lmb { mouse.press(MouseButton::Left); } else { mouse.release(MouseButton::Left); }
}

fn record(cfg: &BotCfg, st: &mut BotState, g: &crate::game::Game, p: &Player) {
    st.in_run = false;
    let row = Row { class: p.cls, seed: cfg.seed + st.run * 7919, time: g.run_t, won: g.flags.contains_key("victory"), died: g.over || p.dead, level: g.lvl, kills: g.kills, taken: g.dmg_taken, healed: g.healed, gold: g.gold,
        bosses: g.flags.get("bosses").copied().unwrap_or(0.0) as u32, relics: g.mods.iter().filter(|(k, v)| k.starts_with("r.") && **v > 0).count() as u32 };
    st.run += 1;
    let path = format!("{}.csv", cfg.out);
    let new = !std::path::Path::new(&path).exists();
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        if new { let _ = writeln!(f, "run,class,ember,seed,survived_s,won,died,level,kills,damage_taken,healed,gold,bosses,relics"); }
        let _ = writeln!(f, "{},{},{},{},{:.0},{},{},{},{},{:.0},{:.0},{},{},{}", st.run, class_name(row.class), cfg.ember, row.seed, row.time, row.won, row.died, row.level, row.kills, row.taken, row.healed, row.gold, row.bosses, row.relics);
    }
    println!("[bot] run {} {}: {} after {:.0}s, level {}, {} kills, took {:.0}, healed {:.0}", st.run, class_name(row.class), if row.won { "WON" } else if row.died { "fell" } else { "reached the time cap" }, row.time, row.level, row.kills, row.taken, row.healed);
    st.rows.push(row);
}

fn finish(cfg: &BotCfg, st: &BotState, exit: &mut EventWriter<AppExit>) {
    let mut out = format!("Emberwild balance bot: {} runs, Ember {}, run cap {} minutes\n\n", st.rows.len(), cfg.ember, cfg.minutes);
    out += "class     runs  win%  died%  avg survived  avg level  kills/min  dmg taken/min  healed/min  heal:damage  gold\n";
    for c in &cfg.classes {
        let r: Vec<&Row> = st.rows.iter().filter(|r| r.class == *c).collect();
        if r.is_empty() { continue; }
        let n = r.len() as f32;
        let avg = |f: &dyn Fn(&Row) -> f32| r.iter().map(|x| f(x)).sum::<f32>() / n;
        let mins = avg(&|x| x.time) / 60.0;
        let (tk, th) = (avg(&|x| x.taken), avg(&|x| x.healed));
        out += &format!("{:<9} {:>4}  {:>4.0}  {:>5.0}  {:>8.1} min  {:>9.1}  {:>9.1}  {:>13.1}  {:>10.1}  {:>11.2}  {:>4.0}\n", class_name(*c), r.len(), avg(&|x| x.won as i32 as f32) * 100.0, avg(&|x| x.died as i32 as f32) * 100.0, mins, avg(&|x| x.level as f32),
            avg(&|x| x.kills as f32) / mins.max(0.01), tk / mins.max(0.01), th / mins.max(0.01), th / tk.max(1.0), avg(&|x| x.gold as f32));
    }
    print!("\n{}", out);
    let _ = std::fs::write(format!("{}-summary.txt", cfg.out), out);
    exit.send(AppExit::Success);
}
