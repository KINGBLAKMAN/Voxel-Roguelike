// Hide the console window on Windows release builds.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
//! Emberwild: a voxel horde-survival roguelike.

mod abilities;
mod audio;
mod augments;
mod combat;
mod director;
mod enemies;
mod fx;
mod game;
mod mesher;
mod models;
mod noise;
mod physics;
mod player;
mod relics;
mod scene;
mod testkit;
mod world;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::window::{PresentMode, WindowMode};
use game::*;
use models::*;
use player::*;
use scene::*;

/// tunables the renderer and streaming read; `focus` overrides the streaming centre (menus, tests)
#[derive(Resource)]
pub struct GameCfg { pub view: i32, pub chunk_budget: usize, pub time_runs: bool, pub focus: Option<Vec3> }
impl Default for GameCfg { fn default() -> Self { GameCfg { view: 8, chunk_budget: 3, time_runs: true, focus: None } } }

/// the simulation is frozen (menus, level-up cards)
#[derive(Resource, Default)]
pub struct Paused(pub bool);

fn main() {
    let test = testkit::TestCfg::from_env();
    let fullscreen = test.is_none() && std::env::var("EMBERWILD_WINDOWED").is_err();
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Emberwild".into(), resolution: (1280.0f32, 720.0f32).into(), present_mode: PresentMode::AutoVsync,
            mode: if fullscreen { WindowMode::BorderlessFullscreen(MonitorSelection::Primary) } else { WindowMode::Windowed }, ..default()
        }),
        ..default()
    }))
    .init_resource::<GameCfg>()
    .init_resource::<Palette>()
    .init_resource::<Player>()
    .init_resource::<Controls>()
    .init_resource::<CamRig>()
    .init_resource::<Paused>()
    .init_resource::<Game>()
    .init_resource::<audio::Sfx>()
    .add_plugins((ScenePlugin, fx::FxPlugin))
    .add_systems(Startup, (start_world, audio::setup_audio))
    .add_systems(Update, (cursor_lock, mouse_look, game_tick, audio::play_sfx).chain());
    if let Some(t) = test { app.insert_resource(t).add_systems(Update, testkit::run_test.before(game_tick)); }
    app.run();
}

fn start_world(mut c: Commands, mut pal: ResMut<Palette>, mut mats: ResMut<Assets<StandardMaterial>>, mut p: ResMut<Player>, mut ctl: ResMut<Controls>, test: Option<Res<testkit::TestCfg>>) {
    let seed = test.as_ref().map(|t| t.seed).unwrap_or_else(|| rand::random::<u32>() % 1_000_000_000);
    let mut w = world::Voxels::new(seed);
    let sp = w.spawn_point();
    p.cls = test.as_ref().map(|t| t.class).unwrap_or(Class::Warrior);
    p.body.pos = sp; p.vis_y = sp.y;
    build_player(&mut c, &mut pal, &mut mats, &mut p);
    ctl.want_lock = true;
    c.insert_resource(w);
}

#[derive(SystemParam)]
pub struct Sim<'w, 's> {
    c: Commands<'w, 's>, g: ResMut<'w, Game>, w: Option<ResMut<'w, world::Voxels>>, p: ResMut<'w, Player>, pal: ResMut<'w, Palette>,
    mats: ResMut<'w, Assets<StandardMaterial>>, meshes: ResMut<'w, Assets<Mesh>>, fx: ResMut<'w, fx::Fx>, q: Query<'w, 's, &'static mut Transform>,
    cam: ResMut<'w, CamRig>, dc: Res<'w, DayCycle>, sfx: ResMut<'w, audio::Sfx>,
}

/// one frame of the game: input, the player, weapons, enemies, projectiles, zones, pickups, the director, relics, camera
fn game_tick(mut s: Sim, time: Res<Time>, keys: Res<ButtonInput<KeyCode>>, mouse: Res<ButtonInput<MouseButton>>, paused: Res<Paused>, ctl: Res<Controls>,
    mut camq: Query<(Entity, &mut Projection), With<MainCam>>, mut test: Option<ResMut<testkit::TestCfg>>) {
    let s = &mut s;
    let Some(w) = s.w.as_mut() else { return };
    let w: &mut world::Voxels = w;
    let raw = time.delta_secs().min(0.05);
    let Ok((ce, mut proj)) = camq.get_single_mut() else { return };
    if paused.0 { return; }
    let mut dt = raw;
    if s.g.hitstop > 0.0 { s.g.hitstop -= raw; dt = raw * 0.08; }
    let mut cx = Ctx { c: &mut s.c, g: &mut s.g, w, p: &mut s.p, pal: &mut s.pal, mats: &mut s.mats, meshes: &mut s.meshes, fx: &mut s.fx, q: &mut s.q, cam: &mut s.cam, dc: &s.dc, sfx: &mut s.sfx, dt };
    // scripted test actions
    let mut hold = false;
    if let Some(t) = test.as_mut() {
        hold = t.acts.iter().any(|(f, a)| a == "hold" && *f <= t.frame);
        let now: Vec<String> = t.acts.iter().filter(|(f, _)| *f == t.frame).map(|(_, a)| a.clone()).collect();
        for a in now { test_act(&mut cx, &a); }
        if t.acts.iter().any(|(f, a)| a == "god" && *f <= t.frame) { cx.p.hp = cx.p.max_hp; }
    }
    cx.g.run_t += dt;
    let lmb = (mouse.pressed(MouseButton::Left) && ctl.locked) || hold;
    cx.p.firing = lmb;
    // the player
    let st = cx.g.st;
    move_player(cx.w, cx.p, &keys, dt, st.speed * if cx.g.buff("haste") > 0.0 { 1.3 } else { 1.0 }, 1.0, st.stam_max);
    cx.p.invuln = (cx.p.invuln - dt).max(0.0);
    cx.p.hit_t = (cx.p.hit_t - dt).max(0.0);
    cx.p.cast_kick = (cx.p.cast_kick - dt * 4.0).max(0.0);
    for v in cx.g.buffs.values_mut() { *v -= dt; }
    cx.g.buffs.retain(|_, v| *v > 0.0);
    for c in cx.g.cds.iter_mut() { *c = (*c - dt).max(0.0); }
    cx.g.flash = (cx.g.flash - dt).max(0.0);
    if !cx.p.dead && st.regen > 0.0 { cx.g.regen_acc += dt; if cx.g.regen_acc >= 1.0 { cx.g.regen_acc -= 1.0; cx.heal(st.regen, true); } }
    if keys.just_pressed(KeyCode::KeyQ) { cx.p.widx = 1 - cx.p.widx; cx.p.charging = false; attach_weapon(cx.c, cx.pal, cx.mats, cx.p); cx.play("swap"); }
    for (i, k) in [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4].into_iter().enumerate() { if keys.just_pressed(k) { abilities::use_ability(&mut cx, i); } }
    update_aim(&mut cx, ce);
    combat::update_primary(&mut cx, lmb);
    enemies::update_enemies(&mut cx);
    combat::update_projs(&mut cx);
    abilities::update_abilities(&mut cx);
    combat::update_zones(&mut cx);
    abilities::update_traps(&mut cx);
    update_pickups(&mut cx);
    enemies::update_corpses(&mut cx);
    run_timers(&mut cx);
    director::update_director(&mut cx);
    director::update_pois(&mut cx);
    relics::update_relics(&mut cx);
    augments::update_augments(&mut cx);
    if keys.just_pressed(KeyCode::KeyE) { if let Some((_, it)) = director::find_interact(&mut cx) { director::interact(&mut cx, it); } }
    // until the HUD drains them, keep the transient UI queues bounded
    cx.g.popups.clear();
    if cx.g.toasts.len() > 6 { cx.g.toasts.remove(0); }
    animate_player(cx.p, cx.q, dt);
    let mut ct = *cx.q.get(ce).unwrap();
    let mut rng = noise::mulberry32((time.elapsed_secs() * 1000.0) as u32);
    update_camera(cx.w, cx.p, cx.cam, &mut ct, &mut proj, raw, &mut rng);
    *cx.q.get_mut(ce).unwrap() = ct;
}

/// what the crosshair points at: the first enemy or block along the camera ray
fn update_aim(cx: &mut Ctx, ce: Entity) {
    let Ok(ct) = cx.q.get(ce) else { return };
    let (o, d) = (ct.translation, ct.forward().as_vec3());
    let mut best = world::voxel_ray(cx.w, o, d, 90.0).map(|h| h.1).unwrap_or(90.0);
    for e in cx.g.enemies.iter() {
        if e.dead || e.t.passive { continue; }
        let c = e.body.pos + Vec3::Y * e.body.h * 0.5;
        let t = (c - o).dot(d);
        if t < 0.0 || t > best { continue; }
        let r = e.body.w + 0.3 + e.body.h * 0.3;
        if (o + d * t).distance_squared(c) < r * r { best = t; }
    }
    // never aim behind the shoulder: push the point past the player
    let pp = cx.p.body.pos + Vec3::Y * 1.4;
    let t0 = (pp - o).dot(d) + 1.5;
    cx.p.aim_dist = best.max(t0);
    cx.p.aim_pt = o + d * cx.p.aim_dist;
}

fn test_act(cx: &mut Ctx, a: &str) {
    let (k, v) = a.split_once('=').unwrap_or((a, ""));
    match k {
        "horde" | "hordefar" => {
            let n: u32 = v.parse().unwrap_or(10); let (r0, r1) = if k == "horde" { (7.0, 12.0) } else { (15.0, 21.0) };
            let pp = cx.p.body.pos; let f = cx.p.fwd();
            for i in 0..n {
                let an = (i as f32 / n as f32 - 0.5) * 2.2 + f.x.atan2(f.z);
                let r = cx.rr(r0, r1);
                if let Some(sp) = cx.w.surface_spot(pp.x + an.sin() * r, pp.z + an.cos() * r) {
                    let k = cx.pick(&[enemies::Kind::Zombie, enemies::Kind::Zombie, enemies::Kind::Skeleton, enemies::Kind::Spider, enemies::Kind::Slime, enemies::Kind::Brute]);
                    enemies::spawn_enemy(cx, k, sp, enemies::SpawnOpts { force: true, ..default() });
                }
            }
        }
        "pitch" => { cx.p.pitch = v.parse().unwrap_or(0.0); }
        "yaw" => { cx.p.yaw = v.parse().unwrap_or(0.0); }
        "mod" => { *cx.g.mods.entry(v.to_string()).or_insert(0) += 1; }
        "minutes" => { cx.g.run_t = v.parse::<f32>().unwrap_or(5.0) * 60.0; }
        "weapon" => { cx.p.widx = v.parse().unwrap_or(0); attach_weapon(cx.c, cx.pal, cx.mats, cx.p); }
        "chest" => { let p = cx.p.body.pos + cx.p.fwd() * 4.0; director::drop_chest(cx, p, false); }
        "ab" => { let i: usize = v.parse().unwrap_or(0); cx.g.cds[i] = 0.0; abilities::use_ability(cx, i); }
        "bomb" => { let p = cx.p.body.pos + cx.p.fwd() * 2.0; cx.drop_pick(p, PickKind::Bomb, 0.0); }
        _ => {}
    }
}
