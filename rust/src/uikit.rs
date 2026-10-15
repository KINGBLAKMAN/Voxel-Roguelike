//! The medieval voxel UI kit. Every panel, button and frame is painted procedurally at startup as chunky pixel
//! art (each "voxel pixel" is a 4x4 block, sampled without smoothing) and drawn with 9-slice scaling so it stays
//! crisp at any size: carved stone panels with iron rivets, wooden plaques banded in iron for buttons, parchment for
//! reading, cloth banners, ribbons and gold frames. Buttons lift and glow when hovered, sink and flash when pressed,
//! and screens slide in when they open.
use crate::hud::UiBtn;
use crate::models::srgb;
use crate::noise::hash2;
use bevy::image::{ImageSampler, ImageSamplerDescriptor, ImageFilterMode};
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::sprite::{BorderRect, SliceScaleMode, TextureSlicer};
use bevy::ui::widget::NodeImageMode;

/// size of one painted "voxel pixel" in texture pixels
const C: usize = 4;

#[derive(Resource, Clone)]
pub struct UiTex {
    pub stone: Handle<Image>, pub wood: Handle<Image>, pub parch: Handle<Image>, pub slot: Handle<Image>, pub frame: Handle<Image>,
    pub gold: Handle<Image>, pub banner: Handle<Image>, pub ribbon: Handle<Image>, pub dark: Handle<Image>, pub iron: Handle<Image>,
}

// ---------------- painting ----------------
fn hx(hex: u32) -> Vec3 { Vec3::new(((hex >> 16) & 255) as f32 / 255.0, ((hex >> 8) & 255) as f32 / 255.0, (hex & 255) as f32 / 255.0) }
fn img(images: &mut Assets<Image>, w: usize, h: usize, f: impl Fn(i32, i32) -> Vec4) -> Handle<Image> {
    let (pw, ph) = (w * C, h * C);
    let mut d = vec![0u8; pw * ph * 4];
    for y in 0..h { for x in 0..w {
        let v = f(x as i32, y as i32);
        let px = [(v.x.clamp(0.0, 1.0) * 255.0) as u8, (v.y.clamp(0.0, 1.0) * 255.0) as u8, (v.z.clamp(0.0, 1.0) * 255.0) as u8, (v.w.clamp(0.0, 1.0) * 255.0) as u8];
        for yy in 0..C { for xx in 0..C { let i = ((y * C + yy) * pw + x * C + xx) * 4; d[i..i + 4].copy_from_slice(&px); } }
    } }
    let mut im = Image::new(Extent3d { width: pw as u32, height: ph as u32, depth_or_array_layers: 1 }, TextureDimension::D2, d, TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::RENDER_WORLD);
    im.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor { mag_filter: ImageFilterMode::Nearest, min_filter: ImageFilterMode::Nearest, ..default() });
    images.add(im)
}
fn n(x: i32, y: i32, s: u32) -> f32 { hash2(x, y, s) }
/// distance (in cells) to the nearest edge
fn edge(x: i32, y: i32, w: usize, h: usize) -> i32 { x.min(y).min(w as i32 - 1 - x).min(h as i32 - 1 - y) }
fn rivet(x: i32, y: i32, w: usize, h: usize, inset: i32) -> Option<f32> {
    for (cx, cy) in [(inset, inset), (w as i32 - 1 - inset, inset), (inset, h as i32 - 1 - inset), (w as i32 - 1 - inset, h as i32 - 1 - inset)] {
        let (dx, dy) = (x - cx, y - cy);
        if dx.abs() <= 1 && dy.abs() <= 1 { return Some(if dx <= 0 && dy <= 0 { 1.0 } else if dx >= 0 && dy >= 0 { 0.45 } else { 0.75 }); }
    }
    None
}

pub fn build(mut c: Commands, mut images: ResMut<Assets<Image>>) {
    let images = &mut *images;
    // carved stone: blocks in courses, a bevelled rim, iron rivets in the corners
    let stone = img(images, 24, 24, |x, y| {
        let e = edge(x, y, 24, 24);
        if let Some(k) = rivet(x, y, 24, 24, 2) { return (hx(0x8a8e96) * k).extend(1.0); }
        if e == 0 { return hx(0x16141a).extend(1.0); }
        if e <= 3 {
            let lit = x + y < 24; let base = if lit { 0x6a6a74 } else { 0x3a3a42 };
            return (hx(base) * (0.9 + 0.2 * n(x, y, 3))).extend(1.0);
        }
        let row = (y - 4) / 4; let off = if row % 2 == 0 { 0 } else { 3 };
        let mortar = (y - 4) % 4 == 3 || (x - 4 + off) % 6 == 5;
        let b = n((x - 4 + off) / 6, row, 7);
        if mortar { (hx(0x24222a) * 1.0).extend(0.96) } else { (hx(0x34343c).lerp(hx(0x44444e), b) * (0.92 + 0.12 * n(x, y, 9))).extend(0.96) }
    });
    // darker stone for full-screen backdrops
    let dark = img(images, 16, 16, |x, y| { let e = edge(x, y, 16, 16); if e == 0 { hx(0x0c0a0e).extend(0.96) } else if e == 1 { hx(0x2a2830).extend(0.94) } else { (hx(0x16141a) * (0.9 + 0.2 * n(x / 2, y / 2, 11))).extend(0.9) } });
    // a wooden plaque: planks banded in iron with a rivet at each corner
    let wood = img(images, 16, 12, |x, y| {
        let e = edge(x, y, 16, 12);
        if let Some(k) = rivet(x, y, 16, 12, 1) { if e >= 1 { return (hx(0xb8bcc4) * k).extend(1.0); } }
        if e == 0 { return hx(0x1a1410).extend(1.0); }
        if e == 1 { return (hx(0x4a4e56) * if y <= 1 || x <= 1 { 1.2 } else { 0.8 }).extend(1.0); }
        let plank = (y - 2) / 4; let seam = (y - 2) % 4 == 3;
        let grain = n(x / 3, y, 13 + plank as u32) * 0.25 + n(x, y, 17) * 0.1;
        let base = hx(0x7a4e2c).lerp(hx(0x9a6a3c), n(plank, 0, 19)) * (0.85 + grain);
        (if seam { base * 0.55 } else if y == 2 { base * 1.18 } else { base }).extend(1.0)
    });
    // parchment with darkened, irregular edges
    let parch = img(images, 24, 24, |x, y| {
        let e = edge(x, y, 24, 24) as f32 + n(x, y, 21) * 1.5;
        if e < 0.6 { return Vec4::ZERO; }
        let blot = n(x / 3, y / 3, 23) * 0.12 + n(x, y, 29) * 0.06;
        let burn = (1.0 - (e / 4.0).min(1.0)) * 0.45;
        (hx(0xe8d8b0) * (1.0 - blot - burn)).lerp(hx(0x6a4a2a), burn * 0.6).extend(1.0)
    });
    // an ability slot: a stone frame round a dark well
    let slot = img(images, 20, 20, |x, y| {
        let e = edge(x, y, 20, 20);
        if e == 0 { return hx(0x0e0c10).extend(1.0); }
        if e <= 3 { let lit = x + y < 20; return (hx(if lit { 0x7a7a84 } else { 0x44444c }) * (0.88 + 0.2 * n(x, y, 31))).extend(1.0); }
        if e == 4 { return hx(0x0a080c).extend(1.0); }
        (hx(0x18161c) * (0.9 + 0.15 * n(x, y, 37))).extend(0.82)
    });
    // a wooden bar frame with a hollow middle (fills show through)
    let frame = img(images, 12, 6, |x, y| {
        let e = edge(x, y, 12, 6);
        if e == 0 { hx(0x1a1410).extend(1.0) } else if e == 1 { (hx(0x8a5a32) * if y == 1 { 1.2 } else { 0.85 } * (0.9 + 0.2 * n(x, y, 41))).extend(1.0) } else { Vec4::ZERO }
    });
    // a gilded frame: for the selected item, the worn cosmetic, an Unbound ability
    let gold = img(images, 12, 12, |x, y| {
        let e = edge(x, y, 12, 12);
        if e == 0 { hx(0x5a3a0a).extend(1.0) } else if e == 1 { (hx(if x + y < 12 { 0xffe8a0 } else { 0xd8a840 })).extend(1.0) } else if e == 2 { hx(0xa8782a).extend(1.0) } else { Vec4::ZERO }
    });
    // iron: a thin dark frame for small widgets
    let iron = img(images, 8, 8, |x, y| { let e = edge(x, y, 8, 8); if e == 0 { hx(0x101014).extend(1.0) } else if e == 1 { (hx(0x6a6e76) * if x + y < 8 { 1.2 } else { 0.8 }).extend(1.0) } else { (hx(0x1a1820)).extend(0.85) } });
    // a cloth banner (tinted per use): woven cloth, a trim band at the top and a swallowtail at the foot
    let banner = img(images, 16, 24, |x, y| {
        let tail = y >= 19 && ((x as f32 - 7.5).abs() < (y - 19) as f32 * 1.6);
        if tail { return Vec4::ZERO; }
        if x == 0 || x == 15 { return (Vec3::splat(0.55)).extend(1.0); }
        if y <= 1 { return Vec3::splat(0.95).extend(1.0); }
        if y == 2 || y == 3 { return Vec3::splat(0.7).extend(1.0); }
        let weave = if (x + y) % 2 == 0 { 1.0 } else { 0.975 };
        (Vec3::splat(0.82 * weave) * (0.95 + 0.08 * n(x, y, 43))).extend(1.0)
    });
    // a ribbon with notched ends (tinted per use)
    let ribbon = img(images, 16, 5, |x, y| {
        let notch = (x == 0 || x == 15) && y == 2;
        if notch { return Vec4::ZERO; }
        let v = if y == 0 { 1.0 } else if y == 4 { 0.6 } else { 0.85 };
        Vec3::splat(v * (0.95 + 0.06 * n(x, y, 47))).extend(1.0)
    });
    c.insert_resource(UiTex { stone, wood, parch, slot, frame, gold, banner, ribbon, dark, iron });
}

/// a 9-slice image node; `b` is the border in texture pixels (which is also its size on screen)
pub fn sliced(h: &Handle<Image>, b: f32) -> ImageNode {
    ImageNode::new(h.clone()).with_mode(NodeImageMode::Sliced(TextureSlicer { border: BorderRect::square(b), center_scale_mode: SliceScaleMode::Stretch, sides_scale_mode: SliceScaleMode::Stretch, max_corner_scale: 1.0 }))
}
/// like `sliced`, but the middle and edges repeat instead of stretching, so patterns (bricks, grain) keep their size
pub fn tiled(h: &Handle<Image>, b: f32) -> ImageNode {
    ImageNode::new(h.clone()).with_mode(NodeImageMode::Sliced(TextureSlicer { border: BorderRect::square(b), center_scale_mode: SliceScaleMode::Tile { stretch_value: 1.0 }, sides_scale_mode: SliceScaleMode::Tile { stretch_value: 1.0 }, max_corner_scale: 1.0 }))
}
pub fn stone(t: &UiTex) -> ImageNode { tiled(&t.stone, 16.0) }
pub fn wood(t: &UiTex) -> ImageNode { tiled(&t.wood, 12.0) }
pub fn parch(t: &UiTex) -> ImageNode { tiled(&t.parch, 16.0) }
pub fn dark(t: &UiTex) -> ImageNode { sliced(&t.dark, 8.0) }
pub fn slot(t: &UiTex) -> ImageNode { sliced(&t.slot, 20.0) }
pub fn frame(t: &UiTex) -> ImageNode { sliced(&t.frame, 8.0) }
pub fn gold(t: &UiTex) -> ImageNode { sliced(&t.gold, 12.0) }
pub fn iron(t: &UiTex) -> ImageNode { sliced(&t.iron, 8.0) }
pub fn banner(t: &UiTex, hex: u32) -> ImageNode {
    let mut i = ImageNode::new(t.banner.clone()).with_mode(NodeImageMode::Sliced(TextureSlicer { border: BorderRect { left: 8.0, right: 8.0, top: 16.0, bottom: 20.0 }, center_scale_mode: SliceScaleMode::Stretch, sides_scale_mode: SliceScaleMode::Stretch, max_corner_scale: 1.0 }));
    i.color = srgb(hex); i
}
pub fn ribbon(t: &UiTex, hex: u32) -> ImageNode { let mut i = sliced(&t.ribbon, 8.0); i.color = srgb(hex); i }

pub fn px(v: f32) -> Val { Val::Px(v) }
pub fn txt(s: &str, size: f32, hex: u32) -> (Text, TextFont, TextColor) { (Text::new(s), TextFont { font_size: size, ..default() }, TextColor(srgb(hex))) }

/// a big title with a drop shadow (an offset dark copy behind it)
pub fn title(c: &mut Commands, s: &str, size: f32, hex: u32) -> Entity {
    let w = c.spawn(Node { ..default() }).id();
    let sh = c.spawn((txt(s, size, 0x120a06), Node { position_type: PositionType::Absolute, left: px(size * 0.06), top: px(size * 0.07), ..default() })).id();
    let fg = c.spawn(txt(s, size, hex)).id();
    c.entity(w).add_children(&[sh, fg]);
    w
}

/// an animated button: hover lifts and brightens it, press sinks and flashes it
#[derive(Component, Default)]
pub struct Fancy { pub hover: f32, pub press: f32, pub was: bool, pub sel: bool, pub tint: Option<Color> }
/// a gold frame overlay shown when a Fancy button is selected
#[derive(Component)]
pub struct SelFrame;
/// a wooden plaque button with a centred label; returns (button, label)
pub fn button(c: &mut Commands, t: &UiTex, b: UiBtn, w: f32, h: f32, label: &str, size: f32) -> (Entity, Entity) {
    let e = c.spawn((Button, b, Fancy::default(), Node { width: px(w), min_height: px(h), padding: UiRect::axes(px(12.0), px(6.0)), justify_content: JustifyContent::Center, align_items: AlignItems::Center, flex_direction: FlexDirection::Column, ..default() }, wood(t))).id();
    let l = c.spawn((txt(label, size, 0xfff3d8), TextLayout::new_with_justify(JustifyText::Center))).id();
    let f = c.spawn((SelFrame, Node { position_type: PositionType::Absolute, left: px(-3.0), right: px(-3.0), top: px(-3.0), bottom: px(-3.0), ..default() }, gold(t), Visibility::Hidden)).id();
    c.entity(e).add_children(&[l, f]);
    (e, l)
}
/// a button with an arbitrary panel look (banner, parchment...) that still animates
pub fn fancy(c: &mut Commands, t: &UiTex, b: UiBtn, node: Node, look: ImageNode) -> Entity {
    let tint = Some(look.color);
    let e = c.spawn((Button, b, Fancy { tint, ..default() }, node, look)).id();
    let f = c.spawn((SelFrame, Node { position_type: PositionType::Absolute, left: px(-4.0), right: px(-4.0), top: px(-4.0), bottom: px(-4.0), ..default() }, gold(t), Visibility::Hidden)).id();
    c.entity(e).add_child(f);
    e
}

/// hover, press and selection animation for every Fancy button, with a soft wooden knock when the pointer arrives
pub fn animate(time: Res<Time>, mut q: Query<(&Interaction, &mut Fancy, &mut Node, &mut ImageNode, &Children)>, mut frames: Query<&mut Visibility, With<SelFrame>>, mut sfx: ResMut<crate::audio::Sfx>) {
    let dt = time.delta_secs().min(0.05);
    for (i, mut f, mut n, mut im, kids) in q.iter_mut() {
        let (hov, prs) = (matches!(i, Interaction::Hovered | Interaction::Pressed), *i == Interaction::Pressed);
        if hov && !f.was { sfx.play("ui"); }
        f.was = hov;
        f.hover += ((if hov { 1.0 } else { 0.0 }) - f.hover) * (dt * 14.0).min(1.0);
        f.press += ((if prs { 1.0 } else { 0.0 }) - f.press) * (dt * 24.0).min(1.0);
        n.top = px(-3.0 * f.hover + 3.0 * f.press);
        let base = f.tint.unwrap_or(Color::WHITE).to_linear();
        let k = 1.0 + 0.28 * f.hover + 0.5 * f.press + if f.sel { 0.12 } else { 0.0 };
        im.color = Color::LinearRgba(LinearRgba::new(base.red * k, base.green * k, base.blue * k, base.alpha));
        for &k in kids.iter() { if let Ok(mut v) = frames.get_mut(k) { let want = if f.sel || f.hover > 0.6 { Visibility::Inherited } else { Visibility::Hidden }; if *v != want { *v = want; } } }
    }
}

/// screens slide up and settle when they open
#[derive(Component, Default)]
pub struct Appear { pub t: f32, pub shown: bool }
pub fn appear(time: Res<Time>, mut q: Query<(&mut Appear, &mut Node, &InheritedVisibility)>) {
    let dt = time.delta_secs().min(0.05);
    for (mut a, mut n, v) in q.iter_mut() {
        let vis = v.get();
        if vis && !a.shown { a.t = 0.0; }
        a.shown = vis;
        if !vis { continue; }
        a.t = (a.t + dt / 0.28).min(1.0);
        let e = 1.0 - (1.0 - a.t).powi(3);
        let off = 28.0 * (1.0 - e);
        if n.top != px(off) { n.top = px(off); }
    }
}
