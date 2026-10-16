//! The dev build's side menu (F1 during a run): one-click cheats for testing. It frees the cursor while open; the
//! game keeps running behind it. Actions that need the game world are queued and run inside the next game tick.
use crate::hud::{Screen, UiState};
use crate::models::srgb;
use crate::player::Controls;
use bevy::prelude::*;

/// cheat actions waiting for the game tick (same vocabulary as the scripted test actions)
#[derive(Resource, Default)]
pub struct DevQueue(pub Vec<String>);

#[derive(Component, Clone, Copy)]
pub struct DevBtn(pub &'static str);
#[derive(Resource)]
pub struct DevRefs { root: Entity, hint: Entity, toggles: Vec<(Entity, &'static str)> }
#[derive(Resource, Default)]
pub struct DevMenu { pub open: bool }

/// (section, [(action, label)])
const MENU: [(&str, &[(&str, &str)]); 5] = [
    ("Player", &[("god", "God mode"), ("nocd", "No cooldowns"), ("heal", "Full heal"), ("lvl", "+1 level"), ("lvl5", "+5 levels"), ("gold=500", "+500 run gold"), ("chest", "Drop a chest"), ("rerolls", "+3 rerolls")]),
    ("Build", &[("maxrelics", "Max every relic"), ("allups", "All ability and weapon upgrades"), ("ablv=0,5", "Ability 1 +5 levels"), ("ablv=1,5", "Ability 2 +5 levels"), ("ablv=2,5", "Ability 3 +5 levels"), ("ablv=3,5", "Ability 4 +5 levels"), ("aug", "Random chaos augment"), ("allaug", "+3 to every augment"), ("wlv=5", "+5 weapon levels"), ("swapw", "Swap weapon")]),
    ("Run", &[("minutes+1", "+1 minute"), ("stage_t=288", "Skip to this stage's boss"), ("portal", "Open the portal here"), ("nextstage", "Go to the next stage"), ("day", "Time of day +6h"), ("horde=60", "Spawn a horde"), ("killall", "Kill everything")]),
    ("Spawn boss", &[("spawn=golem,12", "Ancient Golem"), ("spawn=colossus,14", "Bone Colossus"), ("spawn=brood,14", "Broodmother"), ("spawn=king,16", "Ember King")]),
    ("Events", &[("event=0", "Challenge"), ("event=1", "Cursed shrine"), ("event=2", "Treasure goblin"), ("event=3", "War totem"), ("event=4", "Merchant"), ("campgold", "+1000 Camp gold")]),
];

fn col(hex: u32, a: f32) -> Color { srgb(hex).with_alpha(a) }
fn txt(s: &str, size: f32, hex: u32) -> (Text, TextFont, TextColor) { (Text::new(s), TextFont { font_size: size, ..default() }, TextColor(srgb(hex))) }
fn px(v: f32) -> Val { Val::Px(v) }

pub fn setup_dev(mut c: Commands, kit: Res<crate::uikit::UiTex>) {
    if !cfg!(feature = "dev") { return; }
    let hint = c.spawn((txt("DEV BUILD  |  F1 dev menu", 13.0, 0xff9a3d), Node { position_type: PositionType::Absolute, left: px(18.0), top: px(42.0), ..default() }, GlobalZIndex(20), Visibility::Hidden)).id();
    let root = c.spawn((Node { position_type: PositionType::Absolute, left: px(10.0), top: px(64.0), flex_direction: FlexDirection::Column, row_gap: px(3.0), padding: UiRect::all(px(18.0)), ..default() },
        crate::uikit::stone(&kit), GlobalZIndex(40), Visibility::Hidden)).id();
    let t = c.spawn(txt("DEV MENU  (F1 to close)", 15.0, 0xff9a3d)).id(); c.entity(root).add_child(t);
    let cols = c.spawn(Node { column_gap: px(10.0), align_items: AlignItems::FlexStart, ..default() }).id(); c.entity(root).add_child(cols);
    let colv: Vec<Entity> = (0..2).map(|_| c.spawn(Node { flex_direction: FlexDirection::Column, row_gap: px(3.0), width: px(220.0), ..default() }).id()).collect();
    c.entity(cols).add_children(&colv);
    let mut toggles = vec![];
    for (si, (sec, items)) in MENU.into_iter().enumerate() {
        let col_e = colv[if si < 2 { 0 } else { 1 }];
        let h = c.spawn((txt(sec, 12.0, 0x9a8a78), Node { margin: UiRect::top(px(4.0)), ..default() })).id(); c.entity(col_e).add_child(h);
        for (act, label) in items.iter() {
            let b = c.spawn((Button, DevBtn(act), Node { padding: UiRect::axes(px(8.0), px(2.0)), border: UiRect::all(px(1.0)), ..default() }, BackgroundColor(col(0x1a1520, 0.95)), BorderColor(col(0x4a3e34, 1.0)), BorderRadius::all(px(4.0)))).id();
            let l = c.spawn(txt(label, 13.0, 0xe8dccc)).id(); c.entity(b).add_child(l); c.entity(col_e).add_child(b);
            if matches!(*act, "god" | "nocd") { toggles.push((b, *act)); }
        }
    }
    c.insert_resource(DevRefs { root, hint, toggles });
}

#[allow(clippy::too_many_arguments)]
pub fn dev_menu(refs: Option<Res<DevRefs>>, mut menu: ResMut<DevMenu>, ui: Res<UiState>, keys: Res<ButtonInput<KeyCode>>, mut ctl: ResMut<Controls>, mut queue: ResMut<DevQueue>,
    btns: Query<(&Interaction, &DevBtn), Changed<Interaction>>, mut bg: Query<&mut BorderColor>, mut vis: Query<&mut Visibility>, g: Res<crate::game::Game>, mut save: ResMut<crate::meta::Save>, mut dc: ResMut<crate::scene::DayCycle>) {
    let Some(r) = refs else { return };
    let playing = ui.screen == Screen::Playing;
    if playing && keys.just_pressed(KeyCode::F1) { menu.open = !menu.open; if !menu.open { ctl.relock = true; } }
    if !playing { menu.open = false; }
    let show = menu.open && playing;
    if let Ok(mut v) = vis.get_mut(r.root) { *v = if show { Visibility::Inherited } else { Visibility::Hidden }; }
    if let Ok(mut v) = vis.get_mut(r.hint) { *v = if playing && !show { Visibility::Inherited } else { Visibility::Hidden }; }
    if !show { return; }
    // the cursor stays free while the menu is open
    ctl.want_lock = false;
    for (i, b) in btns.iter() {
        if *i != Interaction::Pressed { continue; }
        match b.0 {
            "campgold" => { save.gold += 1000; save.write(); }
            "day" => dc.time += 0.25,
            a => queue.0.push(a.to_string()),
        }
    }
    for (e, k) in &r.toggles {
        let on = g.flags.contains_key(if *k == "god" { "dev_god" } else { "dev_nocd" });
        if let Ok(mut b) = bg.get_mut(*e) { b.0 = if on { col(0x7cff6a, 1.0) } else { col(0x4a3e34, 1.0) }; }
    }
}
