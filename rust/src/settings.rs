//! The settings screen (saved with the rest of the player's progress) and gamepad support. The gamepad is
//! bridged onto the keyboard and mouse inputs the game already reads, with the right stick driving the camera.
use crate::hud::{Screen, UiBtn, UiState};
use crate::meta::{Save, Settings};
use crate::models::srgb;
use crate::player::{CamRig, Controls, Player};
use bevy::input::gamepad::{Gamepad, GamepadAxis, GamepadButton};
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowMode};

pub const ROWS: [&str; 11] = ["Mouse sensitivity", "Invert look", "Sound volume", "Music volume", "Field of view", "View distance", "Graphics quality", "Shadows", "Screen shake", "Damage numbers", "Fullscreen"];
pub const QUALITY: [&str; 4] = ["Low", "Medium", "High", "Ultra"];
fn value(s: &Settings, i: usize) -> String {
    let onoff = |b: bool| if b { "On".to_string() } else { "Off".to_string() };
    match i { 0 => format!("{:.1}", s.sens), 1 => onoff(s.invert_y), 2 => format!("{:.0}%", s.volume * 100.0), 3 => format!("{:.0}%", s.music * 100.0), 4 => format!("{:.0}", s.fov), 5 => format!("{} chunks", s.view),
        6 => QUALITY[s.quality.min(3) as usize].to_string(), 7 => onoff(s.shadows), 8 => format!("{:.0}%", s.shake * 100.0), 9 => onoff(s.damage_numbers), _ => onoff(s.fullscreen) }
}
pub fn change(s: &mut Settings, i: usize, up: bool) {
    let d = if up { 1.0 } else { -1.0 };
    match i {
        0 => s.sens = (s.sens + 0.1 * d).clamp(0.2, 3.0), 1 => s.invert_y = !s.invert_y, 2 => s.volume = (s.volume + 0.1 * d).clamp(0.0, 1.0), 3 => s.music = (s.music + 0.1 * d).clamp(0.0, 1.0),
        4 => s.fov = (s.fov + 5.0 * d).clamp(55.0, 105.0), 5 => s.view = (s.view + d as i32).clamp(4, 14), 6 => s.quality = if up { (s.quality + 1).min(3) } else { s.quality.saturating_sub(1) },
        7 => s.shadows = !s.shadows, 8 => s.shake = (s.shake + 0.25 * d).clamp(0.0, 1.5), 9 => s.damage_numbers = !s.damage_numbers, _ => s.fullscreen = !s.fullscreen,
    }
}

#[derive(Resource)]
pub struct SettingsRefs { root: Entity, vals: Vec<Entity> }
fn col(hex: u32, a: f32) -> Color { srgb(hex).with_alpha(a) }
fn txt(s: &str, size: f32, hex: u32) -> (Text, TextFont, TextColor) { (Text::new(s), TextFont { font_size: size, ..default() }, TextColor(srgb(hex))) }
fn px(v: f32) -> Val { Val::Px(v) }
fn btn(c: &mut Commands, t: &crate::uikit::UiTex, b: UiBtn, w: f32, label: &str) -> Entity { crate::uikit::button(c, t, b, w, 34.0, label, 17.0).0 }
pub fn setup_settings(mut c: Commands, kit: Res<crate::uikit::UiTex>) {
    let kit = &*kit;
    let back_root = c.spawn((Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
        BackgroundColor(col(0x0a0806, 0.62)), Visibility::Hidden, crate::uikit::Appear::default())).id();
    let root = c.spawn((Node { flex_direction: FlexDirection::Column, align_items: AlignItems::Center, row_gap: px(5.0), padding: UiRect::axes(px(40.0), px(22.0)), ..default() }, crate::uikit::stone(kit))).id();
    c.entity(back_root).add_child(root);
    let t = crate::uikit::title(&mut c, "SETTINGS", 40.0, 0xffd27a); c.entity(root).add_child(t);
    let mut vals = vec![];
    for (i, name) in ROWS.iter().enumerate() {
        let row = c.spawn(Node { column_gap: px(10.0), align_items: AlignItems::Center, ..default() }).id();
        let l = c.spawn((txt(name, 18.0, 0xe8dccc), Node { width: px(240.0), ..default() })).id();
        let dn = btn(&mut c, kit, UiBtn::Set(i as u8, false), 44.0, "<");
        let v = c.spawn((txt("", 18.0, 0xffd27a), Node { width: px(150.0), ..default() }, TextLayout::new_with_justify(JustifyText::Center))).id();
        let up = btn(&mut c, kit, UiBtn::Set(i as u8, true), 44.0, ">");
        c.entity(row).add_children(&[l, dn, v, up]); c.entity(root).add_child(row); vals.push(v);
    }
    let pad = c.spawn((txt("Gamepad: left stick move, right stick look, RT attack, LT block, A jump, B dodge, X Y LB RB abilities,\nD-pad up interact, left swap, down eat, right character panel, Start pause. On level-ups X Y B pick cards 1 2 3.", 13.0, 0x9a8a78), Node { margin: UiRect::top(px(10.0)), ..default() }, TextLayout::new_with_justify(JustifyText::Center))).id();
    let back = btn(&mut c, kit, UiBtn::Back, 240.0, "Back  (Esc)");
    c.entity(root).add_children(&[pad, back]);
    c.insert_resource(SettingsRefs { root: back_root, vals });
}
pub fn update_settings_ui(ui: Res<UiState>, save: Res<Save>, r: Res<SettingsRefs>, mut q: Query<(Option<&mut Text>, Option<&mut Visibility>), Without<UiBtn>>) {
    if let Ok((_, Some(mut v))) = q.get_mut(r.root) { *v = if ui.screen == Screen::Settings { Visibility::Inherited } else { Visibility::Hidden }; }
    if ui.screen != Screen::Settings { return; }
    for (i, e) in r.vals.iter().enumerate() { if let Ok((Some(mut t), _)) = q.get_mut(*e) { let s = value(&save.settings, i); if t.0 != s { t.0 = s; } } }
}

/// push the saved settings into the systems that use them whenever they change
#[allow(clippy::too_many_arguments)]
pub fn apply_settings(save: Res<Save>, mut ctl: ResMut<Controls>, mut cr: ResMut<CamRig>, mut sfx: ResMut<crate::audio::Sfx>, mut cfg: ResMut<crate::GameCfg>,
    mut sun: Query<(Entity, &mut DirectionalLight), With<crate::scene::Sun>>, mut win: Query<&mut Window, With<PrimaryWindow>>, test: Option<Res<crate::testkit::TestCfg>>,
    mut c: Commands, cam: Query<Entity, With<crate::scene::MainCam>>, mut smap: ResMut<bevy::pbr::DirectionalLightShadowMap>, mut last_q: Local<Option<u8>>) {
    if !save.is_changed() { return; }
    // graphics quality: ambient occlusion, god rays, anti-aliasing and shadow resolution
    let q = save.settings.quality.min(3);
    if *last_q != Some(q) {
        *last_q = Some(q);
        use bevy::core_pipeline::smaa::{Smaa, SmaaPreset};
        use bevy::pbr::{ScreenSpaceAmbientOcclusion as Ssao, ScreenSpaceAmbientOcclusionQualityLevel as Sq, VolumetricFog, CascadeShadowConfigBuilder};
        if let Ok(e) = cam.get_single() {
            let mut ec = c.entity(e);
            ec.insert(Smaa { preset: [SmaaPreset::Low, SmaaPreset::Medium, SmaaPreset::High, SmaaPreset::Ultra][q as usize] });
            if q >= 1 { ec.insert(Ssao { quality_level: [Sq::Low, Sq::Low, Sq::Medium, Sq::High][q as usize], constant_object_thickness: 0.3 }); } else { ec.remove::<Ssao>(); }
            if q >= 2 { ec.insert(VolumetricFog { ambient_intensity: 0.0, jitter: 0.5, step_count: if q >= 3 { 48 } else { 24 }, ..default() }); } else { ec.remove::<VolumetricFog>(); }
        }
        smap.size = [1024, 2048, 2048, 4096][q as usize];
        if let Ok((e, _)) = sun.get_single() {
            c.entity(e).insert(CascadeShadowConfigBuilder { num_cascades: if q >= 3 { 4 } else if q == 0 { 2 } else { 3 }, minimum_distance: 0.5, first_cascade_far_bound: 18.0, maximum_distance: if q == 0 { 70.0 } else { 120.0 }, overlap_proportion: 0.25 }.build());
        }
    }
    let s = &save.settings;
    ctl.look_scale = s.sens; ctl.invert_y = s.invert_y;
    sfx.master = s.volume; sfx.music = s.music;
    cr.base_fov = s.fov; cr.shake_mul = s.shake;
    cfg.view = s.view; cfg.damage_numbers = s.damage_numbers;
    for (_, mut l) in sun.iter_mut() { l.shadows_enabled = s.shadows; }
    if test.is_none() && std::env::var("EMBERWILD_WINDOWED").is_err() {
        if let Ok(mut w) = win.get_single_mut() { let want = if s.fullscreen { WindowMode::BorderlessFullscreen(MonitorSelection::Primary) } else { WindowMode::Windowed }; if w.mode != want { w.mode = want; } }
    }
}

/// gamepads drive the same inputs as keyboard and mouse
#[derive(Resource, Default)]
pub struct PadState { held: Vec<KeyCode>, mouse: Vec<MouseButton> }
pub fn gamepad_bridge(pads: Query<&Gamepad>, mut keys: ResMut<ButtonInput<KeyCode>>, mut mouse: ResMut<ButtonInput<MouseButton>>, mut st: ResMut<PadState>, mut p: ResMut<Player>,
    mut ctl: ResMut<Controls>, time: Res<Time>, ui: Res<UiState>) {
    // release what we pressed last frame, then press again from the current pad state
    for k in st.held.drain(..) { keys.release(k); }
    for b in st.mouse.drain(..) { mouse.release(b); }
    let Some(pad) = pads.iter().next() else { ctl.pad = false; return };
    let mut press = |k: KeyCode, st: &mut PadState, keys: &mut ButtonInput<KeyCode>| { keys.press(k); st.held.push(k); };
    let (lx, ly) = (pad.get(GamepadAxis::LeftStickX).unwrap_or(0.0), pad.get(GamepadAxis::LeftStickY).unwrap_or(0.0));
    let (rx, ry) = (pad.get(GamepadAxis::RightStickX).unwrap_or(0.0), pad.get(GamepadAxis::RightStickY).unwrap_or(0.0));
    let any = lx.abs() > 0.3 || ly.abs() > 0.3 || rx.abs() > 0.2 || ry.abs() > 0.2 || pad.get_pressed().next().is_some();
    if any { ctl.pad = true; }
    if !ctl.pad { return; }
    let playing = ui.screen == Screen::Playing;
    if playing {
        if ly > 0.35 { press(KeyCode::KeyW, &mut st, &mut keys); } if ly < -0.35 { press(KeyCode::KeyS, &mut st, &mut keys); }
        if lx > 0.35 { press(KeyCode::KeyD, &mut st, &mut keys); } if lx < -0.35 { press(KeyCode::KeyA, &mut st, &mut keys); }
        if (lx * lx + ly * ly).sqrt() > 0.95 && pad.pressed(GamepadButton::LeftThumb) { press(KeyCode::ShiftLeft, &mut st, &mut keys); }
        let dt = time.delta_secs(); let k = ctl.look_scale.max(0.1) * 2.6;
        let dz = |v: f32| if v.abs() < 0.12 { 0.0 } else { v.signum() * (v.abs() - 0.12) / 0.88 };
        p.yaw -= dz(rx) * dt * k; p.pitch = (p.pitch + dz(ry) * dt * k * 0.75 * if ctl.invert_y { -1.0 } else { 1.0 }).clamp(-1.25, 1.05);
        if pad.pressed(GamepadButton::RightTrigger2) { mouse.press(MouseButton::Left); st.mouse.push(MouseButton::Left); }
        if pad.pressed(GamepadButton::LeftTrigger2) { mouse.press(MouseButton::Right); st.mouse.push(MouseButton::Right); }
        if pad.pressed(GamepadButton::South) { press(KeyCode::Space, &mut st, &mut keys); }
    }
    let map: [(GamepadButton, KeyCode); 11] = if playing {
        [(GamepadButton::DPadRight, KeyCode::Tab), (GamepadButton::East, KeyCode::ControlLeft), (GamepadButton::West, KeyCode::Digit1), (GamepadButton::North, KeyCode::Digit2), (GamepadButton::LeftTrigger, KeyCode::Digit3), (GamepadButton::RightTrigger, KeyCode::Digit4),
         (GamepadButton::DPadUp, KeyCode::KeyE), (GamepadButton::DPadLeft, KeyCode::KeyQ), (GamepadButton::DPadDown, KeyCode::KeyF), (GamepadButton::Start, KeyCode::Escape), (GamepadButton::Select, KeyCode::KeyM)]
    } else {
        [(GamepadButton::DPadRight, KeyCode::Tab), (GamepadButton::West, KeyCode::Digit1), (GamepadButton::North, KeyCode::Digit2), (GamepadButton::East, KeyCode::Digit3), (GamepadButton::South, KeyCode::Digit4), (GamepadButton::Start, KeyCode::Escape),
         (GamepadButton::LeftTrigger, KeyCode::KeyR), (GamepadButton::RightTrigger, KeyCode::KeyC), (GamepadButton::DPadUp, KeyCode::KeyE), (GamepadButton::Select, KeyCode::Escape), (GamepadButton::DPadDown, KeyCode::KeyM)]
    };
    for (b, k) in map { if pad.just_pressed(b) { press(k, &mut st, &mut keys); } }
}
