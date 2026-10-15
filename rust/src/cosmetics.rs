//! Cosmetics: everything the Camp sells. Meta progression is purely visual, so nothing here changes how strong you
//! are: outfits recolour a class, headgear and capes add pieces to the model, weapon finishes repaint every
//! weapon, and auras trail motes around you. Choices are saved per class and sent to co-op partners.
use crate::models::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CosKind { Outfit, Head, Cape, Weapon, Aura }
pub const KINDS: [(CosKind, &str); 5] = [(CosKind::Outfit, "Outfits"), (CosKind::Head, "Headgear"), (CosKind::Cape, "Capes"), (CosKind::Weapon, "Weapon finish"), (CosKind::Aura, "Auras")];

/// one item: three colours whose meaning depends on the kind
pub struct CosItem { pub id: &'static str, pub name: &'static str, pub kind: CosKind, pub class: Option<Class>, pub cost: u32, pub a: u32, pub b: u32, pub c: u32 }
const fn it(id: &'static str, name: &'static str, kind: CosKind, class: Option<Class>, cost: u32, a: u32, b: u32, c: u32) -> CosItem { CosItem { id, name, kind, class, cost, a, b, c } }
use CosKind::*;
const R: Option<Class> = Some(Class::Ranger); const M: Option<Class> = Some(Class::Mage); const W: Option<Class> = Some(Class::Warrior);
const P: Option<Class> = Some(Class::Priest); const N: Option<Class> = Some(Class::Necromancer); const G: Option<Class> = Some(Class::Gunslinger);

/// outfits: a = coat/robe, b = trousers, c = the class's hat or hood. Capes: a = cloth, b = trim.
/// Weapon finishes: a = metal/wood, b = the glow. Auras: a, b = mote colours. Headgear: a, b = colours.
pub const COSMETICS: [CosItem; 38] = [
    it("r.autumn", "Autumn Stalker", Outfit, R, 200, 0x8a4a22, 0x4a3020, 0x6a3418), it("r.winter", "Winter Scout", Outfit, R, 250, 0xd8dce0, 0x6a7078, 0xe8ecf0), it("r.night", "Nightshade", Outfit, R, 300, 0x2a2236, 0x18141e, 0x3a2a4a),
    it("m.ember", "Ember Robes", Outfit, M, 200, 0x8a2a1a, 0x3a1410, 0x6a1a10), it("m.frost", "Frostweave", Outfit, M, 250, 0x4a8ab8, 0x2a4a6a, 0xa8d8f0), it("m.void", "Void Silk", Outfit, M, 300, 0x18141e, 0x0c0a10, 0x2a1a3a),
    it("w.gilded", "Gilded Plate", Outfit, W, 300, 0xb89a4a, 0x5a4a2a, 0xe0c060), it("w.black", "Blackguard", Outfit, W, 250, 0x3a3a42, 0x1a1a1e, 0x2a2a30), it("w.crimson", "Crimson Guard", Outfit, W, 200, 0x9a2a22, 0x4a1a14, 0xb8bec5),
    it("p.sun", "Sunlit Vestments", Outfit, P, 250, 0xf0d88a, 0xc8a85a, 0xfff0c0), it("p.twilight", "Twilight Habit", Outfit, P, 200, 0x4a4a6a, 0x2a2a3a, 0x5a5a7a), it("p.rose", "Rose Cleric", Outfit, P, 300, 0xd88a9a, 0x8a4a5a, 0xf0c0c8),
    it("n.bone", "Bone Lord", Outfit, N, 300, 0xd8d2c0, 0x8a8478, 0x6a6458), it("n.plague", "Plague Doctor", Outfit, N, 250, 0x2a3a22, 0x1a2414, 0x3a4a2a), it("n.blood", "Blood Rite", Outfit, N, 200, 0x5a1414, 0x2a0a0a, 0x3a0c0c),
    it("g.desperado", "Desperado", Outfit, G, 250, 0x2a2a2a, 0x1a1a1a, 0x121212), it("g.sheriff", "Sheriff", Outfit, G, 200, 0x8a6a4a, 0x4a3a2c, 0xd8c8a8), it("g.outlaw", "Outlaw Red", Outfit, G, 300, 0x8a2a1e, 0x3a2418, 0x5a1a14),
    it("h.crown", "Iron Crown", Head, None, 400, 0xe0b84a, 0xff4a4a, 0), it("h.horns", "Demon Horns", Head, None, 350, 0x2a1a1a, 0xe8dcc0, 0), it("h.laurel", "Laurel Wreath", Head, None, 250, 0x5a9a3a, 0x8ac85a, 0),
    it("h.halo", "Halo", Head, None, 450, 0xffe48a, 0xffffff, 0), it("h.plume", "Red Plume", Head, None, 200, 0xd8343a, 0xb8202a, 0), it("h.antlers", "Antlers", Head, None, 350, 0x8a6a4a, 0x6a4a2a, 0),
    it("c.royal", "Royal Cape", Cape, None, 300, 0x5a2a8a, 0xf0c95a, 0), it("c.ember", "Ember Cape", Cape, None, 350, 0x2a1a14, 0xff7a2a, 0), it("c.frost", "Frost Cape", Cape, None, 300, 0xd8ecf8, 0x8fe6ff, 0),
    it("c.shadow", "Shadow Cape", Cape, None, 300, 0x141218, 0x9a2a3a, 0), it("c.forest", "Forest Cape", Cape, None, 250, 0x3a5a2a, 0xa8c87a, 0),
    it("x.gilded", "Gilded", Weapon, None, 400, 0xe0b84a, 0xfff0a0, 0), it("x.obsidian", "Obsidian", Weapon, None, 400, 0x1c1426, 0xb46aff, 0), it("x.bone", "Bone", Weapon, None, 300, 0xe8e2d0, 0x7affb0, 0),
    it("x.frost", "Frostforged", Weapon, None, 350, 0xbfe8ff, 0x8fe6ff, 0), it("x.blood", "Bloodsteel", Weapon, None, 350, 0x6a1a1a, 0xff3a3a, 0),
    it("a.embers", "Embers", Aura, None, 500, 0xff7a2a, 0xffd27a, 0), it("a.frost", "Frost Motes", Aura, None, 500, 0x8fe6ff, 0xffffff, 0), it("a.fireflies", "Fireflies", Aura, None, 450, 0xd8ff5a, 0xffe48a, 0),
    it("a.wisps", "Shadow Wisps", Aura, None, 550, 0x9a5aff, 0x2a1a3a, 0),
];

/// what a character wears: an index + 1 into COSMETICS for each slot (0 = the class default)
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cos { pub outfit: u8, pub head: u8, pub cape: u8, pub weapon: u8, pub aura: u8 }
impl Cos {
    pub fn get(&self, k: CosKind) -> u8 { match k { Outfit => self.outfit, Head => self.head, Cape => self.cape, Weapon => self.weapon, Aura => self.aura } }
    pub fn set(&mut self, k: CosKind, v: u8) { match k { Outfit => self.outfit = v, Head => self.head = v, Cape => self.cape = v, Weapon => self.weapon = v, Aura => self.aura = v } }
    pub fn item(&self, k: CosKind) -> Option<&'static CosItem> { let i = self.get(k); if i == 0 { None } else { COSMETICS.get(i as usize - 1) } }
}
pub fn index_of(id: &str) -> u8 { COSMETICS.iter().position(|c| c.id == id).map(|i| i as u8 + 1).unwrap_or(0) }
/// the items one class can wear in one slot
pub fn for_class(cls: Class, k: CosKind) -> Vec<(u8, &'static CosItem)> {
    COSMETICS.iter().enumerate().filter(|(_, c)| c.kind == k && c.class.map(|x| x == cls).unwrap_or(true)).map(|(i, c)| (i as u8 + 1, c)).collect()
}

/// recolour the base body for an outfit
pub fn dress(mut lk: Look, cos: &Cos) -> Look {
    if let Some(o) = cos.item(Outfit) {
        lk.shirt = o.a; if lk.sleeve.is_some() { lk.sleeve = Some(o.a); } if lk.robe.is_some() { lk.robe = Some(darker(o.a)); } lk.pants = o.b;
    }
    lk
}
/// the outfit's hat or hood colour, or the class default
pub fn accent(cos: &Cos, default: u32) -> u32 { cos.item(Outfit).map(|o| o.c).unwrap_or(default) }
fn darker(c: u32) -> u32 { let f = |s: u32| ((((c >> s) & 255) as f32) * 0.85) as u32; (f(16) << 16) | (f(8) << 8) | f(0) }
/// how high the top of the class's own headwear sits, so accessories go on top of it
fn crown_y(cls: Class) -> f32 { match cls { Class::Mage => 1.12, Class::Gunslinger => 0.74, Class::Warrior => 0.62, Class::Necromancer => 0.7, _ => 0.6 } }

/// headgear pieces added on top of the head
pub fn build_head(b: &mut Builder, head: Entity, cls: Class, cos: &Cos) {
    let Some(h) = cos.item(Head) else { return };
    let y = crown_y(cls); let v = Vec3::new;
    match h.id {
        "h.crown" => { for (x, z) in [(-0.22, 0.0), (0.22, 0.0), (0.0, -0.22), (0.0, 0.22)] { b.bx(head, v(if x == 0.0 { 0.46 } else { 0.06 }, 0.12, if z == 0.0 { 0.46 } else { 0.06 }), h.a, v(x, y, z)); }
            for (x, z) in [(-0.2, -0.2), (0.2, -0.2), (-0.2, 0.2), (0.2, 0.2), (0.0, 0.22)] { b.bx(head, v(0.07, 0.12, 0.07), h.a, v(x, y + 0.11, z)); }
            b.glow(head, v(0.07, 0.07, 0.03), h.b, v(0.0, y + 0.02, 0.24)); }
        "h.horns" => { for sx in [-1.0f32, 1.0] { let p = b.pivot(head, v(sx * 0.24, y - 0.08, 0.0)); b.rot(p, 0.0, 0.0, -sx * 0.6); b.bx(p, v(0.1, 0.22, 0.1), h.a, v(0.0, 0.1, 0.0)); b.bx(p, v(0.07, 0.18, 0.07), h.b, v(sx * 0.04, 0.27, 0.03)); } }
        "h.laurel" => { for k in 0..10 { let a = k as f32 / 10.0 * std::f32::consts::TAU; let e = b.bx(head, v(0.1, 0.05, 0.16), if k % 2 == 0 { h.a } else { h.b }, v(a.sin() * 0.27, y - 0.02, a.cos() * 0.27)); b.rot(e, 0.3, a, 0.0); } }
        "h.halo" => { let torus = b.pal.torus.clone(); b.mesh(head, torus, h.a, Mk::Glow, Transform::from_xyz(0.0, y + 0.2, -0.04).with_scale(Vec3::new(0.24, 0.05, 0.24))); }
        "h.plume" => { for k in 0..4 { let e = b.bx(head, v(0.06, 0.26 - k as f32 * 0.03, 0.08), if k % 2 == 0 { h.a } else { h.b }, v(0.12, y + 0.08 + k as f32 * 0.04, -0.08 - k as f32 * 0.08)); b.rot(e, -0.5 - k as f32 * 0.2, 0.0, 0.2); } }
        "h.antlers" => { for sx in [-1.0f32, 1.0] { let p = b.pivot(head, v(sx * 0.18, y - 0.05, 0.0)); b.rot(p, 0.0, 0.0, -sx * 0.4); b.bx(p, v(0.06, 0.4, 0.06), h.a, v(0.0, 0.2, 0.0)); for k in 0..3 { let e = b.bx(p, v(0.05, 0.16, 0.05), h.b, v(sx * 0.08, 0.14 + k as f32 * 0.1, 0.0)); b.rot(e, 0.0, 0.0, -sx * 0.8); } } }
        _ => {}
    }
}
/// the cape's colours: a cape cosmetic overrides the class cape (and gives one to classes without)
pub fn cape_colors(cos: &Cos, default: Option<(u32, u32)>) -> Option<(u32, u32)> { cos.item(Cape).map(|c| (c.a, c.b)).or(default) }
/// repaint a weapon's parts in a finish: metals and woods take the finish colour by brightness, glows take its glow
pub fn finish_weapon(b: &mut Builder, cos: &Cos, parts: &[(Entity, u32, Mk)]) {
    let Some(f) = cos.item(Weapon) else { return };
    for &(e, col, k) in parts {
        let new = if k == Mk::Glow { f.b } else { let l = luma(col); shade(f.a, 0.55 + l * 0.9) };
        let m = b.pal.mat(b.mats, new, k);
        b.c.entity(e).insert(MeshMaterial3d(m));
    }
}
fn luma(c: u32) -> f32 { let (r, g, b) = (((c >> 16) & 255) as f32, ((c >> 8) & 255) as f32, (c & 255) as f32); (0.3 * r + 0.59 * g + 0.11 * b) / 255.0 }
fn shade(c: u32, k: f32) -> u32 { let f = |s: u32| (((((c >> s) & 255) as f32) * k).min(255.0)) as u32; (f(16) << 16) | (f(8) << 8) | f(0) }
/// mote colours for an equipped aura
pub fn aura(cos: &Cos) -> Option<(u32, u32, &'static str)> { cos.item(Aura).map(|a| (a.a, a.b, a.id)) }
