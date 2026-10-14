//! Scene and rendering: camera (HDR, bloom, filmic tone mapping, fog), sun and sky, water, clouds,
//! the day/night cycle and chunk streaming around the camera.
use crate::mesher::mesh_chunk;
use crate::noise::lerp;
use crate::world::*;
use crate::GameCfg;
use bevy::asset::load_internal_asset;
use bevy::core_pipeline::bloom::Bloom;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey, CascadeShadowConfigBuilder, ExtendedMaterial, MaterialExtension, NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;
use bevy::render::mesh::MeshVertexBufferLayoutRef;
use bevy::render::render_resource::*;

pub const SKY_SHADER: Handle<Shader> = Handle::weak_from_u128(0x5e3c_11a2_8f00_4d8b_a001_0000_0000_0001);
pub const WATER_SHADER: Handle<Shader> = Handle::weak_from_u128(0x5e3c_11a2_8f00_4d8b_a001_0000_0000_0002);
pub const WIND_SHADER: Handle<Shader> = Handle::weak_from_u128(0x5e3c_11a2_8f00_4d8b_a001_0000_0000_0003);

#[derive(Clone, ShaderType, Default)]
pub struct SkyU { pub top: Vec4, pub hor: Vec4, pub sun_dir: Vec4, pub sun_col: Vec4, pub moon_col: Vec4, pub params: Vec4 }
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct SkyMaterial { #[uniform(0)] pub u: SkyU }
impl Material for SkyMaterial {
    fn vertex_shader() -> ShaderRef { SKY_SHADER.into() }
    fn fragment_shader() -> ShaderRef { SKY_SHADER.into() }
    fn specialize(_: &MaterialPipeline<Self>, d: &mut RenderPipelineDescriptor, layout: &MeshVertexBufferLayoutRef, _: MaterialPipelineKey<Self>) -> Result<(), SpecializedMeshPipelineError> {
        d.vertex.buffers = vec![layout.0.get_layout(&[Mesh::ATTRIBUTE_POSITION.at_shader_location(0)])?];
        d.primitive.cull_mode = None;
        Ok(())
    }
}
#[derive(Clone, ShaderType, Default)]
pub struct WaterU { pub deep: Vec4, pub shallow: Vec4, pub sky: Vec4, pub sun: Vec4, pub fog: Vec4, pub params: Vec4 }
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct WaterMaterial { #[uniform(0)] pub u: WaterU }
impl Material for WaterMaterial {
    fn vertex_shader() -> ShaderRef { WATER_SHADER.into() }
    fn fragment_shader() -> ShaderRef { WATER_SHADER.into() }
    fn alpha_mode(&self) -> AlphaMode { AlphaMode::Blend }
    fn specialize(_: &MaterialPipeline<Self>, d: &mut RenderPipelineDescriptor, layout: &MeshVertexBufferLayoutRef, _: MaterialPipelineKey<Self>) -> Result<(), SpecializedMeshPipelineError> {
        d.vertex.buffers = vec![layout.0.get_layout(&[Mesh::ATTRIBUTE_POSITION.at_shader_location(0)])?];
        d.primitive.cull_mode = None;
        Ok(())
    }
}
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct WindExt { #[uniform(100)] pub strength: f32 }
impl MaterialExtension for WindExt { fn vertex_shader() -> ShaderRef { WIND_SHADER.into() } }
pub type FoliageMaterial = ExtendedMaterial<StandardMaterial, WindExt>;

#[derive(Resource)]
pub struct SceneHandles { pub chunk_mat: Handle<StandardMaterial>, pub glow_mat: Handle<StandardMaterial>, pub fol_mat: Handle<FoliageMaterial>, pub sky: Handle<SkyMaterial>, pub water: Handle<WaterMaterial> }
#[derive(Component)] pub struct MainCam;
#[derive(Component)] pub struct Sun;
#[derive(Component)] pub struct SkyDome;
#[derive(Component)] pub struct WaterPlane;
#[derive(Component)] pub struct Cloud { pub x: f32, pub z: f32, pub ox: f32, pub oz: f32, pub y: f32 }
#[derive(Component)] pub struct ChunkMesh;

/// time of day and the derived lighting, readable by gameplay (night spawns, music) and the HUD
#[derive(Resource)]
pub struct DayCycle { pub time: f32, pub day: u32, pub night: bool, pub blood: bool, pub daylight: f32, pub sun_dir: Vec3, pub speed: f32 }
impl Default for DayCycle { fn default() -> Self { DayCycle { time: 0.06, day: 1, night: false, blood: false, daylight: 1.0, sun_dir: Vec3::Y, speed: 1.0 } } }
pub const DAY_LEN: f32 = 360.0;

pub struct ScenePlugin;
impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(app, SKY_SHADER, "shaders/sky.wgsl", Shader::from_wgsl);
        load_internal_asset!(app, WATER_SHADER, "shaders/water.wgsl", Shader::from_wgsl);
        load_internal_asset!(app, WIND_SHADER, "shaders/wind.wgsl", Shader::from_wgsl);
        app.add_plugins((MaterialPlugin::<SkyMaterial>::default(), MaterialPlugin::<WaterMaterial>::default(), MaterialPlugin::<FoliageMaterial>::default()))
            .insert_resource(ClearColor(Color::srgb(0.6, 0.8, 0.95)))
            .insert_resource(AmbientLight { color: Color::WHITE, brightness: 300.0 })
            .init_resource::<DayCycle>()
            .add_systems(Startup, setup_scene)
            .add_systems(Update, (update_day, stream_chunks, follow_sky_water, drift_clouds));
    }
}

fn setup_scene(mut c: Commands, mut meshes: ResMut<Assets<Mesh>>, mut mats: ResMut<Assets<StandardMaterial>>, mut sky_m: ResMut<Assets<SkyMaterial>>,
    mut water_m: ResMut<Assets<WaterMaterial>>, mut fol_m: ResMut<Assets<FoliageMaterial>>) {
    c.spawn((Camera3d::default(), Camera { hdr: true, ..default() }, Tonemapping::AcesFitted, Bloom { intensity: 0.12, ..Bloom::NATURAL },
        Projection::Perspective(PerspectiveProjection { fov: 72f32.to_radians(), near: 0.1, far: 1400.0, ..default() }),
        DistanceFog { color: Color::srgb(0.7, 0.82, 0.95), falloff: FogFalloff::Linear { start: 60.0, end: 170.0 }, ..default() },
        Msaa::Sample4, Transform::from_xyz(0.0, 40.0, 0.0).looking_at(Vec3::new(10.0, 30.0, 10.0), Vec3::Y), MainCam));
    c.spawn((DirectionalLight { illuminance: 9000.0, shadows_enabled: true, color: Color::srgb(1.0, 0.95, 0.85), ..default() },
        CascadeShadowConfigBuilder { num_cascades: 3, minimum_distance: 0.5, first_cascade_far_bound: 18.0, maximum_distance: 110.0, overlap_proportion: 0.25 }.build(),
        Transform::from_xyz(0.0, 50.0, 0.0).looking_at(Vec3::new(10.0, 0.0, 5.0), Vec3::Y), Sun));
    let chunk_mat = mats.add(StandardMaterial { base_color: Color::WHITE, perceptual_roughness: 0.92, reflectance: 0.12, ..default() });
    let glow_mat = mats.add(StandardMaterial { base_color: Color::WHITE, unlit: true, ..default() });
    let fol_mat = fol_m.add(ExtendedMaterial { base: StandardMaterial { base_color: Color::WHITE, perceptual_roughness: 0.85, double_sided: false, cull_mode: None, ..default() }, extension: WindExt { strength: 1.0 } });
    let sky = sky_m.add(SkyMaterial { u: SkyU::default() });
    let water = water_m.add(WaterMaterial { u: WaterU::default() });
    c.spawn((Mesh3d(meshes.add(Sphere::new(900.0).mesh().uv(48, 24))), MeshMaterial3d(sky.clone()), Transform::default(), NotShadowCaster, NotShadowReceiver, SkyDome));
    c.spawn((Mesh3d(meshes.add(Plane3d::default().mesh().size(420.0, 420.0).subdivisions(120))), MeshMaterial3d(water.clone()), Transform::from_xyz(0.0, WATER_Y, 0.0), NotShadowCaster, NotShadowReceiver, WaterPlane));
    // puffy clouds: clusters of flattened boxes drifting with the wind
    let cloud_mat = mats.add(StandardMaterial { base_color: Color::srgba(1.0, 1.0, 1.0, 0.86), alpha_mode: AlphaMode::Blend, unlit: false, perceptual_roughness: 1.0, ..default() });
    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let mut rng = crate::noise::mulberry32(77);
    for _ in 0..46 {
        let (cx, cz, cy, w) = (rng() * 600.0 - 300.0, rng() * 600.0 - 300.0, 95.0 + rng() * 20.0, 14.0 + rng() * 18.0);
        let parts = 2 + (rng() * 3.0) as usize;
        for k in 0..parts {
            let (ox, oz) = if k == 0 { (0.0, 0.0) } else { ((rng() - 0.5) * w * 0.9, (rng() - 0.5) * w * 0.6) };
            let sc = if k == 0 { Vec3::new(w, 2.4 + rng(), w * (0.5 + rng() * 0.3)) } else { Vec3::new(w * (0.35 + rng() * 0.25), 2.0 + rng() * 1.5, w * (0.3 + rng() * 0.2)) };
            c.spawn((Mesh3d(cube.clone()), MeshMaterial3d(cloud_mat.clone()), Transform::from_scale(sc), NotShadowCaster, NotShadowReceiver,
                Cloud { x: cx, z: cz, ox, oz, y: cy + if k > 0 { rng() * 2.0 } else { 0.0 } }));
        }
    }
    c.insert_resource(SceneHandles { chunk_mat, glow_mat, fol_mat, sky, water });
}

fn mix3(a: Vec3, b: Vec3, t: f32) -> Vec3 { a + (b - a) * t }
fn rgb(hex: u32) -> Vec3 { let l = lin(hex); Vec3::new(l[0], l[1], l[2]) }
fn smooth(a: f32, b: f32, x: f32) -> f32 { crate::noise::smooth(a, b, x) }

fn update_day(time: Res<Time>, mut dc: ResMut<DayCycle>, cfg: Res<GameCfg>, h: Option<Res<SceneHandles>>, mut sky_m: ResMut<Assets<SkyMaterial>>, mut water_m: ResMut<Assets<WaterMaterial>>,
    mut sun: Query<(&mut DirectionalLight, &mut Transform), With<Sun>>, mut fog: Query<(&mut DistanceFog, &Transform), (With<MainCam>, Without<Sun>)>,
    mut amb: ResMut<AmbientLight>, mut clear: ResMut<ClearColor>) {
    let Some(h) = h else { return };
    if cfg.time_runs { dc.time += time.delta_secs() * dc.speed / DAY_LEN; }
    let t = dc.time.fract();
    let ang = t * std::f32::consts::TAU;
    let sd = Vec3::new(ang.cos() * 0.85, ang.sin(), 0.4).normalize();
    let day = smooth(-0.15, 0.2, sd.y);
    let dusk = 1.0 - (sd.y * 4.0).clamp(-1.0, 1.0).abs();
    dc.day = dc.time.floor() as u32 + 1;
    dc.night = day < 0.35;
    dc.blood = dc.day % 3 == 0 && dc.night;
    dc.daylight = day;
    dc.sun_dir = sd;
    let (nt, nh) = if dc.blood { (rgb(0x1a0408), rgb(0x5a1014)) } else { (rgb(0x060a1e), rgb(0x18224a)) };
    let mut top = mix3(nt, rgb(0x3f86d6), day);
    let mut hor = mix3(nh, rgb(0xb4dcf5), day);
    hor = mix3(hor, rgb(0xf29a5a), dusk * 0.55 * smooth(-0.3, 0.1, sd.y));
    top = mix3(top, rgb(0x3a4a8a), dusk * 0.3);
    let up = sd.y > 0.0;
    if let Some(m) = sky_m.get_mut(&h.sky) {
        m.u.top = top.extend(1.0); m.u.hor = hor.extend(1.0); m.u.sun_dir = sd.extend(0.0);
        let sc = if day > 0.01 { Vec3::new(1.0, 0.85 + day * 0.1, 0.6 + day * 0.3) } else { Vec3::ZERO };
        m.u.sun_col = sc.extend(1.0);
        m.u.moon_col = (if dc.blood { rgb(0xff5a4a) } else { rgb(0xdfe8ff) }).extend(1.0);
        m.u.params = Vec4::new(1.0 - day, 0.0, 0.0, 0.0);
    }
    let (fog_start, fog_end) = (cfg.view as f32 * 16.0 * 0.4, cfg.view as f32 * 16.0 - 8.0);
    if let Ok((mut f, _)) = fog.get_single_mut() {
        f.color = Color::linear_rgb(hor.x, hor.y, hor.z);
        f.falloff = FogFalloff::Linear { start: fog_start, end: fog_end };
    }
    clear.0 = Color::linear_rgb(hor.x, hor.y, hor.z);
    if let Some(w) = water_m.get_mut(&h.water) {
        w.u.deep = rgb(0x17557f).extend(1.0); w.u.shallow = rgb(0x36b0cf).extend(1.0);
        w.u.sky = mix3(hor, top, 0.3).extend(1.0); w.u.sun = sd.extend(0.0);
        w.u.fog = hor.extend(fog_start); w.u.params = Vec4::new(0.3 + 0.7 * day, fog_end, 0.0, 0.0);
    }
    if let Ok((mut l, mut tr)) = sun.get_single_mut() {
        let ldir = if up { sd } else { -sd };
        let center = fog.get_single().map(|(_, t)| t.translation).unwrap_or(Vec3::ZERO);
        *tr = Transform::from_translation(center + ldir * 100.0).looking_at(center, Vec3::Y);
        l.illuminance = if up { 1800.0 + 7200.0 * day } else { 2200.0 * (1.0 - day) };
        let mut sc = if up { rgb(0xfff0d8) } else if dc.blood { rgb(0xff7a6a) } else { rgb(0x9ab0ff) };
        if up { sc = mix3(sc, rgb(0xffa060), dusk * 0.6); }
        l.color = Color::linear_rgb(sc.x, sc.y, sc.z);
    }
    // sky-tinted fill light: blue-white by day, cool moonlit blue at night (red under a blood moon)
    let fill = if dc.blood { Vec3::new(0.7, 0.35, 0.35) } else { mix3(Vec3::new(0.45, 0.55, 0.9), Vec3::new(lerp(top.x, 1.0, 0.5), lerp(top.y, 1.0, 0.5), lerp(top.z, 1.0, 0.5)), day) };
    amb.color = Color::linear_rgb(fill.x, fill.y, fill.z);
    amb.brightness = 260.0 + 160.0 * day;
}

fn follow_sky_water(cam: Query<&Transform, With<MainCam>>, mut q: ParamSet<(Query<&mut Transform, (With<SkyDome>, Without<MainCam>)>, Query<&mut Transform, (With<WaterPlane>, Without<MainCam>)>)>) {
    let Ok(ct) = cam.get_single() else { return };
    let p = ct.translation;
    for mut t in q.p0().iter_mut() { t.translation = p; }
    for mut t in q.p1().iter_mut() { t.translation.x = (p.x / 8.0).round() * 8.0; t.translation.z = (p.z / 8.0).round() * 8.0; }
}
fn drift_clouds(time: Res<Time>, dc: Res<DayCycle>, cam: Query<&Transform, With<MainCam>>, mut q: Query<(&mut Cloud, &mut Transform), Without<MainCam>>) {
    let Ok(ct) = cam.get_single() else { return };
    let p = ct.translation;
    let _ = dc;
    for (mut cl, mut t) in q.iter_mut() {
        cl.x += time.delta_secs() * 1.6;
        let rx = ((cl.x - p.x + 300.0).rem_euclid(600.0)) - 300.0;
        let rz = ((cl.z - p.z + 300.0).rem_euclid(600.0)) - 300.0;
        t.translation = Vec3::new(p.x + rx + cl.ox, cl.y, p.z + rz + cl.oz);
    }
}

/// generate and mesh chunks nearest the camera first, a few per frame, and drop far ones
pub fn stream_chunks(mut c: Commands, world: Option<ResMut<Voxels>>, cfg: Res<GameCfg>, cam: Query<&Transform, With<MainCam>>, h: Option<Res<SceneHandles>>, mut meshes: ResMut<Assets<Mesh>>) {
    let (Some(mut world), Some(h)) = (world, h) else { return };
    let world: &mut Voxels = &mut world;
    let Ok(ct) = cam.get_single() else { return };
    let focus = cfg.focus.unwrap_or(ct.translation);
    let (pcx, pcz) = ((focus.x / CS as f32).floor() as i32, (focus.z / CS as f32).floor() as i32);
    let v = cfg.view;
    let mut need = vec![];
    for dz in -v..=v { for dx in -v..=v {
        let dd = dx * dx + dz * dz;
        if dd > v * v + 1 { continue; }
        let k = (pcx + dx, pcz + dz);
        let meshed = world.chunks.get(&k).map(|ch| !ch.dirty && ch.entity.is_some()).unwrap_or(false);
        if !meshed { need.push((dd, k)); }
    } }
    need.sort_by_key(|n| n.0);
    for (_, (cx, cz)) in need.into_iter().take(cfg.chunk_budget) {
        for dz in -1..=1 { for dx in -1..=1 { world.ensure(cx + dx, cz + dz); } }
        let m = mesh_chunk(world, cx, cz);
        let ch = world.chunks.get_mut(&(cx, cz)).unwrap();
        if let Some(e) = ch.entity.take() { c.entity(e).despawn_recursive(); }
        let root = c.spawn((Transform::default(), Visibility::default(), ChunkMesh)).id();
        if !m.solid.is_empty() { let e = c.spawn((Mesh3d(meshes.add(m.solid.into_mesh())), MeshMaterial3d(h.chunk_mat.clone()), Transform::default())).id(); c.entity(root).add_child(e); }
        if !m.glow.is_empty() { let e = c.spawn((Mesh3d(meshes.add(m.glow.into_mesh())), MeshMaterial3d(h.glow_mat.clone()), Transform::default(), NotShadowCaster)).id(); c.entity(root).add_child(e); }
        if !m.foliage.is_empty() { let e = c.spawn((Mesh3d(meshes.add(m.foliage.into_mesh())), MeshMaterial3d(h.fol_mat.clone()), Transform::default(), NotShadowCaster)).id(); c.entity(root).add_child(e); }
        ch.entity = Some(root);
        ch.dirty = false;
    }
    // unload far chunks entirely so long runs don't grow memory forever
    let far = (v + 3) * (v + 3);
    let drop: Vec<(i32, i32)> = world.chunks.keys().filter(|(x, z)| (x - pcx).pow(2) + (z - pcz).pow(2) > far + 30).copied().collect();
    for k in drop { if let Some(ch) = world.chunks.remove(&k) { if let Some(e) = ch.entity { c.entity(e).despawn_recursive(); } } }
    for ch in world.chunks.values_mut() { if (ch.cx - pcx).pow(2) + (ch.cz - pcz).pow(2) > far { if let Some(e) = ch.entity.take() { c.entity(e).despawn_recursive(); } } }
}
