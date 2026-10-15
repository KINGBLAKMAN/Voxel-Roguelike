//! The sixteen abilities. Every spell is a real thing in the world, not just a flash of light:
//! Arrow Storm raises a storm cloud that rains real arrows that stick in the ground; Meteor drops a burning rock
//! that carves a crater and stays embedded, cooling; Snare Trap throws a steel bear trap whose jaws snap shut;
//! Frost Nova erupts rings of ice crystals; Smite drives a giant sword of light into the earth; Ground Slam
//! heaves pillars of the ground itself; Battle Cry plants war banners; Mend blooms a lotus of light.
use crate::combat::*;
use crate::fx::{hexc, Blend, Ease, Tween};
use crate::game::*;
use crate::models::*;
use crate::player::*;
use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;
use std::f32::consts::{FRAC_PI_2, PI, TAU};

pub struct AbInfo { pub id: &'static str, pub name: &'static str, pub cd: f32, pub color: u32, pub desc: &'static str }
pub const ABIL: [AbInfo; 24] = [
    AbInfo { id: "volley", name: "Volley", cd: 5.0, color: 0x9be07a, desc: "Fan of 9 piercing arrows." },
    AbInfo { id: "roll", name: "Evasive Roll", cd: 3.0, color: 0xc9e6b0, desc: "Dash and dodge all damage briefly." },
    AbInfo { id: "snare", name: "Snare Trap", cd: 8.0, color: 0xd9b46a, desc: "Throw a steel bear trap that snaps shut on the first foe." },
    AbInfo { id: "storm", name: "Arrow Storm", cd: 13.0, color: 0x7fd46a, desc: "A storm cloud rains real arrows on an area for 4s." },
    AbInfo { id: "blink", name: "Blink", cd: 4.5, color: 0xc4a8ff, desc: "Shatter and reform 11m where you aim." },
    AbInfo { id: "nova", name: "Frost Nova", cd: 8.0, color: 0x8fe6ff, desc: "Rings of ice crystals erupt and freeze everything near." },
    AbInfo { id: "orb", name: "Arcane Orb", cd: 5.0, color: 0xd08aff, desc: "A slow gyroscope of arcane force that pierces and bursts." },
    AbInfo { id: "meteor", name: "Meteor", cd: 14.0, color: 0xff8a3d, desc: "Call down a burning rock that leaves a crater." },
    AbInfo { id: "charge", name: "Charge", cd: 5.5, color: 0xff8a6a, desc: "Rush forward, stunning all you hit." },
    AbInfo { id: "whirl", name: "Whirlwind", cd: 9.0, color: 0xffd0a0, desc: "Become a tornado of steel for 2.5s." },
    AbInfo { id: "slam", name: "Ground Slam", cd: 8.0, color: 0xd9a46a, desc: "Leap and heave pillars of earth out of the ground." },
    AbInfo { id: "cry", name: "Battle Cry", cd: 18.0, color: 0xff5b4c, desc: "Plant war banners: +40% damage and lifesteal for 8s." },
    AbInfo { id: "mend", name: "Mend", cd: 9.0, color: 0x8dff9e, desc: "A lotus of light blooms: heal 25% plus regen." },
    AbInfo { id: "smite", name: "Smite", cd: 4.5, color: 0xffe48a, desc: "Drive a giant sword of light into your target. Stuns." },
    AbInfo { id: "shield", name: "Divine Shield", cd: 16.0, color: 0xfff3b8, desc: "Six holy shields orbit you: invulnerable for 3s." },
    AbInfo { id: "consecrate", name: "Consecration", cd: 11.0, color: 0xffd35a, desc: "Raise a ring of holy obelisks: burns foes, heals you." },
    AbInfo { id: "raise", name: "Raise Dead", cd: 12.0, color: 0x7affb0, desc: "Three skeleton warriors claw out of the ground and fight for you for 15s." },
    AbInfo { id: "bonespear", name: "Bone Spear", cd: 5.0, color: 0xe8e2d0, desc: "A line of bone spikes erupts toward your aim, impaling everything on it." },
    AbInfo { id: "corpse", name: "Corpse Explosion", cd: 7.0, color: 0x9aff6a, desc: "The corpses near your aim burst in gore and green fire." },
    AbInfo { id: "drain", name: "Soul Drain", cd: 10.0, color: 0x6adf9a, desc: "Tether three enemies and drain their life into yours for 2.5s." },
    AbInfo { id: "fan", name: "Fan the Hammer", cd: 5.0, color: 0xffb05a, desc: "Fan six shots across a cone in a heartbeat." },
    AbInfo { id: "smoke", name: "Smoke Roll", cd: 4.0, color: 0xc8c0b0, desc: "Roll away and leave a smoke cloud that blinds and scatters enemies." },
    AbInfo { id: "dynamite", name: "Dynamite", cd: 7.0, color: 0xff6a3a, desc: "Throw a lit bundle of dynamite. Big boom." },
    AbInfo { id: "deadeye", name: "Deadeye", cd: 14.0, color: 0xff4a3a, desc: "Time slows: mark up to 6 enemies, then shoot every one for a guaranteed crit." },
];
pub fn ab_info(id: &str) -> &'static AbInfo { ABIL.iter().find(|a| a.id == id).unwrap_or(&ABIL[0]) }
pub fn class_abilities(c: Class) -> [&'static str; 4] {
    match c { Class::Ranger => ["volley", "roll", "snare", "storm"], Class::Mage => ["blink", "nova", "orb", "meteor"], Class::Warrior => ["charge", "whirl", "slam", "cry"], Class::Priest => ["mend", "smite", "shield", "consecrate"], Class::Necromancer => ["raise", "bonespear", "corpse", "drain"], Class::Gunslinger => ["fan", "smoke", "dynamite", "deadeye"] }
}
/// (ability, key, name, description, max rank, mastery)
pub const MODS: [(&str, &str, &str, &str, u32, bool); 96] = [
    ("volley", "more", "Splinter Volley", "+3 arrows per volley.", 3, false), ("volley", "split", "Splitting Arrows", "Every volley arrow splits into 3 when it hits.", 1, false), ("volley", "fire", "Incendiary Volley", "Volley arrows set enemies ablaze.", 1, false), ("volley", "hail", "Hail of Arrows", "Mastery: every Volley fires two more waves.", 1, true),
    ("roll", "cd", "Nimble", "Roll recharges 22% faster.", 2, false), ("roll", "shot", "Parting Shot", "Fire a ring of 10 arrows as you roll.", 1, false), ("roll", "decoy", "Decoy", "Leave a straw decoy that draws enemies for 4s.", 1, false), ("roll", "dance", "Blade Dance", "Mastery: rolling through enemies slashes them, and each hit refunds 0.4s of cooldown.", 1, true),
    ("snare", "twin", "Twin Snares", "Throw +1 trap at once.", 2, false), ("snare", "boom", "Explosive Snare", "Traps explode, hitting everything nearby.", 1, false), ("snare", "spiked", "Serrated Jaws", "+60% trap damage and +1s root.", 2, false), ("snare", "field", "Minefield", "Mastery: sprung traps scatter 4 mines around them.", 1, true),
    ("storm", "wide", "Wider Storm", "+25% storm radius.", 2, false), ("storm", "long", "Downpour", "Storm lasts 1.5s longer.", 2, false), ("storm", "frost", "Frost-tipped", "Storm arrows slow enemies.", 1, false), ("storm", "eye", "Eye of the Storm", "Mastery: the storm follows you and lasts 2s longer.", 1, true),
    ("blink", "far", "Far Step", "Blink 5m further.", 1, false), ("blink", "wake", "Arcane Wake", "Blink detonates where you leave and land.", 1, false), ("blink", "cd", "Flicker", "Blink recharges 22% faster.", 2, false), ("blink", "mirror", "Mirror Image", "Mastery: blinking leaves an image that fires arcane bolts for 4s.", 1, true),
    ("nova", "wide", "Wider Nova", "+25% nova radius.", 2, false), ("nova", "deep", "Deep Freeze", "Freeze lasts 1.2s longer.", 2, false), ("nova", "shards", "Shatterburst", "Nova fires 10 ice shards outward.", 2, false), ("nova", "zero", "Absolute Zero", "Mastery: Frost Nova pulses two more times, each wider.", 1, true),
    ("orb", "split", "Splitting Orb", "The orb bursts into 3 smaller orbs (5 at rank 2).", 2, false), ("orb", "twin", "Twin Orbs", "Cast two orbs at once.", 1, false), ("orb", "big", "Greater Orb", "The orb is 35% larger and stronger.", 2, false), ("orb", "sing", "Singularity", "Mastery: the orb drags enemies into its path and collapses in a bigger blast.", 1, true),
    ("meteor", "shower", "Meteor Shower", "+2 smaller meteors follow the first.", 2, false), ("meteor", "inferno", "Inferno", "Impact leaves burning ground for 5s.", 1, false), ("meteor", "cata", "Cataclysm", "+40% meteor damage and +20% radius.", 2, false), ("meteor", "arma", "Armageddon", "Mastery: meteors keep raining on enemies around you for 4s.", 1, true),
    ("charge", "mom", "Momentum", "Charge travels 50% further.", 1, false), ("charge", "wave", "Shockwave", "Charge ends in a stunning shockwave.", 1, false), ("charge", "cd", "Relentless", "Charge recharges 22% faster.", 2, false), ("charge", "ruin", "Trail of Ruin", "Mastery: Charge leaves a blazing trail and ends in an eruption.", 1, true),
    ("whirl", "long", "Cyclone", "Whirlwind lasts 1.5s longer.", 2, false), ("whirl", "vortex", "Vortex", "Whirlwind drags enemies inward.", 1, false), ("whirl", "wide", "Wide Arc", "+25% whirlwind radius.", 2, false), ("whirl", "blades", "Bladestorm", "Mastery: Whirlwind hurls spinning blades outward.", 1, true),
    ("slam", "after", "Aftershock", "+1 follow-up shockwave.", 2, false), ("slam", "wide", "Earthsplitter", "+30% slam radius.", 2, false), ("slam", "quake", "Quake", "Slam stuns for 1s longer.", 1, false), ("slam", "fissure", "Fissures", "Mastery: the slam tears 6 erupting fissures through the ground.", 1, true),
    ("cry", "rally", "Rallying Cry", "Battle Cry heals 20% more and lasts 1s longer.", 1, false), ("cry", "terror", "Terrify", "Enemies nearby flee for 3s.", 1, false), ("cry", "rage", "Bloodrage", "+2s duration and +10% lifesteal.", 2, false), ("cry", "warlord", "Warlord", "Mastery: during Battle Cry every swing launches a shockwave.", 1, true),
    ("mend", "prayer", "Prayer of Healing", "Mend also burns every enemy within 6m.", 1, false), ("mend", "greater", "Greater Mend", "Mend heals 15% more.", 2, false), ("mend", "renew", "Renew", "Healing over time lasts twice as long.", 1, false), ("mend", "bloom", "Lifebloom", "Mastery: the lotus stays for 6s, healing you and burning foes.", 1, true),
    ("smite", "chain", "Chain Smite", "+2 swords strike nearby enemies.", 2, false), ("smite", "wrath", "Wrath", "+40% smite damage.", 2, false), ("smite", "cd", "Swift Judgment", "Smite recharges 22% faster.", 2, false), ("smite", "judg", "Judgment", "Mastery: two more swords fall on the same spot.", 1, true),
    ("shield", "aegis", "Aegis", "The orbiting shields bash enemies they touch.", 1, false), ("shield", "retri", "Retribution", "The shields burst outward when it ends.", 1, false), ("shield", "long", "Lasting Grace", "Shield lasts 1.5s longer.", 2, false), ("shield", "thorns", "Thorns of Light", "Mastery: while shielded, you scorch every enemy within 4.5m.", 1, true),
    ("consecrate", "wide", "Hallowed Ground", "+25% radius.", 2, false), ("consecrate", "purge", "Purge", "+50% damage.", 2, false), ("consecrate", "sanct", "Sanctuary", "Doubles the healing.", 1, false), ("consecrate", "march", "Holy March", "Mastery: Consecration follows you and lasts 3s longer.", 1, true),
    ("raise", "more", "Bone Legion", "+1 skeleton per cast.", 2, false), ("raise", "archer", "Skeletal Archers", "Two skeleton archers rise as well.", 1, false), ("raise", "long", "Undying", "Skeletons last 6s longer.", 2, false), ("raise", "legion", "Army of the Dead", "Mastery: skeletons burst when they fall, and each kill they make can raise another.", 1, true),
    ("bonespear", "long", "Impale", "+50% spear length.", 2, false), ("bonespear", "twin", "Twin Spears", "Fire two spears in a V.", 1, false), ("bonespear", "wall", "Ossuary Wall", "Spikes stay for 3s, slowing whatever touches them.", 1, false), ("bonespear", "shards", "Marrow Burst", "Mastery: every spike bursts into bone shards.", 1, true),
    ("corpse", "wide", "Gore Blast", "+30% blast radius.", 2, false), ("corpse", "plague", "Plague", "Blasts leave poison clouds.", 1, false), ("corpse", "more", "Mass Grave", "+2 corpses explode, and one more is made if none lie there.", 2, false), ("corpse", "cascade", "Cascade", "Mastery: anything a blast kills explodes too.", 1, true),
    ("drain", "more", "Many Souls", "+2 tethers.", 2, false), ("drain", "long", "Long Night", "Drain lasts 1.5s longer.", 1, false), ("drain", "feast", "Feast", "Drain heals 50% more.", 2, false), ("drain", "harvest", "Soul Harvest", "Mastery: drained enemies that die rise as skeletons.", 1, true),
    ("fan", "more", "Six More", "+3 shots.", 2, false), ("fan", "fire", "Dragon's Breath", "Fan shots set enemies ablaze.", 1, false), ("fan", "cd", "Quick Draw", "Fan recharges 22% faster.", 2, false), ("fan", "hellfire", "Hellfire", "Mastery: fan the hammer twice, the second time all the way round.", 1, true),
    ("smoke", "cd", "Light Feet", "Smoke Roll recharges 22% faster.", 2, false), ("smoke", "flash", "Flashbang", "The cloud stuns enemies caught in it for 1.2s.", 1, false), ("smoke", "caltrops", "Caltrops", "Leave caltrops that cut and slow.", 1, false), ("smoke", "ghost", "Ghost", "Mastery: vanish for 2s after rolling; your shots crit while hidden.", 1, true),
    ("dynamite", "cluster", "Cluster Charges", "Two extra sticks scatter and blow.", 2, false), ("dynamite", "big", "More Powder", "+30% blast radius and damage.", 2, false), ("dynamite", "sticky", "Sticky Bomb", "The bundle sticks to the first enemy it hits.", 1, false), ("dynamite", "keg", "Powder Keg", "Mastery: the blast leaves a burning crater and an aftershock.", 1, true),
    ("deadeye", "more", "Many Marks", "+3 marks.", 2, false), ("deadeye", "exec", "Executioner", "Marked targets below 30% health die outright.", 1, false), ("deadeye", "cd", "Steady Breath", "Deadeye recharges 22% faster.", 2, false), ("deadeye", "noon", "High Noon", "Mastery: marks every enemy in sight.", 1, true),
];
/// (weapon, key, name, description, max rank)
pub const WMODS: [(&str, &str, &str, &str, u32); 36] = [
    ("longbow", "multi", "Multishot", "Fire 2 extra arrows at half damage.", 2), ("longbow", "split", "Splinter Tips", "Fully drawn arrows split into 3 on hit.", 1), ("longbow", "blast", "Hunter's Mark", "Fully drawn arrows explode where they land.", 1),
    ("daggers", "venom", "Venom", "Hits poison enemies.", 1), ("daggers", "flurry", "Flurry", "Attack 20% faster.", 2), ("daggers", "shadow", "Shadow Strikes", "Every third strike also hurls a spectral dagger.", 1),
    ("firestaff", "cluster", "Cluster Fire", "Explosions scatter 3 bomblets (+2 per rank).", 2), ("firestaff", "big", "Conflagration", "+30% explosion radius.", 2), ("firestaff", "trail", "Phoenix Fire", "Fireballs leave burning ground.", 1),
    ("frostwand", "fork", "Forked Ice", "Fire 2 extra shards.", 2), ("frostwand", "brittle", "Brittle", "Targets freeze after 3 hits instead of 5.", 1), ("frostwand", "lance", "Glacial Lance", "Shards pierce through every enemy.", 1),
    ("greatsword", "wave", "Sword Wave", "Swings launch a piercing wave.", 1), ("greatsword", "exec", "Executioner", "+60% damage to enemies below 35% health.", 1), ("greatsword", "reach", "Titan's Reach", "+30% reach and a wider arc.", 1),
    ("swordshield", "bash", "Shield Bash", "Every 3rd swing stuns.", 1), ("swordshield", "bulwark", "Bulwark", "The shield blocks 10% more damage.", 2), ("swordshield", "riposte", "Riposte", "Taking a hit blasts enemies around you (every 2s).", 1),
    ("mace", "nova", "Holy Nova", "Every 4th hit bursts with light and heals you.", 1), ("mace", "heavy", "Heavy Blows", "+25% mace damage.", 2), ("mace", "thunder", "Thunder Mace", "Every third hit calls lightning on the target.", 1),
    ("scepter", "split", "Splitting Light", "Bolts split into 3 seekers on hit.", 1), ("scepter", "twin", "Twin Bolts", "Fire +1 bolt.", 2), ("scepter", "nova", "Radiant Bolts", "Bolts burst in holy light on impact.", 1),
    ("scythe", "reach", "Long Reap", "+30% reach and a wider arc.", 1), ("scythe", "leech", "Soul Reaper", "Each hit heals 1 (2 at rank 2).", 2), ("scythe", "wave", "Spectral Scythe", "Every third swing hurls a spinning spectral scythe.", 1),
    ("skullstaff", "twin", "Twin Skulls", "Fire +1 skull.", 2), ("skullstaff", "plague", "Plague Skulls", "Skulls poison what they hit.", 1), ("skullstaff", "burst", "Bone Burst", "Skulls burst into bone shards.", 1),
    ("revolvers", "quick", "Quick Hands", "Reload 40% faster and fire 15% faster.", 2), ("revolvers", "rico", "Ricochet", "Bullets bounce to another enemy.", 1), ("revolvers", "hollow", "Hollow Points", "+35% bullet damage.", 2),
    ("rifle", "big", "Big Iron", "+40% rifle damage.", 2), ("rifle", "explo", "Explosive Rounds", "Rifle shots explode on impact.", 1), ("rifle", "lever", "Lever Action", "Fire 30% faster.", 2),
];

/// state for abilities that run across frames
#[derive(Default)]
pub struct AbState { pub charge: Option<ChargeSt>, pub roll: Option<RollSt>, pub slam: Option<SlamSt> }
pub struct ChargeSt { hit: Vec<u32>, drop: f32, ruin: bool, wave: bool, wedge: Entity, dust: f32 }
impl ChargeSt { pub fn wedge(&self) -> Entity { self.wedge } }
pub struct RollSt { hit: Vec<u32>, dance: bool }
#[derive(Clone, Copy)]
pub struct SlamSt { to: Vec3, r: f32, stun: f32, after: u32, fis: bool }

pub fn cd_of(cx: &Ctx, id: &str) -> f32 { ab_info(id).cd * cx.g.st.cd * (1.0 - 0.22 * cx.m(&format!("{}.cd", id)) as f32) }

pub fn use_ability(cx: &mut Ctx, i: usize) {
    if cx.p.dead || cx.g.cds[i] > 0.0 || cx.p.leap.is_some() || cx.g.over { return; }
    let id = class_abilities(cx.p.cls)[i];
    if !cast(cx, id) { return; }
    cx.g.cds[i] = cd_of(cx, id);
    { let aim = cx.p.aim_pt.to_array(); let me = cx.g.net.my_id; cx.g.net.send(crate::net::Msg::Cast { id: me, ab: id.to_string(), aim }); }
    if crate::augments::on_cast(cx, false) {
        cx.later(0.35, move |cx| {
            if cx.p.dead || cx.p.leap.is_some() { return; }
            if cast(cx, id) { let p = cx.p.body.pos + Vec3::Y * 2.4; cx.pop(p, "ECHO".into(), 3); afterimage(cx, 0xff7ae8, 0.5); crate::augments::on_cast(cx, true); }
        });
    }
}
pub fn cast_remote(cx: &mut Ctx, id: &str) { cast(cx, id); }
fn cast(cx: &mut Ctx, id: &str) -> bool {
    match id {
        "volley" => volley(cx), "roll" => roll(cx), "snare" => snare(cx), "storm" => storm(cx),
        "blink" => blink(cx), "nova" => nova(cx), "orb" => orb(cx), "meteor" => meteor(cx),
        "charge" => charge(cx), "whirl" => whirl(cx), "slam" => slam(cx), "cry" => cry(cx),
        "mend" => mend(cx), "smite" => smite(cx), "shield" => shield(cx), "consecrate" => consecrate(cx),
        id => crate::abilities2::cast(cx, id),
    }
}

// ---------------- shared helpers ----------------
/// the aim point, pulled in to a maximum range
fn clamp_aim(cx: &Ctx, max: f32) -> Vec3 {
    let pp = cx.p.body.pos; let a = cx.p.aim_pt;
    let d = Vec2::new(a.x - pp.x, a.z - pp.z);
    if d.length() <= max { a } else { let n = d.normalize() * max; Vec3::new(pp.x + n.x, pp.y, pp.z + n.y) }
}
fn ground_hex(cx: &mut Ctx, p: Vec3) -> u32 {
    let b = cx.w.get(p.x.floor() as i32, p.y.floor() as i32 - 1, p.z.floor() as i32);
    if b == 0 { return 0x8a5b34; }
    let d = crate::world::block_def(b); let c = d.top.unwrap_or(d.c);
    let s = Color::linear_rgb(c[0], c[1], c[2]).to_srgba();
    ((s.red * 255.0) as u32) << 16 | ((s.green * 255.0) as u32) << 8 | (s.blue * 255.0) as u32
}
fn mesh_part(cx: &mut Ctx, parent: Entity, mesh: Handle<Mesh>, mat: Handle<StandardMaterial>, t: Transform) -> Entity {
    let e = cx.c.spawn((Mesh3d(mesh), MeshMaterial3d(mat), t, NotShadowCaster)).id();
    cx.c.entity(parent).add_child(e); e
}
fn empty(cx: &mut Ctx, t: Transform) -> Entity { cx.c.spawn((t, Visibility::default())).id() }
fn set_tf(cx: &mut Ctx, e: Entity, f: impl FnOnce(&mut Transform)) { if let Ok(mut t) = cx.q.get_mut(e) { f(&mut t); } }
/// translucent copy of the player that thins away: dashes, blinks and echoes leave these
pub fn afterimage(cx: &mut Ctx, hex: u32, life: f32) {
    let (pos, yaw, cls) = (Vec3::new(cx.p.body.pos.x, cx.p.vis_y, cx.p.body.pos.z), cx.p.yaw, cx.p.cls);
    let root = ghost_model(cx, pos, yaw, cls, hex);
    let mut t = Tween::new(root, life, pos); t.rot = Quat::from_rotation_y(yaw); t.s0 = Vec3::ONE; t.s1 = Vec3::new(0.05, 1.05, 0.05); t.ease = Ease::InOut;
    cx.fx.tweens.push(t);
}
fn ghost_model(cx: &mut Ctx, pos: Vec3, yaw: f32, cls: Class, hex: u32) -> Entity {
    let (rig, parts) = { let mut b = cx.builder(); b.shadow = false; let r = player_model(&mut b, pos, cls); (r, std::mem::take(&mut b.parts)) };
    let m = cx.pal.mat(cx.mats, hex, Mk::Ghost);
    for (e, _, _) in parts { cx.c.entity(e).insert((MeshMaterial3d(m.clone()), NotShadowCaster)); }
    let root = rig.root.unwrap();
    cx.c.entity(root).insert(Transform::from_translation(pos).with_rotation(Quat::from_rotation_y(yaw)));
    root
}
/// shove of dirt and grass: tumbling clods in the colour of the ground
fn kick_dirt(cx: &mut Ctx, p: Vec3, n: usize, speed: f32) { let g = ground_hex(cx, p); cx.gfx().debris(p + Vec3::Y * 0.1, &[g, 0x8a5b34, 0x6a4a2c], n, speed, 0.16, 5.0); }

// ================= RANGER =================
fn volley(cx: &mut Ctx) -> bool {
    let n = 9 + cx.g.st.extra_proj as u32 + 3 * cx.m("volley.more");
    let fire = cx.m("volley.fire") > 0; let split = if cx.m("volley.split") > 0 { 3 } else { 0 };
    cx.p.pose(Pose::Fan, 0.55);
    volley_wave(cx, n, fire, split);
    if cx.m("volley.hail") > 0 { for i in 1..=2 { cx.later(0.22 * i as f32, move |cx| { if !cx.p.dead { volley_wave(cx, n, fire, split); } }); } }
    true
}
fn volley_wave(cx: &mut Ctx, n: u32, fire: bool, split: u32) {
    let from = cx.hand(); let base = (cx.p.aim_pt - from).normalize();
    let step = (1.25 / n as f32).min(0.07);
    cx.play("bow"); cx.p.cast_kick = 1.0;
    let glow = if fire { 0xff8a3a } else { 0x9be07a };
    for k in 0..n {
        let mut d = Quat::from_rotation_y((k as f32 - (n - 1) as f32 / 2.0) * step) * base; d.y += 0.03;
        player_arrow(cx, from, d.normalize() * 58.0, 15.0, 4.0, true, 2.2, 0x4a7a3a, Some(glow), split, fire, false);
    }
    // the string snaps: a spray of leaves and wind lines fanning out with the arrows
    cx.fx.burst(from, glow, 18, 5.0, 0.35, 0.25, 0.0, 0.0);
    { let mut g = cx.gfx(); g.speed_lines(from + base * 3.0, base, 12, 5.0, 1.4, 0xf4ffe8, 0.22); g.glint(from, glow, 1.4, 0.3); g.card(from + base * 0.6, glow, g.fx.tex.burst.clone(), Vec3::splat(0.4), Vec3::splat(2.2), 0.18, 0.0, 0.0, 6.0); }
    for _ in 0..8 { let d = Quat::from_rotation_y(cx.rr(-0.6, 0.6)) * base; let a = from + Vec3::new(cx.rr(-0.3, 0.3), cx.rr(-0.3, 0.3), cx.rr(-0.3, 0.3)); let l = cx.rr(3.0, 6.0); cx.gfx().streak(a, a + d * l, 0xeaffd8, 0.18, 0.035); }
    let lc = hexc(0x6fb83a);
    for _ in 0..10 { let v = Quat::from_rotation_y(cx.rr(-0.7, 0.7)) * base * cx.rr(4.0, 9.0) + Vec3::Y * cx.rr(0.0, 2.0); cx.fx.emit(false, from, v, cx.rr(0.6, 1.0), 0.12, 0.1, lc, 4.0, 2.0, 1.0); }
}

fn roll(cx: &mut Ctx) -> bool {
    let mut d = cx.p.move_dir; if d.length_squared() < 0.01 { d = -cx.p.fwd(); }
    d.y = 0.0; let d = d.normalize();
    let dance = cx.m("roll.dance") > 0;
    cx.p.dash = Some(Dash { dir: d, speed: 25.0, t: 0.3, kind: 1 });
    cx.p.roll_t = 0.3; cx.p.invuln = 0.45;
    cx.g.ab.roll = Some(RollSt { hit: vec![], dance });
    cx.play("dash");
    let pp = cx.p.body.pos;
    cx.fx.smoke(pp, 0xd8d0c0, 10, 2.0, 0.7, 0.9);
    cx.gfx().speed_lines(pp + Vec3::Y * 0.9 - d * 1.2, d, 10, 3.0, 0.8, 0xffffff, 0.25);
    kick_dirt(cx, pp, 8, 4.0);
    afterimage(cx, 0x9be07a, 0.35);
    if cx.m("roll.decoy") > 0 { spawn_decoy(cx, pp); }
    if cx.m("roll.shot") > 0 { let c = pp + Vec3::Y * 1.2; for i in 0..10 { let an = i as f32 / 10.0 * TAU; player_arrow(cx, c, Vec3::new(an.cos(), 0.05, an.sin()) * 44.0, 12.0, 3.0, false, 1.2, 0x4a7a3a, Some(0x9be07a), 0, false, false); } }
    true
}
/// a straw dummy in a green hood: the horde can't resist it
fn spawn_decoy(cx: &mut Ctx, pos: Vec3) {
    if cx.g.rel.decoys.len() >= 3 { let d = cx.g.rel.decoys.remove(0); cx.c.entity(d.e).despawn_recursive(); }
    let yaw = cx.p.yaw;
    let mut b = cx.builder();
    let root = b.root(pos);
    let v = Vec3::new;
    b.bx(root, v(0.12, 1.5, 0.12), 0x6a4528, v(0.0, 0.75, 0.0)); b.bx(root, v(1.3, 0.1, 0.1), 0x6a4528, v(0.0, 1.25, 0.0));
    b.bx(root, v(0.55, 0.7, 0.32), 0xc8a85a, v(0.0, 1.05, 0.0));
    let head = b.bx(root, v(0.42, 0.42, 0.42), 0xb89a68, v(0.0, 1.72, 0.0)); let _ = head;
    b.bx(root, v(0.1, 0.03, 0.02), 0x2a1a10, v(-0.1, 1.76, 0.22)); b.bx(root, v(0.1, 0.03, 0.02), 0x2a1a10, v(0.1, 1.76, 0.22)); b.bx(root, v(0.22, 0.03, 0.02), 0x2a1a10, v(0.0, 1.62, 0.22));
    b.bx(root, v(0.6, 0.18, 0.5), 0x3a6a2c, v(0.0, 1.96, -0.02)); b.bx(root, v(0.6, 0.9, 0.08), 0x3a6a2c, v(0.0, 1.2, -0.22));
    for (x, y) in [(-0.62, 1.2), (0.62, 1.2), (-0.15, 0.5), (0.15, 0.48)] { b.bx(root, v(0.16, 0.22, 0.1), 0xe0c070, v(x, y, 0.0)); }
    b.rot(root, 0.0, yaw, 0.0);
    cx.g.rel.decoys.push(crate::relics::Decoy { pos, t: 4.0, hp: 80.0, e: root });
    cx.fx.smoke(pos, 0xd8d0c0, 8, 1.5, 0.8, 0.9);
    cx.gfx().debris(pos + Vec3::Y, &[0xe0c070, 0xc8a85a], 6, 3.0, 0.1, 3.0);
}

fn snare(cx: &mut Ctx) -> bool {
    let cnt = 1 + cx.m("snare.twin");
    let fl = cx.p.fwd(); let rt = cx.p.right(); let pp = cx.p.body.pos;
    let mut base = cx.p.aim_pt; if pp.distance(base) > 22.0 { base = pp + fl * 22.0; }
    cx.p.pose(Pose::Throw, 0.45); cx.play("whoosh");
    for i in 0..cnt {
        let off = if i == 0 { 0.0 } else { (if i % 2 == 1 { 1.0 } else { -1.0 }) * ((i + 1) / 2) as f32 * 2.4 };
        let mut to = base + rt * off; to.y = cx.ground(to.x, to.z, to.y + 1.0);
        throw_trap(cx, to, false);
    }
    true
}
/// the steel trap: a base plate with chain and stake, and two toothed jaws hinged at the sides
fn trap_model(cx: &mut Ctx, pos: Vec3, mini: bool) -> (Entity, [Entity; 2]) {
    let mut b = cx.builder(); b.shadow = false;
    let root = b.root(pos);
    let v = Vec3::new;
    let (iron, dark, brass) = (0x9aa0a8, 0x4a4c52, 0xb08a3a);
    b.bx(root, v(1.0, 0.06, 0.24), dark, v(0.0, 0.03, 0.0)); b.bx(root, v(0.24, 0.06, 0.9), dark, v(0.0, 0.03, 0.0));
    b.bx(root, v(0.36, 0.05, 0.36), brass, v(0.0, 0.07, 0.0));
    for x in [-0.38, 0.38] { b.bx(root, v(0.12, 0.1, 0.5), iron, v(x, 0.05, 0.0)); }
    // chain to a stake
    for k in 0..4 { b.bx(root, v(0.08, 0.05, 0.12), iron, v(0.0, 0.03, -0.55 - k as f32 * 0.12)); }
    b.bx(root, v(0.1, 0.4, 0.1), 0x6a4528, v(0.0, 0.1, -1.08));
    let mut jaws = [root; 2];
    for (j, s) in [(0, -1.0f32), (1, 1.0f32)] {
        let hinge = b.pivot(root, v(s * 0.46, 0.08, 0.0));
        b.bx(hinge, v(0.05, 0.5, 0.9), iron, v(0.0, 0.25, 0.0));
        for t in 0..6 { b.bx(hinge, v(0.06, 0.16, 0.06), 0xd8dde4, v(0.0, 0.55, -0.38 + t as f32 * 0.152)); }
        b.rot(hinge, 0.0, 0.0, -s * 1.45);
        jaws[j] = hinge;
    }
    b.c.entity(root).insert(Transform::from_translation(pos).with_scale(Vec3::splat(if mini { 0.8 } else { 1.4 })));
    (root, jaws)
}
fn throw_trap(cx: &mut Ctx, to: Vec3, mini: bool) {
    let from = if mini { to + Vec3::Y * 2.0 } else { cx.hand() };
    let (root, jaws) = trap_model(cx, from, mini);
    // closed while it flies, tumbling end over end
    for (j, s) in [(0usize, -1.0f32), (1, 1.0)] { let h = jaws[j]; cx.c.entity(h).queue(move |mut ew: EntityWorldMut| { if let Some(mut t) = ew.get_mut::<Transform>() { t.rotation = Quat::from_rotation_z(s * 1.1); } }); }
    let dur = if mini { 0.35 } else { 0.5 + from.distance(to) * 0.012 };
    let mut z = Zone::new(to, 0.0, dur);
    let sc = if mini { 0.8 } else { 1.4 };
    z.on_frame = Some(Box::new(move |cx, z| {
        let k = (z.t / dur).min(1.0);
        let p = from.lerp(to, k) + Vec3::Y * (k * PI).sin() * from.distance(to) * 0.22;
        let spin = if k < 1.0 { k * TAU * 1.5 } else { 0.0 };
        set_tf(cx, root, |t| { t.translation = p; t.rotation = Quat::from_rotation_x(spin); t.scale = Vec3::splat(sc); });
    }));
    z.on_end = Some(Box::new(move |cx, _| {
        set_tf(cx, root, |t| { t.translation = to; t.rotation = Quat::IDENTITY; });
        cx.play("heavy");
        cx.fx.burst(to + Vec3::Y * 0.3, 0xd9b46a, if mini { 6 } else { 12 }, 3.0, 0.35, 0.22, 0.0, 6.0);
        kick_dirt(cx, to, 5, 2.5);
        cx.gfx().ring(to, 0xd9b46a, 0.3, if mini { 1.0 } else { 1.6 }, 0.4);
        if cx.g.traps.len() >= 24 { let t = cx.g.traps.remove(0); cx.c.entity(t.root).despawn_recursive(); }
        let spiked = cx.m("snare.spiked"); let boom = cx.m("snare.boom");
        let field = !mini && cx.m("snare.field") > 0;
        let ghost = cx.g.net.ghost;
        cx.g.traps.push(Trap { ghost, pos: to, life: 30.0, armed: 0.35, spiked, boom, field, mini, root, jaws, snap: -1.0 });
    }));
    zone(cx, z);
}
pub fn update_traps(cx: &mut Ctx) {
    let dt = cx.dt;
    let mut i = 0;
    while i < cx.g.traps.len() {
        let (pos, mini, jaws, root) = { let t = &mut cx.g.traps[i]; t.life -= dt; t.armed -= dt; if t.snap >= 0.0 { t.snap += dt; } (t.pos, t.mini, t.jaws, t.root) };
        let (life, armed, snap) = { let t = &cx.g.traps[i]; (t.life, t.armed, t.snap) };
        // the jaws spring open as it arms, slam shut when sprung
        let ang = if snap >= 0.0 { crate::noise::lerp(1.45, -1.1, (snap / 0.07).min(1.0)) } else { crate::noise::lerp(-1.1, 1.45, (1.0 - armed / 0.35).clamp(0.0, 1.0)) };
        for (j, s) in [(0usize, -1.0f32), (1, 1.0)] { set_tf(cx, jaws[j], |t| t.rotation = Quat::from_rotation_z(-s * ang)); }
        if life <= 0.0 || snap > 1.8 {
            let t = cx.g.traps.swap_remove(i);
            cx.c.entity(root).despawn_recursive();
            if snap > 0.0 { cx.gfx().debris(t.pos + Vec3::Y * 0.2, &[0x9aa0a8, 0x4a4c52], 8, 3.0, 0.12, 4.0); }
            continue;
        }
        if snap >= 0.0 || armed > 0.0 { i += 1; continue; }
        cx.g.net.ghost = cx.g.traps[i].ghost;
        if cx.r() < 0.25 { let j = Vec3::new(cx.rr(-0.6, 0.6), 0.25, cx.rr(-0.6, 0.6)); cx.fx.emit(true, pos + j, Vec3::Y * 0.8, 0.5, 0.18, 0.04, hexc(0xffe0a0) * 2.5, 0.0, 0.0, 0.9); }
        let rad = if mini { 0.9 } else { 1.3 };
        let victim = cx.g.enemies.iter().position(|e| !e.dead && !e.t.passive && !e.t.prop && !e.t.fly && Vec2::new(e.body.pos.x - pos.x, e.body.pos.z - pos.z).length() < rad + e.body.w && (e.body.pos.y - pos.y).abs() < 1.5);
        if let Some(v) = victim {
            let (spiked, boom, field) = { let t = &mut cx.g.traps[i]; t.snap = 0.0; (t.spiked, t.boom, t.field) };
            // haul the victim onto the plate and hold it there
            { let e = &mut cx.g.enemies[v]; e.body.pos.x = pos.x; e.body.pos.z = pos.z; e.body.vel = Vec3::ZERO; }
            cx.play("snap"); cx.shake(0.12);
            { let mut g = cx.gfx(); g.flare(pos + Vec3::Y * 0.4, 0xffe0a0, if mini { 2.0 } else { 3.5 }, 0.25, true); }
            cx.fx.sparks(pos + Vec3::Y * 0.4, 0xffe0a0, 12, 8.0);
            let dm = (32.0 * (1.0 + 0.6 * spiked as f32) * if mini { 0.6 } else { 1.0 }).round();
            cx.damage(v, dm, Hit { col: 0xd9b46a, ..default() });
            cx.status(v, Status { stun: (3.0 + spiked as f32) * if mini { 0.5 } else { 1.0 }, ..default() });
            if boom > 0 { explode_fire(cx, pos + Vec3::Y * 0.5, if mini { 2.5 } else { 4.0 }, if mini { 20.0 } else { 40.0 }, 0, false); }
            if field { cx.later(0.05, move |cx| { for k in 0..4 { let an = k as f32 / 4.0 * TAU + cx.rr(-0.4, 0.4); let d = cx.rr(2.0, 3.5); let mut to = Vec3::new(pos.x + an.cos() * d, 0.0, pos.z + an.sin() * d); to.y = cx.ground(to.x, to.z, pos.y + 2.0); throw_trap(cx, to, true); } }); }
        }
        cx.g.net.ghost = false;
        i += 1;
    }
}

fn storm(cx: &mut Ctx) -> bool {
    let eye = cx.m("storm.eye") > 0;
    let mut c = if eye { cx.p.body.pos } else { clamp_aim(cx, 30.0) }; c.y = cx.ground(c.x, c.z, c.y + 1.0);
    let r = 6.0 * cx.g.st.area * (1.0 + 0.25 * cx.m("storm.wide") as f32);
    let life = 4.0 + 1.5 * cx.m("storm.long") as f32 + if eye { 2.0 } else { 0.0 };
    let frost = cx.m("storm.frost") > 0;
    cx.p.pose(Pose::BowUp, 0.5); cx.play("bow");
    { let pp = cx.p.body.pos + Vec3::Y * 3.0; cx.gfx().speed_lines(pp, Vec3::Y, 12, 5.0, 0.8, 0xeaffd8, 0.3); }
    // a single glowing arrow shot straight up calls the storm
    let from = cx.p.body.pos + Vec3::Y * 1.8;
    let e = arrow_mesh(cx, 0x4a7a3a, 0xd8dde4, Some(0xd8ffb8));
    let mut p = Proj::new(from, Vec3::Y * 60.0); p.e = Some(e); p.life = 0.45; p.no_world = true; p.pierce = true; p.r = 0.1;
    p.trail = Some(Box::new(|cx, p, _| cx.fx.emit(true, p.pos, Vec3::ZERO, 0.4, 0.35, 0.05, hexc(0xd8ffb8) * 2.5, 0.0, 0.0, 1.0)));
    p.on_expire = Some(Box::new(|cx, p, _| { cx.gfx().flare(p.pos, 0xeaffd8, 4.0, 0.3, true); cx.play("zap"); }));
    spawn_proj(cx, p);
    cx.later(0.4, move |cx| storm_cloud(cx, c, r, life, frost, eye));
    true
}
fn storm_cloud(cx: &mut Ctx, c: Vec3, r: f32, life: f32, frost: bool, eye: bool) {
    let h = 10.0;
    // the cloud: a heap of dark voxel billows that churn overhead
    let root = empty(cx, Transform::from_translation(c + Vec3::Y * h).with_scale(Vec3::splat(0.01)));
    let cols = if frost { [0x8a9aaa, 0xaab8c8, 0x6a7888, 0xc8d4e0] } else { [0x4a4f5a, 0x5a606a, 0x3a3e48, 0x7a808a] };
    let mut puffs = vec![];
    let n = (16.0 * r / 6.0) as usize + 10;
    for k in 0..n {
        // a heaped thunderhead: big dark billows low, smaller pale ones piled on top
        let an = k as f32 / n as f32 * TAU * 2.6 + cx.rr(-0.3, 0.3); let rr = r * cx.r().sqrt() * 1.05;
        let top = k % 3 == 0;
        let s = cx.rr(1.8, 3.4) * if top { 0.75 } else { 1.0 };
        let y = if top { cx.rr(0.8, 2.2) } else { cx.rr(-0.6, 0.6) } * (1.0 - rr / (r * 1.3));
        let m = cx.pal.mat(cx.mats, cols[if top { 3 } else { k % 3 }], Mk::Lit);
        let cube = cx.pal.cube.clone();
        let e = mesh_part(cx, root, cube, m, Transform::from_xyz(an.cos() * rr, y, an.sin() * rr).with_scale(Vec3::new(s * 1.2, s * 0.8, s)).with_rotation(Quat::from_rotation_y(cx.rr(0.0, 1.5))));
        puffs.push((e, an, rr, cx.rr(0.0, 6.0), y));
    }
    // a dark shadow pooling on the ground beneath
    { let tx = cx.fx.tex.glow.clone(); let m = cx.gfx().fmat(0x101418, 0.0, Some(tx), Blend::Alpha, true); let mesh = cx.fx.disk.clone(); let rr = r * 1.15;
      cx.gfx().spawn_tween(mesh, m, life + 0.8, c, |t| { t.s0 = Vec3::splat(rr * 0.3); t.s1 = Vec3::splat(rr); t.alpha = 0.45; t.fade_in = 0.15; t.fade_out = 0.2; }); }
    cx.play("rumble");
    let mut z = Zone::new(c, r, life + 0.6); z.every = 0.25; z.follow = eye;
    z.ents.push(root);
    let mut acc = 0.0f32;
    z.on_frame = Some(Box::new(move |cx, z| {
        let k_in = (z.t / 0.5).min(1.0); let k_out = ((z.life - z.t) / 0.6).clamp(0.0, 1.0);
        let raining = z.t > 0.35 && z.t < z.life - 0.5;
        let cp = z.pos + Vec3::Y * h;
        set_tf(cx, root, |t| { t.translation = cp; t.scale = Vec3::splat((k_in * k_out).max(0.01)); t.rotation = Quat::from_rotation_y(z.t * 0.25); });
        for &(e, an, rr, ph, y) in puffs.iter() { let bob = (z.t * 1.7 + ph).sin() * 0.25; set_tf(cx, e, |t| { t.translation = Vec3::new(an.cos() * rr, y + bob, an.sin() * rr); }); }
        // lightning flickers inside the cloud
        if cx.r() < 0.04 { let a = cp + Vec3::new(cx.rr(-r, r) * 0.6, -0.5, cx.rr(-r, r) * 0.6); let b = a + Vec3::new(cx.rr(-2.0, 2.0), cx.rr(-3.0, -1.0), cx.rr(-2.0, 2.0)); cx.gfx().lightning(a, b, if frost { 0xd8f8ff } else { 0xeaffd8 }, 0.12, 0.12); }
        if !raining { return; }
        // the rain: real arrows falling out of the cloud
        acc += cx.dt * 16.0 * (r / 6.0).max(1.0);
        while acc >= 1.0 {
            acc -= 1.0;
            let an = cx.rr(0.0, TAU); let rr = cx.r().sqrt() * z.r;
            let from = Vec3::new(z.pos.x + an.cos() * rr, cp.y - 0.8, z.pos.z + an.sin() * rr);
            let v = Vec3::new(cx.rr(-2.0, 2.0), -46.0, cx.rr(-2.0, 2.0));
            storm_arrow(cx, from, v, frost);
        }
    }));
    z.on_tick = Some(Box::new(move |cx, z| {
        if z.t < 0.35 || z.t > z.life - 0.5 { return; }
        cx.play("bow");
        let (p, rr) = (z.pos, z.r);
        cx.aoe(p, rr, &mut |cx, i, _| { cx.damage(i, 5.0, Hit { kb: 0.5, src: Some(p), quiet: true, ..default() }); if frost { cx.status(i, Status { slow: 1.5, ..default() }); } });
    }));
    zone(cx, z);
}
fn storm_arrow(cx: &mut Ctx, from: Vec3, v: Vec3, frost: bool) {
    let e = arrow_mesh(cx, if frost { 0x8fb8d0 } else { 0x4a7a3a }, 0xd8dde4, if frost { Some(0xbff4ff) } else { None });
    cx.c.entity(e).insert(Transform::from_translation(from).with_scale(Vec3::splat(1.5)));
    let mut p = Proj::new(from, v); p.e = Some(e); p.grav = 10.0; p.life = 1.5; p.r = 0.25; p.stick = true; p.dmg = 10.0;
    p.trail = Some(Box::new(move |cx, p, _| { if cx.r() < 0.5 { cx.fx.emit(true, p.pos, Vec3::ZERO, 0.15, 0.14, 0.02, hexc(if frost { 0xd8f8ff } else { 0xeaffd8 }) * 1.5, 0.0, 0.0, 0.6); } }));
    p.on_hit = Some(Box::new(move |cx, p, i| { if let Some(i) = i { let s = p.pos; let d = p.dmg; cx.damage(i, d, Hit { kb: 0.5, src: Some(s), quiet: true, col: 0xb8ff8a, ..default() }); if frost { cx.status(i, Status { slow: 1.5, ..default() }); } cx.fx.sparks(s, 0xeaffd8, 3, 5.0); } }));
    p.on_world = Some(Box::new(|cx, p, _| { if cx.r() < 0.3 { cx.fx.emit(false, p.pos, Vec3::Y * 0.8, 0.5, 0.15, 0.35, hexc(0xb8a888), 0.0, 1.0, 0.5); } }));
    spawn_proj(cx, p);
}

// ================= MAGE =================
fn blink(cx: &mut Ctx) -> bool {
    let mut f = cx.p.look(); f.y = 0.0; let f = f.normalize_or_zero();
    let from = cx.p.body.pos;
    let far = 11.0 + 5.0 * cx.m("blink.far") as f32;
    let mut ok = None; let mut d = far;
    while d >= 1.0 {
        let (x, z) = (from.x + f.x * d, from.z + f.z * d);
        let gy = cx.ground(x, z, from.y + 2.0);
        if (gy - from.y).abs() <= 6.0 && !crate::physics::collides(cx.w, x, gy, z, 0.3, 1.8) { ok = Some(Vec3::new(x, gy, z)); break; }
        d -= 0.5;
    }
    let Some(to) = ok else { return false };
    cx.play("arcane");
    // the mage shatters like violet glass, streams across as motes, and reassembles
    afterimage(cx, 0xc4a8ff, 0.4);
    blink_rift(cx, from, f, true);
    cx.gfx().debris(from + Vec3::Y, &[0xc4a8ff, 0x8a5aff, 0xffffff], 14, 5.0, 0.12, 3.0);
    if cx.m("blink.mirror") > 0 { mirror_image(cx, from); }
    cx.p.body.pos = to; cx.p.vis_y = to.y; cx.p.body.vel = Vec3::ZERO; cx.p.invuln = 0.25;
    blink_rift(cx, to, f, false);
    { let mid = from.lerp(to, 0.5) + Vec3::Y; let len = from.distance(to); let mut g = cx.gfx(); g.speed_lines(mid, f, 18, len, 1.1, 0xe8dcff, 0.3); g.impact(to + Vec3::Y, 0xc4a8ff, 1.5); g.glint(from + Vec3::Y * 1.2, 0xffffff, 1.2, 0.3); }
    cx.punch(0.25);
    for i in 0..40 { let t = i as f32 / 40.0; let p = from.lerp(to, t) + Vec3::Y * (1.0 + (t * PI).sin() * 0.6); let v = Vec3::new(cx.rr(-0.6, 0.6), cx.rr(-0.6, 0.6), cx.rr(-0.6, 0.6)) + f * 6.0; let c = hexc(if i % 3 == 0 { 0xffffff } else { 0xa183ff }) * 2.5; cx.fx.emit(true, p, v, 0.45, 0.3, 0.04, c, 0.0, 3.0, 1.0); }
    if cx.m("blink.wake") > 0 {
        for p in [from, to] {
            { let mut g = cx.gfx(); g.ring(p, 0xd08aff, 0.5, 3.5, 0.4); g.shock(p, 0xd08aff, 0.4, 3.5, 0.35, 1.4); }
            cx.fx.burst(p + Vec3::Y, 0xd08aff, 30, 6.0, 0.5, 0.4, 0.0, 0.0);
            cx.aoe(p, 3.5, &mut |cx, i, _| cx.damage(i, 30.0, Hit { kb: 6.0, src: Some(p), col: 0xd08aff, ..default() }));
        }
    }
    true
}
/// a standing ring torn in the air that collapses (departure) or opens (arrival)
fn blink_rift(cx: &mut Ctx, p: Vec3, f: Vec3, closing: bool) {
    let m = cx.gfx().fmat(0xc4a8ff, 6.0, None, Blend::Add, true); let mesh = cx.pal.torus.clone();
    let rot = Quat::from_rotation_arc(Vec3::Y, f);
    cx.gfx().spawn_tween(mesh, m, 0.4, p + Vec3::Y * 1.0, |t| { t.rot = rot; let big = Vec3::new(0.8, 1.0, 1.3); let small = Vec3::new(0.05, 1.0, 0.05); if closing { t.s0 = big; t.s1 = small; } else { t.s0 = small; t.s1 = big; } t.ease = Ease::Out; t.fade_in = 0.02; t.fade_out = 0.5; t.spin = Vec3::new(0.0, 6.0, 0.0); });
    cx.gfx().flare(p + Vec3::Y, 0xc4a8ff, 3.0, 0.3, true);
    cx.fx.burst(p + Vec3::Y, 0xc4a8ff, 20, 4.0, 0.4, 0.3, 0.0, 0.0);
}
fn mirror_image(cx: &mut Ctx, pos: Vec3) {
    let (yaw, cls) = (cx.p.yaw, cx.p.cls);
    let root = ghost_model(cx, pos, yaw, cls, 0x8a5aff);
    cx.gfx().ring(pos, 0xd08aff, 0.4, 2.2, 0.4);
    let mut z = Zone::new(pos, 0.0, 4.0); z.every = 0.4; z.ents.push(root);
    z.on_tick = Some(Box::new(|cx, z| {
        let Some(&f) = cx.foes(z.pos, 18.0).first() else { return };
        let from = z.pos + Vec3::Y * 1.4; let e = &cx.g.enemies[f]; let d = e.body.pos + Vec3::Y * e.body.h * 0.5 - from;
        cx.play("arcane"); crate::relics::seek_bolt(cx, from, d, 16.0, 0xd08aff, false, 0.2, 26.0, 4.0);
    }));
    z.on_frame = Some(Box::new(move |cx, z| { let y = yaw + z.t * 0.8; let p = z.pos; set_tf(cx, root, |t| t.rotation = Quat::from_rotation_y(y)); if cx.r() < 0.4 { let j = Vec3::new(cx.rr(-0.4, 0.4), cx.rr(0.0, 1.8), cx.rr(-0.4, 0.4)); cx.fx.emit(true, p + j, Vec3::Y, 0.5, 0.3, 0.04, hexc(0xd08aff) * 2.0, 0.0, 0.0, 0.8); } }));
    z.on_end = Some(Box::new(|cx, z| { let p = z.pos + Vec3::Y; cx.fx.burst(p, 0xd08aff, 20, 4.0, 0.4, 0.3, 0.0, 0.0); cx.gfx().debris(p, &[0xc4a8ff, 0x8a5aff], 10, 4.0, 0.12, 3.0); }));
    zone(cx, z);
}

fn nova(cx: &mut Ctx) -> bool {
    let r = 7.0 * cx.g.st.area * (1.0 + 0.25 * cx.m("nova.wide") as f32);
    let fr = 2.6 + 1.2 * cx.m("nova.deep") as f32;
    let p0 = cx.p.body.pos;
    cx.p.pose(Pose::Slam, 0.5);
    nova_pulse(cx, p0, r, fr, 15.0);
    if cx.m("nova.zero") > 0 { for i in 1..=2 { cx.later(0.45 * i as f32, move |cx| { if !cx.p.dead { let p = cx.p.body.pos; nova_pulse(cx, p, r * (1.0 + 0.25 * i as f32), fr * 0.6, 12.0); } }); } }
    let ns = 10 * cx.m("nova.shards");
    for i in 0..ns {
        let an = i as f32 / ns as f32 * TAU;
        let e = shard_mesh(cx, 0xbff4ff, 2.6);
        let mut p = Proj::new(p0 + Vec3::Y, Vec3::new(an.cos(), 0.0, an.sin()) * 30.0); p.e = Some(e); p.r = 0.3; p.life = 0.8; p.pierce = true;
        p.trail = Some(Box::new(|cx, p, _| cx.fx.emit(true, p.pos, Vec3::ZERO, 0.25, 0.3, 0.04, hexc(0x8fe6ff) * 2.5, 0.0, 0.0, 0.8)));
        p.on_hit = Some(Box::new(|cx, p, i| { if let Some(i) = i { let s = p.pos; cx.damage(i, 9.0, Hit { kb: 2.0, src: Some(s), col: 0x8fe6ff, ..default() }); cx.status(i, Status { slow: 2.0, ..default() }); } }));
        spawn_proj(cx, p);
    }
    true
}
/// three rings of ice crystals punch out of the ground one after another, frost spreads, and everything inside freezes solid
fn nova_pulse(cx: &mut Ctx, p0: Vec3, r: f32, fr: f32, dmg: f32) {
    cx.play("ice"); cx.shake(0.25);
    cx.gfx().impact(p0 + Vec3::Y, 0xbff4ff, 2.4); cx.punch(0.35);
    { let mut g = cx.gfx(); for _ in 0..6 { let an = g.fx.rr(0.0, TAU); let rr = g.fx.rr(1.0, r * 0.8); g.glint(p0 + Vec3::new(an.cos() * rr, g.fx.rr(0.3, 1.8), an.sin() * rr), 0xe8fbff, 0.8, 0.5); } }
    { let mut g = cx.gfx(); g.ring(p0, 0xffffff, 0.5, r, 0.4); g.flare(p0 + Vec3::Y, 0xbff4ff, 5.0, 0.3, true); }
    // a sheet of hoarfrost on the ground
    { let tx = cx.fx.tex.circles["frost"].clone(); let m = cx.gfx().fmat(0xe8fbff, 1.2, Some(tx), Blend::Add, true); let mesh = cx.fx.quad.clone(); let rot = Quat::from_rotation_y(cx.rr(0.0, TAU));
      cx.gfx().spawn_tween(mesh, m, 2.6, p0 + Vec3::Y * 0.06, |t| { t.s0 = Vec3::splat(0.5); t.s1 = Vec3::splat(r * 1.05); t.rot = rot; t.alpha = 0.7; t.fade_in = 0.02; t.fade_out = 0.4; t.ease = Ease::Out; }); }
    for (k, f) in [0.35f32, 0.68, 1.0].into_iter().enumerate() {
        let n = (8.0 + f * r * 2.6) as usize; let rr = r * f;
        cx.later(0.06 * k as f32, move |cx| { cx.gfx().spikes(p0, 0xbff4ff, n, rr, 1.8 - k as f32 * 0.2, 1.6 - k as f32 * 0.35, true); cx.gfx().debris(p0 + Vec3::Y * 0.3, &[0xbff4ff, 0xe8fbff, 0x8fe6ff], 6, rr * 1.4, 0.16, 5.0); });
    }
    // cold mist rolling outward
    for _ in 0..30 { let an = cx.rr(0.0, TAU); let s = cx.rr(2.0, r * 1.6); cx.fx.emit(false, p0 + Vec3::Y * 0.3, Vec3::new(an.cos() * s, cx.rr(0.2, 1.0), an.sin() * s), cx.rr(0.8, 1.3), 0.6, 1.6, hexc(0xe8fbff), -0.3, 2.5, 0.5); }
    for _ in 0..40 { let an = cx.rr(0.0, TAU); let s = cx.rr(4.0, r * 2.2); cx.fx.emit(true, p0 + Vec3::Y * 0.5, Vec3::new(an.cos() * s, cx.rr(0.0, 1.5), an.sin() * s), 0.5, 0.35, 0.05, hexc(0xbff4ff) * 2.5, 0.0, 3.0, 1.0); }
    cx.aoe(p0, r, &mut |cx, i, _| { cx.damage(i, dmg, Hit { kb: 3.0, src: Some(p0), col: 0x8fe6ff, ..default() }); cx.status(i, Status { freeze: fr, ..default() }); });
}

fn orb(cx: &mut Ctx) -> bool {
    let from = cx.hand(); let dir = (cx.p.aim_pt - from).normalize();
    cx.play("arcane"); cx.p.cast_kick = 1.0; cx.p.pose(Pose::Push, 0.4);
    let sc = 1.0 + 0.35 * cx.m("orb.big") as f32; let n = 1 + cx.m("orb.twin");
    let (split, sing) = (cx.m("orb.split"), cx.m("orb.sing") > 0);
    for k in 0..n { let d = if n > 1 { Quat::from_rotation_y(if k > 0 { 0.14 } else { -0.14 }) * dir } else { dir }; make_orb(cx, from, d, sc, (26.0 * sc).round(), split, sing); }
    cx.fx.burst(from, 0xd08aff, 20, 4.0, 0.4, 0.35, 0.0, 0.0);
    true
}
/// the orb is a gyroscope: a white-hot core in a violet shell, two rings spinning on crossed axes, and runes in orbit
fn make_orb(cx: &mut Ctx, from: Vec3, dir: Vec3, sc: f32, dmg: f32, split: u32, sing: bool) {
    let root = glow_ball(cx, 0xb46aff, 0.55 * sc);
    let ring_m = cx.pal.mat(cx.mats, 0xe0b8ff, Mk::Glow); let torus = cx.pal.torus.clone();
    let r1 = mesh_part(cx, root, torus.clone(), ring_m.clone(), Transform::from_scale(Vec3::splat(0.95 * sc)));
    let r2 = mesh_part(cx, root, torus, ring_m, Transform::from_scale(Vec3::splat(1.2 * sc)));
    let rune_m = cx.pal.mat(cx.mats, 0xffffff, Mk::Glow); let cube = cx.pal.cube.clone();
    let mut runes = vec![];
    for _ in 0..4 { runes.push(mesh_part(cx, root, cube.clone(), rune_m.clone(), Transform::from_scale(Vec3::splat(0.14 * sc)))); }
    let mut p = Proj::new(from, dir * 12.0); p.e = Some(root); p.r = 0.9 * sc; p.life = 2.4; p.pierce = true; p.orient = Orient::Fixed; p.dmg = dmg;
    let mut pull = 0.0f32;
    p.trail = Some(Box::new(move |cx, p, _| {
        let a = p.age;
        set_tf(cx, r1, |t| t.rotation = Quat::from_euler(EulerRot::XYZ, a * 4.0, a * 1.5, 0.0));
        set_tf(cx, r2, |t| t.rotation = Quat::from_euler(EulerRot::XYZ, 0.0, a * 2.0, a * 5.0 + 1.2));
        for (k, &e) in runes.iter().enumerate() { let an = a * 3.0 + k as f32 * TAU / 4.0; set_tf(cx, e, |t| { t.translation = Vec3::new(an.cos() * 1.6 * sc, (an * 2.0).sin() * 0.4 * sc, an.sin() * 1.6 * sc); t.rotation = Quat::from_rotation_y(a * 6.0); }); }
        for _ in 0..3 { let an = cx.rr(0.0, TAU); let rr = cx.rr(0.8, 1.6) * sc * if sing { 2.2 } else { 1.0 }; let o = Vec3::new(an.cos() * rr, cx.rr(-0.8, 0.8) * sc, an.sin() * rr); let v = -o.normalize() * 3.0 * if sing { 2.0 } else { 1.0 };
            let c = hexc(cx.pick(&[0xd08aff, 0x8a5aff, 0xffffff])) * 2.5; cx.fx.emit(true, p.pos + o, v, 0.35, 0.3, 0.05, c, 0.0, 0.0, 1.0); }
        if sing { pull -= cx.dt; if pull <= 0.0 { pull = 0.2; let mut n = 0; for i in 0..cx.g.enemies.len() { let e = &cx.g.enemies[i]; if e.dead || e.t.passive || e.t.prop || n >= 10 { continue; } let (dx, dz) = (p.pos.x - e.body.pos.x, p.pos.z - e.body.pos.z); let d = (dx * dx + dz * dz).sqrt(); if d < 6.0 && d > 0.8 { n += 1; cx.status(i, Status { push: Some(Vec3::new(dx / d * 7.0, 1.5, dz / d * 7.0)), ..default() }); } } } }
    }));
    p.on_hit = Some(Box::new(|cx, p, i| { if let Some(i) = i { let s = p.pos; let d = p.dmg; cx.damage(i, d, Hit { kb: 2.0, src: Some(s), col: 0xd08aff, ..default() }); let e = &cx.g.enemies[i]; let hp = e.body.pos + Vec3::Y * e.body.h * 0.5; cx.fx.burst(hp, 0xd08aff, 14, 4.0, 0.4, 0.3, 0.0, 0.0); } }));
    let bsc = sc * if sing { 1.5 } else { 1.0 };
    p.on_world = Some(Box::new(move |cx, p, _| orb_burst(cx, p.pos, bsc, split)));
    p.on_expire = Some(Box::new(move |cx, p, _| orb_burst(cx, p.pos, bsc, split)));
    spawn_proj(cx, p);
}
/// the orb implodes to a point, then blows apart
fn orb_burst(cx: &mut Ctx, c: Vec3, sc: f32, split: u32) {
    cx.play("arcane");
    let r = 3.5 * sc;
    { let m = cx.gfx().fmat(0xb46aff, 3.0, None, Blend::Add, true); let mesh = cx.pal.sphere.clone(); cx.gfx().spawn_tween(mesh, m, 0.14, c, |t| { t.s0 = Vec3::splat(r * 0.6); t.s1 = Vec3::splat(0.1); t.ease = Ease::Lin; t.alpha = 0.5; t.fade_in = 0.01; t.fade_out = 0.01; }); }
    cx.later(0.14, move |cx| {

        cx.gfx().impact(c, 0xd08aff, 2.0 * sc); cx.punch(0.2 * sc);        cx.play("boom");
        { let mut g = cx.gfx(); g.flare(c, 0xd08aff, 6.0 * sc, 0.35, true); g.sphere(c, 0xb46aff, 0.3, r, 0.35); g.ring(c - Vec3::Y * 0.5, 0xd08aff, 0.5, r, 0.4); g.shock(c - Vec3::Y * 0.8, 0xd08aff, 0.4, r, 0.35, 1.4); }
        cx.fx.burst(c, 0xd08aff, 40, 6.0, 0.5, 0.45, 0.0, 0.0); cx.shake(0.15);
        cx.aoe(c, r, &mut |cx, i, _| cx.damage(i, (20.0 * sc).round(), Hit { kb: 6.0, src: Some(c), col: 0xd08aff, ..default() }));
        if split > 0 { let n = 1 + 2 * split; for i in 0..n { let an = i as f32 / n as f32 * TAU;
            let e = glow_ball(cx, 0xd08aff, 0.3);
            let mut p = Proj::new(c, Vec3::new(an.cos(), 0.1, an.sin()) * 15.0); p.e = Some(e); p.r = 0.55; p.life = 0.75; p.pierce = true; p.orient = Orient::Fixed;
            p.trail = Some(Box::new(|cx, p, _| cx.fx.emit(true, p.pos, Vec3::ZERO, 0.3, 0.4, 0.05, hexc(0xd08aff) * 2.5, 0.0, 0.0, 1.0)));
            p.on_hit = Some(Box::new(|cx, p, i| { if let Some(i) = i { let s = p.pos; cx.damage(i, 12.0, Hit { kb: 3.0, src: Some(s), col: 0xd08aff, ..default() }); } }));
            p.on_expire = Some(Box::new(|cx, p, _| { let s = p.pos; cx.fx.burst(s, 0xd08aff, 14, 4.0, 0.35, 0.3, 0.0, 0.0); cx.aoe(s, 2.0, &mut |cx, i, _| cx.damage(i, 10.0, Hit { kb: 3.0, src: Some(s), col: 0xd08aff, ..default() })); }));
            spawn_proj(cx, p); } }
    });
}

fn meteor(cx: &mut Ctx) -> bool {
    let mut c = clamp_aim(cx, 40.0);
    c.y = cx.ground(c.x, c.z, c.y + 1.0);
    let cata = cx.m("meteor.cata") as f32;
    let r = 5.5 * cx.g.st.area * (1.0 + 0.2 * cata);
    let dmg = (95.0 * (1.0 + 0.4 * cata)).round();
    let inf = cx.m("meteor.inferno") > 0;
    cx.play("arcane"); cx.p.pose(Pose::Raise, 0.9);
    let pp = cx.p.body.pos; cx.gfx().circle(pp, 0xff6a1a, 2.0, 0.9, "fire", 3.0, None, None);
    call_meteor(cx, c, r, dmg, true, inf);
    let sh = 2 * cx.m("meteor.shower");
    for i in 0..sh {
        let an = i as f32 / sh as f32 * TAU + cx.rr(-0.3, 0.3); let d = r * cx.rr(0.8, 1.4);
        let off = Vec3::new(c.x + an.cos() * d, c.y, c.z + an.sin() * d);
        cx.later(0.3 + i as f32 * 0.28, move |cx| { let mut o = off; o.y = cx.ground(o.x, o.z, c.y + 2.0); call_meteor(cx, o, r * 0.55, (dmg * 0.4).round(), false, false); });
    }
    if cx.m("meteor.arma") > 0 {
        cx.later(1.2, move |cx| {
            let mut z = Zone::new(cx.p.body.pos, 0.0, 4.0); z.follow = true; z.every = 0.38;
            z.on_tick = Some(Box::new(move |cx, z| {
                let fs: Vec<usize> = cx.foes(z.pos, 16.0).into_iter().take(8).collect();
                let mut t = if fs.is_empty() { z.pos + Vec3::new(cx.rr(-10.0, 10.0), 0.0, cx.rr(-10.0, 10.0)) } else { let j = cx.pick(&fs); cx.g.enemies[j].body.pos };
                t.y = cx.ground(t.x, t.z, z.pos.y + 3.0);
                call_meteor(cx, t, r * 0.5, (dmg * 0.35).round(), false, false);
            }));
            zone(cx, z);
        });
    }
    true
}
/// a jagged rock of dark basalt split by glowing magma, wrapped in a corona of fire
fn meteor_rock(cx: &mut Ctx, pos: Vec3, s: f32) -> Entity {
    let root = empty(cx, Transform::from_translation(pos));
    let rock = cx.pal.mat(cx.mats, 0x3a2a24, Mk::Lit); let rock2 = cx.pal.mat(cx.mats, 0x52382a, Mk::Lit); let magma = cx.pal.mat(cx.mats, 0xff6a1a, Mk::Glow);
    let ico = cx.pal.ico.clone(); let cube = cx.pal.cube.clone();
    let body = empty(cx, Transform::default()); cx.c.entity(root).add_child(body);
    mesh_part(cx, body, ico.clone(), rock.clone(), Transform::from_scale(Vec3::new(1.15, 0.95, 1.05) * s));
    for k in 0..6 {
        let d = Vec3::new(cx.rr(-1.0, 1.0), cx.rr(-1.0, 1.0), cx.rr(-1.0, 1.0)).normalize() * 0.7 * s;
        let ls = cx.rr(0.35, 0.6) * s;
        mesh_part(cx, body, ico.clone(), if k % 2 == 0 { rock.clone() } else { rock2.clone() }, Transform::from_translation(d).with_scale(Vec3::splat(ls)).with_rotation(Quat::from_euler(EulerRot::XYZ, cx.rr(0.0, 3.0), cx.rr(0.0, 3.0), 0.0)));
    }
    for _ in 0..12 {
        let d = Vec3::new(cx.rr(-1.0, 1.0), cx.rr(-1.0, 1.0), cx.rr(-1.0, 1.0)).normalize();
        let t = Transform::from_translation(d * 0.9 * s).looking_to(d.cross(Vec3::Y).normalize_or(Vec3::X), d).with_scale(Vec3::new(0.14, 0.14, cx.rr(0.6, 1.1)) * s);
        mesh_part(cx, body, cube.clone(), magma.clone(), t);
    }
    // licks of flame streaming off the leading face
    let flame = cx.pal.mat(cx.mats, 0xffa040, Mk::Glow); let cone = cx.fx.cone.clone();
    for _ in 0..5 { let d = Vec3::new(cx.rr(-0.5, 0.5), 1.0, cx.rr(-0.5, 0.5)).normalize(); let t = Transform::from_translation(d * 0.9 * s).with_rotation(Quat::from_rotation_arc(Vec3::Y, d)).with_scale(Vec3::new(0.35, 1.2, 0.35) * s); mesh_part(cx, root, cone.clone(), flame.clone(), t); }
    root
}
fn call_meteor(cx: &mut Ctx, c: Vec3, r: f32, dmg: f32, big: bool, inferno: bool) {
    let s = if big { 1.9 } else { 0.9 };
    let tt = 1.2;
    // the impact point is marked by a fiery sigil that tightens as the rock comes in
    cx.gfx().circle(c, 0xff6a1a, r, tt + 0.1, "fire", -2.0, None, None);
    let (f, rt) = (cx.p.fwd(), cx.p.right()); let from = c + f * 22.0 + rt * 12.0 + Vec3::Y * 55.0;
    let rock = meteor_rock(cx, from, s);
    let dir = (c - from).normalize();
    cx.play("rumble");
    let mut z = Zone::new(c, r, tt);
    z.ents.push(rock);
    z.on_frame = Some(Box::new(move |cx, z| {
        let k = (z.t / tt).min(1.0);
        let p = from.lerp(c, k * k);
        set_tf(cx, rock, |t| { t.translation = p; t.rotation = Quat::from_euler(EulerRot::XYZ, z.t * 5.0, z.t * 2.0, 0.0); });
        // a roaring tail of fire and black smoke
        for _ in 0..if big { 7 } else { 3 } {
            let j = Vec3::new(cx.rr(-0.8, 0.8), cx.rr(-0.8, 0.8), cx.rr(-0.8, 0.8)) * s;
            let v = -dir * cx.rr(4.0, 10.0) + Vec3::new(cx.rr(-1.0, 1.0), cx.rr(0.0, 2.0), cx.rr(-1.0, 1.0));
            let col = hexc(cx.pick(&[0xff6a1a, 0xffb03a, 0xffd27a])) * 1.7;
            cx.fx.emit(true, p + j, v, cx.rr(0.3, 0.5), if big { 1.2 } else { 0.7 }, 0.1, col, -1.0, 1.0, 1.0);
        }
        cx.fx.emit(false, p, -dir * 3.0 + Vec3::Y, 1.6, 0.8 * s, 2.6 * s, hexc(0x2a2420), -0.5, 0.5, 0.6);
    }));
    z.on_end = Some(Box::new(move |cx, _| meteor_impact(cx, c, r, dmg, big, inferno, s)));
    zone(cx, z);
}
fn meteor_impact(cx: &mut Ctx, c: Vec3, r: f32, dmg: f32, big: bool, inferno: bool, s: f32) {
    cx.play("boom"); cx.shake(if big { 1.0 } else { 0.4 }); if big { cx.g.hitstop = 0.08; }
    cx.gfx().impact(c + Vec3::Y * 1.2, 0xffb03a, if big { 5.0 } else { 2.2 }); if big { cx.punch(1.0); }
    if big { let war = cx.fx.tex.circles["war"].clone(); let mut g = cx.gfx(); g.card(c + Vec3::Y * 2.0, 0xffd27a, war, Vec3::splat(2.0), Vec3::splat(r * 2.2), 0.3, 0.0, 0.0, 6.0); }
    cx.fx.burst(c + Vec3::Y * 0.5, 0xff7a22, if big { 90 } else { 35 }, r * 3.0, 0.8, if big { 0.9 } else { 0.6 }, 3.0, 0.0);
    cx.fx.burst(c + Vec3::Y * 0.5, 0xffe0a0, if big { 30 } else { 10 }, r * 2.0, 0.5, 0.6, 0.0, 0.0);
    cx.fx.smoke(c, 0x4a4440, if big { 14 } else { 6 }, 3.5, 1.6, 1.6);
    cx.fx.sparks(c + Vec3::Y * 0.5, 0xffd27a, if big { 24 } else { 8 }, 14.0);
    {
        let mut g = cx.gfx();
        g.decal(c, false, r * if big { 0.95 } else { 0.6 }, if big { 14.0 } else { 7.0 }, None);
        if big { g.decal(c, true, r * 0.8, 10.0, Some(0xff7a2a)); }
        g.flare(c + Vec3::Y, 0xffb070, if big { 14.0 } else { 6.0 }, 0.45, true);
        g.sphere(c + Vec3::Y, 0xff8a3a, 0.5, r * 1.1, 0.4);
        g.ring(c, 0xff8a3a, 0.5, r * 1.5, 0.6); g.ring(c, 0xffffff, 0.5, r, 0.35);
        g.shock(c, 0xff8a3a, 0.5, r * 1.3, 0.5, if big { 3.0 } else { 1.5 });
    }
    // carve a crater; the blasted blocks fly off as tumbling chunks in their own colours
    let cr: f32 = if big { 2.6 } else { 1.4 }; let (bx, by, bz) = (c.x.floor() as i32, c.y.floor() as i32 - 1, c.z.floor() as i32);
    let ri = cr.ceil() as i32;
    for dy in -ri..=ri { for dz in -ri..=ri { for dx in -ri..=ri {
        if (dx * dx + dz * dz) as f32 + (dy * dy) as f32 * 1.5 > cr * cr { continue; }
        let y = by + dy; if y < 2 { continue; }
        let b = cx.w.get(bx + dx, y, bz + dz); if b == 0 { continue; }
        cx.set_block(bx + dx, y, bz + dz, 0);
        if cx.r() < 0.6 {
            let d = crate::world::block_def(b); let col = d.top.unwrap_or(d.c); let sc = Color::linear_rgb(col[0], col[1], col[2]).to_srgba();
            let hex = ((sc.red * 255.0) as u32) << 16 | ((sc.green * 255.0) as u32) << 8 | (sc.blue * 255.0) as u32;
            let an = cx.rr(0.0, TAU); let sp = cx.rr(4.0, 10.0);
            let v = Vec3::new(an.cos() * sp * 0.6, cx.rr(6.0, 13.0), an.sin() * sp * 0.6); let l = cx.rr(1.5, 2.5);
            let sz = cx.rr(0.25, 0.5); cx.gfx().debris_one(Vec3::new((bx + dx) as f32 + 0.5, y as f32 + 0.5, (bz + dz) as f32 + 0.5), v, hex, sz, l);
        }
    } } }
    // the crater floor is scorched to dark stone, with a few embers glowing in it
    if big { for dz in -3..=3i32 { for dx in -3..=3i32 {
        if dx * dx + dz * dz > 9 || crate::noise::hash2(bx + dx, bz + dz, cx.w.seed.wrapping_add(77)) > 0.45 { continue; }
        let t = cx.w.top_at(bx + dx, bz + dz); let b = cx.w.get(bx + dx, t, bz + dz);
        if b != 0 && b != crate::world::b::EMBER { cx.set_block(bx + dx, t, bz + dz, if crate::noise::hash2(bx + dx, bz + dz, 5) < 0.2 { crate::world::b::EMBER } else { crate::world::b::DSTONE }); }
    } } }
    // molten chunks of the meteor itself
    for _ in 0..if big { 14 } else { 5 } { let an = cx.rr(0.0, TAU); let sp = cx.rr(3.0, 9.0); let v = Vec3::new(an.cos() * sp, cx.rr(5.0, 11.0), an.sin() * sp); let col = cx.pick(&[0x3a2a24, 0x52382a, 0xff6a1a]); let sz = cx.rr(0.2, 0.4); let l = cx.rr(1.4, 2.2); cx.gfx().debris_one(c + Vec3::Y * 0.6, v, col, sz, l); }
    cx.aoe(c, r, &mut |cx, i, _| { cx.damage(i, dmg, Hit { kb: if big { 12.0 } else { 6.0 }, up: if big { 10.0 } else { 5.0 }, src: Some(c), col: 0xff8a3a, ..default() }); cx.status(i, Status { burn: 3.0, ..default() }); });
    if big && Vec2::new(cx.p.body.pos.x - c.x, cx.p.body.pos.z - c.z).length() < r * 0.6 { cx.p.body.vel.y = 8.0; }
    // the rock stays half-buried in its crater, smoking and cooling until it crumbles
    let mut rest = c; rest.y = c.y - 0.5 * s;
    let rock = meteor_rock(cx, rest, s * 0.85);
    let life = if big { 5.0 } else { 2.5 };
    let mut z = Zone::new(rest, 0.0, life); z.ents.push(rock);
    z.on_frame = Some(Box::new(move |cx, z| {
        let k = z.t / life;
        if cx.r() < 0.5 * (1.0 - k) { let j = Vec3::new(cx.rr(-0.6, 0.6), 0.5, cx.rr(-0.6, 0.6)) * s; cx.fx.emit(false, z.pos + j, Vec3::new(cx.rr(-0.3, 0.3), cx.rr(1.0, 2.0), cx.rr(-0.3, 0.3)), 2.0, 0.5 * s, 1.8 * s, hexc(0x3a3430), -0.4, 0.4, 0.5); }
        if cx.r() < 0.3 * (1.0 - k) { let j = Vec3::new(cx.rr(-1.0, 1.0), 0.3, cx.rr(-1.0, 1.0)) * s; cx.fx.emit(true, z.pos + j, Vec3::Y * cx.rr(0.5, 1.5), 0.6, 0.15, 0.03, hexc(0xff8a3a) * 3.0, -1.0, 0.5, 1.0); }
    }));
    z.on_end = Some(Box::new(move |cx, z| { cx.gfx().debris(z.pos + Vec3::Y * 0.4, &[0x3a2a24, 0x52382a, 0x2a2420], if big { 16 } else { 6 }, 3.0, 0.3, 4.0); cx.fx.smoke(z.pos, 0x3a3430, 4, 1.0, 1.0, 1.0); }));
    zone(cx, z);
    if inferno {
        let mut z = Zone::new(c, r * 0.8, 5.0); z.every = 0.5;
        z.on_tick = Some(Box::new(|cx, z| { let (p, rr) = (z.pos, z.r); cx.aoe(p, rr, &mut |cx, i, _| { cx.damage(i, 6.0, Hit { quiet: true, col: 0xff7a22, ..default() }); cx.status(i, Status { burn: 2.0, ..default() }); }); }));
        z.on_frame = Some(Box::new(|cx, z| { for _ in 0..5 { let an = cx.rr(0.0, TAU); let rr = cx.r().sqrt() * z.r; let col = hexc(cx.pick(&[0xff6a1a, 0xffb03a, 0xff3a10])) * 2.5; let v = Vec3::new(cx.rr(-0.3, 0.3), cx.rr(1.5, 3.5), cx.rr(-0.3, 0.3)); cx.fx.emit(true, z.pos + Vec3::new(an.cos() * rr, 0.2, an.sin() * rr), v, 0.6, 0.6, 0.08, col, -1.0, 0.5, 1.0); } }));
        zone(cx, z);
    }
}

// ================= WARRIOR =================
fn charge(cx: &mut Ctx) -> bool {
    let d = cx.p.fwd();
    cx.play("dash"); cx.play("shout");
    let tt = 0.38 * (1.0 + 0.5 * cx.m("charge.mom") as f32);
    cx.p.dash = Some(Dash { dir: d, speed: 30.0, t: tt, kind: 2 });
    cx.p.invuln = 0.3; cx.p.pose(Pose::Lunge, tt + 0.1);
    { let pp = cx.p.body.pos + Vec3::Y; let mut g = cx.gfx(); g.speed_lines(pp + d * 3.0, d, 20, 6.0, 1.5, 0xffd0c0, 0.35); g.card(pp + d * 0.8, 0xff6a4a, g.fx.tex.burst.clone(), Vec3::splat(0.5), Vec3::splat(3.0), 0.2, 0.0, 0.0, 6.0); }
    cx.punch(0.3);
    // a spectral ram's wedge of red force rides in front of the warrior
    let wedge = empty(cx, Transform::from_translation(cx.p.body.pos + Vec3::Y));
    let m = cx.pal.mat(cx.mats, 0xff6a4a, Mk::Ghost); let cone = cx.fx.cone.clone(); let tor = cx.pal.torus.clone();
    mesh_part(cx, wedge, cone, m.clone(), Transform::from_xyz(0.0, 0.0, 1.0).with_rotation(Quat::from_rotation_x(FRAC_PI_2)).with_scale(Vec3::new(2.4, 1.6, 2.0)));
    for k in 0..3 { mesh_part(cx, wedge, tor.clone(), m.clone(), Transform::from_xyz(0.0, 0.0, 0.4 - k as f32 * 0.6).with_rotation(Quat::from_rotation_x(FRAC_PI_2)).with_scale(Vec3::splat(1.2 - k as f32 * 0.2))); }
    cx.g.ab.charge = Some(ChargeSt { hit: vec![], drop: 0.0, ruin: cx.m("charge.ruin") > 0, wave: cx.m("charge.wave") > 0, wedge, dust: 0.0 });
    true
}
fn whirl(cx: &mut Ctx) -> bool {
    let life = 2.5 + 1.5 * cx.m("whirl.long") as f32;
    cx.p.whirl = life; cx.play("heavy");
    let r = 4.2 * cx.g.st.area * (1.0 + 0.25 * cx.m("whirl.wide") as f32);
    let (vortex, storm) = (cx.m("whirl.vortex") > 0, cx.m("whirl.blades") > 0);
    let pp = cx.p.body.pos;
    { let mut g = cx.gfx(); for k in 0..6 { let an = k as f32 / 6.0 * TAU; let p = pp + Vec3::new(an.cos() * r * 0.6, 1.0 + (k % 2) as f32 * 0.5, an.sin() * r * 0.6); g.slash(p, 0xfff0d8, r * 0.55, 0.2, an); } }
    // a funnel of wind (three stacked, counter-spinning rings of air) and four ghostly greatswords orbiting in it
    let root = empty(cx, Transform::from_translation(pp));
    let mut layers = vec![];
    for k in 0..3 {
        let m = cx.mats.add(StandardMaterial { base_color: srgb(if vortex { 0xffb070 } else { 0xfff0d8 }).with_alpha(0.45), emissive: lrgb(0xffd0a0, 1.4), unlit: true, alpha_mode: AlphaMode::Add, double_sided: true, cull_mode: None, ..default() });
        let cyl = cx.fx.cyl.clone();
        let sc = r * (0.55 + k as f32 * 0.22);
        layers.push((mesh_part(cx, root, cyl, m, Transform::from_xyz(0.0, k as f32 * 0.7, 0.0).with_scale(Vec3::new(sc, 1.4, sc))), (), k));
    }
    let mut swords = vec![];
    for k in 0..4 {
        let s = empty(cx, Transform::default()); cx.c.entity(root).add_child(s);
        let gm = cx.pal.mat(cx.mats, 0xffe0b0, Mk::Ghost); let cube = cx.pal.cube.clone();
        mesh_part(cx, s, cube.clone(), gm.clone(), Transform::from_xyz(0.0, 0.0, 0.9).with_scale(Vec3::new(0.12, 0.3, 1.8)));
        mesh_part(cx, s, cube.clone(), gm.clone(), Transform::from_xyz(0.0, 0.0, -0.05).with_scale(Vec3::new(0.12, 0.7, 0.12)));
        mesh_part(cx, s, cube, gm, Transform::from_xyz(0.0, 0.0, -0.35).with_scale(Vec3::new(0.1, 0.1, 0.5)));
        swords.push((s, k as f32 / 4.0 * TAU));
    }
    let mut z = Zone::new(pp, r, life); z.follow = true; z.every = 0.25; z.ents.push(root);
    let mut tk = 0u32;
    z.on_tick = Some(Box::new(move |cx, z| {
        cx.play("swing"); tk += 1;
        let c = z.pos;
        let n = cx.aoe(c, r, &mut |cx, i, _| { cx.damage(i, 11.0, Hit { kb: if vortex { 0.0 } else { 3.0 }, src: Some(c), col: 0xffd0a0, quiet: true, ..default() }); if vortex { let e = &cx.g.enemies[i]; let p = Vec3::new((c.x - e.body.pos.x) * 1.6, 0.0, (c.z - e.body.pos.z) * 1.6); cx.status(i, Status { push: Some(p), ..default() }); } });
        if n > 0 && cx.g.st.ww_heal > 0.0 { let h = n.min(6) as f32 * cx.g.st.ww_heal; cx.sustain(h); }
        if storm && tk % 2 == 0 { let off = cx.rr(0.0, TAU); for i in 0..3 { let an = off + i as f32 / 3.0 * TAU; spin_blade(cx, c + Vec3::Y, Vec3::new(an.sin(), 0.0, an.cos()), 14.0, 0xffe0b0); } }
    }));
    z.on_frame = Some(Box::new(move |cx, z| {
        let c = z.pos; let t = z.t; let fade = ((z.life - t) / 0.4).clamp(0.0, 1.0) * (t / 0.2).min(1.0);
        set_tf(cx, root, |tr| { tr.translation = c; tr.scale = Vec3::new(fade.max(0.01), 1.0, fade.max(0.01)); });
        for &(e, _, k) in layers.iter() { let dir = if k % 2 == 0 { 1.0 } else { -1.3 }; set_tf(cx, e, |tr| tr.rotation = Quat::from_rotation_y(t * 9.0 * dir)); }
        for &(s, a0) in swords.iter() { let an = a0 + t * 11.0; let p = Vec3::new(an.cos() * r * 0.75, 1.0 + (t * 4.0 + a0).sin() * 0.2, an.sin() * r * 0.75); set_tf(cx, s, |tr| { tr.translation = p; tr.rotation = Quat::from_rotation_y(-an) * Quat::from_rotation_z(0.3); }); }
        // dust, leaves and grass torn up into the spiral
        let g = 0x8a7a5a;
        for _ in 0..6 { let an = cx.rr(0.0, TAU); let rr = r * cx.rr(0.5, 1.0); let p = c + Vec3::new(an.cos() * rr, cx.rr(0.1, 2.4), an.sin() * rr); let v = Vec3::new(-an.sin() * 9.0, cx.rr(1.0, 3.0), an.cos() * 9.0) - (p - c).with_y(0.0) * 0.8; cx.fx.emit(true, p, v, 0.25, 0.3, 0.05, hexc(0xffe0b0) * 1.6, 0.0, 2.0, 0.7); if cx.r() < 0.5 { cx.fx.emit(false, p, v * 0.6, 0.6, 0.2, 0.8, hexc(g), -1.0, 1.5, 0.5); } }
        if cx.r() < 0.25 { let an = cx.rr(0.0, TAU); let v = Vec3::new(-an.sin() * 7.0, cx.rr(3.0, 6.0), an.cos() * 7.0); let col = cx.pick(&[0x6cb243, 0x5a9638, 0xc8a85a]); cx.gfx().debris_one(c + Vec3::new(an.cos() * r * 0.6, 0.3, an.sin() * r * 0.6), v, col, 0.1, 0.8); }
    }));
    zone(cx, z);
    true
}
fn spin_blade(cx: &mut Ctx, from: Vec3, dir: Vec3, dmg: f32, c: u32) {
    let root = empty(cx, Transform::from_translation(from));
    let m = cx.pal.mat(cx.mats, c, Mk::Glow); let st = cx.pal.mat(cx.mats, 0xd8e2ec, Mk::Lit); let cube = cx.pal.cube.clone();
    mesh_part(cx, root, cube.clone(), st.clone(), Transform::from_scale(Vec3::new(0.8, 0.05, 0.16)));
    mesh_part(cx, root, cube.clone(), st, Transform::from_scale(Vec3::new(0.16, 0.05, 0.8)));
    mesh_part(cx, root, cube, m, Transform::from_scale(Vec3::new(0.22, 0.07, 0.22)));
    let mut p = Proj::new(from, dir * 19.0); p.e = Some(root); p.r = 0.6; p.life = 0.75; p.pierce = true; p.no_world = true; p.orient = Orient::Spin(-24.0);
    p.trail = Some(Box::new(move |cx, p, _| { if cx.r() < 0.6 { cx.fx.emit(true, p.pos, Vec3::ZERO, 0.2, 0.35, 0.05, hexc(c) * 2.0, 0.0, 0.0, 0.8); } }));
    p.on_hit = Some(Box::new(move |cx, p, i| { if let Some(i) = i { let s = p.pos; cx.damage(i, dmg, Hit { kb: 3.0, src: Some(s), col: c, ..default() }); cx.fx.sparks(s, c, 4, 6.0); } }));
    spawn_proj(cx, p);
}

fn slam(cx: &mut Ctx) -> bool {
    let pp = cx.p.body.pos; let f = cx.p.fwd();
    let mut to = cx.p.aim_pt;
    let dd = Vec2::new(to.x - pp.x, to.z - pp.z).length();
    if dd > 13.0 { to = pp + f * 13.0; } if dd < 2.0 { to = pp + f * 3.0; }
    to.y = cx.ground(to.x, to.z, to.y + 2.0);
    if crate::physics::collides(cx.w, to.x, to.y, to.z, 0.3, 1.8) { to.y += 1.0; }
    let r = 6.0 * cx.g.st.area * (1.0 + 0.3 * cx.m("slam.wide") as f32);
    let stun = 1.0 + cx.m("slam.quake") as f32;
    cx.gfx().circle(to, 0xd9a46a, r, 0.6, "war", 2.0, None, None);
    cx.play("heavy"); cx.p.invuln = 0.6;
    cx.p.leap = Some(Leap { from: pp, to, t: 0.0, dur: 0.55, height: 5.0 });
    cx.g.ab.slam = Some(SlamSt { to, r, stun, after: cx.m("slam.after"), fis: cx.m("slam.fissure") > 0 });
    true
}
/// pillars of the ground itself heave up out of the earth, hold, and sink back
pub fn earth_pillars(cx: &mut Ctx, pts: &[(Vec3, f32)], life: f32) {
    if pts.is_empty() { return; }
    let mut ents = vec![];
    for &(p, h) in pts {
        let hex = ground_hex(cx, p);
        let top = cx.pal.mat(cx.mats, hex, Mk::Lit); let dirt = cx.pal.mat(cx.mats, 0x7a5a3a, Mk::Lit); let stone = cx.pal.mat(cx.mats, 0x8c9097, Mk::Lit);
        let w = cx.rr(0.8, 1.2);
        let tilt = Quat::from_euler(EulerRot::XYZ, cx.rr(-0.25, 0.25), cx.rr(0.0, TAU), cx.rr(-0.25, 0.25));
        let root = empty(cx, Transform::from_translation(p - Vec3::Y * h).with_rotation(tilt));
        let cube = cx.pal.cube.clone();
        // grass cap, dirt band, stone core: a real chunk of the ground
        let e1 = cx.c.spawn((Mesh3d(cube.clone()), MeshMaterial3d(stone), Transform::from_xyz(0.0, h * 0.4, 0.0).with_scale(Vec3::new(w, h * 0.8, w)))).id();
        let e2 = cx.c.spawn((Mesh3d(cube.clone()), MeshMaterial3d(dirt), Transform::from_xyz(0.0, h * 0.86, 0.0).with_scale(Vec3::new(w * 1.02, h * 0.14, w * 1.02)))).id();
        let e3 = cx.c.spawn((Mesh3d(cube), MeshMaterial3d(top), Transform::from_xyz(0.0, h * 0.97, 0.0).with_scale(Vec3::new(w * 1.04, h * 0.08, w * 1.04)))).id();
        cx.c.entity(root).add_children(&[e1, e2, e3]);
        ents.push((root, p, h, tilt));
        kick_dirt(cx, p, 3, 4.0);
    }
    let mut z = Zone::new(pts[0].0, 0.0, life);
    z.ents = ents.iter().map(|e| e.0).collect();
    z.on_frame = Some(Box::new(move |cx, z| {
        let t = z.t;
        let rise = 1.0 - (1.0 - (t / 0.12).min(1.0)).powi(3);
        let sink = ((z.life - t) / 0.45).clamp(0.0, 1.0);
        let k = rise.min(sink);
        for &(e, p, h, rot) in ents.iter() { let y = p.y - h + h * k + if t < 0.2 { (t * 60.0).sin() * 0.05 } else { 0.0 }; set_tf(cx, e, |tr| { tr.translation = Vec3::new(p.x, y, p.z); tr.rotation = rot; }); }
    }));
    zone(cx, z);
}
fn slam_land(cx: &mut Ctx, s: SlamSt) {
    let (to, r) = (s.to, s.r);

    cx.gfx().impact(to + Vec3::Y * 0.8, 0xffd0a0, 3.5); cx.punch(0.8);
    { let war = cx.fx.tex.circles["war"].clone(); let mut g = cx.gfx(); g.card(to + Vec3::Y * 1.0, 0xffe0b0, war, Vec3::splat(1.5), Vec3::splat(r * 1.8), 0.28, 0.0, 0.0, 6.0); }    cx.play("boom"); cx.shake(1.0); cx.g.hitstop = 0.08;
    {
        let mut g = cx.gfx();
        g.decal(to, true, r * 0.85, 9.0, Some(0xff9a4a)); g.flare(to + Vec3::Y * 0.8, 0xffd0a0, 8.0, 0.35, true);
        g.ring(to, 0xffd0a0, 0.5, r, 0.5); g.shock(to, 0xffd0a0, 0.6, r, 0.45, 2.0);
    }
    cx.fx.smoke(to, 0xb8a888, 16, 3.0, 1.2, 1.4);
    // a ring of earth pillars and a jagged inner crown
    let mut pts = vec![];
    let n = (r * 2.2) as usize;
    for i in 0..n { let an = i as f32 / n as f32 * TAU + cx.rr(-0.1, 0.1); let rr = r * cx.rr(0.55, 0.8); let mut p = Vec3::new(to.x + an.cos() * rr, 0.0, to.z + an.sin() * rr); p.y = cx.ground(p.x, p.z, to.y + 2.0); pts.push((p, cx.rr(1.6, 2.8))); }
    for i in 0..6 { let an = i as f32 / 6.0 * TAU + 0.5; let rr = r * 0.3; let mut p = Vec3::new(to.x + an.cos() * rr, 0.0, to.z + an.sin() * rr); p.y = cx.ground(p.x, p.z, to.y + 2.0); pts.push((p, cx.rr(0.8, 1.4))); }
    earth_pillars(cx, &pts, 1.6);
    let g = ground_hex(cx, to);
    for _ in 0..26 { let an = cx.rr(0.0, TAU); let sp = cx.rr(3.0, 9.0); let v = Vec3::new(an.cos() * sp, cx.rr(5.0, 11.0), an.sin() * sp); let col = cx.pick(&[g, 0x8a5b34, 0x8c9097]); let sz = cx.rr(0.2, 0.45); let l = cx.rr(1.0, 2.0); cx.gfx().debris_one(to + Vec3::new(an.cos(), 0.3, an.sin()), v, col, sz, l); }
    cx.aoe(to, r, &mut |cx, i, _| { cx.damage(i, 42.0, Hit { kb: 9.0, up: 8.0, src: Some(to), col: 0xffd0a0, ..default() }); cx.status(i, Status { stun: s.stun, ..default() }); });
    for i in 1..=s.after {
        cx.later(0.45 * i as f32, move |cx| {
            let rr = r * (1.0 + 0.25 * i as f32);
            cx.play("heavy"); cx.shake(0.4);
            { let mut g = cx.gfx(); g.ring(to, 0xffb070, 0.5, rr, 0.45); g.shock(to, 0xffb070, 0.5, rr, 0.4, 1.2); }
            cx.fx.smoke(to, 0xb8a888, 8, 2.5, 1.0, 1.2);
            let mut pts = vec![]; let n = (rr * 1.6) as usize;
            for k in 0..n { let an = k as f32 / n as f32 * TAU + i as f32; let mut p = Vec3::new(to.x + an.cos() * rr * 0.9, 0.0, to.z + an.sin() * rr * 0.9); p.y = cx.ground(p.x, p.z, to.y + 2.0); pts.push((p, cx.rr(0.8, 1.5))); }
            earth_pillars(cx, &pts, 0.9);
            cx.aoe(to, rr, &mut |cx, j, _| { cx.damage(j, 25.0, Hit { kb: 6.0, up: 5.0, src: Some(to), col: 0xffb070, ..default() }); cx.status(j, Status { stun: 0.6, ..default() }); });
        });
    }
    if s.fis {
        let off = cx.rr(0.0, TAU);
        for j in 0..6 { let an = off + j as f32 / 6.0 * TAU; let (dx, dz) = (an.cos(), an.sin());
            for k in 1..=5 { cx.later(0.07 * k as f32, move |cx| {
                let mut p = Vec3::new(to.x + dx * (r * 0.5 + k as f32 * 1.7), 0.0, to.z + dz * (r * 0.5 + k as f32 * 1.7)); p.y = cx.ground(p.x, p.z, to.y + 3.0);
                cx.play("heavy"); cx.fx.smoke(p, 0x8a7458, 3, 1.5, 0.7, 0.9); cx.fx.burst(p + Vec3::Y * 0.3, 0xff9a4a, 8, 4.0, 0.35, 0.35, 4.0, 0.0);
                let h = cx.rr(1.4, 2.4); earth_pillars(cx, &[(p, h)], 0.9);
                cx.gfx().decal(p, true, 1.4, 5.0, Some(0xff9a4a));
                cx.aoe(p, 1.7, &mut |cx, i, _| { cx.damage(i, 22.0, Hit { kb: 2.0, up: 7.0, src: Some(p), col: 0xffb070, quiet: true }); cx.status(i, Status { stun: 0.4, ..default() }); });
            }); }
        }
    }
}

fn cry(cx: &mut Ctx) -> bool {
    let rage = cx.m("cry.rage") as f32; let rally = cx.m("cry.rally") > 0;
    let dur = 8.0 + 2.0 * rage + if rally { 1.0 } else { 0.0 };
    cx.g.buffs.insert("cry", dur);
    cx.g.flags.insert("cry_leech", 0.1 + 0.1 * rage);
    cx.heal(if rally { 24.0 } else { 20.0 }, false);
    cx.play("shout"); cx.shake(0.4); cx.p.pose(Pose::Shout, 0.8);
    { let pp = cx.p.body.pos + Vec3::Y * 1.6; let war = cx.fx.tex.circles["war"].clone(); let lines = cx.fx.tex.burst.clone(); let mut g = cx.gfx(); g.card(pp, 0xff5b4c, war, Vec3::splat(1.0), Vec3::splat(5.0), 0.4, 0.0, 0.0, 6.0); g.card(pp, 0xffd0a0, lines, Vec3::splat(2.0), Vec3::splat(9.0), 0.35, 0.3, 0.0, 5.0); }
    cx.punch(0.5);
    let pp = cx.p.body.pos;
    // the roar: rings of sound you can see, rolling out at head height
    for k in 0..4 {
        cx.later(0.08 * k as f32, move |cx| {
            let m = cx.gfx().fmat(if k % 2 == 0 { 0xff5b4c } else { 0xffb03a }, 3.0, None, Blend::Add, true); let mesh = cx.pal.torus.clone();
            cx.gfx().spawn_tween(mesh, m, 0.6, pp + Vec3::Y * 1.6, |t| { t.s0 = Vec3::new(0.5, 3.0, 0.5); t.s1 = Vec3::new(9.0, 6.0, 9.0); t.fade_in = 0.02; t.fade_out = 0.7; });
        });
    }
    { let mut g = cx.gfx(); g.shock(pp, 0xff5b4c, 0.5, 9.0, 0.5, 2.4); g.circle(pp, 0xff5b4c, 4.0, 1.0, "war", 3.0, None, None); }
    cx.fx.burst(pp + Vec3::Y * 1.2, 0xff5b4c, 40, 7.0, 0.6, 0.4, 2.0, 0.0);
    // four war banners crash down around the warrior and stand for the length of the cry
    for k in 0..4 {
        let an = cx.p.yaw + PI / 4.0 + k as f32 * FRAC_PI_2;
        let mut p = pp + Vec3::new(an.sin() * 3.2, 0.0, an.cos() * 3.2); p.y = cx.ground(p.x, p.z, pp.y + 2.0);
        cx.later(0.05 * k as f32, move |cx| banner(cx, p, an, dur));
    }
    let terror = cx.m("cry.terror") > 0; let wl = cx.m("cry.warlord") > 0;
    let rr = if terror { 11.0 } else { 9.0 } * if wl { 1.5 } else { 1.0 };
    cx.aoe(pp, rr, &mut |cx, i, _| cx.status(i, if terror { Status { fear: 3.0, ..default() } } else { Status { stun: 0.8, fear: 1.5, ..default() } }));
    true
}
fn banner(cx: &mut Ctx, p: Vec3, yaw: f32, dur: f32) {
    let mut b = cx.builder();
    let root = b.root(p + Vec3::Y * 7.0);
    let v = Vec3::new;
    b.bx(root, v(0.12, 3.4, 0.12), 0x5a3a22, v(0.0, 1.5, 0.0));
    b.bx(root, v(1.2, 0.1, 0.1), 0x5a3a22, v(0.0, 3.0, 0.0));
    b.bx(root, v(0.18, 0.3, 0.18), 0xe0b84a, v(0.0, 3.3, 0.0));
    let cloth = b.pivot(root, v(0.0, 2.95, 0.0));
    b.bx(cloth, v(1.1, 1.6, 0.05), 0xa8282a, v(0.0, -0.8, 0.0));
    b.bx(cloth, v(1.1, 0.12, 0.06), 0xe0b84a, v(0.0, -0.06, 0.0)); b.bx(cloth, v(1.1, 0.12, 0.06), 0xe0b84a, v(0.0, -1.55, 0.0));
    for (x, y) in [(-0.35, -1.75), (0.0, -1.85), (0.35, -1.75)] { b.bx(cloth, v(0.3, 0.3, 0.05), 0xa8282a, v(x, y, 0.0)); }
    b.glow(cloth, v(0.42, 0.42, 0.07), 0xffb03a, v(0.0, -0.8, 0.0));
    b.rot(root, 0.0, yaw, 0.0);
    let mut z = Zone::new(p, 0.0, dur);
    z.ents.push(root);
    let mut landed = false;
    z.on_frame = Some(Box::new(move |cx, z| {
        let t = z.t;
        let fall = (t / 0.18).min(1.0); let sink = ((z.life - t) / 0.5).clamp(0.0, 1.0);
        let y = p.y + 7.0 * (1.0 - fall * fall) - (1.0 - sink) * 3.6;
        set_tf(cx, root, |tr| { tr.translation = Vec3::new(p.x, y, p.z); tr.rotation = Quat::from_rotation_y(yaw); });
        set_tf(cx, cloth, |tr| tr.rotation = Quat::from_euler(EulerRot::XYZ, (t * 3.0 + yaw).sin() * 0.12, 0.0, (t * 2.3).sin() * 0.05));
        if fall >= 1.0 && !landed { landed = true; cx.play("heavy"); kick_dirt(cx, p, 6, 3.0); cx.fx.smoke(p, 0xb8a888, 4, 1.2, 0.8, 0.8); cx.gfx().ring(p, 0xff5b4c, 0.3, 2.0, 0.4); }
        if cx.r() < 0.15 { let j = Vec3::new(cx.rr(-0.5, 0.5), cx.rr(1.5, 3.0), cx.rr(-0.5, 0.5)); cx.fx.emit(true, z.pos + j, Vec3::Y * 1.2, 0.8, 0.2, 0.04, hexc(0xff7a3a) * 2.5, -0.5, 0.0, 1.0); }
    }));
    zone(cx, z);
}

// ================= PRIEST =================
fn mend(cx: &mut Ctx) -> bool {
    let amt = cx.p.max_hp * 0.25 * (1.0 + 0.15 * cx.m("mend.greater") as f32);
    let hot = 4.0 * (1.0 + cx.m("mend.renew") as f32);
    cx.heal(amt, false); cx.g.buffs.insert("hot", hot);
    cx.play("heal"); cx.p.pose(Pose::Pray, 0.9);
    let pp = cx.p.body.pos;
    cx.gfx().pillar(pp, 0x8dff9e, 1.1, 6.0, 0.9);
    { let mut g = cx.gfx(); for k in 0..10 { let an = k as f32 * 2.4; let rr = g.fx.rr(0.6, 1.6); let gp = pp + Vec3::new(an.cos() * rr, 0.4 + k as f32 * 0.25, an.sin() * rr); g.glint(gp, if k % 3 == 0 { 0xffffff } else { 0x8dff9e }, g.fx.rr(0.5, 1.0), 0.6 + k as f32 * 0.04); } }
    for i in 0..30 { let an = i as f32 / 30.0 * TAU * 3.0; let v = Vec3::new(-an.sin() * 1.5, 2.5, an.cos() * 1.5); let c = hexc(cx.pick(&[0x8dff9e, 0xffffff, 0xffe48a])) * 2.5; cx.fx.emit(true, pp + Vec3::new(an.cos() * 1.1, i as f32 * 0.06, an.sin() * 1.1), v, 0.9, 0.4, 0.05, c, 0.0, 0.5, 1.0); }
    let bloom = cx.m("mend.bloom") > 0;
    lotus(cx, pp, if bloom { 2.2 } else { 1.4 }, if bloom { 6.0 } else { 1.8 }, bloom);
    if cx.g.st.radiance > 0.0 || cx.m("mend.prayer") > 0 {
        { let mut g = cx.gfx(); g.ring(pp, 0xffe48a, 0.5, 6.0, 0.4); g.shock(pp, 0xffe48a, 0.5, 6.0, 0.4, 1.6); }
        cx.aoe(pp, 6.0, &mut |cx, i, _| cx.damage(i, 40.0, Hit { kb: 4.0, src: Some(pp), col: 0xffe48a, ..default() }));
    }
    true
}
/// a lotus of light: eight outer and six inner petals unfold from the ground, glow, then close and sink
fn lotus(cx: &mut Ctx, pos: Vec3, s: f32, life: f32, bloom: bool) {
    let mut c = pos; c.y = cx.ground(c.x, c.z, c.y + 1.0);
    let root = empty(cx, Transform::from_translation(c + Vec3::Y * 0.05).with_scale(Vec3::splat(s)));
    let outer = cx.pal.mat(cx.mats, 0x8dff9e, Mk::Ghost); let inner = cx.pal.mat(cx.mats, 0xffffff, Mk::Ghost); let core = cx.pal.mat(cx.mats, 0xffe48a, Mk::Glow);
    let cube = cx.pal.cube.clone();
    let mut hinges = vec![];
    for (ring, n, len, m, off) in [(0, 8, 0.95, outer, 0.0), (1, 6, 0.65, inner, 0.4)] {
        for i in 0..n {
            let yaw = i as f32 / n as f32 * TAU + off;
            let yp = empty(cx, Transform::from_rotation(Quat::from_rotation_y(yaw))); cx.c.entity(root).add_child(yp);
            let hinge = empty(cx, Transform::from_xyz(0.0, 0.0, 0.15)); cx.c.entity(yp).add_child(hinge);
            mesh_part(cx, hinge, cube.clone(), m.clone(), Transform::from_xyz(0.0, 0.0, len * 0.5).with_scale(Vec3::new(0.34 * len * 1.3, 0.03, len)));
            mesh_part(cx, hinge, cube.clone(), m.clone(), Transform::from_xyz(0.0, 0.0, len * 0.5).with_scale(Vec3::new(0.18 * len, 0.05, len * 0.85)));
            hinges.push((hinge, ring));
        }
    }
    mesh_part(cx, root, cube, core, Transform::from_xyz(0.0, 0.15, 0.0).with_scale(Vec3::splat(0.22)).with_rotation(Quat::from_euler(EulerRot::XYZ, 0.6, 0.6, 0.0)));
    let mut z = Zone::new(c, 5.0 * cx.g.st.area, life); z.ents.push(root);
    if bloom { z.every = 0.5; }
    z.on_frame = Some(Box::new(move |cx, z| {
        let t = z.t;
        let open = (t / 0.5).min(1.0); let close = ((z.life - t) / 0.45).clamp(0.0, 1.0);
        let k = (1.0 - (1.0 - open).powi(3)) * close;
        for &(h, ring) in hinges.iter() { let a = crate::noise::lerp(-1.35, if ring == 0 { -0.25 } else { -0.7 }, k); set_tf(cx, h, |tr| tr.rotation = Quat::from_rotation_x(a)); }
        set_tf(cx, root, |tr| tr.rotation = Quat::from_rotation_y(t * 0.4));
        if cx.r() < 0.5 { let an = cx.rr(0.0, TAU); let rr = cx.rr(0.2, 1.2) * s; let col = hexc(cx.pick(&[0x8dff9e, 0xffb8e0, 0xffffff])) * 2.5; cx.fx.emit(true, z.pos + Vec3::new(an.cos() * rr, 0.3, an.sin() * rr), Vec3::new(0.0, cx.rr(0.8, 2.0), 0.0), 1.1, 0.3, 0.05, col, 0.0, 0.5, 0.9); }
    }));
    if bloom {
        z.on_tick = Some(Box::new(|cx, z| {
            let (p, r) = (z.pos, z.r);
            cx.aoe(p, r, &mut |cx, i, _| cx.damage(i, 9.0, Hit { src: Some(p), col: 0xffe48a, quiet: true, ..default() }));
            if !cx.p.dead && Vec2::new(cx.p.body.pos.x - p.x, cx.p.body.pos.z - p.z).length() < r { cx.sustain(cx.p.max_hp * 0.012); }
        }));
        let r = z.r; cx.gfx().circle(c, 0x8dff9e, r, life, "nature", 0.5, None, None);
    }
    zone(cx, z);
}

fn smite(cx: &mut Ctx) -> bool {
    let mut c = clamp_aim(cx, 38.0);
    c.y = cx.ground(c.x, c.z, c.y + 1.0);
    let r = 2.9 * cx.g.st.area;
    let dmg = (48.0 * (1.0 + 0.4 * cx.m("smite.wrath") as f32)).round();
    cx.play("holy"); cx.p.cast_kick = 1.0; cx.p.pose(Pose::Point, 0.6);
    holy_sword(cx, c, r, dmg, 1.0);
    if cx.m("smite.judg") > 0 { for i in 1..=2 { cx.later(0.5 * i as f32, move |cx| holy_sword(cx, c, r * (1.0 + 0.15 * i as f32), (dmg * 0.7).round(), 0.85)); } }
    let chain = 2 * cx.m("smite.chain") as usize;
    if chain > 0 {
        cx.later(0.45, move |cx| {
            let mut cand: Vec<(f32, Vec3)> = cx.g.enemies.iter().filter(|e| !e.dead && !e.t.passive).map(|e| (Vec2::new(e.body.pos.x - c.x, e.body.pos.z - c.z).length(), e.body.pos)).filter(|(d, _)| *d < 12.0 && *d > r).collect();
            cand.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            for (i, (_, p)) in cand.into_iter().take(chain).enumerate() { cx.later(0.12 * i as f32, move |cx| { let mut g = p; g.y = cx.ground(g.x, g.z, g.y + 1.0); holy_sword(cx, g, r * 0.8, (dmg * 0.7).round(), 0.7); }); }
        });
    }
    true
}
/// a colossal sword of light falls point-first out of the sky and drives into the ground
fn holy_sword(cx: &mut Ctx, c: Vec3, r: f32, dmg: f32, s: f32) {
    cx.gfx().circle(c, 0xffe48a, r * 1.2, 0.9, "holy", 2.0, None, None);
    let mut b = cx.builder(); b.shadow = false;
    let root = b.root(c + Vec3::Y * 34.0);
    let v = Vec3::new;
    let (gold, white, edge) = (0xffd35a, 0xfff8e0, 0xffe48a);
    // built tip-down: the origin is the point of the blade
    b.glow(root, v(0.5, 7.0, 0.14), white, v(0.0, 3.6, 0.0));
    b.glow(root, v(0.26, 7.2, 0.18), edge, v(0.0, 3.6, 0.0));
    let tip = b.glow(root, v(0.36, 0.36, 0.12), white, v(0.0, 0.1, 0.0)); b.rot(tip, 0.0, 0.0, PI / 4.0);
    b.glow(root, v(2.6, 0.32, 0.34), gold, v(0.0, 7.25, 0.0));
    for x in [-1.35, 1.35] { b.glow(root, v(0.36, 0.5, 0.36), gold, v(x, 7.35, 0.0)); }
    b.glow(root, v(0.22, 1.5, 0.22), 0xc89a3a, v(0.0, 8.15, 0.0));
    b.glow(root, v(0.44, 0.44, 0.44), gold, v(0.0, 9.05, 0.0));
    b.c.entity(root).insert(Transform::from_translation(c + Vec3::Y * 34.0).with_scale(Vec3::splat(s)));
    let fall = 0.32; let hold = 0.9;
    let mut z = Zone::new(c, r, fall + hold);
    z.ents.push(root);
    let mut hit = false;
    z.on_frame = Some(Box::new(move |cx, z| {
        let t = z.t;
        if t < fall {
            let k = t / fall; let y = c.y + 34.0 * (1.0 - k * k) - 1.6 * s * k * k;
            set_tf(cx, root, |tr| tr.translation = Vec3::new(c.x, y, c.z));
            let p = Vec3::new(c.x, y + 8.0 * s, c.z); cx.fx.emit(true, p, Vec3::Y * 6.0, 0.3, 0.9, 0.1, hexc(0xffe48a) * 2.0, 0.0, 0.0, 0.8);
        } else {
            if !hit { hit = true; smite_impact(cx, c, r, dmg); }
            // embedded: it quivers, then thins away into light
            let k = ((t - fall) / hold).min(1.0); let w = (1.0 - ((k - 0.6) / 0.4).max(0.0)).max(0.02);
            let q = if k < 0.25 { ((t - fall) * 70.0).sin() * 0.03 * (1.0 - k * 4.0) } else { 0.0 };
            set_tf(cx, root, |tr| { tr.translation = Vec3::new(c.x, c.y - 1.6 * s, c.z); tr.rotation = Quat::from_rotation_z(q); tr.scale = Vec3::new(w * s, s, w * s); });
            if k > 0.6 && cx.r() < 0.8 { let h = cx.rr(0.0, 8.0) * s; cx.fx.emit(true, Vec3::new(c.x, c.y - 1.6 * s + h, c.z), Vec3::new(cx.rr(-0.5, 0.5), cx.rr(1.0, 3.0), cx.rr(-0.5, 0.5)), 0.6, 0.3, 0.05, hexc(0xfff3b8) * 2.5, 0.0, 0.5, 1.0); }
        }
    }));
    zone(cx, z);
}
fn smite_impact(cx: &mut Ctx, c: Vec3, r: f32, dmg: f32) {
    cx.play("boom"); cx.shake(0.35);
    cx.gfx().impact(c + Vec3::Y * 1.0, 0xfff3b8, 2.6); cx.punch(0.45);
    { let holy = cx.fx.tex.circles["holy"].clone(); let mut g = cx.gfx(); g.card(c + Vec3::Y * 1.2, 0xffe48a, holy, Vec3::splat(1.0), Vec3::splat(r * 2.0), 0.35, 0.0, 0.0, 6.0); }
    { let mut g = cx.gfx(); g.flare(c + Vec3::Y * 1.5, 0xfff3b8, 7.0, 0.4, true); g.decal(c, true, r * 0.9, 4.0, Some(0xffe48a)); g.ring(c, 0xffe48a, 0.4, r * 1.6, 0.45); g.shock(c, 0xffe48a, 0.4, r * 1.4, 0.35, 1.4); }
    cx.fx.burst(c + Vec3::Y * 0.5, 0xffe48a, 40, 7.0, 0.6, 0.4, 3.0, 0.0); cx.fx.sparks(c + Vec3::Y * 0.5, 0xffffff, 12, 11.0);
    kick_dirt(cx, c, 10, 6.0);
    cx.aoe(c, r, &mut |cx, i, _| { cx.damage(i, dmg, Hit { kb: 3.0, up: 5.0, src: Some(c), col: 0xffe48a, ..default() }); cx.status(i, Status { stun: 0.9, ..default() }); });
}

fn shield(cx: &mut Ctx) -> bool {
    let dur = 3.0 + 1.5 * cx.m("shield.long") as f32;
    cx.g.buffs.insert("shield", dur);
    cx.play("holy"); cx.p.pose(Pose::Raise, 0.6);
    let pp = cx.p.body.pos;
    { let mut g = cx.gfx(); g.ring(pp, 0xfff3b8, 0.5, 5.0, 0.5); g.shock(pp, 0xfff3b8, 0.5, 5.0, 0.4, 2.0); }
    { let mut g = cx.gfx(); for k in 0..8 { let an = k as f32 / 8.0 * TAU; g.glint(pp + Vec3::new(an.cos() * 1.6, 1.0 + (k % 3) as f32 * 0.6, an.sin() * 1.6), 0xfff3b8, 0.9, 0.5); } g.impact(pp + Vec3::Y * 1.2, 0xfff3b8, 1.6); }
    cx.punch(0.25);
    cx.aoe(pp, 4.5, &mut |cx, i, _| { let e = &cx.g.enemies[i]; let p = Vec3::new((e.body.pos.x - pp.x) * 3.0, 5.0, (e.body.pos.z - pp.z) * 3.0); cx.status(i, Status { push: Some(p), ..default() }); });
    // a golden dome, and six kite shields of light circling the priest
    let root = empty(cx, Transform::from_translation(pp));
    let dome_m = cx.mats.add(StandardMaterial { base_color: srgb(0xffe48a).with_alpha(0.16), emissive: lrgb(0xffe48a, 0.5), unlit: true, alpha_mode: AlphaMode::Add, double_sided: true, cull_mode: None, ..default() });
    let sph = cx.pal.sphere.clone();
    let dome = mesh_part(cx, root, sph, dome_m, Transform::from_xyz(0.0, 0.9, 0.0).with_scale(Vec3::new(1.7, 1.5, 1.7)));
    let mut shields = vec![];
    for k in 0..6 {
        let s = empty(cx, Transform::default()); cx.c.entity(root).add_child(s);
        let mut b = cx.builder(); b.shadow = false;
        let v = Vec3::new;
        b.glow(s, v(0.7, 0.75, 0.06), 0xffd35a, v(0.0, 0.1, 0.0));
        let low = b.glow(s, v(0.5, 0.5, 0.06), 0xffd35a, v(0.0, -0.35, 0.0)); b.rot(low, 0.0, 0.0, PI / 4.0);
        b.glow(s, v(0.1, 0.8, 0.08), 0xffffff, v(0.0, 0.0, 0.0)); b.glow(s, v(0.5, 0.1, 0.08), 0xffffff, v(0.0, 0.15, 0.0));
        shields.push((s, k as f32 / 6.0 * TAU));
    }
    let aegis = cx.m("shield.aegis") > 0; let thorns = cx.m("shield.thorns") > 0; let retri = cx.m("shield.retri") > 0;
    let mut z = Zone::new(pp, 4.5, dur); z.follow = true; z.ents.push(root);
    if thorns || aegis { z.every = 0.5; }
    z.on_frame = Some(Box::new(move |cx, z| {
        let t = z.t; let c = z.pos;
        let k = (t / 0.25).min(1.0) * ((z.life - t) / 0.3).clamp(0.0, 1.0);
        set_tf(cx, root, |tr| { tr.translation = c; tr.scale = Vec3::splat(k.max(0.01)); });
        set_tf(cx, dome, |tr| tr.rotation = Quat::from_rotation_y(t * 0.6));
        for &(s, a0) in shields.iter() { let an = a0 + t * 2.4; set_tf(cx, s, |tr| { tr.translation = Vec3::new(an.sin() * 1.8, 1.0 + (t * 3.0 + a0).sin() * 0.15, an.cos() * 1.8); tr.rotation = Quat::from_rotation_y(an); }); }
        if cx.r() < 0.4 { let an = cx.rr(0.0, TAU); cx.fx.emit(true, c + Vec3::new(an.cos() * 1.7, cx.rr(0.2, 1.8), an.sin() * 1.7), Vec3::new(an.cos(), 0.5, an.sin()) * 2.0, 0.35, 0.25, 0.04, hexc(0xfff3b8) * 2.5, 0.0, 1.0, 1.0); }
    }));
    z.on_tick = Some(Box::new(move |cx, z| {
        let c = z.pos;
        if thorns { cx.gfx().ring(c, 0xfff3b8, 0.5, 4.5, 0.3); cx.aoe(c, 4.5, &mut |cx, i, _| cx.damage(i, 14.0, Hit { kb: 4.0, src: Some(c), col: 0xfff3b8, quiet: true, ..default() })); }
        if aegis { cx.aoe(c, 2.4, &mut |cx, i, _| { cx.damage(i, 10.0, Hit { kb: 7.0, src: Some(c), col: 0xffd35a, quiet: true, ..default() }); }); }
    }));
    if retri { z.on_end = Some(Box::new(|cx, z| {
        let c = z.pos; cx.play("boom");
        { let mut g = cx.gfx(); g.ring(c, 0xffe48a, 0.5, 6.5, 0.5); g.shock(c, 0xffe48a, 0.5, 6.5, 0.45, 2.2); g.pillar(c, 0xffe48a, 1.5, 12.0, 0.6); }
        cx.fx.burst(c + Vec3::Y, 0xffe48a, 50, 8.0, 0.6, 0.4, 0.0, 0.0);
        for k in 0..6 { let an = k as f32 / 6.0 * TAU; spin_blade(cx, c + Vec3::Y, Vec3::new(an.sin(), 0.0, an.cos()), 20.0, 0xffd35a); }
        cx.aoe(c, 6.5, &mut |cx, i, _| { cx.damage(i, 50.0, Hit { kb: 8.0, up: 5.0, src: Some(c), col: 0xffe48a, ..default() }); cx.status(i, Status { stun: 0.8, ..default() }); });
    })); }
    zone(cx, z);
    true
}

fn consecrate(cx: &mut Ctx) -> bool {
    let march = cx.m("consecrate.march") > 0;
    let c = cx.p.body.pos;
    let r = 6.0 * cx.g.st.area * (1.0 + 0.25 * cx.m("consecrate.wide") as f32);
    let dmg = (9.0 * (1.0 + 0.5 * cx.m("consecrate.purge") as f32)).round();
    let sanct = cx.m("consecrate.sanct") > 0;
    let life = 6.0 + if march { 3.0 } else { 0.0 };
    cx.play("holy"); cx.p.pose(Pose::Kneel, 0.8);
    { let holy = cx.fx.tex.circles["holy"].clone(); let mut g = cx.gfx(); g.card(c + Vec3::Y * 1.5, 0xffd35a, holy, Vec3::splat(1.0), Vec3::splat(r * 1.4), 0.4, 0.0, 0.6, 5.0); g.impact(c + Vec3::Y * 1.0, 0xffd35a, 1.8); }
    { let mut g = cx.gfx(); g.ring(c, 0xffd35a, 0.5, r, 0.5); g.shock(c, 0xffd35a, 0.5, r, 0.4, 1.4); }
    let follow = if march { cx.p.rig.root } else { None };
    cx.gfx().circle(c, 0xffd35a, r, life, "holy", 0.4, None, follow);
    // eight white obelisks with burning gold runes rise around the edge of the holy ground
    let root = empty(cx, Transform::from_translation(c));
    let mut obs = vec![];
    for k in 0..8 {
        let an = k as f32 / 8.0 * TAU;
        let o = empty(cx, Transform::from_xyz(an.cos() * r, -3.0, an.sin() * r)); cx.c.entity(root).add_child(o);
        let mut b = cx.builder();
        let v = Vec3::new;
        b.bx(o, v(0.6, 0.3, 0.6), 0xcfc6b2, v(0.0, 0.15, 0.0));
        b.bx(o, v(0.42, 2.0, 0.42), 0xe8e0cc, v(0.0, 1.3, 0.0));
        let cap = b.bx(o, v(0.3, 0.3, 0.3), 0xe8e0cc, v(0.0, 2.4, 0.0)); b.rot(cap, PI / 4.0, 0.0, PI / 4.0);
        for y in [0.8, 1.3, 1.8] { b.glow(o, v(0.44, 0.1, 0.2), 0xffd35a, v(0.0, y, 0.0)); }
        b.glow(o, v(0.16, 0.16, 0.16), 0xfff3b8, v(0.0, 2.85, 0.0));
        b.rot(o, 0.0, -an, 0.0);
        obs.push((o, an));
    }
    let mut z = Zone::new(c, r, life); z.every = 0.5; z.follow = march; z.ents.push(root);
    z.on_tick = Some(Box::new(move |cx, z| {
        let (p, rr) = (z.pos, z.r);
        cx.aoe(p, rr, &mut |cx, i, _| cx.damage(i, dmg, Hit { src: Some(p), col: 0xffd35a, quiet: true, ..default() }));
        if Vec2::new(cx.p.body.pos.x - p.x, cx.p.body.pos.z - p.z).length() < rr { let h = cx.p.max_hp * 0.012 * if sanct { 1.6 } else { 1.0 }; cx.sustain(h); }
    }));
    z.on_frame = Some(Box::new(move |cx, z| {
        let t = z.t; let p = z.pos;
        let k = (1.0 - (1.0 - (t / 0.5).min(1.0)).powi(3)) * ((z.life - t) / 0.5).clamp(0.0, 1.0);
        set_tf(cx, root, |tr| tr.translation = p);
        for &(o, an) in obs.iter() { let y = -3.0 + 3.0 * k + (t * 2.0 + an).sin() * 0.06; set_tf(cx, o, |tr| { tr.translation = Vec3::new(an.cos() * r, y, an.sin() * r); }); }
        for _ in 0..4 { let an = cx.rr(0.0, TAU); let rr = cx.r().sqrt() * r; let col = hexc(cx.pick(&[0xffd35a, 0xffffff, 0xffb03a])) * 2.5; cx.fx.emit(true, p + Vec3::new(an.cos() * rr, 0.2, an.sin() * rr), Vec3::Y * cx.rr(1.0, 3.0), 0.9, 0.35, 0.05, col, 0.0, 0.5, 0.9); }
        if cx.r() < 0.3 { let (o, an) = obs[(cx.r() * 8.0) as usize % 8]; let _ = o; let top = p + Vec3::new(an.cos() * r, 2.9 * k, an.sin() * r); cx.fx.emit(true, top, Vec3::Y * 1.5, 0.8, 0.4, 0.05, hexc(0xfff3b8) * 3.0, 0.0, 0.5, 1.0); }
    }));
    zone(cx, z);
    true
}

/// per-frame work for abilities in motion: charge and roll contact, slam landings, the Battle Cry aura
pub fn update_abilities(cx: &mut Ctx) {
    let dt = cx.dt;
    let pp = cx.p.body.pos;
    // ---- charge
    if let Some(mut st) = cx.g.ab.charge.take() {
        let d = cx.p.dash.as_ref().map(|d| d.dir);
        set_tf(cx, st.wedge, |t| { t.translation = pp + Vec3::Y * 1.0 + d.unwrap_or(Vec3::Z) * 0.6; if let Some(d) = d { t.rotation = Quat::from_rotation_y(d.x.atan2(d.z)); } });
        if let Some(d) = d {
            let cands: Vec<usize> = (0..cx.g.enemies.len()).filter(|&i| { let e = &cx.g.enemies[i]; !e.dead && !st.hit.contains(&e.id) && Vec2::new(e.body.pos.x - pp.x, e.body.pos.z - pp.z).length() < 1.6 + e.body.w && (e.body.pos.y - pp.y).abs() < 2.5 }).collect();
            for i in cands {
                st.hit.push(cx.g.enemies[i].id);
                let ep = cx.g.enemies[i].body.pos;
                cx.damage(i, 26.0, Hit { kb: 10.0, src: Some(pp), col: 0xff8a6a, ..default() });
                cx.status(i, Status { stun: 1.3, ..default() });
                cx.fx.burst(ep + Vec3::Y, 0xffd0a0, 16, 5.0, 0.4, 0.3, 0.0, 0.0); cx.fx.sparks(ep + Vec3::Y, 0xffd0a0, 8, 10.0);
                cx.g.hitstop = 0.05;
            }
            for _ in 0..2 { let j = Vec3::new(cx.rr(-0.4, 0.4), cx.rr(0.3, 1.6), cx.rr(-0.4, 0.4)); cx.fx.emit(true, pp + j, -d * 6.0, 0.3, 0.4, 0.05, hexc(0xff8a6a) * 2.0, 0.0, 2.0, 0.9); }
            // the ground tears up behind the charge
            st.dust -= dt;
            if st.dust <= 0.0 { st.dust = 0.05; kick_dirt(cx, pp, 2, 3.0); cx.fx.smoke(pp, 0xb8a888, 1, 1.0, 0.6, 0.8); let g = Vec3::new(pp.x, cx.ground(pp.x, pp.z, pp.y + 1.0), pp.z); cx.gfx().decal(g, true, 0.8, 3.0, None); }
            if st.ruin { st.drop -= dt; if st.drop <= 0.0 { st.drop = 0.07; let g = Vec3::new(pp.x, cx.ground(pp.x, pp.z, pp.y + 1.0), pp.z); fire_pool(cx, g, 1.5, 4.0, 6.0, false); } }
            cx.g.ab.charge = Some(st);
        } else {
            cx.c.entity(st.wedge).despawn_recursive();
            { let mut g = cx.gfx(); g.ring(pp, 0xff8a6a, 0.3, 3.0, 0.3); g.shock(pp, 0xff8a6a, 0.3, 3.0, 0.3, 1.0); }
            cx.shake(0.2);
            if st.wave { let c = pp; cx.play("boom"); { let mut g = cx.gfx(); g.ring(c, 0xffd0a0, 0.5, 4.5, 0.45); g.shock(c, 0xffd0a0, 0.5, 4.5, 0.4, 1.6); } cx.fx.smoke(c, 0xb8a888, 10, 2.5, 1.0, 1.2); cx.aoe(c, 4.5, &mut |cx, i, _| { cx.damage(i, 30.0, Hit { kb: 8.0, up: 6.0, src: Some(c), ..default() }); cx.status(i, Status { stun: 1.0, ..default() }); }); }
            if st.ruin { explode_fire(cx, pp + Vec3::Y * 0.6, 4.5, 40.0, 0, false); cx.gfx().pillar(pp, 0xff7a2a, 1.6, 10.0, 0.6); }
        }
    }
    // ---- roll (Blade Dance)
    if let Some(mut st) = cx.g.ab.roll.take() {
        if cx.p.dash.is_some() {
            if cx.r() < 0.5 { cx.fx.smoke(pp, 0xd8d0c0, 1, 0.8, 0.5, 0.6); }
            if st.dance {
                let cands: Vec<usize> = (0..cx.g.enemies.len()).filter(|&i| { let e = &cx.g.enemies[i]; !e.dead && !e.t.passive && !st.hit.contains(&e.id) && Vec2::new(e.body.pos.x - pp.x, e.body.pos.z - pp.z).length() < 1.6 + e.body.w && (e.body.pos.y - pp.y).abs() < 2.2 }).collect();
                for i in cands {
                    st.hit.push(cx.g.enemies[i].id); let ep = cx.g.enemies[i].body.pos;
                    cx.damage(i, 35.0, Hit { kb: 5.0, src: Some(pp), col: 0xd8f0ff, ..default() });
                    let yaw = (ep.x - pp.x).atan2(ep.z - pp.z); let tilt = cx.rr(-0.5, 0.5);
                    cx.gfx().swoosh(pp + Vec3::Y, yaw, 2.2, 2.4, 0xd8f0ff, 0.18, tilt, 1.0); cx.fx.sparks(ep + Vec3::Y, 0xd8f0ff, 6, 8.0);
                    if let Some(k) = class_abilities(cx.p.cls).iter().position(|&a| a == "roll") { cx.g.cds[k] = (cx.g.cds[k] - 0.4).max(0.0); }
                }
            }
            cx.g.ab.roll = Some(st);
        }
    }
    // ---- slam landing
    if let Some(s) = cx.g.ab.slam { if cx.p.leap.is_none() { cx.g.ab.slam = None; slam_land(cx, s); } }
    // ---- Battle Cry: the warrior burns with a red aura while it lasts
    if cx.g.buff("cry") > 0.0 && cx.r() < 0.6 { let an = cx.rr(0.0, TAU); let p = pp + Vec3::new(an.cos() * 0.5, cx.rr(0.2, 1.8), an.sin() * 0.5); let c = hexc(cx.pick(&[0xff5b4c, 0xffb03a])) * 2.5; cx.fx.emit(true, p, Vec3::Y * cx.rr(1.0, 2.5), 0.4, 0.3, 0.04, c, -1.0, 0.5, 1.0); }
    if cx.g.buff("hot") > 0.0 { let a = cx.g.flags.entry("hot_acc").or_insert(0.0); *a += dt; if *a >= 1.0 { *a -= 1.0; let h = cx.p.max_hp * 0.025; cx.sustain(h); } }
}
