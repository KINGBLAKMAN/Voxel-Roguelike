// Hide the console window on Windows release builds.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
//! Emberwild: a voxel horde-survival roguelike.

mod abilities;
mod anim;
mod audio;
mod camp;
mod charsheet;
mod icons;
mod augments;
mod bot;
mod combat;
mod devmenu;
mod director;
mod enemies;
mod events;
mod fx;
mod game;
mod horde;
mod hud;
mod meta;
mod net;
mod mesher;
mod models;
mod noise;
mod physics;
mod player;
mod progression;
mod relics;
mod scene;
mod settings;
mod testkit;
mod textures;
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
pub struct GameCfg { pub view: i32, pub chunk_budget: usize, pub time_runs: bool, pub focus: Option<Vec3>, pub damage_numbers: bool, pub no_foliage: bool }
impl Default for GameCfg { fn default() -> Self { GameCfg { view: 8, chunk_budget: 3, time_runs: true, focus: None, damage_numbers: true, no_foliage: false } } }

/// the simulation is frozen (menus, level-up cards)
#[derive(Resource, Default)]
pub struct Paused(pub bool);

/// write panics to a crash log next to the save, since the Windows build has no console to show them
fn install_crash_log() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let bt = std::backtrace::Backtrace::force_capture();
        let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let body = format!("Emberwild {} crashed (unix time {})\nOS: {} {}\n\n{}\n\nBacktrace:\n{}\n", env!("CARGO_PKG_VERSION"), stamp, std::env::consts::OS, std::env::consts::ARCH, info, bt);
        let dir = meta::data_dir(); let _ = std::fs::create_dir_all(&dir);
        let path = dir.join(format!("crash-{}.log", stamp));
        if std::fs::write(&path, &body).is_ok() { eprintln!("crash log written to {}", path.display()); }
        default(info);
    }));
}

fn main() {
    install_crash_log();
    let bot = bot::from_env();
    let test = testkit::TestCfg::from_env();
    let fullscreen = test.is_none() && bot.is_none() && std::env::var("EMBERWILD_WINDOWED").is_err();
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: if bot.is_some() { "Emberwild balance bot".into() } else if cfg!(feature = "dev") { "Emberwild (dev build)".into() } else { "Emberwild".into() },
            resolution: if bot.is_some() { (640.0f32, 360.0f32).into() } else { (1280.0f32, 720.0f32).into() },
            present_mode: if bot.is_some() { PresentMode::AutoNoVsync } else { PresentMode::AutoVsync },
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
    .init_resource::<hud::UiState>()
    .insert_resource(meta::Save::load())
    .init_resource::<settings::PadState>()
    .add_plugins((ScenePlugin, fx::FxPlugin))
    .add_systems(Startup, (start_world, audio::setup_audio, audio::setup_music, (hud::setup_hud, camp::setup_camp, settings::setup_settings).chain()))
    .add_systems(Update, (settings::gamepad_bridge, settings::apply_settings, hud::ui_flow, run_control.after(scene::stream_chunks), cursor_lock, mouse_look, game_tick, hud::update_hud, net_day, hud::button_hover, camp::update_camp, camp::update_shop, camp::update_join, settings::update_settings_ui, hud::menu_camera, audio::play_sfx, audio::update_music).chain());
    app.add_systems(Update, player::weapon_trails.after(game_tick));
    app.init_resource::<devmenu::DevQueue>().init_resource::<devmenu::DevMenu>()
        .add_systems(Startup, devmenu::setup_dev.after(hud::setup_hud))
        .add_systems(Update, devmenu::dev_menu.after(hud::ui_flow).before(cursor_lock));
    app.init_resource::<icons::Icons>().init_resource::<charsheet::Panel>()
        .add_systems(Startup, (icons::build_icons, charsheet::setup_tips))
        .add_systems(Update, (charsheet::update_panel, charsheet::update_tips).chain().after(hud::update_hud));
    if let Some(t) = test { app.insert_resource(t).add_systems(Update, testkit::run_test.before(game_tick)); }
    if let Some(b) = bot { app.insert_resource(b).init_resource::<bot::BotState>().add_systems(Update, bot::bot_drive.after(settings::gamepad_bridge).before(hud::ui_flow)); }
    app.run();
}

fn start_world(mut c: Commands, mut g: ResMut<Game>, mut pal: ResMut<Palette>, mut mats: ResMut<Assets<StandardMaterial>>, mut p: ResMut<Player>, mut ctl: ResMut<Controls>, mut ui: ResMut<hud::UiState>, test: Option<Res<testkit::TestCfg>>) {
    let seed = test.as_ref().map(|t| t.seed).unwrap_or_else(|| rand::random::<u32>() % 1_000_000_000);
    if test.as_ref().map(|t| t.no_foliage).unwrap_or(false) { c.queue(|w: &mut bevy::prelude::World| w.resource_mut::<GameCfg>().no_foliage = true); }
    let map = test.as_ref().map(|t| t.map).unwrap_or_default();
    let mut w = world::Voxels::new(seed, map);
    let sp = w.spawn_point();
    p.body.pos = sp; p.vis_y = sp.y;
    // scripted tests skip the title screen
    if let Some(t) = test.as_ref() { if t.camp { ui.screen = if t.class == Class::Mage { hud::Screen::Settings } else { hud::Screen::Camp }; } if !t.menu && !t.camp { ui.screen = hud::Screen::Playing; begin_player(&mut c, &mut pal, &mut mats, &mut p, t.class); ctl.want_lock = true; } }
    // a test on a later map plays as if the run got there
    g.map = map; g.stage = match map { world::MapKind::Forest => 0, world::MapKind::Volcanic => 2, m => { g.route = m; 1 } };
    c.insert_resource(w);
}
fn class_stats(c: Class) -> (f32, f32) { match c { Class::Ranger => (100.0, 1.08), Class::Mage => (85.0, 1.0), Class::Warrior => (140.0, 0.98), Class::Priest => (110.0, 1.0) } }
fn begin_player(c: &mut Commands, pal: &mut Palette, mats: &mut Assets<StandardMaterial>, p: &mut Player, cls: Class) {
    let pos = p.body.pos;
    let old = p.rig.root.take();
    if let Some(r) = old { c.entity(r).despawn_recursive(); }
    *p = Player::default();
    p.cls = cls; p.body.pos = pos; p.vis_y = pos.y;
    let (hp, _) = class_stats(cls); p.hp = hp; p.max_hp = hp; p.ghost_hp = hp;
    build_player(c, pal, mats, p);
    p.spawned = true;
}

/// starting a run from the menu, and tearing one down to start again
#[allow(clippy::too_many_arguments)]
fn run_control(link: Option<Res<net::Link>>, save: Res<meta::Save>, mut c: Commands, mut ui: ResMut<hud::UiState>, mut g: ResMut<Game>, mut p: ResMut<Player>, mut pal: ResMut<Palette>, mut mats: ResMut<Assets<StandardMaterial>>,
    mut fx: ResMut<fx::Fx>, w: Option<ResMut<world::Voxels>>, mut dc: ResMut<DayCycle>, mut ctl: ResMut<Controls>, mut sfx: ResMut<audio::Sfx>) {
    if ui.req_host && link.is_none() {
        ui.req_host = false;
        match net::Link::host() { Ok(l) => { ui.net_msg = l.status.lock().unwrap().clone(); c.insert_resource(l); g.net.mode = net::Mode::Host; g.net.my_id = 1; } Err(e) => ui.net_msg = format!("Could not host: {}", e) }
    }
    if let Some(addr) = ui.req_join.take() {
        if link.is_none() { match net::Link::join(&addr) { Ok(l) => { ui.net_msg = l.status.lock().unwrap().clone(); c.insert_resource(l); g.net.mode = net::Mode::Client; } Err(e) => ui.net_msg = format!("Could not connect: {}", e) } }
    }
    if let Some(l) = link.as_ref() { let s = l.status.lock().unwrap().clone(); if ui.net_msg != s && !ui.net_msg.starts_with("Could") { ui.net_msg = s; } }
    // joining a host means playing in the host's world
    let join_seed = g.net.seed.take();
    if join_seed.is_some() { ui.restart = true; }
    if let Some(cls) = ui.start.take() {
        begin_player(&mut c, &mut pal, &mut mats, &mut p, cls);
        meta::apply_start(&save, &mut g, &mut p);
        g.route = match ui.route { 1 => world::MapKind::Swamp, 2 => world::MapKind::Desert, _ => if rand::random::<bool>() { world::MapKind::Swamp } else { world::MapKind::Desert } };
        ui.screen = hud::Screen::Playing; ui.dead_t = 0.0; ui.ghost_hp = 1.0;
        ctl.want_lock = true; ctl.relock = true;
        sfx.play("horn");
    }
    if let (Some((stage, map, seed)), false) = (g.pending_stage, ui.restart) {
        g.pending_stage = None;
        next_stage(&mut c, &mut g, &mut p, &mut fx, w, &mut dc, stage, map, seed);
        return;
    }
    if !ui.restart { return; }
    ui.restart = false;
    let g = &mut *g;
    clear_run_world(&mut c, g, &mut fx);
    let netstate = std::mem::take(&mut g.net);
    *g = Game::default();
    g.net = netstate; g.net.out.clear();
    if let Some(r) = p.rig.root.take() { c.entity(r).despawn_recursive(); }
    // a fresh world
    if let Some(w) = w { clear_chunks(&mut c, &w); }
    let seed = join_seed.unwrap_or_else(|| rand::random::<u32>() % 1_000_000_000);
    // a client joining mid-run lands on the host's current map
    let (map, stage) = g.net.join_map.take().filter(|_| join_seed.is_some()).unwrap_or((world::MapKind::Forest, 0));
    g.map = map; g.stage = stage;
    let mut nw = world::Voxels::new(seed, map);
    if let Some(l) = link.as_ref() { if l.host { for id in l.peer_ids() { l.send(Some(id), &net::Msg::Welcome { id, seed, map: map.id(), stage: 0 }); } } }
    let sp = nw.spawn_point();
    *p = Player::default(); p.body.pos = sp; p.vis_y = sp.y;
    c.insert_resource(nw);
    *dc = DayCycle::default();
    ui.screen = hud::Screen::Menu; ui.dead_t = 0.0; ui.toast = None;
}

/// despawn everything a stage put into the world (monsters, corpses, loot, projectiles, zones, chests, the portal)
fn clear_run_world(c: &mut Commands, g: &mut Game, fx: &mut fx::Fx) {
    let mut ents: Vec<Entity> = vec![];
    for e in g.enemies.drain(..) { ents.extend(e.rig.root); ents.extend(e.ice); }
    ents.extend(g.corpses.drain(..).map(|x| x.root));
    ents.extend(g.pickups.drain(..).map(|x| x.e));
    for pr in g.projs.drain(..) { ents.extend(pr.e); }
    for z in g.zones.iter_mut() { ents.append(&mut z.ents); }
    g.zones.clear();
    ents.extend(g.traps.drain(..).map(|t| t.root));
    ents.extend(g.dir.chests.drain(..).map(|c| c.root));
    if let Some(pt) = g.dir.portal.take() { ents.push(pt.root); }
    for l in g.rel.objs.values() { ents.extend(l.iter().copied()); }
    g.rel.objs.clear();
    ents.extend(g.rel.decoys.drain(..).map(|d| d.e));
    ents.extend(g.rel.stuck.drain(..).map(|s| s.e));
    if let Some(ch) = g.ab.charge.take() { ents.push(ch.wedge()); }
    for t in fx.tweens.drain(..) { ents.push(t.e); }
    for d in fx.debris.drain(..) { c.entity(d.e).insert(Visibility::Hidden); fx.debris_free.push(d.e); }
    fx.add.clear(); fx.blend.clear();
    for e in ents { if let Some(mut ec) = c.get_entity(e) { ec.despawn_recursive(); } }
}
fn clear_chunks(c: &mut Commands, w: &world::Voxels) {
    for ch in w.chunks.values() { for e in [ch.entity, ch.fol_entity].into_iter().flatten() { if let Some(mut ec) = c.get_entity(e) { ec.despawn_recursive(); } } }
}
/// carry the party to the next map: a fresh world, the same character, and the clock keeps its intensity
#[allow(clippy::too_many_arguments)]
fn next_stage(c: &mut Commands, g: &mut Game, p: &mut Player, fx: &mut fx::Fx, w: Option<ResMut<world::Voxels>>, dc: &mut DayCycle, stage: u32, map: world::MapKind, seed: u32) {
    clear_run_world(c, g, fx);
    if let Some(w) = w.as_ref() { clear_chunks(c, w); }
    g.timers.clear(); g.boss = None; g.popups.clear();
    g.dir = Default::default(); g.ev = Default::default();
    g.stage = stage; g.stage_t = 0.0; g.map = map;
    let mut nw = world::Voxels::new(seed, map);
    let sp = nw.spawn_point();
    p.body.pos = sp; p.body.vel = Vec3::ZERO; p.vis_y = sp.y; p.dash = None; p.leap = None;
    if p.dead { p.dead = false; p.hp = p.max_hp * 0.5; }
    c.insert_resource(nw);
    // each new map starts at dawn of the next day
    dc.time = dc.time.floor() + 1.02;
    let (k, _, _) = director::stage_boss(map);
    g.toasts.push((format!("Stage {}: {}", stage + 1, map.name()), format!("Survive five minutes, then face the {}", enemies::etype(k).name), 4.5));
    g.flash = 0.6;
}

#[derive(SystemParam)]
pub struct Sim<'w, 's> {
    c: Commands<'w, 's>, g: ResMut<'w, Game>, w: Option<ResMut<'w, world::Voxels>>, p: ResMut<'w, Player>, pal: ResMut<'w, Palette>,
    mats: ResMut<'w, Assets<StandardMaterial>>, meshes: ResMut<'w, Assets<Mesh>>, fx: ResMut<'w, fx::Fx>, q: Query<'w, 's, &'static mut Transform>,
    cam: ResMut<'w, CamRig>, dc: Res<'w, DayCycle>, sfx: ResMut<'w, audio::Sfx>,
}

/// one frame of the game: input, the player, weapons, enemies, projectiles, zones, pickups, the director, relics, camera
#[allow(clippy::too_many_arguments)]
fn game_tick(mut s: Sim, time: Res<Time>, keys_in: Res<ButtonInput<KeyCode>>, mouse: Res<ButtonInput<MouseButton>>, paused: Res<Paused>, ctl: Res<Controls>,
    mut camq: Query<(Entity, &mut Projection), With<MainCam>>, mut test: Option<ResMut<testkit::TestCfg>>, ui: Res<hud::UiState>, link: Option<Res<net::Link>>, bot: Option<Res<bot::BotCfg>>, mut devq: Option<ResMut<devmenu::DevQueue>>) {
    let s = &mut s;
    let Ok((ce, mut proj)) = camq.get_single_mut() else { return };
    if paused.0 && link.is_none() { return; }
    // the balance bot runs several fixed steps per frame
    let (steps, raw) = match bot.as_ref() { Some(b) => (b.turbo, 1.0 / 30.0), None => (1, time.delta_secs().min(0.05)) };
    for _step in 0..steps {
    let Some(w) = s.w.as_mut() else { return };
    let w: &mut world::Voxels = w;
    if paused.0 && link.is_none() { return; }
    let mut dt = raw;
    if s.g.hitstop > 0.0 { s.g.hitstop -= raw; dt = raw * 0.08; }
    let mut cx = Ctx { c: &mut s.c, g: &mut s.g, w, p: &mut s.p, pal: &mut s.pal, mats: &mut s.mats, meshes: &mut s.meshes, fx: &mut s.fx, q: &mut s.q, cam: &mut s.cam, dc: &s.dc, sfx: &mut s.sfx, dt };
    let link = link.as_deref();
    if let Some(l) = link { net::process(&mut cx, l); }
    if !cx.p.spawned { if let Some(l) = link { net::flush(&mut cx, l, raw); } return; }
    // co-op never pauses, so menus only take the player's hands off the controls
    let input = ui.screen == hud::Screen::Playing;
    let idle = ButtonInput::<KeyCode>::default();
    let keys: &ButtonInput<KeyCode> = if input { &keys_in } else { &idle };
    // scripted test actions
    let mut hold = false;
    if let Some(t) = test.as_mut() {
        hold = t.acts.iter().any(|(f, a)| a == "hold" && *f <= t.frame);
        let now: Vec<String> = t.acts.iter().filter(|(f, _)| *f == t.frame).map(|(_, a)| a.clone()).collect();
        for a in now { test_act(&mut cx, &a); }
        if t.acts.iter().any(|(f, a)| a == "god" && *f <= t.frame) { cx.p.hp = cx.p.max_hp; }
    }
    // dev menu cheats
    if let Some(q) = devq.as_mut() { for a in std::mem::take(&mut q.0) { test_act(&mut cx, &a); } }
    if cx.g.flags.contains_key("dev_god") { cx.p.hp = cx.p.max_hp; cx.p.dead = false; }
    if cx.g.flags.contains_key("dev_nocd") { cx.g.cds = [0.0; 4]; }
    cx.g.run_t += dt; cx.g.stage_t += dt;
    // guard and dodge, for every class
    let rmb = input && mouse.pressed(MouseButton::Right) && (ctl.locked || ctl.pad);
    if rmb && !cx.p.dead && cx.p.stam > 0.0 && cx.p.dash.is_none() && cx.p.leap.is_none() { if cx.p.blocking <= 0.0 { cx.play("swing"); } cx.p.blocking += dt; } else { cx.p.blocking = 0.0; }
    cx.p.dodge_cd -= dt;
    if keys.just_pressed(KeyCode::ControlLeft) || keys.just_pressed(KeyCode::AltLeft) { dodge(&mut cx); }
    let lmb = ((input && mouse.pressed(MouseButton::Left) && (ctl.locked || ctl.pad)) || hold) && cx.p.blocking <= 0.0;
    cx.p.firing = lmb;
    // the player
    let st = cx.g.st;
    let cspeed = class_stats(cx.p.cls).1;
    let guard = if cx.p.blocking > 0.0 { 0.5 } else { 1.0 };
    let slowed = guard * if cx.g.buff("web") > 0.0 || cx.g.buff("chill") > 0.0 { 0.55 } else { 1.0 };
    move_player(cx.w, cx.p, keys, dt, st.speed * slowed * if cx.g.buff("haste") > 0.0 { 1.3 } else { 1.0 }, cspeed, st.stam_max);
    liquid_fx(&mut cx);
    // hunger: food keeps you going; starving bleeds health
    if !cx.p.dead {
        cx.p.hunger = (cx.p.hunger - dt * 100.0 / 330.0).max(0.0);
        if cx.p.hunger <= 0.0 { let a = cx.g.flags.entry("starve").or_insert(0.0); *a += dt; if *a >= 1.0 { *a -= 1.0; cx.p.hp -= 1.0; if cx.p.hp <= 0.0 { cx.kill_player(); } } }
    }
    if keys.just_pressed(KeyCode::KeyF) { eat(&mut cx); }
    let cap = cx.p.max_hp * 0.04; cx.g.heal_budget = (cx.g.heal_budget + dt * cap).min(cap);
    cx.p.invuln = (cx.p.invuln - dt).max(0.0);
    cx.p.hit_t = (cx.p.hit_t - dt).max(0.0);
    cx.p.cast_kick = (cx.p.cast_kick - dt * 4.0).max(0.0);
    for v in cx.g.buffs.values_mut() { *v -= dt; }
    cx.g.buffs.retain(|_, v| *v > 0.0);
    for c in cx.g.cds.iter_mut() { *c = (*c - dt).max(0.0); }
    cx.g.flash = (cx.g.flash - dt).max(0.0);
    if !cx.p.dead && st.regen > 0.0 { cx.g.regen_acc += dt; if cx.g.regen_acc >= 1.0 { cx.g.regen_acc -= 1.0; cx.sustain(st.regen); } }
    if keys.just_pressed(KeyCode::KeyQ) { cx.p.widx = 1 - cx.p.widx; cx.p.charging = false; attach_weapon(cx.c, cx.pal, cx.mats, cx.p); cx.play("swap"); }
    for (i, k) in [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4].into_iter().enumerate() { if keys.just_pressed(k) { abilities::use_ability(&mut cx, i); } }
    update_aim(&mut cx, ce);
    combat::update_primary(&mut cx, lmb);
    if cx.g.net.client() { net::puppets(&mut cx); } else { enemies::update_enemies(&mut cx); }
    net::remotes(&mut cx);
    combat::update_projs(&mut cx);
    abilities::update_abilities(&mut cx);
    combat::update_zones(&mut cx);
    abilities::update_traps(&mut cx);
    update_pickups(&mut cx);
    enemies::update_corpses(&mut cx);
    run_timers(&mut cx);
    if !cx.g.net.client() {
        director::update_director(&mut cx);
        director::update_pois(&mut cx);
        events::update(&mut cx);
    }
    director::update_portal(&mut cx);
    if cx.g.net.host() { cx.g.net.day = Some(cx.dc.time); }
    relics::update_relics(&mut cx);
    augments::update_augments(&mut cx);
    let it = director::find_interact(&mut cx);
    let evp = events::interact_prompt(cx.g, cx.p.body.pos);
    cx.g.prompt = match (&it, &evp) { (Some((l, _)), _) => format!("[E]  {}", l), (None, Some((_, l))) => l.clone(), _ => String::new() };
    if cx.g.net.on() { if let Some(pr) = net::revive_prompt(&mut cx, input && keys_in.pressed(KeyCode::KeyE)) { cx.g.prompt = pr; } }
    if keys.just_pressed(KeyCode::KeyE) && !cx.g.net.client() {
        if let Some((_, it)) = it { director::interact(&mut cx, it); }
        else if let Some((i, _)) = evp { if events::use_event(&mut cx, i) { cx.g.flags.insert("open_shop", 1.0); } }
    }
    animate_player(cx.p, cx.q, dt);
    let mut ct = *cx.q.get(ce).unwrap();
    let mut rng = noise::mulberry32((time.elapsed_secs() * 1000.0) as u32);
    update_camera(cx.w, cx.p, cx.cam, &mut ct, &mut proj, raw, &mut rng);
    *cx.q.get_mut(ce).unwrap() = ct;
    if let Some(l) = link { net::flush(&mut cx, l, raw); }
    }
}
/// splashes, ripples and swim strokes in water; burns, sizzles and a hop out of lava
fn liquid_fx(cx: &mut Ctx) {
    let dt = cx.dt;
    let (pos, vel, depth, swim) = (cx.p.body.pos, cx.p.body.vel, cx.p.body.depth, cx.p.body.in_water);
    let lava = cx.w.kind.lava();
    let was = cx.g.flags.get("in_liquid").copied().unwrap_or(0.0) > 0.0;
    let wet = depth > 0.05;
    if wet { cx.g.flags.insert("in_liquid", 1.0); } else { cx.g.flags.remove("in_liquid"); }
    let surf = Vec3::new(pos.x, world::WATER_Y, pos.z);
    let (c1, c2) = if lava { (0xff8a2a, 0xffd27a) } else { (0x1c2a34, 0xffffff) };
    if wet && !was && vel.y < -2.5 {
        cx.play(if lava { "sizzle" } else { "splash" });
        let n = (12.0 + vel.y.abs() * 3.0) as usize;
        for _ in 0..n { let a = cx.rr(0.0, std::f32::consts::TAU); let sp = cx.rr(1.5, 4.0); let v = Vec3::new(a.cos() * sp, cx.rr(3.0, 7.0), a.sin() * sp); let l = cx.rr(0.4, 0.8); cx.fx.emit(lava, surf, v, l, 0.14, 0.05, fx::hexc(c1) * if lava { 2.5 } else { 1.0 }, 14.0, 0.5, 0.9); }
        cx.gfx().ring(surf - Vec3::Y * 0.1, c2, 0.3, 2.4, 0.6);
    }
    if !wet || cx.p.dead { return; }
    if lava {
        // lava burns hard and throws you back up, so a misstep hurts but doesn't trap you
        let t = cx.g.flags.entry("lava_t").or_insert(0.0); *t -= dt;
        if *t <= 0.0 {
            *t = 0.45;
            let d = 6.0 + cx.p.max_hp * 0.06;
            cx.hurt_player(d, None, 0.0); cx.p.body.vel.y = cx.p.body.vel.y.max(8.5);
            cx.play("sizzle"); cx.fx.burst(surf + Vec3::Y * 0.3, 0xff6a1a, 18, 4.0, 0.5, 0.25, 3.0, 4.0); cx.fx.smoke(surf, 0x3a2a24, 4, 1.2, 1.0, 1.0);
        }
        return;
    }
    // strokes and a wake while swimming; slow rings while treading water; ripples around wading legs
    let hs = Vec2::new(vel.x, vel.z).length();
    let t = cx.g.flags.entry("stroke_t").or_insert(0.0); *t -= dt;
    if *t <= 0.0 {
        if swim && hs > 1.5 {
            *t = 0.42; cx.play("stroke");
            let side = if cx.r() < 0.5 { 1.0 } else { -1.0 }; let r = cx.p.right() * side * 0.45 + cx.p.fwd() * 0.5;
            for _ in 0..6 { let v = Vec3::new(cx.rr(-1.0, 1.0), cx.rr(2.0, 3.6), cx.rr(-1.0, 1.0)); let l = cx.rr(0.35, 0.6); cx.fx.emit(false, surf + r, v, l, 0.1, 0.04, fx::hexc(c2), 14.0, 0.5, 0.85); }
            cx.gfx().ring(surf - Vec3::Y * 0.1, c1, 0.4, 1.4, 0.7);
        } else if hs > 1.0 { *t = 0.3; cx.gfx().ring(surf - Vec3::Y * 0.1, c1, 0.3, 1.1, 0.6); }
        else { *t = 1.1; cx.gfx().ring(surf - Vec3::Y * 0.1, c1, 0.3, 1.6, 1.2); }
    }
}
/// clients follow the host's time of day
fn net_day(g: Res<Game>, mut dc: ResMut<DayCycle>) { if g.net.client() { if let Some(t) = g.net.day { dc.time = t; } } }

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

/// a quick roll with a moment of invulnerability; costs stamina
fn dodge(cx: &mut Ctx) {
    if cx.p.dead || cx.p.dodge_cd > 0.0 || cx.p.stam < 25.0 || cx.p.dash.is_some() || cx.p.leap.is_some() { return; }
    let mut d = cx.p.move_dir; if d.length_squared() < 0.01 { d = cx.p.fwd(); }
    d.y = 0.0; let d = d.normalize();
    cx.p.dash = Some(player::Dash { dir: d, speed: 20.0, t: 0.24, kind: 3 });
    cx.p.roll_t = 0.3; cx.p.invuln = cx.p.invuln.max(0.35); cx.p.stam -= 25.0; cx.p.dodge_cd = 0.55; cx.p.blocking = 0.0;
    cx.g.flags.insert("dodge_at", cx.g.run_t); cx.g.flags.insert("perfect_used", 0.0);
    cx.play("dash");
    let pp = cx.p.body.pos; cx.fx.smoke(pp, 0xd8d0c0, 6, 1.5, 0.6, 0.8);
    abilities::afterimage(cx, 0xbfe8ff, 0.3);
}

fn eat(cx: &mut Ctx) {
    if cx.p.dead { return; }
    let meat = cx.g.food_meat > 0;
    if !meat && cx.g.food_berries == 0 { cx.toast("No food", "Hunt animals or open chests for food", 1.6); return; }
    if cx.p.hunger >= 99.0 && cx.p.hp >= cx.p.max_hp { cx.toast("Not hungry", "", 1.0); return; }
    let f = cx.g.st.food;
    if meat { cx.g.food_meat -= 1; cx.p.hunger = (cx.p.hunger + 38.0 * f).min(100.0); cx.heal(12.0 * f, false); } else { cx.g.food_berries -= 1; cx.p.hunger = (cx.p.hunger + 16.0 * f).min(100.0); cx.heal(4.0 * f, false); }
    cx.play("eat");
    let p = cx.p.body.pos + Vec3::Y * 1.6; cx.fx.burst(p, if meat { 0xd06a6a } else { 0xd23a4e }, 10, 2.0, 0.4, 0.2, 0.0, 8.0);
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
        "spawn" => {
            let (name, dist) = v.split_once(',').map(|(a, b)| (a, b.parse().unwrap_or(9.0))).unwrap_or((v, 9.0));
            if let Some(k) = horde::kind_by_name(name) {
                let p = cx.p.body.pos + cx.p.fwd() * dist;
                if let Some(s) = cx.w.surface_spot(p.x, p.z) { let boss = horde::is_boss_kind(k); if let Some(i) = enemies::spawn_enemy(cx, k, s, enemies::SpawnOpts { force: true, boss, no_rise: true, ..default() }) { if boss { cx.g.boss = Some(cx.g.enemies[i].id); } } }
            }
        }
        "event" => events::debug_start(cx, v.parse().unwrap_or(0)),
        "useev" => { if let Some((i, _)) = events::interact_prompt(cx.g, cx.p.body.pos) { if events::use_event(cx, i) { cx.g.flags.insert("open_shop", 1.0); } } else if let Some(i) = (0..cx.g.ev.list.len()).next() { let p = cx.g.ev.list[i].pos; cx.p.body.pos = p + Vec3::new(2.5, 0.5, 0.0); } }
        "pitch" => { cx.p.pitch = v.parse().unwrap_or(0.0); }
        "yaw" => { cx.p.yaw = v.parse().unwrap_or(0.0); }
        "lvl" => { let n = cx.g.xp_need(); cx.gain_xp(n); }
        "mod" => { *cx.g.mods.entry(v.to_string()).or_insert(0) += 1; }
        "minutes" => { cx.g.run_t = v.parse::<f32>().unwrap_or(5.0) * 60.0; }
        "stage_t" => { let t = v.parse::<f32>().unwrap_or(290.0); cx.g.run_t += (t - cx.g.stage_t).max(0.0); cx.g.stage_t = t; }
        "water" => {
            // drop into the nearest deep liquid
            let pp = cx.p.body.pos;
            'find: for r in (0..90).step_by(2) { for k in 0..24 { let a = k as f32 / 24.0 * std::f32::consts::TAU; let (x, z) = (pp.x + a.cos() * r as f32, pp.z + a.sin() * r as f32);
                if cx.w.top_at(x.floor() as i32, z.floor() as i32) <= world::SEA - 2 && world::in_bounds(x, z, 4.0) { cx.p.body.pos = Vec3::new(x, world::WATER_Y - 1.2, z); cx.p.vis_y = cx.p.body.pos.y; break 'find; } } }
        }
        "nextstage" => { director::advance_stage(cx); }
        "portal" => { let p = cx.p.body.pos + cx.p.fwd() * 5.0; let mut at = p; at.y = cx.ground(p.x, p.z, p.y + 2.0); let col = director::map_color(cx.g.stage_map(cx.g.stage + 1)); director::open_portal(cx, at, col); }
        "weapon" => { cx.p.widx = v.parse().unwrap_or(0); attach_weapon(cx.c, cx.pal, cx.mats, cx.p); }
        "chest" => { let p = cx.p.body.pos + cx.p.fwd() * 4.0; director::drop_chest(cx, p, false); }
        "ab" => { let i: usize = v.parse().unwrap_or(0); cx.g.cds[i] = 0.0; abilities::use_ability(cx, i); }
        "bomb" => { let p = cx.p.body.pos + cx.p.fwd() * 2.0; cx.drop_pick(p, PickKind::Bomb, 0.0); }
        "god" => { if cx.g.flags.remove("dev_god").is_none() { cx.g.flags.insert("dev_god", 1.0); } }
        "nocd" => { if cx.g.flags.remove("dev_nocd").is_none() { cx.g.flags.insert("dev_nocd", 1.0); } }
        "heal" => { cx.p.hp = cx.p.max_hp; cx.p.stam = cx.g.st.stam_max; cx.p.hunger = 100.0; }
        "lvl5" => { for _ in 0..5 { let n = cx.g.xp_need(); cx.gain_xp(n); } }
        "gold" => { cx.g.gold += v.parse::<u32>().unwrap_or(500); }
        "rerolls" => { cx.g.rerolls += 3; }
        "minutes+1" => { cx.g.run_t += 60.0; cx.g.stage_t += 60.0; }
        "swapw" => { cx.p.widx = 1 - cx.p.widx; cx.p.charging = false; attach_weapon(cx.c, cx.pal, cx.mats, cx.p); }
        "killall" => { for i in 0..cx.g.enemies.len() { if !cx.g.enemies[i].dead && !cx.g.enemies[i].t.passive { cx.damage(i, 1.0e7, Hit { quiet: true, ..default() }); } } }
        "maxrelics" => { for r in relics::RELICS.iter() { cx.g.mods.insert(format!("r.{}", r.id), relics::RMAX); } }
        "allups" => {
            for ab in abilities::class_abilities(cx.p.cls) { for m in abilities::MODS.iter().filter(|m| m.0 == ab) { cx.g.mods.insert(format!("{}.{}", ab, m.1), m.4); } }
            for w in player::class_weapons(cx.p.cls) { for m in abilities::WMODS.iter().filter(|m| m.0 == w) { cx.g.mods.insert(format!("w.{}.{}", w, m.1), m.4); } }
        }
        "aug" => { let a = &augments::AUGS[(cx.r() * augments::AUGS.len() as f32) as usize % augments::AUGS.len()]; let k = format!("x.{}", a.id); let r = cx.g.m(&k); if (r as usize) < a.ranks.len() { cx.g.mods.insert(k, r + 1); cx.toast(a.name, a.ranks[r as usize], 2.0); } }
        _ => {}
    }
}
