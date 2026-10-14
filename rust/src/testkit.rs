//! Scripted test runs for headless checks: `EW_TEST="seed=5;class=mage;time=0.3;yaw=1.2;shot=90:/tmp/a.png;exit=120"`.
//! Steps run on fixed frame numbers; `act=<frame>:<action>` triggers gameplay actions once the game layer exists.
use crate::models::Class;
use crate::player::Player;
use crate::scene::DayCycle;
use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};

#[derive(Resource, Clone)]
pub struct TestCfg { pub seed: u32, pub class: Class, pub time: Option<f32>, pub yaw: Option<f32>, pub pitch: Option<f32>, pub shots: Vec<(u32, String)>, pub acts: Vec<(u32, String)>, pub exit: u32, pub frame: u32, pub menu: bool, pub autopick: bool }

impl TestCfg {
    pub fn from_env() -> Option<Self> {
        let s = std::env::var("EW_TEST").ok()?;
        let mut t = TestCfg { seed: 12345, class: Class::Warrior, time: None, yaw: None, pitch: None, shots: vec![], acts: vec![], exit: 200, frame: 0, menu: false, autopick: false };
        for kv in s.split(';') {
            let Some((k, v)) = kv.split_once('=') else { continue };
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
                _ => {}
            }
        }
        Some(t)
    }
}

pub fn run_test(mut c: Commands, mut t: ResMut<TestCfg>, mut p: ResMut<Player>, mut dc: ResMut<DayCycle>, mut ex: EventWriter<AppExit>) {
    t.frame += 1;
    let f = t.frame;
    if f <= 2 {
        if let Some(v) = t.time { dc.time = v; }
        if let Some(v) = t.yaw { p.yaw = v; }
        if let Some(v) = t.pitch { p.pitch = v; }
    }
    for (sf, path) in t.shots.clone() { if sf == f { c.spawn(Screenshot::primary_window()).observe(save_to_disk(path)); } }
    if f >= t.exit { ex.send(AppExit::Success); }
}
