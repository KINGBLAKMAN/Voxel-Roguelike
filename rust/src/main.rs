// Hide the console window on Windows release builds.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
//! Emberwild: a voxel horde-survival roguelike.

mod mesher;
mod models;
mod noise;
mod physics;
mod player;
mod scene;
mod testkit;
mod world;

use bevy::prelude::*;
use bevy::window::{PresentMode, WindowMode};
use models::*;
use player::*;
use scene::*;

/// tunables the renderer and streaming read; `focus` overrides the streaming centre (menus, tests)
#[derive(Resource)]
pub struct GameCfg { pub view: i32, pub chunk_budget: usize, pub time_runs: bool, pub focus: Option<Vec3> }
impl Default for GameCfg { fn default() -> Self { GameCfg { view: 8, chunk_budget: 3, time_runs: true, focus: None } } }

fn main() {
    let test = testkit::TestCfg::from_env();
    let fullscreen = test.is_none() && std::env::var("EMBERWILD_WINDOWED").is_err();
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Emberwild".into(), resolution: (1280.0, 720.0).into(), present_mode: PresentMode::AutoVsync,
            mode: if fullscreen { WindowMode::BorderlessFullscreen(MonitorSelection::Primary) } else { WindowMode::Windowed }, ..default()
        }),
        ..default()
    }))
    .init_resource::<GameCfg>()
    .init_resource::<Palette>()
    .init_resource::<Player>()
    .init_resource::<Controls>()
    .init_resource::<CamRig>()
    .add_plugins(ScenePlugin)
    .add_systems(Startup, start_world)
    .add_systems(Update, (cursor_lock, mouse_look, player_frame).chain());
    if let Some(t) = test { app.insert_resource(t).add_systems(Update, testkit::run_test); }
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

fn player_frame(time: Res<Time>, keys: Res<ButtonInput<KeyCode>>, mouse: Res<ButtonInput<MouseButton>>, mut w: Option<ResMut<world::Voxels>>, mut p: ResMut<Player>, mut cr: ResMut<CamRig>,
    mut q: Query<&mut Transform>, mut cam: Query<(Entity, &mut Projection), With<MainCam>>) {
    let Some(w) = w.as_mut() else { return };
    let dt = time.delta_secs().min(0.05);
    move_player(w, &mut p, &keys, dt, 1.0, 1.0, 100.0);
    p.firing = mouse.pressed(MouseButton::Left);
    animate_player(&mut p, &mut q, dt);
    let Ok((ce, mut proj)) = cam.get_single_mut() else { return };
    let mut ct = *q.get(ce).unwrap();
    let mut rng = noise::mulberry32((time.elapsed_secs() * 1000.0) as u32);
    update_camera(w, &p, &mut cr, &mut ct, &mut proj, dt, &mut rng);
    *q.get_mut(ce).unwrap() = ct;
}
