// Hide the console window on Windows release builds.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
//! Emberwild: a voxel horde-survival roguelike.

mod abilities;
mod audio;
mod camp;
mod augments;
mod combat;
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
pub struct GameCfg { pub view: i32, pub chunk_budget: usize, pub time_runs: bool, pub focus: Option<Vec3>, pub damage_numbers: bool }
impl Default for GameCfg { fn default() -> Self { GameCfg { view: 8, chunk_budget: 3, time_runs: true, focus: None, damage_numbers: true } } }

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
    .init_resource::<hud::UiState>()
    .insert_resource(meta::Save::load())
    .init_resource::<settings::PadState>()
    .add_plugins((ScenePlugin, fx::FxPlugin))
    .add_systems(Startup, (start_world, audio::setup_audio, audio::setup_music, (hud::setup_hud, camp::setup_camp, settings::setup_settings).chain()))
    .add_systems(Update, (settings::gamepad_bridge, settings::apply_settings, hud::ui_flow, run_control.after(scene::stream_chunks), cursor_lock, mouse_look, game_tick, hud::update_hud, net_day, hud::button_hover, camp::update_camp, camp::update_shop, camp::update_join, settings::update_settings_ui, hud::menu_camera, audio::play_sfx, audio::update_music).chain());
    if let Some(t) = test { app.insert_resource(t).add_systems(Update, testkit::run_test.before(game_tick)); }
    app.run();
}

fn start_world(mut c: Commands, mut pal: ResMut<Palette>, mut mats: ResMut<Assets<StandardMaterial>>, mut p: ResMut<Player>, mut ctl: ResMut<Controls>, mut ui: ResMut<hud::UiState>, test: Option<Res<testkit::TestCfg>>) {
    let seed = test.as_ref().map(|t| t.seed).unwrap_or_else(|| rand::random::<u32>() % 1_000_000_000);
    let mut w = world::Voxels::new(seed);
    let sp = w.spawn_point();
    p.body.pos = sp; p.vis_y = sp.y;
    // scripted tests skip the title screen
    if let Some(t) = test.as_ref() { if t.camp { ui.screen = if t.class == Class::Mage { hud::Screen::Settings } else { hud::Screen::Camp }; } if !t.menu && !t.camp { ui.screen = hud::Screen::Playing; begin_player(&mut c, &mut pal, &mut mats, &mut p, t.class); ctl.want_lock = true; } }
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
        ui.screen = hud::Screen::Playing; ui.dead_t = 0.0; ui.ghost_hp = 1.0;
        ctl.want_lock = true; ctl.relock = true;
        sfx.play("horn");
    }
    if !ui.restart { return; }
    ui.restart = false;
    let g = &mut *g;
    // everything the run spawned into the world goes away
    let mut ents: Vec<Entity> = vec![];
    for e in g.enemies.drain(..) { ents.extend(e.rig.root); ents.extend(e.ice); }
    ents.extend(g.corpses.drain(..).map(|x| x.root));
    ents.extend(g.pickups.drain(..).map(|x| x.e));
    for pr in g.projs.drain(..) { ents.extend(pr.e); }
    for z in g.zones.iter_mut() { ents.append(&mut z.ents); }
    ents.extend(g.traps.drain(..).map(|t| t.root));
    ents.extend(g.dir.chests.drain(..).map(|c| c.root));
    for l in g.rel.objs.values() { ents.extend(l.iter().copied()); }
    ents.extend(g.rel.decoys.drain(..).map(|d| d.e));
    ents.extend(g.rel.stuck.drain(..).map(|s| s.e));
    if let Some(ch) = g.ab.charge.take() { ents.push(ch.wedge()); }
    for t in fx.tweens.drain(..) { ents.push(t.e); }
    let fx = &mut *fx;
    for d in fx.debris.drain(..) { c.entity(d.e).insert(Visibility::Hidden); fx.debris_free.push(d.e); }
    fx.add.clear(); fx.blend.clear();
    for e in ents { if let Some(mut ec) = c.get_entity(e) { ec.despawn_recursive(); } }
    let netstate = std::mem::take(&mut g.net);
    *g = Game::default();
    g.net = netstate; g.net.out.clear();
    if let Some(r) = p.rig.root.take() { c.entity(r).despawn_recursive(); }
    // a fresh world
    if let Some(w) = w { for ch in w.chunks.values() { for e in [ch.entity, ch.fol_entity].into_iter().flatten() { if let Some(mut ec) = c.get_entity(e) { ec.despawn_recursive(); } } } }
    let seed = join_seed.unwrap_or_else(|| rand::random::<u32>() % 1_000_000_000);
    let mut nw = world::Voxels::new(seed);
    if let Some(l) = link.as_ref() { if l.host { for id in l.peer_ids() { l.send(Some(id), &net::Msg::Welcome { id, seed }); } } }
    let sp = nw.spawn_point();
    *p = Player::default(); p.body.pos = sp; p.vis_y = sp.y;
    c.insert_resource(nw);
    *dc = DayCycle::default();
    ui.screen = hud::Screen::Menu; ui.dead_t = 0.0; ui.toast = None;
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
    mut camq: Query<(Entity, &mut Projection), With<MainCam>>, mut test: Option<ResMut<testkit::TestCfg>>, ui: Res<hud::UiState>, link: Option<Res<net::Link>>) {
    let s = &mut s;
    let Some(w) = s.w.as_mut() else { return };
    let w: &mut world::Voxels = w;
    let raw = time.delta_secs().min(0.05);
    let Ok((ce, mut proj)) = camq.get_single_mut() else { return };
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
    cx.g.run_t += dt;
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
        "weapon" => { cx.p.widx = v.parse().unwrap_or(0); attach_weapon(cx.c, cx.pal, cx.mats, cx.p); }
        "chest" => { let p = cx.p.body.pos + cx.p.fwd() * 4.0; director::drop_chest(cx, p, false); }
        "ab" => { let i: usize = v.parse().unwrap_or(0); cx.g.cds[i] = 0.0; abilities::use_ability(cx, i); }
        "bomb" => { let p = cx.p.body.pos + cx.p.fwd() * 2.0; cx.drop_pick(p, PickKind::Bomb, 0.0); }
        _ => {}
    }
}
