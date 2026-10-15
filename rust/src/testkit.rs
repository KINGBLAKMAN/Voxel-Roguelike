//! Scripted test runs for headless checks: `EW_TEST="seed=5;class=mage;time=0.3;yaw=1.2;shot=90:/tmp/a.png;exit=120"`.
//! Steps run on fixed frame numbers; `act=<frame>:<action>` triggers gameplay actions once the game layer exists.
use crate::models::Class;
use crate::player::Player;
use crate::scene::DayCycle;
use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};

#[derive(Resource, Clone)]
pub struct TestCfg { pub seed: u32, pub class: Class, pub time: Option<f32>, pub yaw: Option<f32>, pub pitch: Option<f32>, pub shots: Vec<(u32, String)>, pub acts: Vec<(u32, String)>, pub exit: u32, pub frame: u32, pub menu: bool, pub autopick: bool, pub camp: bool, pub orbit: f32, pub dist: f32, pub held: Vec<KeyCode>, pub no_foliage: bool }

/// scripted tests only exist in dev builds
pub fn active() -> bool { cfg!(feature = "dev") && std::env::var("EW_TEST").is_ok() }
impl TestCfg {
    pub fn from_env() -> Option<Self> {
        if !cfg!(feature = "dev") { return None; }
        let s = std::env::var("EW_TEST").ok()?;
        let mut t = TestCfg { seed: 12345, class: Class::Warrior, time: None, yaw: None, pitch: None, shots: vec![], acts: vec![], exit: 200, frame: 0, menu: false, autopick: false, camp: false, orbit: 0.0, dist: 6.6, held: vec![], no_foliage: false };
        for kv in s.split(';') {
            let (k, v) = kv.split_once('=').unwrap_or((kv, ""));
            match k {
                "seed" => t.seed = v.parse().unwrap_or(1),
                "class" => t.class = match v { "ranger" => Class::Ranger, "mage" => Class::Mage, "priest" => Class::Priest, _ => Class::Warrior },
                "time" => t.time = v.parse().ok(),
                "yaw" => t.yaw = v.parse().ok(),
                "pitch" => t.pitch = v.parse().ok(),
                "shot" => if let Some((f, p)) = v.split_once(':') { t.shots.push((f.parse().unwrap_or(60), p.to_string())) },
                "act" => if let Some((f, a)) = v.split_once(':') { t.acts.push((f.parse().unwrap_or(10), a.to_string())) },
                "exit" => t.exit = v.parse().unwrap_or(200),
                "menu" => t.menu = true,
                "autopick" => t.autopick = true,
                "camp" => t.camp = true,
                "nofoliage" => t.no_foliage = true,
                "orbit" => t.orbit = v.parse().unwrap_or(0.0),
                "dist" => t.dist = v.parse().unwrap_or(6.6),
                _ => {}
            }
        }
        Some(t)
    }
}

#[allow(clippy::too_many_arguments)]
pub fn run_test(mut c: Commands, mut t: ResMut<TestCfg>, mut ui: ResMut<crate::hud::UiState>, mut keys: ResMut<ButtonInput<KeyCode>>, mut cr: ResMut<crate::player::CamRig>, mut win: Query<&mut Window>, mut p: ResMut<Player>, mut dc: ResMut<DayCycle>, mut ex: EventWriter<AppExit>) {
    t.frame += 1;
    let f = t.frame;
    if f <= 2 {
        if let Some(v) = t.time { dc.time = v; }
        if let Some(v) = t.yaw { p.yaw = v; }
        if let Some(v) = t.pitch { p.pitch = v; }
        cr.orbit = t.orbit; cr.max_d = t.dist;
    }
    keys.release(KeyCode::KeyQ); keys.release(KeyCode::F1);
    for k in t.held.clone() { keys.press(k); }
    for (af, a) in t.acts.clone() {
        if af != f { continue; }
        match a.as_str() {
            "host" => ui.req_host = true, "join" => ui.req_join = Some("127.0.0.1".into()), "start" => ui.start = Some(t.class),
            // animation stills: freeze attack progress, pick the combo step, swap weapons
            "swap" => keys.press(KeyCode::KeyQ),
            "f1" => keys.press(KeyCode::F1),
            "tab" => ui.screen = crate::hud::Screen::Character,
            // hold movement keys: run, sprint, strafe, backpedal; "stop" lets go
            "w" => t.held.push(KeyCode::KeyW), "s" => t.held.push(KeyCode::KeyS), "d" => t.held.push(KeyCode::KeyD), "shift" => t.held.push(KeyCode::ShiftLeft),
            "stop" => { for k in t.held.drain(..) { keys.release(k); } }
            a if a.starts_with("cursor") => { if let Some((x, y)) = a[6..].split_once(',') { if let (Ok(x), Ok(y), Ok(mut w)) = (x.parse::<f32>(), y.parse::<f32>(), win.get_single_mut()) { w.set_cursor_position(Some(Vec2::new(x, y))); } } }
            "noft" => crate::player::FREEZE.store(f32::NAN.to_bits(), std::sync::atomic::Ordering::Relaxed),
            a if a.starts_with("ft") => { if let Ok(v) = a[2..].parse::<f32>() { crate::player::FREEZE.store(v.to_bits(), std::sync::atomic::Ordering::Relaxed); } }
            a if a.starts_with("cb") => { p.swing_kind = a[2..].parse().unwrap_or(0); }
            _ => {}
        }
    }
    for (sf, path) in t.shots.clone() { if sf == f { c.spawn(Screenshot::primary_window()).observe(save_to_disk(path)); } }
    if f >= t.exit { ex.send(AppExit::Success); }
}
