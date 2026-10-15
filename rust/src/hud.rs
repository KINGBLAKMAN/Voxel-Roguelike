//! The interface: the in-run HUD (health, experience, stamina, ability bar with cooldown sweeps, run clock and
//! threat, boss bar, toasts, buffs and relics, minimap, floating damage numbers, crosshair) and the screens
//! (class select, level-up cards, pause, death). Screen flow lives here too.
use crate::abilities::{ab_info, class_abilities};
use crate::combat::weapon_info;
use crate::director::threat;
use crate::game::*;
use crate::models::{srgb, Class};
use crate::player::{Controls, Player};
use crate::progression::*;
use crate::scene::{DayCycle, MainCam};
use crate::world::{block_def, Voxels, CS};
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen { Menu, Playing, LevelUp, Paused, Dead, Victory, Camp, Shop, Settings, Join, Character }

#[derive(Resource)]
pub struct UiState {
    pub screen: Screen, pub cards: Vec<Card>, pub relic_only: bool, pub dead_t: f32, pub toast: Option<(String, String, f32, f32)>,
    pub ghost_hp: f32, pub map_t: f32, pub menu_t: f32, pub start: Option<Class>, pub restart: bool, pub quit: bool, pub cards_dirty: bool,
    pub pops: Vec<(Vec3, f32, bool)>, pub rng: u32, pub paid: u32, pub recorded: bool, pub last_gain: u32, pub settings_from: Option<Screen>, pub req_host: bool, pub req_join: Option<String>, pub join_text: String, pub net_msg: String,
    /// the second stage picked on the title screen: 0 random, 1 swamp, 2 desert
    pub route: u8,
}
impl Default for UiState {
    fn default() -> Self { UiState { screen: Screen::Menu, cards: vec![], relic_only: false, dead_t: 0.0, toast: None, ghost_hp: 100.0, map_t: 0.0, menu_t: 0.0, start: None, restart: false, quit: false, cards_dirty: true, pops: vec![], rng: 0x1234567, paid: 0, recorded: false, last_gain: 0, settings_from: None, req_host: false, req_join: None, join_text: String::new(), net_msg: String::new(), route: 0 } }
}
impl UiState { fn rnd(&mut self) -> f32 { let mut x = self.rng; x ^= x << 13; x ^= x >> 17; x ^= x << 5; self.rng = x; (x % 1_000_000) as f32 / 1_000_000.0 } }

#[derive(Component, Clone, Copy, PartialEq, Debug)]
pub enum UiBtn { Class(Class), Card(usize), Resume, Restart, Quit, Endless, Camp, Back, Buy(usize), Unlock(usize), EmberUp, EmberDown, ShopBuy(usize), Settings, Set(u8, bool), Host, Join, Route(u8) }

#[derive(Resource)]
pub struct HudRefs {
    hud: Entity, hp_fill: Entity, hp_ghost: Entity, hp_text: Entity, xp_fill: Entity, lvl: Entity, stam_fill: Entity,
    slots: Vec<(Entity, Entity, Entity, Entity)>, weapon: Entity, food: Entity, timer: Entity, threat: Entity, kills: Entity,
    boss: Entity, boss_fill: Entity, boss_name: Entity, toast: Entity, toast_a: Entity, toast_b: Entity, buffs: Entity, relics: Entity,
    interact: Entity, cross: Vec<Entity>, map: Handle<Image>, day: Entity, pops: Vec<Entity>, flash: Entity, hurt: Entity,
    pub menu: Entity, level: Entity, lv_title: Entity, lv_sub: Entity, cards: Vec<(Entity, Entity, Entity, Entity)>, reroll: Entity,
    pause: Entity, dead: Entity, dead_stats: Entity, victory: Entity, vic_stats: Entity,
}

const EMBER: u32 = 0xff9a3d;
fn col(hex: u32, a: f32) -> Color { srgb(hex).with_alpha(a) }
fn txt(s: &str, size: f32, hex: u32) -> (Text, TextFont, TextColor) { (Text::new(s), TextFont { font_size: size, ..default() }, TextColor(srgb(hex))) }
fn abs(l: Val, t: Val, r: Val, b: Val) -> Node { Node { position_type: PositionType::Absolute, left: l, top: t, right: r, bottom: b, ..default() } }
fn px(v: f32) -> Val { Val::Px(v) }
const A: Val = Val::Auto;

pub fn setup_hud(mut c: Commands, mut images: ResMut<Assets<Image>>) {
    let map = images.add(Image::new_fill(Extent3d { width: 128, height: 128, depth_or_array_layers: 1 }, TextureDimension::D2, &[20, 24, 30, 255], TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::all()));
    let panel = |a: f32| BackgroundColor(col(0x0c0a10, a));
    // ---------------- HUD ----------------
    let hud = c.spawn((Node { width: Val::Percent(100.0), height: Val::Percent(100.0), position_type: PositionType::Absolute, ..default() }, Visibility::Hidden)).id();
    let flash = c.spawn((Node { width: Val::Percent(100.0), height: Val::Percent(100.0), position_type: PositionType::Absolute, ..default() }, BackgroundColor(col(0xfff0c0, 0.0)))).id();
    let hurt = c.spawn((Node { width: Val::Percent(100.0), height: Val::Percent(100.0), position_type: PositionType::Absolute, border: UiRect::all(px(28.0)), ..default() }, BorderColor(col(0xb01020, 0.0)))).id();
    c.entity(hud).add_children(&[flash, hurt]);
    // crosshair
    let mut cross = vec![];
    for (w, h, dx, dy) in [(2.0, 8.0, -1.0, -14.0), (2.0, 8.0, -1.0, 6.0), (8.0, 2.0, -14.0, -1.0), (8.0, 2.0, 6.0, -1.0), (3.0, 3.0, -1.5, -1.5)] {
        let e = c.spawn((Node { position_type: PositionType::Absolute, left: Val::Percent(50.0), top: Val::Percent(50.0), width: px(w), height: px(h), margin: UiRect { left: px(dx), top: px(dy), ..default() }, ..default() }, BackgroundColor(col(0xffffff, 0.85)))).id();
        c.entity(hud).add_child(e); cross.push(e);
    }
    // bottom panel: health, experience, stamina and the action bar
    let bottom = c.spawn(Node { position_type: PositionType::Absolute, bottom: px(18.0), left: px(0.0), right: px(0.0), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, row_gap: px(5.0), ..default() }).id();
    c.entity(hud).add_child(bottom);
    let interact = c.spawn((txt("", 18.0, 0xfff3d8), Node { margin: UiRect::bottom(px(8.0)), ..default() })).id();
    let hp_bar = c.spawn((Node { width: px(560.0), height: px(20.0), border: UiRect::all(px(2.0)), ..default() }, BackgroundColor(col(0x1a0e10, 0.85)), BorderColor(col(0x000000, 0.9)), BorderRadius::all(px(5.0)))).id();
    let hp_ghost = c.spawn((Node { position_type: PositionType::Absolute, height: Val::Percent(100.0), width: Val::Percent(100.0), ..default() }, BackgroundColor(col(0xffd0c0, 0.55)), BorderRadius::all(px(3.0)))).id();
    let hp_fill = c.spawn((Node { position_type: PositionType::Absolute, height: Val::Percent(100.0), width: Val::Percent(100.0), ..default() }, BackgroundColor(col(0xd8343a, 1.0)), BorderRadius::all(px(3.0)))).id();
    let hp_shine = c.spawn((Node { position_type: PositionType::Absolute, height: Val::Percent(40.0), width: Val::Percent(100.0), ..default() }, BackgroundColor(col(0xffffff, 0.1)))).id();
    let hp_text = c.spawn((txt("100 / 100", 14.0, 0xffffff), Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), ..default() }, TextLayout::new_with_justify(JustifyText::Center))).id();
    c.entity(hp_bar).add_children(&[hp_ghost, hp_fill, hp_shine, hp_text]);
    let xp_row = c.spawn(Node { width: px(560.0), align_items: AlignItems::Center, column_gap: px(8.0), ..default() }).id();
    let lvl = c.spawn((txt("Lv 1", 15.0, 0x8fd0ff), Node { width: px(44.0), ..default() })).id();
    let xp_bar = c.spawn((Node { flex_grow: 1.0, height: px(8.0), ..default() }, BackgroundColor(col(0x0e1620, 0.85)), BorderRadius::all(px(4.0)))).id();
    let xp_fill = c.spawn((Node { height: Val::Percent(100.0), width: Val::Percent(0.0), ..default() }, BackgroundColor(col(0x5ab8ff, 1.0)), BorderRadius::all(px(4.0)))).id();
    c.entity(xp_bar).add_child(xp_fill);
    let stam_bar = c.spawn((Node { width: px(120.0), height: px(5.0), ..default() }, BackgroundColor(col(0x1e1a10, 0.85)), BorderRadius::all(px(3.0)))).id();
    let stam_fill = c.spawn((Node { height: Val::Percent(100.0), width: Val::Percent(100.0), ..default() }, BackgroundColor(col(0xf0c64a, 1.0)), BorderRadius::all(px(3.0)))).id();
    c.entity(stam_bar).add_child(stam_fill);
    c.entity(xp_row).add_children(&[lvl, xp_bar, stam_bar]);
    let bar = c.spawn(Node { column_gap: px(8.0), margin: UiRect::top(px(4.0)), align_items: AlignItems::FlexEnd, ..default() }).id();
    let side_slot = |c: &mut Commands, key: &str, label: &str| -> (Entity, Entity) {
        let s = c.spawn((Node { width: px(76.0), height: px(54.0), flex_direction: FlexDirection::Column, justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(px(2.0)), ..default() }, panel(0.72), BorderColor(col(0x4a3e34, 1.0)), BorderRadius::all(px(6.0)))).id();
        let k = c.spawn(txt(key, 12.0, 0x9a8a78)).id();
        let t = c.spawn((txt(label, 13.0, 0xf0e6d8), TextLayout::new_with_justify(JustifyText::Center))).id();
        c.entity(s).add_children(&[k, t]); (s, t)
    };
    let (wroot, weapon) = side_slot(&mut c, "Q  swap", "");
    let mut slots = vec![];
    let mut slot_roots = vec![];
    for i in 0..4 {
        let s = c.spawn((Node { width: px(78.0), height: px(78.0), border: UiRect::all(px(2.0)), flex_direction: FlexDirection::Column, justify_content: JustifyContent::FlexEnd, align_items: AlignItems::Center, overflow: Overflow::clip(), ..default() }, panel(0.8), BorderColor(col(0x4a3e34, 1.0)), BorderRadius::all(px(8.0)),
            Interaction::default(), crate::charsheet::AbilityTip(i))).id();
        let glow = c.spawn((Node { position_type: PositionType::Absolute, top: px(0.0), left: px(0.0), right: px(0.0), height: px(4.0), ..default() }, BackgroundColor(col(0xffffff, 1.0)))).id();
        let cd = c.spawn((Node { position_type: PositionType::Absolute, bottom: px(0.0), left: px(0.0), right: px(0.0), height: Val::Percent(0.0), ..default() }, BackgroundColor(col(0x000000, 0.66)))).id();
        let key = c.spawn((txt(&format!("{}", i + 1), 15.0, 0xfff3d8), Node { position_type: PositionType::Absolute, top: px(6.0), left: px(7.0), ..default() })).id();
        let cdt = c.spawn((txt("", 22.0, 0xffffff), Node { position_type: PositionType::Absolute, top: px(20.0), ..default() })).id();
        let name = c.spawn((txt("", 11.0, 0xf0e6d8), TextLayout::new_with_justify(JustifyText::Center), Node { margin: UiRect::bottom(px(5.0)), max_width: px(72.0), ..default() })).id();
        c.entity(s).add_children(&[glow, cd, key, cdt, name]);
        slots.push((s, cd, cdt, name)); slot_roots.push((s, glow));
    }
    let (froot, food) = side_slot(&mut c, "F  eat", "");
    c.entity(bottom).add_children(&[interact, hp_bar, xp_row, bar]);
    // top centre: clock, threat, kills, boss bar, toasts
    let top = c.spawn(Node { position_type: PositionType::Absolute, top: px(14.0), left: px(0.0), right: px(0.0), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, row_gap: px(2.0), ..default() }).id();
    c.entity(hud).add_child(top);
    let timer = c.spawn(txt("0:00", 40.0, 0xfff3d8)).id();
    let threat_t = c.spawn(txt("Calm", 15.0, 0x9fd8ff)).id();
    let kills = c.spawn(txt("0 slain", 13.0, 0xb8aca0)).id();
    let boss = c.spawn((Node { flex_direction: FlexDirection::Column, align_items: AlignItems::Center, margin: UiRect::top(px(10.0)), row_gap: px(3.0), ..default() }, Visibility::Hidden)).id();
    let boss_name = c.spawn(txt("Ancient Golem", 16.0, 0xffd0a0)).id();
    let boss_bar = c.spawn((Node { width: px(460.0), height: px(14.0), border: UiRect::all(px(2.0)), ..default() }, BackgroundColor(col(0x140c0c, 0.85)), BorderColor(col(0x6ff3ff, 0.8)), BorderRadius::all(px(4.0)))).id();
    let boss_fill = c.spawn((Node { height: Val::Percent(100.0), width: Val::Percent(100.0), ..default() }, BackgroundColor(col(0x9a3a2a, 1.0)), BorderRadius::all(px(3.0)))).id();
    c.entity(boss_bar).add_child(boss_fill);
    c.entity(boss).add_children(&[boss_name, boss_bar]);
    let toast = c.spawn((Node { flex_direction: FlexDirection::Column, align_items: AlignItems::Center, margin: UiRect::top(px(16.0)), padding: UiRect::axes(px(26.0), px(8.0)), ..default() }, BackgroundColor(col(0x0c0a10, 0.5)), BorderRadius::all(px(8.0)), Visibility::Hidden)).id();
    let toast_a = c.spawn(txt("", 26.0, 0xffd27a)).id();
    let toast_b = c.spawn(txt("", 15.0, 0xe8dccc)).id();
    c.entity(toast).add_children(&[toast_a, toast_b]);
    c.entity(top).add_children(&[timer, threat_t, kills, boss, toast]);
    // top left: buffs and relics; top right: minimap and the day
    let buffs = c.spawn((txt("", 14.0, 0xe8dccc), abs(px(18.0), px(16.0), A, A))).id();
    let relics = c.spawn((txt("", 13.0, 0xcfe6ff), abs(px(18.0), px(120.0), A, A))).id();
    let mapbox = c.spawn((Node { position_type: PositionType::Absolute, top: px(14.0), right: px(14.0), width: px(168.0), height: px(168.0), border: UiRect::all(px(3.0)), flex_direction: FlexDirection::Column, overflow: Overflow::clip(), ..default() }, BorderColor(col(0x2a2420, 0.95)), BorderRadius::all(px(84.0)), BackgroundColor(col(0x000000, 0.4)))).id();
    let mapimg = c.spawn((ImageNode::new(map.clone()), Node { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() })).id();
    let me = c.spawn((Node { position_type: PositionType::Absolute, left: px(78.0), top: px(78.0), width: px(6.0), height: px(6.0), ..default() }, BackgroundColor(col(0xffffff, 1.0)), BorderRadius::all(px(3.0)))).id();
    c.entity(mapbox).add_children(&[mapimg, me]);
    let day = c.spawn((txt("Day 1", 14.0, 0xe8dccc), abs(A, px(188.0), px(18.0), A), TextLayout::new_with_justify(JustifyText::Right))).id();
    c.entity(hud).add_children(&[buffs, relics, mapbox, day]);
    // floating damage numbers
    let mut pops = vec![];
    for _ in 0..48 { let e = c.spawn((txt("", 18.0, 0xffffff), Node { position_type: PositionType::Absolute, ..default() }, Visibility::Hidden)).id(); c.entity(hud).add_child(e); pops.push(e); }

    // ---------------- menu ----------------
    let full = || Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, justify_content: JustifyContent::Center, align_items: AlignItems::Center, row_gap: px(10.0), ..default() };
    let menu = c.spawn((full(), BackgroundColor(col(0x08060c, 0.5)))).id();
    let title = c.spawn(txt("EMBERWILD", 92.0, EMBER)).id();
    let sub = c.spawn(txt("The horde never stops coming. Grow strong enough to outlast it.", 18.0, 0xe8dccc)).id();
    let row = c.spawn(Node { column_gap: px(10.0), margin: UiRect::top(px(20.0)), ..default() }).id();
    for (i, cls) in crate::models::CLASSES.into_iter().enumerate() {
        let (role, hp) = match cls { Class::Ranger => ("Fast skirmisher who wins from range and never stands still.", 100), Class::Mage => ("Glass cannon. Fire for damage, frost for control.", 85), Class::Warrior => ("Armored brawler who closes in and refuses to fall.", 140), Class::Priest => ("Holy bruiser who heals through anything.", 110),
            Class::Necromancer => ("Raises the dead to fight for him and reaps souls to survive.", 95), Class::Gunslinger => ("Fast hands, loud guns and a stick of dynamite for every problem.", 105) };
        let cc = class_color(cls);
        let b = c.spawn((Button, UiBtn::Class(cls), Node { width: px(196.0), height: px(290.0), flex_direction: FlexDirection::Column, padding: UiRect::all(px(12.0)), row_gap: px(8.0), border: UiRect::all(px(2.0)), ..default() }, BackgroundColor(col(0x100c14, 0.86)), BorderColor(col(cc, 0.5)), BorderRadius::all(px(10.0)))).id();
        let band = c.spawn((Node { height: px(4.0), width: Val::Percent(100.0), ..default() }, BackgroundColor(col(cc, 1.0)))).id();
        let n = c.spawn(txt(class_name(cls), 30.0, cc)).id();
        let r = c.spawn(txt(role, 14.0, 0xd8ccbc)).id();
        let h = c.spawn(txt(&format!("{} health", hp), 13.0, 0x9a8a78)).id();
        let abl = class_abilities(cls).iter().enumerate().map(|(k, a)| format!("{}  {}", k + 1, ab_info(a).name)).collect::<Vec<_>>().join("\n");
        let a = c.spawn(txt(&abl, 14.0, 0xf0e6d8)).id();
        let k = c.spawn((txt(&format!("Press {}", i + 1), 13.0, 0x9a8a78), Node { margin: UiRect::top(Val::Auto), ..default() })).id();
        c.entity(b).add_children(&[band, n, r, h, a, k]);
        c.entity(row).add_child(b);
    }
    let help = c.spawn((txt("WASD move  |  Left click attack  |  Right click block/parry  |  Ctrl dodge  |  1-4 abilities  |  Q swap  |  E interact  |  F eat  |  Tab character  |  Esc pause", 14.0, 0x9a8a78), Node { margin: UiRect::top(px(26.0)), ..default() })).id();
    c.entity(menu).add_children(&[title, sub, row, help]);

    // ---------------- level up ----------------
    let level = c.spawn((full(), BackgroundColor(col(0x06040a, 0.62)), Visibility::Hidden)).id();
    let lv_title = c.spawn(txt("LEVEL UP", 52.0, 0x8fd0ff)).id();
    let lv_sub = c.spawn(txt("Choose one", 17.0, 0xe8dccc)).id();
    let crow = c.spawn(Node { column_gap: px(18.0), margin: UiRect::top(px(20.0)), ..default() }).id();
    let mut cards = vec![];
    for i in 0..3 {
        let b = c.spawn((Button, UiBtn::Card(i), Node { width: px(300.0), height: px(250.0), flex_direction: FlexDirection::Column, padding: UiRect::all(px(18.0)), row_gap: px(12.0), border: UiRect::all(px(2.0)), ..default() }, BackgroundColor(col(0x100c14, 0.92)), BorderColor(col(0xffffff, 0.5)), BorderRadius::all(px(12.0)))).id();
        let tag = c.spawn(txt("", 13.0, 0xffffff)).id();
        let name = c.spawn(txt("", 28.0, 0xffffff)).id();
        let desc = c.spawn(txt("", 17.0, 0xe8dccc)).id();
        let key = c.spawn((txt(&format!("{}", i + 1), 15.0, 0x9a8a78), Node { margin: UiRect::top(Val::Auto), ..default() })).id();
        c.entity(b).add_children(&[tag, name, desc, key]);
        c.entity(crow).add_child(b);
        cards.push((b, tag, name, desc));
    }
    let reroll = c.spawn((txt("", 15.0, 0xb8aca0), Node { margin: UiRect::top(px(18.0)), ..default() })).id();
    c.entity(level).add_children(&[lv_title, lv_sub, crow, reroll]);

    // ---------------- pause & death ----------------
    let pause = c.spawn((full(), BackgroundColor(col(0x06040a, 0.6)), Visibility::Hidden)).id();
    let pt = c.spawn(txt("PAUSED", 64.0, 0xfff3d8)).id();
    c.entity(pause).add_child(pt);
    for (label, b) in [("Resume", UiBtn::Resume), ("Settings", UiBtn::Settings), ("Abandon run", UiBtn::Restart), ("Quit to desktop", UiBtn::Quit)] {
        let e = c.spawn((Button, b, Node { width: px(280.0), padding: UiRect::all(px(12.0)), justify_content: JustifyContent::Center, border: UiRect::all(px(2.0)), ..default() }, BackgroundColor(col(0x100c14, 0.9)), BorderColor(col(EMBER, 0.4)), BorderRadius::all(px(8.0)))).id();
        let t = c.spawn(txt(label, 20.0, 0xfff3d8)).id(); c.entity(e).add_child(t); c.entity(pause).add_child(e);
    }
    let dead = c.spawn((full(), BackgroundColor(col(0x140406, 0.7)), Visibility::Hidden)).id();
    let dt = c.spawn(txt("YOU HAVE FALLEN", 64.0, 0xff5a4a)).id();
    let dead_stats = c.spawn((txt("", 20.0, 0xe8dccc), TextLayout::new_with_justify(JustifyText::Center))).id();
    let db = c.spawn((Button, UiBtn::Restart, Node { width: px(300.0), padding: UiRect::all(px(12.0)), justify_content: JustifyContent::Center, border: UiRect::all(px(2.0)), margin: UiRect::top(px(18.0)), ..default() }, BackgroundColor(col(0x100c14, 0.9)), BorderColor(col(EMBER, 0.5)), BorderRadius::all(px(8.0)))).id();
    let dbt = c.spawn(txt("Rise again  (R)", 20.0, 0xfff3d8)).id(); c.entity(db).add_child(dbt);
    c.entity(dead).add_children(&[dt, dead_stats, db]);
    let victory = c.spawn((full(), BackgroundColor(col(0x1a1006, 0.72)), Visibility::Hidden)).id();
    let vt = c.spawn(txt("VICTORY", 80.0, 0xffd35a)).id();
    let vs = c.spawn(txt("The Ember King has fallen. The wild is yours.", 20.0, 0xfff3d8)).id();
    let vic_stats = c.spawn((txt("", 18.0, 0xe8dccc), TextLayout::new_with_justify(JustifyText::Center))).id();
    c.entity(victory).add_children(&[vt, vs, vic_stats]);
    for (label, b) in [("Keep fighting: endless  (C)", UiBtn::Endless), ("Return to camp  (R)", UiBtn::Restart)] {
        let e = c.spawn((Button, b, Node { width: px(340.0), padding: UiRect::all(px(12.0)), justify_content: JustifyContent::Center, border: UiRect::all(px(2.0)), margin: UiRect::top(px(10.0)), ..default() }, BackgroundColor(col(0x100c14, 0.9)), BorderColor(col(EMBER, 0.5)), BorderRadius::all(px(8.0)))).id();
        let t = c.spawn(txt(label, 20.0, 0xfff3d8)).id(); c.entity(e).add_child(t); c.entity(victory).add_child(e);
    }

    // put the side slots and ability slots into the bar in order
    let mut kids = vec![wroot]; kids.extend(slot_roots.iter().map(|s| s.0)); kids.push(froot);
    c.entity(bar).add_children(&kids);
    c.insert_resource(HudRefs { hud, hp_fill, hp_ghost, hp_text, xp_fill, lvl, stam_fill, slots: slots.iter().zip(slot_roots.iter()).map(|(s, r)| (s.1, s.2, s.3, r.1)).collect(),
        weapon, food, timer, threat: threat_t, kills, boss, boss_fill, boss_name, toast, toast_a, toast_b, buffs, relics, interact, cross, map, day, pops, flash, hurt,
        menu, level, lv_title, lv_sub, cards, reroll, pause, dead, dead_stats, victory, vic_stats });
    c.insert_resource(SlotRoots(slot_roots.iter().map(|s| s.0).collect()));
}
#[derive(Resource)]
pub struct SlotRoots(pub Vec<Entity>);
type UiQ<'w, 's> = Query<'w, 's, (Option<&'static mut Node>, Option<&'static mut Text>, Option<&'static mut TextColor>, Option<&'static mut BackgroundColor>, Option<&'static mut Visibility>, Option<&'static mut BorderColor>, Option<&'static mut TextFont>), Without<UiBtn>>;
fn set_text(q: &mut UiQ, e: Entity, s: &str) { if let Ok((_, Some(mut t), ..)) = q.get_mut(e) { if t.0 != s { t.0 = s.to_string(); } } }
fn set_tcol(q: &mut UiQ, e: Entity, c: Color) { if let Ok((_, _, Some(mut t), ..)) = q.get_mut(e) { t.0 = c; } }
fn set_w(q: &mut UiQ, e: Entity, pct: f32) { if let Ok((Some(mut n), ..)) = q.get_mut(e) { n.width = Val::Percent((pct * 100.0).clamp(0.0, 100.0)); } }
fn set_h(q: &mut UiQ, e: Entity, pct: f32) { if let Ok((Some(mut n), ..)) = q.get_mut(e) { n.height = Val::Percent((pct * 100.0).clamp(0.0, 100.0)); } }
fn set_vis(q: &mut UiQ, e: Entity, v: bool) { if let Ok((_, _, _, _, Some(mut vis), ..)) = q.get_mut(e) { *vis = if v { Visibility::Inherited } else { Visibility::Hidden }; } }
fn set_bg(q: &mut UiQ, e: Entity, c: Color) { if let Ok((_, _, _, Some(mut b), ..)) = q.get_mut(e) { b.0 = c; } }
fn set_border(q: &mut UiQ, e: Entity, c: Color) { if let Ok((_, _, _, _, _, Some(mut b), _)) = q.get_mut(e) { b.0 = c; } }
fn set_size(q: &mut UiQ, e: Entity, size: f32) { if let Ok((.., Some(mut f))) = q.get_mut(e) { f.font_size = size; } }
fn set_pos(q: &mut UiQ, e: Entity, x: f32, y: f32) { if let Ok((Some(mut n), ..)) = q.get_mut(e) { n.left = px(x); n.top = px(y); } }

pub fn cd_base(g: &Game, id: &str) -> f32 { ab_info(id).cd * g.st.cd * (1.0 - 0.22 * g.m(&format!("{}.cd", id)) as f32) }
fn fmt_t(t: f32) -> String { let s = t.max(0.0) as u32; format!("{}:{:02}", s / 60, s % 60) }

/// screen flow: menus, level-ups, pause and death
pub fn ui_flow(mut save: ResMut<crate::meta::Save>, mut ui: ResMut<UiState>, keys: Res<ButtonInput<KeyCode>>, btns: Query<(&Interaction, &UiBtn), Changed<Interaction>>, mut g: ResMut<Game>, mut p: ResMut<Player>,
    mut paused: ResMut<crate::Paused>, mut ctl: ResMut<Controls>, time: Res<Time>, mut sfx: ResMut<crate::audio::Sfx>, mut exit: EventWriter<AppExit>, test: Option<Res<crate::testkit::TestCfg>>) {
    let mut clicked = None;
    for (i, b) in btns.iter() { if *i == Interaction::Pressed { clicked = Some(*b); } }
    match ui.screen {
        Screen::Menu => {
            let mut want = None;
            for (i, k) in [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4, KeyCode::Digit5, KeyCode::Digit6].into_iter().enumerate() { if keys.just_pressed(k) { want = Some(crate::models::CLASSES[i]); } }
            if let Some(UiBtn::Class(c)) = clicked { want = Some(c); }
            if let Some(c) = want { if save.class_open(c) { ui.start = Some(c); ui.paid = 0; ui.recorded = false; } else { sfx.play("hurt"); } }
            if keys.just_pressed(KeyCode::KeyC) || clicked == Some(UiBtn::Camp) { ui.screen = Screen::Camp; sfx.play("pick"); }
            if keys.just_pressed(KeyCode::KeyH) || clicked == Some(UiBtn::Host) { ui.req_host = true; sfx.play("pick"); }
            if keys.just_pressed(KeyCode::KeyJ) || clicked == Some(UiBtn::Join) { ui.screen = Screen::Join; sfx.play("pick"); }
            if keys.just_pressed(KeyCode::KeyS) || clicked == Some(UiBtn::Settings) { ui.settings_from = Some(Screen::Menu); ui.screen = Screen::Settings; sfx.play("pick"); }
            if let Some(UiBtn::Route(k)) = clicked { ui.route = k; sfx.play("pick"); }
            if keys.just_pressed(KeyCode::KeyR) { ui.route = (ui.route + 1) % 3; sfx.play("pick"); }
            if clicked == Some(UiBtn::EmberUp) && save.ember < save.max_ember { save.ember += 1; save.write(); sfx.play("pick"); }
            if clicked == Some(UiBtn::EmberDown) && save.ember > 0 { save.ember -= 1; save.write(); sfx.play("pick"); }
        }
        Screen::Playing => {
            if g.flags.remove("open_shop").is_some() { ui.screen = Screen::Shop; sfx.play("chest"); }
            else if keys.just_pressed(KeyCode::Escape) && !g.over { ui.screen = Screen::Paused; sfx.play("pick"); }
            else if keys.just_pressed(KeyCode::Tab) && !g.over && !p.dead { ui.screen = Screen::Character; sfx.play("pick"); }
            else if (g.level_pending > 0 || g.chest_pending > 0) && !g.over && !p.dead {
                ui.relic_only = g.level_pending == 0 && g.flags.remove("relic_chest").is_some();
                ui.screen = Screen::LevelUp; ui.cards_dirty = true; p.charging = false; sfx.play("level");
            }
            if g.over { ui.dead_t += time.delta_secs(); if ui.dead_t > 2.2 { ui.screen = Screen::Dead; } }
            if let Some(&t) = g.flags.get("victory") { if !g.flags.contains_key("endless") && g.run_t - t > 3.0 && !g.over { ui.screen = Screen::Victory; sfx.play("relic"); } }
        }
        Screen::LevelUp => {
            if ui.cards_dirty { ui.cards_dirty = false; let cls = p.cls; let ro = ui.relic_only; let mut rng = ui.rng; let cards = roll(&g, cls, ro, &mut || { rng ^= rng << 13; rng ^= rng >> 17; rng ^= rng << 5; (rng % 1_000_000) as f32 / 1_000_000.0 }); ui.rng = rng; ui.cards = cards; }
            let mut pick = None;
            for (i, k) in [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3].into_iter().enumerate() { if keys.just_pressed(k) { pick = Some(i); } }
            if let Some(UiBtn::Card(i)) = clicked { pick = Some(i); }
            if test.as_ref().map(|t| t.autopick).unwrap_or(false) && !ui.cards.is_empty() { pick = Some((ui.rnd() * ui.cards.len() as f32) as usize % ui.cards.len()); }
            if keys.just_pressed(KeyCode::KeyR) && g.rerolls > 0 { g.rerolls -= 1; ui.cards_dirty = true; sfx.play("pick"); }
            if let Some(i) = pick { if let Some(card) = ui.cards.get(i).cloned() {
                apply(&mut g, &mut p, &card);
                sfx.play(match card.kind { CardKind::Chaos => "bomb", CardKind::Relic | CardKind::Evolution => "relic", _ => "pick" });
                if card.kind == CardKind::Evolution { g.toasts.push((format!("EVOLUTION: {}", card.name), card.desc.clone(), 3.0)); g.flash = 0.3; }
                if card.kind == CardKind::Relic && g.m(&card.key) == 1 { g.toasts.push((card.name.clone(), "A new relic joins your arsenal".into(), 2.0)); }
                if g.level_pending > 0 { g.level_pending -= 1; } else if g.chest_pending > 0 { g.chest_pending -= 1; }
                if g.level_pending > 0 || g.chest_pending > 0 { ui.cards_dirty = true; ui.relic_only = g.level_pending == 0 && g.flags.remove("relic_chest").is_some(); } else { ui.screen = Screen::Playing; ctl.relock = true; }
            } }
        }
        Screen::Paused => {
            if clicked == Some(UiBtn::Settings) { ui.settings_from = Some(Screen::Paused); ui.screen = Screen::Settings; }
            else if keys.just_pressed(KeyCode::Escape) || clicked == Some(UiBtn::Resume) { ui.screen = Screen::Playing; ctl.relock = true; }
            if clicked == Some(UiBtn::Restart) { ui.restart = true; }
            if clicked == Some(UiBtn::Quit) { exit.send(AppExit::Success); }
        }
        Screen::Dead => { if keys.just_pressed(KeyCode::KeyR) || clicked == Some(UiBtn::Restart) { ui.restart = true; } }
        Screen::Join => { if keys.just_pressed(KeyCode::Escape) || clicked == Some(UiBtn::Back) { ui.screen = Screen::Menu; } }
        Screen::Character => { if keys.just_pressed(KeyCode::Tab) || keys.just_pressed(KeyCode::Escape) || g.over { ui.screen = Screen::Playing; ctl.relock = true; } }
        Screen::Settings => {
            if keys.just_pressed(KeyCode::Escape) || clicked == Some(UiBtn::Back) { ui.screen = ui.settings_from.take().unwrap_or(Screen::Menu); save.write(); }
            if let Some(UiBtn::Set(i, up)) = clicked { crate::settings::change(&mut save.settings, i as usize, up); sfx.play("pick"); }
        }
        Screen::Shop => {
            if keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::KeyE) || clicked == Some(UiBtn::Back) { ui.screen = Screen::Playing; ctl.relock = true; }
            let mut pick = None;
            for (i, k) in [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4].into_iter().enumerate() { if keys.just_pressed(k) { pick = Some(i); } }
            if let Some(UiBtn::ShopBuy(i)) = clicked { pick = Some(i); }
            if let Some(i) = pick { let (gg, pp) = (&mut *g, &mut *p); sfx.play(if crate::events::buy(gg, pp, i) { "chest" } else { "hurt" }); }
            if (g.level_pending > 0 || g.chest_pending > 0) && ui.screen == Screen::Shop { ui.relic_only = g.level_pending == 0 && g.flags.remove("relic_chest").is_some(); ui.screen = Screen::LevelUp; ui.cards_dirty = true; }
        }
        Screen::Camp => {
            if keys.just_pressed(KeyCode::Escape) || clicked == Some(UiBtn::Back) { ui.screen = Screen::Menu; }
            if let Some(UiBtn::Buy(i)) = clicked { sfx.play(if save.buy(i) { "relic" } else { "hurt" }); }
            if let Some(UiBtn::Unlock(i)) = clicked { sfx.play(if save.unlock(i) { "chest" } else { "hurt" }); }
        }
        Screen::Victory => {
            if keys.just_pressed(KeyCode::KeyR) || clicked == Some(UiBtn::Restart) { ui.restart = true; }
            if keys.just_pressed(KeyCode::KeyC) || clicked == Some(UiBtn::Endless) { g.flags.insert("endless", 1.0); ui.screen = Screen::Playing; ctl.relock = true; g.toasts.push(("Endless".into(), "Bosses return every four minutes, stronger each time".into(), 3.0)); }
        }
    }
    if matches!(ui.screen, Screen::Dead | Screen::Victory) {
        let total = crate::meta::payout(&g);
        if total > ui.paid {
            let gain = total - ui.paid; ui.paid = total; ui.last_gain = gain;
            if !ui.recorded { ui.recorded = true; crate::meta::record_run(&mut save, &g, p.cls, gain); } else { save.gold += gain; save.write(); }
        }
    }
    paused.0 = ui.screen != Screen::Playing && !g.net.on();
    ctl.want_lock = ui.screen == Screen::Playing;
}

/// hover highlight for every button
pub fn button_hover(mut q: Query<(&Interaction, &UiBtn, &mut BorderColor, &mut BackgroundColor)>, ui: Res<UiState>) {
    for (i, b, mut bc, mut bg) in q.iter_mut() {
        let base = match b { UiBtn::Class(c) => class_color(*c), UiBtn::Card(k) => ui.cards.get(*k).map(|c| c.color).unwrap_or(0xffffff), UiBtn::Route(k) => [0xd8ccbc, 0x6fd4a0, 0xffc86a][*k as usize % 3], _ => EMBER };
        let picked = matches!(b, UiBtn::Route(k) if *k == ui.route);
        let (a, l) = match i { Interaction::Hovered => (1.0, 0.95), Interaction::Pressed => (1.0, 1.0), _ if picked => (1.0, 0.95), _ => (0.4, 0.88) };
        bc.0 = col(base, a); bg.0 = col(0x100c14, l);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn update_hud(mut ui: ResMut<UiState>, r: Res<HudRefs>, mut q: UiQ, mut g: ResMut<Game>, p: Res<Player>, dc: Res<DayCycle>, time: Res<Time>,
    cam: Query<(&Camera, &GlobalTransform), With<MainCam>>, w: Option<Res<Voxels>>, mut images: ResMut<Assets<Image>>, ctl: Res<Controls>, cfg: Res<crate::GameCfg>) {
    let dt = time.delta_secs();
    let scr = ui.screen;
    set_vis(&mut q, r.menu, scr == Screen::Menu);
    set_vis(&mut q, r.hud, !matches!(scr, Screen::Menu | Screen::Camp));
    set_vis(&mut q, r.level, scr == Screen::LevelUp);
    set_vis(&mut q, r.hud, !matches!(scr, Screen::Menu | Screen::Camp | Screen::Shop | Screen::Settings | Screen::Join | Screen::Character));
    set_vis(&mut q, r.pause, scr == Screen::Paused);
    set_vis(&mut q, r.dead, scr == Screen::Dead);
    set_vis(&mut q, r.victory, scr == Screen::Victory);
    if matches!(scr, Screen::Menu | Screen::Camp | Screen::Settings | Screen::Join) { return; }
    // health, experience, stamina
    let hp = (p.hp / p.max_hp).clamp(0.0, 1.0);
    if ui.ghost_hp < hp { ui.ghost_hp = hp; } else { ui.ghost_hp = (ui.ghost_hp - dt * 0.35).max(hp); }
    set_w(&mut q, r.hp_fill, hp); set_w(&mut q, r.hp_ghost, ui.ghost_hp);
    set_text(&mut q, r.hp_text, &format!("{} / {}", p.hp.max(0.0).ceil() as i32, p.max_hp as i32));
    set_bg(&mut q, r.hp_fill, if g.buff("shield") > 0.0 { col(0xffd35a, 1.0) } else if hp < 0.3 { col(0xff2a2a, 0.75 + 0.25 * (time.elapsed_secs() * 8.0).sin()) } else { col(0xd8343a, 1.0) });
    set_w(&mut q, r.xp_fill, g.xp / g.xp_need());
    set_text(&mut q, r.lvl, &format!("Lv {}", g.lvl));
    set_w(&mut q, r.stam_fill, p.stam / g.st.stam_max);
    // ability bar
    let abs_ = class_abilities(p.cls);
    for (i, (cd_e, cdt, name, glow)) in r.slots.iter().enumerate() {
        let id = abs_[i]; let info = ab_info(id);
        let left = g.cds[i]; let base = cd_base(&g, id);
        set_h(&mut q, *cd_e, if left > 0.0 { left / base } else { 0.0 });
        set_text(&mut q, *cdt, &if left > 0.0 { if left < 1.0 { format!("{:.1}", left) } else { format!("{}", left.ceil() as i32) } } else { String::new() });
        set_text(&mut q, *name, info.name);
        let ready = left <= 0.0;
        set_bg(&mut q, *glow, col(info.color, if ready { 1.0 } else { 0.35 }));
    }
    let wid = p.weapon_id();
    set_text(&mut q, r.weapon, &if wid == "revolvers" { if p.reload > 0.0 { format!("{}\nreloading", weapon_info(wid).0) } else { format!("{}\n{} / {}", weapon_info(wid).0, p.ammo, crate::combat::CYLINDER) } } else { weapon_info(wid).0.to_string() });
    set_text(&mut q, r.food, &format!("{} meat  {} berry", g.food_meat, g.food_berries));
    // clock, threat, kills, day
    let m = g.minutes(); let (tn, tc) = threat(m);
    set_text(&mut q, r.timer, &fmt_t(g.stage_t));
    set_text(&mut q, r.threat, tn); set_tcol(&mut q, r.threat, srgb(tc));
    let nb = crate::director::next_boss(&g).map(|(t, n)| if t > 0.0 { format!("  |  {} in {}", n, fmt_t(t)) } else { String::new() }).unwrap_or_default();
    set_text(&mut q, r.kills, &format!("{} slain  |  {} gold{}", g.kills, g.gold, nb));
    let tod = if dc.blood { "Blood moon" } else if dc.night { "Night" } else { "Day" };
    set_text(&mut q, r.day, &format!("Stage {}/3  {}\nDay {}  |  {}", g.stage + 1, g.map.name(), dc.day, tod));
    // boss
    let boss = g.boss.and_then(|id| g.enemies.iter().find(|e| e.id == id && !e.dead));
    set_vis(&mut q, r.boss, boss.is_some());
    if let Some(b) = boss { set_w(&mut q, r.boss_fill, b.hp / b.max_hp); set_text(&mut q, r.boss_name, b.t.name); }
    // toasts: the newest replaces the last
    if let Some(t) = g.toasts.pop() { ui.toast = Some((t.0, t.1, t.2, 0.0)); g.toasts.clear(); }
    let mut show = false;
    if let Some(t) = ui.toast.as_mut() { t.3 += dt; if t.3 < t.2 + 0.5 { show = true; } }
    if show { let t = ui.toast.clone().unwrap(); let a = (t.3 / 0.15).min(1.0) * ((t.2 + 0.5 - t.3) / 0.5).clamp(0.0, 1.0);
        set_text(&mut q, r.toast_a, &t.0); set_text(&mut q, r.toast_b, &t.1); set_tcol(&mut q, r.toast_a, col(0xffd27a, a)); set_tcol(&mut q, r.toast_b, col(0xe8dccc, a)); set_bg(&mut q, r.toast, col(0x0c0a10, 0.5 * a)); }
    set_vis(&mut q, r.toast, show);
    // buffs and relics
    let mut bl = vec![];
    for (k, n) in [("cry", "Battle Cry"), ("shield", "Divine Shield"), ("shrine", "Blessed"), ("hot", "Renewing"), ("haste", "Haste")] { let v = g.buff(k); if v > 0.0 { bl.push(format!("{}  {:.0}s", n, v.ceil())); } }
    if g.dir.trial.is_some() { bl.push(format!("Altar trial  {:.0}s", g.dir.trial.as_ref().unwrap().t.max(0.0))); }
    bl.extend(crate::events::status(&g));
    for r in g.net.remotes.iter().filter(|r| r.state.in_run) { bl.push(if r.state.dead { format!("Player {}  DOWN", r.id) } else { format!("Player {}  {:.0}/{:.0}", r.id, r.state.hp.max(0.0), r.state.max_hp) }); }
    if p.hunger <= 0.0 { bl.push("Starving".into()); } else if p.hunger > 75.0 { bl.push("Well fed".into()); }
    set_text(&mut q, r.buffs, &bl.join("\n"));
    let rl: Vec<String> = crate::relics::RELICS.iter().filter_map(|x| { let k = crate::relics::rk(&g, x.id); if crate::relics::ev(&g, x.id) { crate::relics::EVOS.iter().find(|e| e.0 == x.id).map(|e| format!("{}  *", e.3)) } else if k > 0 { Some(format!("{}  {}", x.name, "I".repeat(k as usize))) } else { None } }).collect();
    let au: Vec<String> = crate::augments::AUGS.iter().filter_map(|x| { let k = g.m(&format!("x.{}", x.id)); if k > 0 { Some(format!("{} {}", x.name, k)) } else { None } }).collect();
    set_text(&mut q, r.relics, &[rl.join("\n"), au.join("\n")].iter().filter(|s| !s.is_empty()).cloned().collect::<Vec<_>>().join("\n\n"));
    // crosshair turns red on a hit
    let hit = g.flags.get("cross_hit").copied().unwrap_or(0.0);
    if hit > 0.0 { g.flags.insert("cross_hit", hit - dt); }
    for &e in r.cross.iter() { set_bg(&mut q, e, if hit > 0.0 { col(0xff5a4a, 1.0) } else { col(0xffffff, if ctl.locked { 0.85 } else { 0.4 }) }); }
    // screen flashes
    set_bg(&mut q, r.flash, col(0xfff0c0, (g.flash * 2.0).min(0.5)));
    set_border(&mut q, r.hurt, col(0xb01020, (p.hit_t * 1.6).min(0.5) + if hp < 0.25 && !p.dead { 0.25 + 0.1 * (time.elapsed_secs() * 4.0).sin() } else { 0.0 }));
    // interact prompt
    let prompt = g.prompt.clone(); set_text(&mut q, r.interact, &prompt);
    // damage numbers rise and fade over the world
    if let Ok((camera, ct)) = cam.get_single() {
        if !cfg.damage_numbers { g.popups.retain(|p| p.kind == 1 || p.kind >= 3); }
        for pu in g.popups.drain(..) {
            let free = ui.pops.iter().position(|x| !x.2).or_else(|| if ui.pops.len() < r.pops.len() { ui.pops.push((Vec3::ZERO, 0.0, false)); Some(ui.pops.len() - 1) } else { None });
            let Some(k) = free.or(Some(0)) else { continue };
            let jx = ui.rnd() - 0.5;
            ui.pops[k] = (pu.pos + Vec3::new(jx * 0.6, 0.0, 0.0), 0.0, true);
            let e = r.pops[k];
            let (size, c) = match pu.kind { 1 => (28.0, 0xffd23a), 2 => (20.0, 0x8dff9e), 3 => (18.0, 0xbfe8ff), 4 => (22.0, 0xff4a3a), _ => (17.0, 0xffffff) };
            set_text(&mut q, e, &pu.text); set_tcol(&mut q, e, srgb(c));
            set_size(&mut q, e, size);
        }
        for (k, it) in ui.pops.iter_mut().enumerate() {
            if !it.2 { continue; }
            it.1 += dt;
            let e = r.pops[k];
            if it.1 > 0.85 { it.2 = false; set_vis(&mut q, e, false); continue; }
            let wp = it.0 + Vec3::Y * (it.1 * 1.6);
            match camera.world_to_viewport(ct, wp) { Ok(v) => { set_vis(&mut q, e, true); set_pos(&mut q, e, v.x - 12.0, v.y - 10.0); let a = ((0.85 - it.1) / 0.3).min(1.0); if let Ok((_, _, Some(mut tc), ..)) = q.get_mut(e) { tc.0 = tc.0.with_alpha(a); } } Err(_) => set_vis(&mut q, e, false) }
        }
    } else { g.popups.clear(); }
    // minimap: terrain from loaded chunks, monsters as dots, north up
    ui.map_t -= dt;
    if ui.map_t <= 0.0 { if let Some(w) = w.as_ref() { ui.map_t = 0.2; draw_map(w, &g, &p, &mut images, &r.map); } }
    // death summary
    if scr == Screen::Victory { set_text(&mut q, r.vic_stats, &format!("{}  |  {}  |  level {}  |  {} slain\n+{} gold", class_name(p.cls), fmt_t(g.run_t), g.lvl, g.kills, ui.last_gain)); }
    if scr == Screen::Dead { set_text(&mut q, r.dead_stats, &format!("{}  |  survived {}  |  level {}  |  {} slain\n+{} gold", class_name(p.cls), fmt_t(g.run_t), g.lvl, g.kills, ui.last_gain)); }
    // level-up cards
    if scr == Screen::LevelUp {
        let chest = g.level_pending == 0;
        set_text(&mut q, r.lv_title, if chest { "TREASURE" } else { "LEVEL UP" });
        set_tcol(&mut q, r.lv_title, srgb(if chest { 0xffd35a } else { 0x8fd0ff }));
        set_text(&mut q, r.lv_sub, &if chest { if ui.relic_only { "The altar offers its power. Choose one.".to_string() } else { "The chest holds a relic. Choose one.".into() } } else { format!("Level {}. Choose one{}", g.lvl - g.level_pending + 1, if g.level_pending > 1 { format!("  |  {} more to come", g.level_pending - 1) } else { String::new() }) });
        for (i, (b, tag, name, desc)) in r.cards.iter().enumerate() {
            match ui.cards.get(i) {
                Some(c) => { set_vis(&mut q, *b, true); set_text(&mut q, *tag, &c.tag.to_uppercase()); set_tcol(&mut q, *tag, srgb(c.color)); set_text(&mut q, *name, &c.name); set_tcol(&mut q, *name, srgb(match c.kind { CardKind::Chaos => 0xff7ae8, CardKind::Mastery | CardKind::Evolution => 0xffd27a, _ => 0xfff3d8 })); set_text(&mut q, *desc, &c.desc); }
                None => set_vis(&mut q, *b, false),
            }
        }
        set_text(&mut q, r.reroll, &if g.rerolls > 0 { format!("R  reroll  ({} left)", g.rerolls) } else { String::new() });
    }
}
fn draw_map(w: &Voxels, g: &Game, p: &Player, images: &mut Assets<Image>, h: &Handle<Image>) {
    let Some(img) = images.get_mut(h) else { return };
    let (px0, pz0) = (p.body.pos.x.floor() as i32, p.body.pos.z.floor() as i32);
    let s = 128i32; let scale = 1;
    let data = &mut img.data;
    for j in 0..s { for i in 0..s {
        let (x, z) = (px0 + (i - s / 2) * scale, pz0 + (j - s / 2) * scale);
        let (cx, cz) = (x.div_euclid(CS), z.div_euclid(CS));
        let o = ((j * s + i) * 4) as usize;
        let c = match w.chunks.get(&(cx, cz)) {
            Some(ch) => { let k = (x.rem_euclid(CS) + z.rem_euclid(CS) * CS) as usize; let (t, b) = (ch.top[k] as i32, ch.top_b[k]);
                if t <= crate::world::SEA { match w.kind { crate::world::MapKind::Volcanic => [230u8, 90, 20], crate::world::MapKind::Swamp => [50, 80, 50], _ => [40, 90, 150] } } else { let d = block_def(b); let c = d.top.unwrap_or(d.c); let s = Color::linear_rgb(c[0], c[1], c[2]).to_srgba(); let sh = 0.75 + (t as f32 - 22.0) * 0.02; [(s.red * 255.0 * sh).min(255.0) as u8, (s.green * 255.0 * sh).min(255.0) as u8, (s.blue * 255.0 * sh).min(255.0) as u8] } }
            None => [18, 20, 26],
        };
        data[o] = c[0]; data[o + 1] = c[1]; data[o + 2] = c[2]; data[o + 3] = 255;
    } }
    let mut dot = |x: f32, z: f32, c: [u8; 3], r: i32| {
        let (i, j) = (((x - p.body.pos.x) / scale as f32) as i32 + s / 2, ((z - p.body.pos.z) / scale as f32) as i32 + s / 2);
        for dy in -r..=r { for dx in -r..=r { let (a, b) = (i + dx, j + dy); if a < 0 || b < 0 || a >= s || b >= s { continue; } let o = ((b * s + a) * 4) as usize; data[o] = c[0]; data[o + 1] = c[1]; data[o + 2] = c[2]; } }
    };
    for c in g.dir.chests.iter() { if !c.open { dot(c.pos.x, c.pos.z, [255, 210, 80], 2); } }
    for e in g.enemies.iter() { if e.dead { continue; } let (c, r) = if e.boss { ([111, 243, 255], 3) } else if e.elite { ([255, 180, 60], 2) } else if e.t.passive { ([240, 230, 220], 0) } else if e.t.prop { ([200, 140, 80], 1) } else { ([230, 50, 50], 1) }; dot(e.body.pos.x, e.body.pos.z, c, r); }
    // facing tick
    let f = p.fwd();
    for k in 1..7 { let (x, z) = (p.body.pos.x + f.x * k as f32, p.body.pos.z + f.z * k as f32); dot(x, z, [255, 255, 255], 0); }
}

/// the title screen camera drifts slowly around the spawn
pub fn menu_camera(ui: Res<UiState>, time: Res<Time>, p: Res<Player>, mut cam: Query<&mut Transform, With<MainCam>>) {
    if ui.screen != Screen::Menu { return; }
    let Ok(mut t) = cam.get_single_mut() else { return };
    let a = time.elapsed_secs() * 0.05; let c = p.body.pos;
    t.translation = c + Vec3::new(a.cos() * 26.0, 14.0, a.sin() * 26.0);
    t.look_at(c + Vec3::Y * 4.0, Vec3::Y);
}
