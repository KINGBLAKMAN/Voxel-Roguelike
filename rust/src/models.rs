//! Box-built models. Every part shares one cube mesh and draws from a cached colour palette, so Bevy
//! batches whole hordes into a handful of draw calls. A Rig names the animated pivots of a model.
use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Mk { Lit, Glow, Ghost, Flash, Ice, Clear }

#[derive(Resource)]
pub struct Palette { pub cube: Handle<Mesh>, pub sphere: Handle<Mesh>, pub ico: Handle<Mesh>, pub octa: Handle<Mesh>, pub torus: Handle<Mesh>, mats: HashMap<(u32, Mk), Handle<StandardMaterial>> }
impl FromWorld for Palette {
    fn from_world(w: &mut bevy::prelude::World) -> Self {
        let mut meshes = w.resource_mut::<Assets<Mesh>>();
        let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
        let sphere = meshes.add(Sphere::new(1.0).mesh().ico(3).unwrap());
        let ico = meshes.add(Sphere::new(1.0).mesh().ico(1).unwrap());
        let octa = meshes.add(octahedron());
        let torus = meshes.add(Torus::new(0.9, 1.0).mesh().minor_resolution(8).major_resolution(32));
        Palette { cube, sphere, ico, octa, torus, mats: HashMap::new() }
    }
}
pub fn srgb(hex: u32) -> Color { Color::srgb_u8((hex >> 16) as u8, (hex >> 8) as u8, hex as u8) }
pub fn lrgb(hex: u32, k: f32) -> LinearRgba { let c = srgb(hex).to_linear(); LinearRgba::rgb(c.red * k, c.green * k, c.blue * k) }

impl Palette {
    pub fn mat(&mut self, mats: &mut Assets<StandardMaterial>, hex: u32, kind: Mk) -> Handle<StandardMaterial> {
        self.mats.entry((hex, kind)).or_insert_with(|| {
            let c = srgb(hex);
            mats.add(match kind {
                Mk::Lit => StandardMaterial { base_color: c, perceptual_roughness: 0.8, reflectance: 0.2, ..default() },
                Mk::Glow => StandardMaterial { base_color: c, emissive: lrgb(hex, 6.0), ..default() },
                Mk::Ghost => StandardMaterial { base_color: c.with_alpha(0.35), emissive: lrgb(hex, 1.5), alpha_mode: AlphaMode::Add, unlit: true, ..default() },
                Mk::Flash => StandardMaterial { base_color: Color::WHITE, emissive: LinearRgba::rgb(1.2, 1.2, 1.2), ..default() },
                Mk::Ice => StandardMaterial { base_color: Color::srgba(0.7, 0.9, 1.0, 0.75), emissive: LinearRgba::rgb(0.15, 0.4, 0.6), alpha_mode: AlphaMode::Blend, perceptual_roughness: 0.1, reflectance: 0.8, ..default() },
                Mk::Clear => StandardMaterial { base_color: c.with_alpha(0.55), alpha_mode: AlphaMode::Blend, perceptual_roughness: 0.15, reflectance: 0.6, ..default() },
            })
        }).clone()
    }
}

pub fn octahedron() -> Mesh {
    let v = [[0., 1., 0.], [1., 0., 0.], [0., 0., 1.], [-1., 0., 0.], [0., 0., -1.], [0., -1., 0.]];
    let tris = [[0, 2, 1], [0, 3, 2], [0, 4, 3], [0, 1, 4], [5, 1, 2], [5, 2, 3], [5, 3, 4], [5, 4, 1]];
    let (mut p, mut n) = (vec![], vec![]);
    for t in tris { let a = Vec3::from(v[t[0]]); let b = Vec3::from(v[t[1]]); let c = Vec3::from(v[t[2]]); let nn = (b - a).cross(c - a).normalize();
        for i in t { p.push(v[i]); n.push(nn.to_array()); } }
    Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList, bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD)
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, p).with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, n)
}

/// a model's animated joints; parts lists every coloured box so hit flashes can swap materials
#[derive(Clone, Default)]
pub struct Rig {
    pub root: Option<Entity>, pub body: Option<Entity>, pub torso: Option<Entity>, pub head: Option<Entity>,
    pub arm_l: Option<Entity>, pub arm_r: Option<Entity>, pub leg_l: Option<Entity>, pub leg_r: Option<Entity>,
    pub hand_l: Option<Entity>, pub hand_r: Option<Entity>, pub cape: Option<Entity>, pub extra: Vec<Entity>,
    pub parts: Vec<(Entity, u32, Mk)>,
}

/// spawns boxes into a hierarchy; `shadow` false for small or numerous things
pub struct Builder<'a, 'w, 's> { pub c: &'a mut Commands<'w, 's>, pub pal: &'a mut Palette, pub mats: &'a mut Assets<StandardMaterial>, pub shadow: bool, pub parts: Vec<(Entity, u32, Mk)> }
impl<'a, 'w, 's> Builder<'a, 'w, 's> {
    pub fn new(c: &'a mut Commands<'w, 's>, pal: &'a mut Palette, mats: &'a mut Assets<StandardMaterial>) -> Self { Builder { c, pal, mats, shadow: true, parts: vec![] } }
    pub fn root(&mut self, pos: Vec3) -> Entity { self.c.spawn((Transform::from_translation(pos), Visibility::default())).id() }
    pub fn pivot(&mut self, parent: Entity, pos: Vec3) -> Entity { let e = self.c.spawn((Transform::from_translation(pos), Visibility::default())).id(); self.c.entity(parent).add_child(e); e }
    pub fn bx(&mut self, parent: Entity, size: Vec3, hex: u32, pos: Vec3) -> Entity { self.bxk(parent, size, hex, pos, Mk::Lit) }
    pub fn glow(&mut self, parent: Entity, size: Vec3, hex: u32, pos: Vec3) -> Entity { self.bxk(parent, size, hex, pos, Mk::Glow) }
    pub fn bxk(&mut self, parent: Entity, size: Vec3, hex: u32, pos: Vec3, k: Mk) -> Entity {
        let m = self.pal.mat(self.mats, hex, k);
        let mut ec = self.c.spawn((Mesh3d(self.pal.cube.clone()), MeshMaterial3d(m), Transform { translation: pos, scale: size, ..default() }));
        if !self.shadow || k != Mk::Lit { ec.insert(NotShadowCaster); }
        let e = ec.id();
        self.c.entity(parent).add_child(e);
        self.parts.push((e, hex, k));
        e
    }
    pub fn rot(&mut self, e: Entity, x: f32, y: f32, z: f32) { self.c.entity(e).queue(move |mut ew: EntityWorldMut| { if let Some(mut t) = ew.get_mut::<Transform>() { t.rotation = Quat::from_euler(EulerRot::XYZ, x, y, z); } }); }
    pub fn mesh(&mut self, parent: Entity, mesh: Handle<Mesh>, hex: u32, k: Mk, t: Transform) -> Entity {
        let m = self.pal.mat(self.mats, hex, k);
        let e = self.c.spawn((Mesh3d(mesh), MeshMaterial3d(m), t, NotShadowCaster)).id();
        self.c.entity(parent).add_child(e);
        self.parts.push((e, hex, k));
        e
    }
}

pub struct Look { pub skin: u32, pub shirt: u32, pub pants: u32, pub boots: Option<u32>, pub belt: Option<u32>, pub robe: Option<u32>, pub sleeve: Option<u32>, pub eye: u32, pub eye_glow: Option<u32>, pub thin: bool }
impl Default for Look { fn default() -> Self { Look { skin: 0xe0b08a, shirt: 0x4b7a3c, pants: 0x5a4632, boots: None, belt: None, robe: None, sleeve: None, eye: 0x1a1a22, eye_glow: None, thin: false } } }

pub fn humanoid(b: &mut Builder, pos: Vec3, o: &Look) -> Rig {
    let root = b.root(pos);
    let body = b.pivot(root, Vec3::ZERO);
    let th = if o.thin { 0.6 } else { 1.0 };
    let leg_l = b.pivot(body, Vec3::new(-0.13, 0.78, 0.0));
    let leg_r = b.pivot(body, Vec3::new(0.13, 0.78, 0.0));
    for l in [leg_l, leg_r] {
        b.bx(l, Vec3::new(0.22 * th, 0.78, 0.24 * th), o.pants, Vec3::new(0.0, -0.39, 0.0));
        if let Some(bt) = o.boots { b.bx(l, Vec3::new(0.25, 0.2, 0.29), bt, Vec3::new(0.0, -0.69, 0.02)); }
    }
    let torso = b.pivot(body, Vec3::new(0.0, 0.78, 0.0));
    b.bx(torso, Vec3::new(0.52 * if o.thin { 0.75 } else { 1.0 }, 0.7, 0.3 * if o.thin { 0.7 } else { 1.0 }), o.shirt, Vec3::new(0.0, 0.35, 0.0));
    if let Some(r) = o.robe { b.bx(torso, Vec3::new(0.56, 0.6, 0.34), r, Vec3::new(0.0, -0.15, 0.0)); }
    if let Some(bl) = o.belt { b.bx(torso, Vec3::new(0.55, 0.08, 0.32), bl, Vec3::new(0.0, 0.06, 0.0)); }
    let head = b.pivot(torso, Vec3::new(0.0, 0.7, 0.0));
    b.bx(head, Vec3::new(0.44, 0.44, 0.44), o.skin, Vec3::new(0.0, 0.22, 0.0));
    for x in [-0.1, 0.1] {
        if let Some(g) = o.eye_glow { b.glow(head, Vec3::new(0.08, 0.06, 0.02), g, Vec3::new(x, 0.24, 0.225)); }
        else { b.bx(head, Vec3::new(0.08, 0.06, 0.02), o.eye, Vec3::new(x, 0.24, 0.225)); }
    }
    let sl = o.sleeve.unwrap_or(o.shirt);
    let arm_l = b.pivot(torso, Vec3::new(-0.35, 0.64, 0.0));
    let arm_r = b.pivot(torso, Vec3::new(0.35, 0.64, 0.0));
    for a in [arm_l, arm_r] {
        b.bx(a, Vec3::new(0.18 * th, 0.6, 0.2 * th), sl, Vec3::new(0.0, -0.28, 0.0));
        b.bx(a, Vec3::new(0.16 * th, 0.12, 0.18 * th), o.skin, Vec3::new(0.0, -0.62, 0.0));
    }
    let hand_l = b.pivot(arm_l, Vec3::new(0.0, -0.62, 0.0));
    let hand_r = b.pivot(arm_r, Vec3::new(0.0, -0.62, 0.0));
    Rig { root: Some(root), body: Some(body), torso: Some(torso), head: Some(head), arm_l: Some(arm_l), arm_r: Some(arm_r), leg_l: Some(leg_l), leg_r: Some(leg_r), hand_l: Some(hand_l), hand_r: Some(hand_r), ..default() }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Class { Ranger, Mage, Warrior, Priest }
pub fn class_look(c: Class) -> Look {
    match c {
        Class::Ranger => Look { skin: 0xe0b08a, shirt: 0x4b7a3c, pants: 0x5a4632, boots: Some(0x3a2a1c), belt: Some(0x6a4528), ..default() },
        Class::Mage => Look { skin: 0xe8c09a, shirt: 0x4b3a8f, pants: 0x2d2550, robe: Some(0x3f3180), belt: Some(0xf0c95a), sleeve: Some(0x4b3a8f), ..default() },
        Class::Warrior => Look { skin: 0xd9a47a, shirt: 0x8d959e, pants: 0x4a4038, boots: Some(0x2a221c), belt: Some(0x5a3a22), sleeve: Some(0x6d757e), ..default() },
        Class::Priest => Look { skin: 0xe6bd98, shirt: 0xece6d8, pants: 0xb9ae98, robe: Some(0xe4dccb), belt: Some(0xe0b84a), sleeve: Some(0xece6d8), ..default() },
    }
}
pub fn player_model(b: &mut Builder, pos: Vec3, cls: Class) -> Rig {
    let mut r = humanoid(b, pos, &class_look(cls));
    let (head, torso) = (r.head.unwrap(), r.torso.unwrap());
    let v = Vec3::new;
    match cls {
        Class::Ranger => {
            let hm = 0x355e2c;
            b.bx(head, v(0.5, 0.14, 0.5), hm, v(0.0, 0.47, -0.02)); b.bx(head, v(0.5, 0.46, 0.1), hm, v(0.0, 0.24, -0.25));
            b.bx(head, v(0.08, 0.46, 0.42), hm, v(-0.25, 0.24, -0.02)); b.bx(head, v(0.08, 0.46, 0.42), hm, v(0.25, 0.24, -0.02));
            let q = b.pivot(torso, v(0.12, 0.45, -0.22)); b.rot(q, 0.0, 0.0, 0.5);
            b.bx(q, v(0.14, 0.52, 0.14), 0x6a4528, Vec3::ZERO);
            for i in 0..3 { b.bx(q, v(0.03, 0.18, 0.03), 0xe8e0c8, v(-0.03 + i as f32 * 0.03, 0.32, 0.0)); }
        }
        Class::Mage => {
            let hm = 0x3a2a78;
            b.bx(head, v(0.66, 0.06, 0.66), hm, v(0.0, 0.46, 0.0)); b.bx(head, v(0.42, 0.2, 0.42), hm, v(0.0, 0.58, -0.02));
            b.bx(head, v(0.3, 0.2, 0.3), hm, v(0.0, 0.76, -0.06)); b.bx(head, v(0.18, 0.2, 0.18), hm, v(0.0, 0.93, -0.12));
            b.bx(head, v(0.1, 0.14, 0.1), hm, v(0.0, 1.06, -0.18));
            b.bx(head, v(0.36, 0.26, 0.06), 0xe8e4dc, v(0.0, 0.04, 0.23)); b.bx(head, v(0.44, 0.05, 0.45), 0xf0c95a, v(0.0, 0.5, 0.0));
            b.glow(head, v(0.08, 0.08, 0.08), 0xd08aff, v(0.0, 1.16, -0.2));
        }
        Class::Warrior => {
            let (st, dk, rd) = (0xa7afb8, 0x1b1b22, 0xb3342a);
            b.bx(head, v(0.5, 0.32, 0.5), st, v(0.0, 0.34, 0.0)); b.bx(head, v(0.5, 0.22, 0.08), st, v(0.0, 0.12, -0.22));
            b.bx(head, v(0.38, 0.05, 0.02), dk, v(0.0, 0.28, 0.255)); b.bx(head, v(0.06, 0.18, 0.3), rd, v(0.0, 0.56, -0.04));
            b.bx(r.arm_l.unwrap(), v(0.28, 0.16, 0.3), st, v(0.0, 0.02, 0.0)); b.bx(r.arm_r.unwrap(), v(0.28, 0.16, 0.3), st, v(0.0, 0.02, 0.0));
            b.bx(torso, v(0.3, 0.52, 0.02), rd, v(0.0, 0.32, 0.16));
        }
        Class::Priest => {
            let (hm, gd) = (0xf2eee4, 0xe0b84a);
            b.bx(head, v(0.5, 0.14, 0.5), hm, v(0.0, 0.47, -0.02)); b.bx(head, v(0.5, 0.46, 0.1), hm, v(0.0, 0.24, -0.25));
            b.bx(torso, v(0.12, 0.7, 0.02), gd, v(0.0, 0.35, 0.16)); b.bx(torso, v(0.4, 0.12, 0.02), gd, v(0.0, 0.48, 0.16));
            let torus = b.pal.torus.clone();
            b.mesh(head, torus, 0xffe48a, Mk::Glow, Transform::from_xyz(0.0, 0.66, -0.06).with_scale(Vec3::new(0.27, 0.06, 0.27)));
        }
    }
    // capes hang from the shoulders and stream out behind when running
    let cape_c = match cls { Class::Warrior => Some((0x9a2a22, 0xd8b04a)), Class::Mage => Some((0x2c2266, 0xf0c95a)), Class::Priest => Some((0xf2eee4, 0xe0b84a)), _ => None };
    if let Some((a, c2)) = cape_c {
        let cape = b.pivot(torso, v(0.0, 0.66, -0.17));
        b.bx(cape, v(0.5, 0.86, 0.04), a, v(0.0, -0.43, 0.0)); b.bx(cape, v(0.52, 0.06, 0.05), c2, v(0.0, -0.85, 0.0)); b.bx(cape, v(0.56, 0.08, 0.08), a, v(0.0, 0.0, 0.02));
        r.cape = Some(cape);
    }
    r.parts = std::mem::take(&mut b.parts);
    r
}

/// weapon models. Each returns its root and a "tip" pivot where projectiles leave.
pub struct Weapon { pub root: Entity, pub tip: Entity, pub string: Option<[Entity; 3]>, pub tip_y: f32, pub tip_z: f32 }
pub fn weapon_model(b: &mut Builder, parent: Entity, id: &str) -> Weapon {
    let root = b.pivot(parent, Vec3::ZERO);
    let v = Vec3::new;
    let mut w = Weapon { root, tip: root, string: None, tip_y: 0.0, tip_z: 0.0 };
    match id {
        // held in the left hand with the arm pointing at the target: -y is forward, +y back towards the archer.
        // The limbs run along z and sweep back towards the archer to recurved tips; the string pulls back when drawn.
        "longbow" | "ebow" => {
            let (wd, wl, gr, gd) = if id == "ebow" { (0x8a8478, 0xa8a294, 0x5a5448, 0x6a6458) } else { (0x6a4224, 0x8a5a32, 0x3a2a1a, 0xc9a24a) };
            let p = |t: f32| (0.26 * t * t - if t > 0.86 { (t - 0.86) * 0.9 } else { 0.0 }, 0.04 + 0.68 * t);
            let n = 7;
            for sg in [1.0f32, -1.0] {
                for i in 0..n {
                    let (y0, z0) = p(i as f32 / n as f32); let (y1, z1) = p((i + 1) as f32 / n as f32);
                    let len = ((y1 - y0).powi(2) + (z1 - z0).powi(2)).sqrt();
                    let e = b.bx(root, v(0.06 + (n - i) as f32 * 0.004, 0.075 - i as f32 * 0.006, len + 0.02), if i % 2 == 1 { wd } else { wl }, v(0.0, (y0 + y1) / 2.0, sg * (z0 + z1) / 2.0));
                    b.rot(e, (-(y1 - y0)).atan2(z1 - z0) * sg, 0.0, 0.0);
                }
                let (ty, tz) = p(1.0);
                b.bx(root, v(0.05, 0.05, 0.07), gd, v(0.0, ty, sg * tz));
                w.tip_y = ty; w.tip_z = tz;
            }
            b.bx(root, v(0.09, 0.11, 0.2), gr, v(0.0, 0.01, 0.0));
            let s1 = b.bx(root, v(0.016, 0.016, 1.0), 0xeee6cc, Vec3::ZERO);
            let s2 = b.bx(root, v(0.016, 0.016, 1.0), 0xeee6cc, Vec3::ZERO);
            let arrow = b.pivot(root, Vec3::ZERO);
            b.bx(arrow, v(0.03, 0.7, 0.03), 0x7a5a3a, v(0.0, -0.35, 0.0)); b.bx(arrow, v(0.06, 0.1, 0.06), 0xd8dde4, v(0.0, -0.72, 0.0)); b.bx(arrow, v(0.1, 0.12, 0.012), 0xf0f0f0, v(0.0, -0.04, 0.0));
            w.string = Some([s1, s2, arrow]);
            w.tip = b.pivot(root, v(0.0, -0.2, 0.0));
        }
        "daggers" => { b.bx(root, v(0.05, 0.03, 0.42), 0xd8dde4, v(0.0, 0.0, 0.28)); b.bx(root, v(0.16, 0.04, 0.05), 0x3a2a1c, v(0.0, 0.0, 0.06)); b.bx(root, v(0.05, 0.05, 0.14), 0x3a2a1c, v(0.0, 0.0, -0.03)); w.tip = b.pivot(root, v(0.0, 0.0, 0.5)); }
        "firestaff" => {
            b.bx(root, v(0.07, 0.07, 1.5), 0x4a2f1c, v(0.0, 0.0, 0.35)); b.bx(root, v(0.14, 0.14, 0.08), 0xd9a640, v(0.0, 0.0, 1.06));
            b.bx(root, v(0.05, 0.18, 0.05), 0xd9a640, v(0.08, 0.0, 1.15)); b.bx(root, v(0.05, 0.18, 0.05), 0xd9a640, v(-0.08, 0.0, 1.15));
            let o = b.glow(root, v(0.17, 0.17, 0.17), 0xff7a22, v(0.0, 0.0, 1.2)); b.rot(o, 0.6, 0.6, 0.0); w.tip = b.pivot(root, v(0.0, 0.0, 1.22));
        }
        "frostwand" => { b.bx(root, v(0.05, 0.05, 0.62), 0xcfd9e8, v(0.0, 0.0, 0.2)); let c = b.glow(root, v(0.11, 0.11, 0.24), 0x8fe6ff, v(0.0, 0.0, 0.58)); b.rot(c, 0.0, 0.0, 0.78); w.tip = b.pivot(root, v(0.0, 0.0, 0.72)); }
        "greatsword" => {
            b.bx(root, v(0.14, 0.04, 1.35), 0xc9d0d8, v(0.0, 0.0, 0.88)); b.bx(root, v(0.06, 0.05, 0.2), 0xc9d0d8, v(0.0, 0.0, 1.62)); b.bx(root, v(0.5, 0.08, 0.09), 0x8a6a3a, v(0.0, 0.0, 0.18));
            b.bx(root, v(0.06, 0.06, 0.34), 0x2e1f16, v(0.0, 0.0, 0.0)); b.bx(root, v(0.1, 0.1, 0.1), 0x8a6a3a, v(0.0, 0.0, -0.18)); w.tip = b.pivot(root, v(0.0, 0.0, 1.6));
        }
        "sword" => { b.bx(root, v(0.1, 0.035, 0.8), 0xd5dbe2, v(0.0, 0.0, 0.55)); b.bx(root, v(0.36, 0.07, 0.07), 0xb08a3a, v(0.0, 0.0, 0.12)); b.bx(root, v(0.05, 0.05, 0.22), 0x3a2618, Vec3::ZERO); w.tip = b.pivot(root, v(0.0, 0.0, 0.95)); }
        "shield" => {
            b.bx(root, v(0.08, 0.66, 0.54), 0x8a3a2a, v(-0.1, 0.0, 0.05)); b.bx(root, v(0.1, 0.7, 0.06), 0xb9bec5, v(-0.1, 0.0, 0.33)); b.bx(root, v(0.1, 0.7, 0.06), 0xb9bec5, v(-0.1, 0.0, -0.23));
            b.bx(root, v(0.1, 0.06, 0.58), 0xb9bec5, v(-0.1, 0.34, 0.05)); b.bx(root, v(0.1, 0.06, 0.58), 0xb9bec5, v(-0.1, -0.34, 0.05)); b.bx(root, v(0.12, 0.14, 0.14), 0xe6c15a, v(-0.13, 0.0, 0.05));
        }
        "mace" => {
            b.bx(root, v(0.06, 0.06, 0.62), 0x5a3a22, v(0.0, 0.0, 0.22)); b.bx(root, v(0.22, 0.22, 0.22), 0xd9b24a, v(0.0, 0.0, 0.6));
            for (x, y, z) in [(0.14, 0.0, 0.0), (-0.14, 0.0, 0.0), (0.0, 0.14, 0.0), (0.0, -0.14, 0.0), (0.0, 0.0, 0.14)] { b.bx(root, v(0.07, 0.07, 0.07), 0xeae4d0, v(x, y, 0.6 + z)); }
            w.tip = b.pivot(root, v(0.0, 0.0, 0.7));
        }
        "scepter" => {
            b.bx(root, v(0.05, 0.05, 0.85), 0xe0b84a, v(0.0, 0.0, 0.3)); b.bx(root, v(0.2, 0.04, 0.04), 0xe0b84a, v(0.0, 0.0, 0.72)); b.bx(root, v(0.04, 0.2, 0.04), 0xe0b84a, v(0.0, 0.0, 0.72));
            let g = b.glow(root, v(0.13, 0.13, 0.13), 0xfff0a0, v(0.0, 0.0, 0.82)); b.rot(g, 0.7, 0.7, 0.0); w.tip = b.pivot(root, v(0.0, 0.0, 0.86));
        }
        _ => {}
    }
    w
}
