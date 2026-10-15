//! The Camp: spend gold between runs on permanent upgrades and unlocks; plus the title screen's camp button,
//! Ember difficulty picker and lifetime stats.
use crate::hud::{HudRefs, Screen, UiBtn, UiState};
use crate::meta::*;
use crate::models::srgb;
use bevy::prelude::*;

#[derive(Resource)]
pub struct CampRefs { root: Entity, gold: Entity, items: Vec<(Entity, Entity, Entity)>, cls_name: Entity, info: Entity, ember: Entity, ember_desc: Entity, history: Entity }

fn col(hex: u32, a: f32) -> Color { srgb(hex).with_alpha(a) }
fn txt(s: &str, size: f32, hex: u32) -> (Text, TextFont, TextColor) { (Text::new(s), TextFont { font_size: size, ..default() }, TextColor(srgb(hex))) }
fn px(v: f32) -> Val { Val::Px(v) }
fn button(c: &mut Commands, b: UiBtn, w: f32) -> Entity {
    c.spawn((Button, b, Node { width: px(w), padding: UiRect::axes(px(10.0), px(4.0)), flex_direction: FlexDirection::Column, border: UiRect::all(px(2.0)), ..default() },
        BackgroundColor(col(0x100c14, 0.9)), BorderColor(col(0xff9a3d, 0.4)), BorderRadius::all(px(8.0)))).id()
}

pub fn setup_camp(mut c: Commands, r: Res<HudRefs>) {
    // title screen additions: camp button, Ember picker, stats
    let row = c.spawn(Node { column_gap: px(14.0), align_items: AlignItems::Center, margin: UiRect::top(px(18.0)), ..default() }).id();
    let camp_b = button(&mut c, UiBtn::Camp, 160.0); let t = c.spawn(txt("Camp  (C)", 20.0, 0xffd27a)).id(); c.entity(camp_b).add_child(t);
    let dn = button(&mut c, UiBtn::EmberDown, 44.0); let t = c.spawn(txt("<", 20.0, 0xfff3d8)).id(); c.entity(dn).add_child(t);
    let ember = c.spawn((txt("Ember 0", 22.0, 0xff7a3a), Node { width: px(110.0), ..default() }, TextLayout::new_with_justify(JustifyText::Center))).id();
    let up = button(&mut c, UiBtn::EmberUp, 44.0); let t = c.spawn(txt(">", 20.0, 0xfff3d8)).id(); c.entity(up).add_child(t);
    let set_b = button(&mut c, UiBtn::Settings, 170.0); let t = c.spawn(txt("Settings  (S)", 20.0, 0xfff3d8)).id(); c.entity(set_b).add_child(t);
    c.entity(row).add_children(&[camp_b, dn, ember, up, set_b]);
    let ember_desc = c.spawn(txt("", 15.0, 0xd8ccbc)).id();
    let row2 = c.spawn(Node { column_gap: px(14.0), align_items: AlignItems::Center, margin: UiRect::top(px(8.0)), ..default() }).id();
    let host_b = button(&mut c, UiBtn::Host, 190.0); let t = c.spawn(txt("Host co-op  (H)", 18.0, 0x9fe0ff)).id(); c.entity(host_b).add_child(t);
    let join_b = button(&mut c, UiBtn::Join, 190.0); let t = c.spawn(txt("Join co-op  (J)", 18.0, 0x9fe0ff)).id(); c.entity(join_b).add_child(t);
    c.entity(row2).add_children(&[host_b, join_b]);
    // the route: which map (and boss) the second stage is played on
    let rrow = c.spawn(Node { column_gap: px(10.0), align_items: AlignItems::Center, margin: UiRect::top(px(8.0)), ..default() }).id();
    let rl = c.spawn(txt("Route  (R):   Greenwood  >", 17.0, 0xd8ccbc)).id(); c.entity(rrow).add_child(rl);
    for (k, label, sub) in [(1u8, "Drowned Fen", "Broodmother"), (2, "Bone Wastes", "Bone Colossus"), (0, "Random", "either")] {
        let b = button(&mut c, UiBtn::Route(k), 150.0);
        let t = c.spawn((txt(label, 16.0, [0xd8ccbc, 0x6fd4a0, 0xffc86a][k as usize]), Node { align_self: AlignSelf::Center, ..default() })).id();
        let t2 = c.spawn((txt(sub, 12.0, 0x9a8a78), Node { align_self: AlignSelf::Center, ..default() })).id();
        c.entity(b).add_children(&[t, t2]); c.entity(rrow).add_child(b);
    }
    let rr = c.spawn(txt(">   Ember Throne", 17.0, 0xff8a5a)).id(); c.entity(rrow).add_child(rr);
    let net = c.spawn(txt("", 14.0, 0x9fe0ff)).id();
    let info = c.spawn((txt("", 14.0, 0x9a8a78), Node { margin: UiRect::top(px(6.0)), ..default() })).id();
    c.entity(r.menu).add_children(&[row, ember_desc, rrow, row2, net, info]);

    // the join screen: type the host's address
    let jroot = c.spawn((Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, justify_content: JustifyContent::Center, row_gap: px(10.0), ..default() },
        BackgroundColor(col(0x0a0806, 0.92)), Visibility::Hidden)).id();
    let jt = c.spawn(txt("JOIN CO-OP", 38.0, 0x9fe0ff)).id();
    let jh = c.spawn(txt("Type the host's address (shown on their title screen) and press Enter", 16.0, 0xd8ccbc)).id();
    let jbox = c.spawn((Node { width: px(420.0), padding: UiRect::all(px(10.0)), border: UiRect::all(px(2.0)), justify_content: JustifyContent::Center, ..default() }, BackgroundColor(col(0x100c14, 0.95)), BorderColor(col(0x9fe0ff, 0.6)), BorderRadius::all(px(8.0)))).id();
    let jtext = c.spawn(txt("", 24.0, 0xfff3d8)).id(); c.entity(jbox).add_child(jtext);
    let jmsg = c.spawn(txt("", 14.0, 0x9fe0ff)).id();
    let back = button(&mut c, UiBtn::Back, 220.0); let t = c.spawn((txt("Back  (Esc)", 18.0, 0xfff3d8), Node { align_self: AlignSelf::Center, ..default() })).id(); c.entity(back).add_child(t);
    c.entity(jroot).add_children(&[jt, jh, jbox, jmsg, back]);
    c.insert_resource(JoinRefs { root: jroot, text: jtext, msg: jmsg, net });

    // the camp itself: a wardrobe. Pick a class, then buy and wear cosmetics; the character turns on the right
    let root = c.spawn((Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Row, ..default() }, Visibility::Hidden)).id();
    let panel = c.spawn((Node { width: px(760.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, padding: UiRect::all(px(18.0)), row_gap: px(6.0), ..default() }, BackgroundColor(col(0x0a0806, 0.9)))).id();
    let title = c.spawn(txt("CAMP  -  WARDROBE", 34.0, 0xffd27a)).id();
    let gold = c.spawn(txt("", 17.0, 0xffc83a)).id();
    let note = c.spawn(txt("Everything here is cosmetic: power is only ever earned inside a run.", 13.0, 0x9a8a78)).id();
    let crow = c.spawn(Node { column_gap: px(10.0), align_items: AlignItems::Center, margin: UiRect::vertical(px(6.0)), ..default() }).id();
    let prev = button(&mut c, UiBtn::CampCls(-1), 44.0); let t = c.spawn(txt("<", 20.0, 0xfff3d8)).id(); c.entity(prev).add_child(t);
    let cls_name = c.spawn((txt("", 24.0, 0xfff3d8), Node { width: px(220.0), ..default() }, TextLayout::new_with_justify(JustifyText::Center))).id();
    let next = button(&mut c, UiBtn::CampCls(1), 44.0); let t = c.spawn(txt(">", 20.0, 0xfff3d8)).id(); c.entity(next).add_child(t);
    c.entity(crow).add_children(&[prev, cls_name, next]);
    let grid = c.spawn(Node { flex_wrap: FlexWrap::Wrap, column_gap: px(14.0), row_gap: px(10.0), ..default() }).id();
    let mut items = vec![];
    for (kind, label) in crate::cosmetics::KINDS {
        let col_e = c.spawn(Node { flex_direction: FlexDirection::Column, row_gap: px(3.0), width: px(232.0), ..default() }).id();
        let h = c.spawn(txt(label, 15.0, 0xd8ccbc)).id(); c.entity(col_e).add_child(h);
        for (i, it) in crate::cosmetics::COSMETICS.iter().enumerate() {
            if it.kind != kind { continue; }
            let b = c.spawn((Button, UiBtn::Buy(i), Node { width: px(232.0), padding: UiRect::axes(px(8.0), px(3.0)), justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, border: UiRect::all(px(2.0)), ..default() },
                BackgroundColor(col(0x100c14, 0.9)), BorderColor(col(0xff9a3d, 0.4)), BorderRadius::all(px(6.0)))).id();
            let a = c.spawn(txt(it.name, 14.0, 0xfff3d8)).id(); let s2 = c.spawn(txt("", 12.0, 0xb8aca0)).id();
            c.entity(b).add_children(&[a, s2]); c.entity(col_e).add_child(b); items.push((b, a, s2));
        }
        c.entity(grid).add_child(col_e);
    }
    let ht = c.spawn((txt("Recent runs", 15.0, 0xd8ccbc), Node { margin: UiRect::top(px(10.0)), ..default() })).id();
    let history = c.spawn(txt("", 12.0, 0xb8aca0)).id();
    let back = button(&mut c, UiBtn::Back, 220.0); let t = c.spawn((txt("Back  (Esc)", 18.0, 0xfff3d8), Node { align_self: AlignSelf::Center, ..default() })).id(); c.entity(back).add_child(t);
    c.entity(panel).add_children(&[title, gold, note, crow, grid, ht, history, back]);
    c.entity(root).add_child(panel);
    c.insert_resource(CampRefs { root, gold, items, cls_name, info, ember, ember_desc, history });

    // the wandering merchant's wares
    let shop = c.spawn((Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, justify_content: JustifyContent::Center, row_gap: px(8.0), ..default() },
        BackgroundColor(col(0x0a0806, 0.7)), Visibility::Hidden)).id();
    let st = c.spawn(txt("WANDERING MERCHANT", 40.0, 0xffc060)).id();
    let sg = c.spawn(txt("", 18.0, 0xffc83a)).id();
    c.entity(shop).add_children(&[st, sg]);
    let mut offers = vec![];
    for i in 0..4 {
        let b = button(&mut c, UiBtn::ShopBuy(i), 460.0);
        let a = c.spawn(txt("", 18.0, 0xfff3d8)).id(); let s = c.spawn(txt("", 14.0, 0xb8aca0)).id();
        c.entity(b).add_children(&[a, s]); c.entity(shop).add_child(b); offers.push((b, a, s));
    }
    let back = button(&mut c, UiBtn::Back, 220.0); let t = c.spawn((txt("Leave  (Esc)", 18.0, 0xfff3d8), Node { align_self: AlignSelf::Center, ..default() })).id(); c.entity(back).add_child(t);
    c.entity(shop).add_child(back);
    c.insert_resource(ShopRefs { root: shop, gold: sg, offers });
}
#[derive(Resource)]
pub struct JoinRefs { root: Entity, text: Entity, msg: Entity, net: Entity }
/// typing on the join screen, and the co-op status line on the title screen
pub fn update_join(mut ui: ResMut<UiState>, r: Res<JoinRefs>, mut kb: EventReader<bevy::input::keyboard::KeyboardInput>, time: Res<Time>, link: Option<Res<crate::net::Link>>, mut was: Local<bool>, mut q: Q) {
    use bevy::input::keyboard::Key;
    if let Ok((_, _, Some(mut v))) = q.get_mut(r.root) { *v = if ui.screen == Screen::Join { Visibility::Inherited } else { Visibility::Hidden }; }
    let peers = link.as_ref().map(|l| l.peer_count()).unwrap_or(0);
    let status = if ui.net_msg.is_empty() { String::new() } else if link.is_some() && peers > 0 { format!("{}   |   {} connected", ui.net_msg, peers) } else { ui.net_msg.clone() };
    set_text(&mut q, r.net, &status);
    let fresh = !*was; *was = ui.screen == Screen::Join;
    if ui.screen != Screen::Join || fresh { kb.clear(); return; }
    for ev in kb.read() {
        if !ev.state.is_pressed() { continue; }
        match &ev.logical_key {
            Key::Character(ch) => for c in ch.chars() { if (c.is_ascii_alphanumeric() || c == '.' || c == ':' || c == '-') && ui.join_text.len() < 60 { ui.join_text.push(c); } },
            Key::Backspace => { ui.join_text.pop(); }
            Key::Enter => { let t = ui.join_text.trim().to_string(); ui.req_join = Some(if t.is_empty() { "127.0.0.1".into() } else { t }); ui.screen = Screen::Menu; }
            _ => {}
        }
    }
    let caret = if (time.elapsed_secs() * 2.0) as u32 % 2 == 0 { "|" } else { " " };
    set_text(&mut q, r.text, &format!("{}{}", if ui.join_text.is_empty() { "127.0.0.1".to_string() } else { ui.join_text.clone() }, caret));
    set_col(&mut q, r.text, if ui.join_text.is_empty() { 0x6a6a6a } else { 0xfff3d8 });
    set_text(&mut q, r.msg, &ui.net_msg);
}
#[derive(Resource)]
pub struct ShopRefs { root: Entity, gold: Entity, offers: Vec<(Entity, Entity, Entity)> }
pub fn update_shop(ui: Res<UiState>, g: Res<crate::game::Game>, r: Res<ShopRefs>, mut q: Q, mut bv: Query<&mut Visibility, With<UiBtn>>) {
    if let Ok((_, _, Some(mut v))) = q.get_mut(r.root) { *v = if ui.screen == Screen::Shop { Visibility::Inherited } else { Visibility::Hidden }; }
    if ui.screen != Screen::Shop { return; }
    set_text(&mut q, r.gold, &format!("You carry {} gold  (spending it lowers what you take home)", g.gold));
    for (i, (b, a, s)) in r.offers.iter().enumerate() {
        let o = g.ev.shop.get(i);
        if let Ok(mut v) = bv.get_mut(*b) { *v = if o.is_some() { Visibility::Inherited } else { Visibility::Hidden }; }
        let Some(o) = o else { continue };
        set_text(&mut q, *a, &format!("{}.  {}", i + 1, o.name));
        set_text(&mut q, *s, &if o.sold { "Sold".to_string() } else { format!("{}  |  {} gold", o.desc, o.cost) });
        set_col(&mut q, *a, if o.sold { 0x6a5a4a } else if g.gold >= o.cost { 0xfff3d8 } else { 0x8a7a6a });
    }
}

type Q<'w, 's> = Query<'w, 's, (Option<&'static mut Text>, Option<&'static mut TextColor>, Option<&'static mut Visibility>), Without<UiBtn>>;
fn set_text(q: &mut Q, e: Entity, s: &str) { if let Ok((Some(mut t), ..)) = q.get_mut(e) { if t.0 != s { t.0 = s.to_string(); } } }
fn set_col(q: &mut Q, e: Entity, c: u32) { if let Ok((_, Some(mut t), _)) = q.get_mut(e) { t.0 = srgb(c); } }
fn fmt_t(t: f32) -> String { let s = t.max(0.0) as u32; format!("{}:{:02}", s / 60, s % 60) }

pub fn update_camp(ui: Res<UiState>, save: Res<Save>, r: Res<CampRefs>, mut q: Q, mut bv: Query<&mut Node, With<UiBtn>>) {
    if let Ok((_, _, Some(mut v))) = q.get_mut(r.root) { *v = if ui.screen == Screen::Camp { Visibility::Inherited } else { Visibility::Hidden }; }
    if !matches!(ui.screen, Screen::Menu | Screen::Camp) { return; }
    set_text(&mut q, r.ember, &format!("Ember {}", save.ember));
    set_text(&mut q, r.ember_desc, &format!("{}{}", EMBERS[save.ember as usize], if save.max_ember > save.ember { String::new() } else if save.ember < 10 { "   (win to unlock the next)".into() } else { String::new() }));
    set_text(&mut q, r.info, &format!("{} gold   |   {} runs, {} won   |   best {} and {} slain{}", save.gold, save.runs, save.wins, fmt_t(save.best_time), save.best_kills,
        if save.refunded > 0 { format!("   |   {} gold refunded from retired upgrades", save.refunded) } else { String::new() }));
    set_text(&mut q, r.gold, &format!("{} gold", save.gold));
    let cls = crate::models::CLASSES[ui.camp_cls % crate::models::CLASSES.len()];
    set_text(&mut q, r.cls_name, crate::progression::class_name(cls));
    set_col(&mut q, r.cls_name, crate::progression::class_color(cls));
    let wear = save.wear(cls);
    for (i, (b, a, s)) in r.items.iter().enumerate() {
        let it = &crate::cosmetics::COSMETICS[i];
        let show = it.class.map(|c| c == cls).unwrap_or(true);
        if let Ok(mut n) = bv.get_mut(*b) { let d = if show { Display::Flex } else { Display::None }; if n.display != d { n.display = d; } }
        if !show { continue; }
        let owned = save.owns(it.id); let worn = wear.get(it.kind) == i as u8 + 1;
        set_text(&mut q, *s, &if worn { "worn".to_string() } else if owned { "owned".to_string() } else { format!("{}g", it.cost) });
        set_col(&mut q, *a, if worn { 0xffd27a } else if owned || save.gold >= it.cost { 0xfff3d8 } else { 0x8a7a6a });
    }
    let h: Vec<String> = save.history.iter().take(8).map(|h| format!("{} {}  {}  lv {}  {} slain  +{}g{}", if h.won { "WON " } else { "fell" }, h.class, fmt_t(h.time), h.level, h.kills, h.gold, if h.ember > 0 { format!("  E{}", h.ember) } else { String::new() })).collect();
    set_text(&mut q, r.history, &if h.is_empty() { "No runs yet".to_string() } else { h.join("\n") });
}

/// the wardrobe's mannequin: the selected class in its current cosmetics, slowly turning beside the panel
#[derive(Default)]
pub struct Preview { plinth: Option<Entity>, rig: Option<crate::models::Rig>, weapon: &'static str, key: Option<(crate::models::Class, crate::cosmetics::Cos)> }
#[allow(clippy::too_many_arguments)]
pub fn camp_preview(mut st: Local<Preview>, ui: Res<UiState>, save: Res<Save>, p: Res<crate::player::Player>, mut c: Commands, mut pal: ResMut<crate::models::Palette>, mut mats: ResMut<Assets<StandardMaterial>>,
    mut q: Query<&mut Transform>, cam: Query<Entity, With<crate::scene::MainCam>>, time: Res<Time>) {
    let on = ui.screen == Screen::Camp;
    let cls = crate::models::CLASSES[ui.camp_cls % crate::models::CLASSES.len()];
    let key = (cls, save.wear(cls));
    if (!on || st.key != Some(key)) && st.rig.is_some() { if let Some(r) = st.rig.take().and_then(|r| r.root) { c.entity(r).despawn_recursive(); } st.key = None; }
    if !on { if let Some(e) = st.plinth.take() { c.entity(e).despawn_recursive(); } return; }
    // the mannequin stands on a stone plinth, clear of the grass
    let base = p.body.pos + Vec3::Y * 1.0;
    if st.plinth.is_none() {
        let mut b = crate::models::Builder::new(&mut c, &mut pal, &mut mats);
        let r = b.root(p.body.pos);
        b.bx(r, Vec3::new(1.6, 1.0, 1.6), 0x5a5a62, Vec3::new(0.0, 0.5, 0.0)); b.bx(r, Vec3::new(1.8, 0.14, 1.8), 0x46464e, Vec3::new(0.0, 0.07, 0.0)); b.bx(r, Vec3::new(1.7, 0.1, 1.7), 0x6a6a72, Vec3::new(0.0, 0.95, 0.0));
        b.glow(r, Vec3::new(1.62, 0.04, 1.62), 0xffb04a, Vec3::new(0.0, 0.82, 0.0));
        st.plinth = Some(r);
    }
    if st.rig.is_none() {
        let mut tmp = crate::player::Player { cls, cos: key.1, ..default() };
        tmp.body.pos = base;
        crate::player::build_player(&mut c, &mut pal, &mut mats, &mut tmp);
        st.weapon = crate::player::class_weapons(cls)[0];
        st.rig = Some(tmp.rig.clone()); st.key = Some(key);
        return;
    }
    let Some(r) = st.rig.clone() else { return };
    let t = time.elapsed_secs();
    if let Some(e) = r.root { if let Ok(mut tr) = q.get_mut(e) { tr.translation = base; tr.rotation = Quat::from_rotation_y(t * 0.6); } }
    // hold the weapon in its ready stance, breathing
    let s = crate::anim::stance(st.weapon);
    let br = (t * 2.0).sin() * 0.03;
    for (e, v) in [(r.arm_r, s.ar + Vec3::X * br), (r.arm_l, s.al - Vec3::X * br), (r.hand_r, s.hr), (r.hand_l, s.hl), (r.torso, s.torso)] { if let Some(e) = e { if let Ok(mut tr) = q.get_mut(e) { tr.rotation = Quat::from_euler(EulerRot::XYZ, v.x, v.y, v.z); } } }
    for (e, x) in [(r.elbow_r, s.er), (r.elbow_l, s.el)] { if let Some(e) = e { if let Ok(mut tr) = q.get_mut(e) { tr.rotation = Quat::from_rotation_x(x); } } }
    if let Ok(ce) = cam.get_single() { if let Ok(mut tr) = q.get_mut(ce) { *tr = Transform::from_translation(base + Vec3::new(0.0, 1.5, 4.6)).looking_at(base + Vec3::new(-2.3, 0.9, 0.0), Vec3::Y); } }
}
