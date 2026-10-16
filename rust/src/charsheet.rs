//! The character panel (Tab): every stat, the abilities and weapons, and every pick so far (relics, evolutions,
//! chaos augments, boons, ability and weapon upgrades) as icons with their rank. Hovering any icon, or an ability
//! slot on the HUD whenever the cursor is free, shows a tooltip.
use crate::abilities::{ab_info, class_abilities, MODS, WMODS};
use crate::augments::{AUGS, AUG_COLOR};
use crate::combat::weapon_info;
use crate::game::Game;
use crate::hud::{cd_base, Screen, UiState};
use crate::icons::{glyph_for, glyph_for_id, Icons};
use crate::models::srgb;
use crate::player::{class_weapons, Player};
use crate::progression::{class_color, class_name, PERKS};
use crate::relics::{rk, EVOS, RELICS, RMAX};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

/// hover text for a UI node
#[derive(Component, Clone, Default)]
pub struct Tip { pub title: String, pub tag: String, pub body: String, pub color: u32 }
/// an ability slot; its tooltip is built live (cooldowns and upgrades change)
#[derive(Component, Clone, Copy)]
pub struct AbilityTip(pub usize);

#[derive(Resource)]
pub struct TipRefs { root: Entity, title: Entity, tag: Entity, body: Entity }
#[derive(Resource, Default)]
pub struct Panel { root: Option<Entity> }

fn col(hex: u32, a: f32) -> Color { srgb(hex).with_alpha(a) }
fn txt(s: &str, size: f32, hex: u32) -> (Text, TextFont, TextColor) { (Text::new(s), TextFont { font_size: size, ..default() }, TextColor(srgb(hex))) }
fn px(v: f32) -> Val { Val::Px(v) }
fn roman(n: u32) -> &'static str { ["", "I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X"][n.min(10) as usize] }

pub fn setup_tips(mut c: Commands, kit: Res<crate::uikit::UiTex>) {
    let root = c.spawn((Node { position_type: PositionType::Absolute, flex_direction: FlexDirection::Column, row_gap: px(4.0), padding: UiRect::all(px(16.0)), width: px(350.0), border: UiRect::top(px(3.0)), ..default() },
        crate::uikit::stone(&kit), BorderColor(col(0xffffff, 0.5)), GlobalZIndex(60), Visibility::Hidden)).id();
    let wrap = || Node { max_width: px(314.0), ..default() };
    let title = c.spawn((txt("", 19.0, 0xfff3d8), wrap())).id();
    let tag = c.spawn((txt("", 13.0, 0xffd27a), wrap())).id();
    let body = c.spawn((txt("", 14.0, 0xd8ccbc), wrap())).id();
    c.entity(root).add_children(&[title, tag, body]);
    c.insert_resource(TipRefs { root, title, tag, body });
}

/// the live tooltip for ability slot i
pub fn ability_tip(g: &Game, p: &Player, i: usize) -> Tip {
    let id = class_abilities(p.cls)[i]; let info = ab_info(id);
    let mut body = info.desc.to_string();
    let ups: Vec<String> = MODS.iter().filter(|m| m.0 == id).filter_map(|m| { let r = g.m(&format!("{}.{}", id, m.1)); (r > 0).then(|| format!("\n- {}{}: {}", m.2, if m.4 > 1 { format!(" {}", roman(r)) } else { String::new() }, m.3)) }).collect();
    if ups.is_empty() { body += "\n\nNo upgrades yet."; } else { body += "\n"; for u in ups { body += &u; } }
    let left = g.cds[i];
    let lv = crate::abilities::ab_level(g, id); let ub = crate::abilities::unbound(g, id);
    body += &format!("\n\n{} {}: {}.", if ub { "UNBOUND, level" } else { "Level" }, lv, crate::abilities::growth(id));
    Tip { title: info.name.into(), tag: format!("Ability {}  |  {:.1}s cooldown{}", i + 1, cd_base(g, id), if left > 0.0 { format!("  |  ready in {:.1}s", left) } else { "  |  ready".into() }), body, color: info.color }
}

pub fn update_tips(r: Res<TipRefs>, hov: Query<(&Interaction, Option<&Tip>, Option<&AbilityTip>, &InheritedVisibility)>, win: Query<&Window, With<PrimaryWindow>>, g: Res<Game>, p: Res<Player>,
    mut q: Query<(Option<&mut Node>, Option<&mut Text>, Option<&mut TextColor>, Option<&mut Visibility>, Option<&mut BorderColor>)>) {
    let mut tip = None;
    for (i, t, a, vis) in hov.iter() {
        if *i == Interaction::None || !vis.get() { continue; }
        if let Some(a) = a { if p.spawned { tip = Some(ability_tip(&g, &p, a.0)); } } else if let Some(t) = t { tip = Some(t.clone()); }
    }
    let cursor = win.get_single().ok().and_then(|w| w.cursor_position().map(|c| (c, w.width(), w.height())));
    let (Some(t), Some((c, ww, wh))) = (tip, cursor) else { if let Ok((_, _, _, Some(mut v), _)) = q.get_mut(r.root) { *v = Visibility::Hidden; } return };
    if let Ok((Some(mut n), _, _, Some(mut v), Some(mut b))) = q.get_mut(r.root) {
        *v = Visibility::Inherited; b.0 = col(t.color, 0.8);
        // keep it on screen: flip to the left or above the cursor near the edges
        let (x, y) = (if c.x > ww - 360.0 { c.x - 350.0 } else { c.x + 18.0 }, if c.y > wh - 220.0 { (c.y - 200.0).max(0.0) } else { c.y + 18.0 });
        n.left = px(x); n.top = px(y);
    }
    for (e, s, hex) in [(r.title, t.title, t.color), (r.tag, t.tag, 0xb8aca0), (r.body, t.body, 0xd8ccbc)] {
        if let Ok((_, Some(mut tx), Some(mut tc), _, _)) = q.get_mut(e) { if tx.0 != s { tx.0 = s; } tc.0 = srgb(if e == r.title { hex } else { hex }); }
    }
}

/// one icon tile: a tinted glyph on a dark tile with a coloured border and an optional rank badge
fn tile(c: &mut Commands, icons: &Icons, glyph: &str, color: u32, border: u32, size: f32, badge: &str, tip: Option<Tip>, ab: Option<usize>) -> Entity {
    let kit = KIT.with(|k| k.borrow().clone()).expect("ui kit");
    let t = c.spawn((Node { width: px(size), height: px(size), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
        if border == 0xffd35a { crate::uikit::gold(&kit) } else { crate::uikit::iron(&kit) }, BackgroundColor(col(0x15111a, 0.0)), Interaction::default())).id();
    if border != 0xffd35a { let edge = c.spawn((Node { position_type: PositionType::Absolute, left: px(4.0), right: px(4.0), bottom: px(4.0), height: px(3.0), ..default() }, BackgroundColor(col(border, 0.9)))).id(); c.entity(t).add_child(edge); }
    if let Some(tp) = tip { c.entity(t).insert(tp); }
    if let Some(i) = ab { c.entity(t).insert(AbilityTip(i)); }
    if let Some(h) = icons.glyphs.get(glyph) { let im = c.spawn((ImageNode { image: h.clone(), color: srgb(color), ..default() }, Node { width: px(size * 0.72), height: px(size * 0.72), ..default() })).id(); c.entity(t).add_child(im); }
    if !badge.is_empty() {
        let b = c.spawn((txt(badge, 12.0, 0xffffff), Node { position_type: PositionType::Absolute, right: px(3.0), bottom: px(1.0), ..default() })).id();
        c.entity(t).add_child(b);
    }
    t
}
fn section(c: &mut Commands, parent: Entity, title: &str, tiles: Vec<Entity>) {
    let h = c.spawn((txt(title, 15.0, 0xffd27a), Node { margin: UiRect::top(px(8.0)), ..default() })).id();
    c.entity(parent).add_child(h);
    if tiles.is_empty() { let n = c.spawn(txt("None yet", 13.0, 0x6a5e52)).id(); c.entity(parent).add_child(n); return; }
    let row = c.spawn(Node { flex_wrap: FlexWrap::Wrap, column_gap: px(6.0), row_gap: px(6.0), max_width: px(640.0), ..default() }).id();
    c.entity(row).add_children(&tiles); c.entity(parent).add_child(row);
}

thread_local! { static KIT: std::cell::RefCell<Option<crate::uikit::UiTex>> = const { std::cell::RefCell::new(None) }; }
fn build(c: &mut Commands, icons: &Icons, g: &Game, p: &Player, kit: &crate::uikit::UiTex) -> Entity {
    KIT.with(|k| *k.borrow_mut() = Some(kit.clone()));
    let root = c.spawn((Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, justify_content: JustifyContent::Center, row_gap: px(10.0), ..default() },
        BackgroundColor(col(0x08060a, 0.7)), GlobalZIndex(30), crate::uikit::Appear::default())).id();
    let secs = g.run_t as u32;
    let head = c.spawn((Node { padding: UiRect::axes(px(34.0), px(10.0)), ..default() }, crate::uikit::stone(kit))).id();
    let title = c.spawn(txt(&format!("{}   |   Level {}   |   {}:{:02}   |   {} slain   |   {} gold", class_name(p.cls).to_uppercase(), g.lvl, secs / 60, secs % 60, g.kills, g.gold), 24.0, class_color(p.cls))).id();
    c.entity(head).add_child(title);
    let cols = c.spawn(Node { column_gap: px(24.0), align_items: AlignItems::FlexStart, ..default() }).id();
    // stats on parchment
    let left = c.spawn((Node { flex_direction: FlexDirection::Column, row_gap: px(3.0), width: px(300.0), padding: UiRect::all(px(22.0)), ..default() }, crate::uikit::parch(kit))).id();
    let h = c.spawn(txt("Stats", 19.0, 0x6a3a0e)).id(); c.entity(left).add_child(h);
    let s = &g.st;
    let pct = |v: f32| if v.abs() < 0.005 { "0%".to_string() } else { format!("{:+.0}%", v * 100.0) };
    let rows: Vec<(&str, String)> = vec![
        ("Health", format!("{:.0} / {:.0}", p.hp.max(0.0), p.max_hp)),
        ("Damage", pct(s.dmg - 1.0)), ("Critical chance", format!("{:.0}%", s.crit * 100.0)), ("Attack speed", pct(s.atk_spd - 1.0)),
        ("Cooldown reduction", format!("{:.0}%", (1.0 - s.cd) * 100.0)), ("Move speed", pct(s.speed - 1.0)), ("Damage reduction", format!("{:.0}%", s.armor * 100.0)),
        ("Lifesteal", format!("{:.0}%", s.leech * 100.0)), ("Regeneration", format!("{:.1} / s", s.regen)), ("Area", pct(s.area - 1.0)),
        ("Elite and boss damage", pct(s.elite)), ("Pickup range", format!("x{:.1}", s.magnet)), ("Experience", pct(s.xp_mul - 1.0)),
        ("Stamina", format!("{:.0}", s.stam_max)), ("Food", format!("{} meat, {} berries", g.food_meat, g.food_berries)), ("Rerolls", format!("{}", g.rerolls)),
        ("Ember", format!("{}", g.ember())),
    ];
    for (k, v) in rows {
        let r = c.spawn(Node { justify_content: JustifyContent::SpaceBetween, ..default() }).id();
        let a = c.spawn(txt(k, 14.0, 0x5a4a3a)).id(); let b = c.spawn(txt(&v, 14.0, 0x2a1a0e)).id();
        c.entity(r).add_children(&[a, b]); c.entity(left).add_child(r);
    }
    // abilities, weapons and picks
    let right = c.spawn((Node { flex_direction: FlexDirection::Column, row_gap: px(2.0), width: px(700.0), padding: UiRect::all(px(20.0)), ..default() }, crate::uikit::stone(kit))).id();
    let abl: Vec<Entity> = class_abilities(p.cls).iter().enumerate().map(|(_, id)| { let info = ab_info(id); let ub = crate::abilities::unbound(g, id); tile(c, icons, glyph_for_id(id), info.color, if ub { 0xffd35a } else { info.color }, 62.0, &format!("{}{}", if ub { "U" } else { "L" }, crate::abilities::ab_level(g, id)), None, Some(class_abilities(p.cls).iter().position(|a| a == id).unwrap_or(0))) }).collect();
    section(c, right, "Abilities", abl);
    let wpn: Vec<Entity> = class_weapons(p.cls).iter().map(|w| {
        let (name, _, desc) = weapon_info(w);
        let ups: String = WMODS.iter().filter(|m| m.0 == *w).filter_map(|m| { let r = g.m(&format!("w.{}.{}", w, m.1)); (r > 0).then(|| format!("\n- {}{}: {}", m.2, if m.4 > 1 { format!(" {}", roman(r)) } else { String::new() }, m.3)) }).collect();
        let eq = p.weapon_id() == *w;
        tile(c, icons, glyph_for_id(w), 0xe8dccc, if eq { 0xffd27a } else { 0x4a3e34 }, 54.0, "", Some(Tip { title: name.into(), tag: if eq { "Weapon  |  equipped".into() } else { "Weapon  |  Q to swap".into() }, body: format!("{}{}", desc, if ups.is_empty() { String::new() } else { format!("\n{}", ups) }), color: 0xffd27a }), None)
    }).collect();
    section(c, right, "Weapons", wpn);
    let mut rel = vec![];
    for rl in RELICS.iter() {
        let r = rk(g, rl.id); if r == 0 { continue; }
        if let Some(e) = EVOS.iter().find(|e| e.0 == rl.id && crate::relics::ev(g, rl.id)) {
            rel.push(tile(c, icons, glyph_for(rl.id, e.3, e.4), 0xffd35a, 0xffd35a, 54.0, "EVO", Some(Tip { title: e.3.into(), tag: format!("Evolved {}", rl.name), body: e.4.into(), color: 0xffd35a }), None));
            continue;
        }
        let body = rl.ranks.iter().take(r as usize).enumerate().map(|(k, t)| format!("{} {}", roman(k as u32 + 1), t)).collect::<Vec<_>>().join("\n");
        let hint = EVOS.iter().find(|e| e.0 == rl.id).map(|e| format!("\n\nAt rank {} it evolves with the {} boon.", RMAX, e.2)).unwrap_or_default();
        rel.push(tile(c, icons, glyph_for(rl.id, rl.name, rl.ranks[0]), rl.color, rl.color, 54.0, roman(r), Some(Tip { title: rl.name.into(), tag: format!("Relic  |  rank {}/{}", r, RMAX), body: format!("{}{}", body, hint), color: rl.color }), None));
    }
    section(c, right, "Relics", rel);
    let aug: Vec<Entity> = AUGS.iter().filter_map(|a| { let r = g.m(&format!("x.{}", a.id)); (r > 0).then(|| (a, r)) }).map(|(a, r)| {
        let mut body = a.ranks.iter().take(r as usize).cloned().collect::<Vec<_>>().join("\n");
        let n = a.ranks.len() as u32;
        if r > n { body += &format!("\nAmplified x{}: each rank {}", r - n, crate::augments::amp_desc(a.id).to_lowercase()); }
        let tag = if r > n { format!("Chaos augment  |  rank {}  (amplified)", r) } else { format!("Chaos augment  |  rank {}/{}", r, n) };
        tile(c, icons, glyph_for(a.id, a.name, a.ranks[0]), AUG_COLOR, AUG_COLOR, 54.0, &if r > 10 { r.to_string() } else { roman(r).to_string() }, Some(Tip { title: a.name.into(), tag, body, color: AUG_COLOR }), None)
    }).collect();
    section(c, right, "Chaos augments", aug);
    let boons: Vec<Entity> = PERKS.iter().filter_map(|b| { let n = g.perks.iter().filter(|q| *q == b.1).count(); (n > 0).then(|| (b, n)) }).map(|(b, n)| {
        let colr = b.3.map(class_color).unwrap_or(0xff9a3d);
        tile(c, icons, glyph_for(b.0, b.1, b.2), colr, colr, 48.0, &if n > 1 { format!("x{}", n) } else { String::new() }, Some(Tip { title: b.1.into(), tag: format!("Boon  |  taken {} time{}", n, if n == 1 { "" } else { "s" }), body: b.2.into(), color: colr }), None)
    }).collect();
    section(c, right, "Boons", boons);
    let mut ups = vec![];
    for ab in class_abilities(p.cls) {
        let info = ab_info(ab);
        for m in MODS.iter().filter(|m| m.0 == ab) {
            let r = g.m(&format!("{}.{}", ab, m.1)); if r == 0 { continue; }
            ups.push(tile(c, icons, glyph_for(m.1, m.2, m.3), info.color, if m.5 { 0xffd35a } else { info.color }, 48.0, if m.4 > 1 { roman(r) } else { "" },
                Some(Tip { title: m.2.into(), tag: if m.5 { format!("{} mastery", info.name) } else { format!("{} upgrade  |  rank {}/{}", info.name, r, m.4) }, body: m.3.into(), color: info.color }), None));
        }
    }
    for w in class_weapons(p.cls) {
        for m in WMODS.iter().filter(|m| m.0 == w) {
            let r = g.m(&format!("w.{}.{}", w, m.1)); if r == 0 { continue; }
            ups.push(tile(c, icons, glyph_for(m.1, m.2, m.3), 0xe8dccc, 0x8a7a6a, 48.0, if m.4 > 1 { roman(r) } else { "" }, Some(Tip { title: m.2.into(), tag: format!("{} upgrade  |  rank {}/{}", weapon_info(w).0, r, m.4), body: m.3.into(), color: 0xffd27a }), None));
        }
    }
    section(c, right, "Ability and weapon upgrades", ups);
    c.entity(cols).add_children(&[left, right]);
    let hint = c.spawn(txt("Hover an icon for details   |   Tab or Esc to close", 13.0, 0x8a7a6a)).id();
    c.entity(root).add_children(&[head, cols, hint]);
    root
}

/// open and close the panel with the screen; it's rebuilt each time so it always shows the current run
pub fn update_panel(mut c: Commands, ui: Res<UiState>, mut panel: ResMut<Panel>, icons: Res<Icons>, g: Res<Game>, p: Res<Player>, kit: Res<crate::uikit::UiTex>) {
    let open = ui.screen == Screen::Character;
    match (open, panel.root) {
        (true, None) => panel.root = Some(build(&mut c, &icons, &g, &p, &kit)),
        (false, Some(e)) => { c.entity(e).despawn_recursive(); panel.root = None; }
        _ => {}
    }
}
