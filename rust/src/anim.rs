//! Keyframed attack animations for every weapon. Each track runs over the attack's duration (0..1): a wind-up,
//! a fast strike timed to land when the hit registers (about a quarter of the way in), a follow-through and a
//! recovery back into the weapon's stance. Melee weapons chain three different attacks in a combo, casters
//! alternate two casts, and the bow has a draw (driven by the charge) and a release.
//!
//! Joint angles are XYZ Euler angles in the parent's frame. For an arm hanging straight down, x < 0 raises it
//! forward, z swings it out to the side (positive for the right arm), and y rolls it about its own length.
//! Elbows bend with negative x. Weapons lie along the hand's +z, so hand angles aim the weapon.
use bevy::prelude::*;

#[derive(Clone, Copy, Default, Debug)]
pub struct Pose { pub ar: Vec3, pub er: f32, pub hr: Vec3, pub al: Vec3, pub el: f32, pub hl: Vec3, pub torso: Vec3, pub step: f32 }
impl Pose {
    fn lerp(&self, o: &Pose, t: f32) -> Pose {
        let l = |a: f32, b: f32| a + (b - a) * t;
        Pose { ar: self.ar.lerp(o.ar, t), er: l(self.er, o.er), hr: self.hr.lerp(o.hr, t), al: self.al.lerp(o.al, t), el: l(self.el, o.el), hl: self.hl.lerp(o.hl, t), torso: self.torso.lerp(o.torso, t), step: l(self.step, o.step) }
    }
}
fn v(x: f32, y: f32, z: f32) -> Vec3 { Vec3::new(x, y, z) }
/// right arm, right elbow, right hand, left arm, left elbow, left hand, torso, forward step
#[allow(clippy::too_many_arguments)]
fn p(ar: Vec3, er: f32, hr: Vec3, al: Vec3, el: f32, hl: Vec3, torso: Vec3, step: f32) -> Pose { Pose { ar, er, hr, al, el, hl, torso, step } }

/// the weapon's ready stance, blended over the walk cycle when not attacking
pub fn stance(id: &str) -> Pose {
    match id {
        "greatsword" => p(v(-0.55, 0.0, -0.3), -1.0, v(0.75, 0.0, 0.0), v(-0.6, 0.0, 0.35), -1.0, Vec3::ZERO, v(0.06, 0.0, 0.0), 0.0),
        "swordshield" => p(v(-0.45, 0.0, 0.12), -0.75, v(0.55, 0.0, 0.0), v(-1.0, 0.0, -0.25), -1.25, Vec3::ZERO, Vec3::ZERO, 0.0),
        "daggers" => p(v(-0.55, 0.0, 0.12), -1.1, v(0.9, 0.0, 0.0), v(-0.55, 0.0, -0.12), -1.1, v(0.9, 0.0, 0.0), v(0.08, 0.0, 0.0), 0.0),
        "mace" => p(v(-0.35, 0.0, 0.15), -0.9, v(0.65, 0.0, 0.0), v(-0.15, 0.0, -0.12), -0.35, Vec3::ZERO, Vec3::ZERO, 0.0),
        "firestaff" => p(v(-0.3, 0.0, 0.14), -1.2, Vec3::ZERO, v(-0.15, 0.0, -0.12), -0.4, Vec3::ZERO, Vec3::ZERO, 0.0),
        "frostwand" => p(v(-0.4, 0.0, 0.12), -1.0, v(0.6, 0.0, 0.0), v(-0.2, 0.0, -0.12), -0.5, Vec3::ZERO, Vec3::ZERO, 0.0),
        "scepter" => p(v(-0.35, 0.0, 0.12), -1.0, v(0.4, 0.0, 0.0), v(-0.15, 0.0, -0.12), -0.4, Vec3::ZERO, Vec3::ZERO, 0.0),
        // bow carried low in the left hand, limbs upright
        "longbow" => p(v(-0.12, 0.0, 0.1), -0.35, Vec3::ZERO, v(-0.3, 0.0, -0.18), -0.45, v(-1.4, 0.0, 0.0), Vec3::ZERO, 0.0),
        _ => Pose::default(),
    }
}

/// (time, pose) keys for one attack; the first and last keys are the stance
fn track(id: &str, combo: u8) -> Vec<(f32, Pose)> {
    let s = stance(id);
    let k = |t: f32, ar: Vec3, er: f32, hr: Vec3, al: Vec3, el: f32, hl: Vec3, to: Vec3, st: f32| (t, p(ar, er, hr, al, el, hl, to, st));
    let z = Vec3::ZERO;
    let mut keys = vec![(0.0, s)];
    match (id, combo % 3) {
        // two-handed: a diagonal cut from high right to low left, a rising backswing, then an overhead slam
        ("greatsword", 0) => keys.extend([
            k(0.2, v(-2.5, 0.0, 0.55), -0.6, v(0.9, 0.0, 0.0), v(-2.4, 0.0, 0.1), -0.8, z, v(-0.15, 0.6, 0.0), -0.05),
            k(0.34, v(-1.3, 0.0, -0.6), -0.2, v(1.4, 0.0, 0.0), v(-1.2, 0.0, 0.0), -0.3, z, v(0.25, -0.55, 0.0), 0.2),
            k(0.6, v(-0.45, 0.0, -0.9), -0.3, v(1.5, 0.0, 0.0), v(-0.5, 0.0, -0.4), -0.4, z, v(0.3, -0.8, 0.0), 0.15)]),
        ("greatsword", 1) => keys.extend([
            k(0.2, v(-0.5, 0.0, -0.9), -0.3, v(1.5, 0.0, 0.0), v(-0.6, 0.0, -0.4), -0.4, z, v(0.2, -0.7, 0.0), 0.0),
            k(0.34, v(-1.9, 0.0, 0.7), -0.4, v(1.0, 0.0, 0.0), v(-1.8, 0.0, 0.3), -0.5, z, v(-0.1, 0.6, 0.0), 0.18),
            k(0.6, v(-2.3, 0.0, 0.9), -0.6, v(0.9, 0.0, 0.0), v(-2.2, 0.0, 0.4), -0.7, z, v(-0.15, 0.8, 0.0), 0.1)]),
        ("greatsword", _) => keys.extend([
            k(0.22, v(-3.0, 0.0, 0.1), -1.1, v(0.6, 0.0, 0.0), v(-3.0, 0.0, -0.1), -1.1, z, v(-0.3, 0.0, 0.0), -0.08),
            k(0.34, v(-0.9, 0.0, -0.1), -0.1, v(1.3, 0.0, 0.0), v(-0.9, 0.0, 0.1), -0.1, z, v(0.5, 0.0, 0.0), 0.25),
            k(0.62, v(-0.7, 0.0, -0.1), -0.1, v(1.3, 0.0, 0.0), v(-0.7, 0.0, 0.1), -0.1, z, v(0.45, 0.0, 0.0), 0.22)]),
        // sword and board: forehand, backhand, then a lunging thrust; the shield stays up
        ("swordshield", 0) => keys.extend([
            k(0.18, v(-1.3, 0.0, 1.2), -0.4, v(1.4, 0.0, 0.0), v(-1.1, 0.0, -0.3), -1.3, z, v(0.0, 0.5, 0.0), 0.0),
            k(0.32, v(-1.4, 0.0, -0.5), -0.2, v(1.5, 0.0, 0.0), v(-0.9, 0.0, -0.2), -1.2, z, v(0.1, -0.5, 0.0), 0.15),
            k(0.58, v(-1.1, 0.0, -0.9), -0.4, v(1.4, 0.0, 0.0), v(-0.95, 0.0, -0.2), -1.2, z, v(0.1, -0.7, 0.0), 0.1)]),
        ("swordshield", 1) => keys.extend([
            k(0.18, v(-1.4, 0.0, -0.8), -1.2, v(1.5, 0.0, 0.0), v(-1.0, 0.0, -0.3), -1.3, z, v(0.05, -0.6, 0.0), 0.0),
            k(0.32, v(-1.4, 0.0, 0.6), -0.3, v(1.5, 0.0, 0.0), v(-0.9, 0.0, -0.2), -1.2, z, v(0.05, 0.5, 0.0), 0.12),
            k(0.58, v(-1.2, 0.0, 1.1), -0.4, v(1.4, 0.0, 0.0), v(-0.95, 0.0, -0.2), -1.2, z, v(0.0, 0.6, 0.0), 0.08)]),
        ("swordshield", _) => keys.extend([
            k(0.2, v(-1.0, 0.0, 0.3), -1.9, v(1.6, 0.0, 0.0), v(-1.1, 0.0, -0.3), -1.3, z, v(-0.05, 0.4, 0.0), -0.1),
            k(0.32, v(-1.6, 0.0, 0.0), 0.0, v(1.55, 0.0, 0.0), v(-0.8, 0.0, -0.2), -1.0, z, v(0.2, -0.3, 0.0), 0.32),
            k(0.6, v(-1.5, 0.0, 0.0), -0.1, v(1.55, 0.0, 0.0), v(-0.85, 0.0, -0.2), -1.1, z, v(0.18, -0.25, 0.0), 0.28)]),
        // daggers: a right slash, a left slash, then a scissor cut with both
        ("daggers", 0) => keys.extend([
            k(0.16, v(-1.6, 0.0, 1.0), -0.8, v(1.2, 0.0, 0.0), s.al, s.el, s.hl, v(0.0, 0.35, 0.0), 0.0),
            k(0.3, v(-1.3, 0.0, -0.7), -0.2, v(1.4, 0.0, 0.0), s.al, s.el, s.hl, v(0.1, -0.4, 0.0), 0.12),
            k(0.55, v(-1.0, 0.0, -0.9), -0.4, v(1.3, 0.0, 0.0), s.al, s.el, s.hl, v(0.1, -0.45, 0.0), 0.1)]),
        ("daggers", 1) => keys.extend([
            k(0.16, s.ar, s.er, s.hr, v(-1.6, 0.0, -1.0), -0.8, v(1.2, 0.0, 0.0), v(0.0, -0.35, 0.0), 0.0),
            k(0.3, s.ar, s.er, s.hr, v(-1.3, 0.0, 0.7), -0.2, v(1.4, 0.0, 0.0), v(0.1, 0.4, 0.0), 0.12),
            k(0.55, s.ar, s.er, s.hr, v(-1.0, 0.0, 0.9), -0.4, v(1.3, 0.0, 0.0), v(0.1, 0.45, 0.0), 0.1)]),
        ("daggers", _) => keys.extend([
            k(0.18, v(-1.5, 0.0, 1.1), -0.5, v(1.3, 0.0, 0.0), v(-1.5, 0.0, -1.1), -0.5, v(1.3, 0.0, 0.0), v(-0.1, 0.0, 0.0), -0.05),
            k(0.32, v(-1.4, 0.0, -0.6), -0.15, v(1.5, 0.0, 0.0), v(-1.4, 0.0, 0.6), -0.15, v(1.5, 0.0, 0.0), v(0.25, 0.0, 0.0), 0.22),
            k(0.58, v(-1.1, 0.0, -0.7), -0.3, v(1.4, 0.0, 0.0), v(-1.1, 0.0, 0.7), -0.3, v(1.4, 0.0, 0.0), v(0.2, 0.0, 0.0), 0.18)]),
        // mace: overhead smash, side swing, uppercut
        ("mace", 0) => keys.extend([
            k(0.22, v(-2.9, 0.0, 0.3), -1.4, v(0.4, 0.0, 0.0), v(-0.5, 0.0, -0.3), -0.6, z, v(-0.2, 0.25, 0.0), -0.05),
            k(0.34, v(-1.0, 0.0, -0.1), -0.1, v(1.2, 0.0, 0.0), v(-0.2, 0.0, -0.3), -0.3, z, v(0.4, -0.1, 0.0), 0.2),
            k(0.62, v(-0.6, 0.0, -0.1), -0.2, v(1.2, 0.0, 0.0), v(-0.2, 0.0, -0.2), -0.3, z, v(0.35, -0.1, 0.0), 0.16)]),
        ("mace", 1) => keys.extend([
            k(0.2, v(-1.2, 0.0, 1.3), -0.9, v(1.3, 0.0, 0.0), v(-0.4, 0.0, -0.4), -0.5, z, v(0.0, 0.6, 0.0), 0.0),
            k(0.33, v(-1.3, 0.0, -0.4), -0.2, v(1.4, 0.0, 0.0), v(-0.3, 0.0, -0.2), -0.4, z, v(0.1, -0.5, 0.0), 0.15),
            k(0.6, v(-1.0, 0.0, -0.8), -0.3, v(1.3, 0.0, 0.0), v(-0.3, 0.0, -0.2), -0.4, z, v(0.1, -0.65, 0.0), 0.1)]),
        ("mace", _) => keys.extend([
            k(0.2, v(-0.2, 0.0, 0.3), -0.2, v(1.6, 0.0, 0.0), v(-0.4, 0.0, -0.3), -0.5, z, v(0.25, 0.3, 0.0), -0.04),
            k(0.33, v(-2.4, 0.0, 0.0), -0.6, v(0.8, 0.0, 0.0), v(-0.3, 0.0, -0.3), -0.4, z, v(-0.25, -0.2, 0.0), 0.15),
            k(0.6, v(-2.6, 0.0, 0.1), -0.7, v(0.7, 0.0, 0.0), v(-0.3, 0.0, -0.3), -0.4, z, v(-0.2, -0.15, 0.0), 0.1)]),
        // staff: pull back and thrust the head at the target with the off hand pushing too; then an overhead cast
        ("firestaff", 0) | ("firestaff", 2) => keys.extend([
            k(0.18, v(-0.6, 0.0, 0.2), -1.6, z, v(-0.8, 0.0, -0.3), -0.8, z, v(-0.1, 0.3, 0.0), -0.05),
            k(0.3, v(-1.5, 0.0, 0.05), -0.3, z, v(-1.4, 0.0, -0.1), -0.2, z, v(0.15, -0.2, 0.0), 0.12),
            k(0.6, v(-1.3, 0.0, 0.05), -0.5, z, v(-1.0, 0.0, -0.15), -0.5, z, v(0.1, -0.1, 0.0), 0.08)]),
        ("firestaff", _) => keys.extend([
            k(0.2, v(-2.8, 0.0, 0.2), -0.8, z, v(-0.6, 0.0, -0.5), -0.6, z, v(-0.2, 0.0, 0.0), -0.04),
            k(0.32, v(-1.3, 0.0, 0.0), -0.4, z, v(-1.3, 0.0, -0.2), -0.2, z, v(0.2, 0.0, 0.0), 0.1),
            k(0.6, v(-1.0, 0.0, 0.05), -0.6, z, v(-0.8, 0.0, -0.2), -0.5, z, v(0.12, 0.0, 0.0), 0.06)]),
        // wand: a snapping flick of the wrist, alternating with an off-hand push
        ("frostwand", 0) | ("frostwand", 2) => keys.extend([
            k(0.18, v(-1.6, 0.0, 0.4), -1.4, v(-0.4, 0.0, 0.0), s.al, s.el, s.hl, v(0.0, 0.2, 0.0), 0.0),
            k(0.3, v(-1.5, 0.0, -0.05), -0.1, v(0.3, 0.0, 0.0), s.al, s.el, s.hl, v(0.05, -0.15, 0.0), 0.06),
            k(0.55, v(-1.3, 0.0, 0.0), -0.3, v(0.4, 0.0, 0.0), s.al, s.el, s.hl, v(0.04, -0.1, 0.0), 0.04)]),
        ("frostwand", _) => keys.extend([
            k(0.18, v(-1.2, 0.0, 0.3), -1.2, v(0.0, 0.0, 0.0), v(-0.9, 0.0, -0.4), -1.2, z, v(0.0, -0.2, 0.0), 0.0),
            k(0.3, v(-1.5, 0.0, 0.0), -0.2, v(0.3, 0.0, 0.0), v(-1.5, 0.0, -0.05), -0.05, z, v(0.06, 0.1, 0.0), 0.06),
            k(0.55, v(-1.2, 0.0, 0.05), -0.4, v(0.4, 0.0, 0.0), v(-1.2, 0.0, -0.1), -0.3, z, v(0.04, 0.05, 0.0), 0.04)]),
        // scepter: raise it overhead, then bring it down to point at the target
        ("scepter", _) => keys.extend([
            k(0.18, v(-2.6, 0.0, 0.2), -0.5, v(0.3, 0.0, 0.0), v(-0.4, 0.0, -0.3), -0.6, z, v(-0.15, 0.1, 0.0), -0.03),
            k(0.3, v(-1.6, 0.0, 0.0), 0.0, v(0.2, 0.0, 0.0), v(-0.6, 0.0, -0.3), -0.8, z, v(0.1, -0.1, 0.0), 0.08),
            k(0.6, v(-1.4, 0.0, 0.05), -0.2, v(0.3, 0.0, 0.0), v(-0.4, 0.0, -0.25), -0.6, z, v(0.06, -0.05, 0.0), 0.05)]),
        _ => {}
    }
    keys.push((1.0, s));
    keys
}

fn ease(t: f32) -> f32 { t * t * t * (t * (t * 6.0 - 15.0) + 10.0) }

/// the pose `t` (0..1) of the way through attack `combo` with weapon `id`
pub fn attack(id: &str, combo: u8, t: f32) -> Pose {
    let keys = track(id, combo);
    let t = t.clamp(0.0, 1.0);
    for w in keys.windows(2) {
        let ((t0, a), (t1, b)) = (w[0], w[1]);
        if t <= t1 { return a.lerp(&b, ease(((t - t0) / (t1 - t0).max(1e-4)).clamp(0.0, 1.0))); }
    }
    keys.last().unwrap().1
}

/// the bow: the left arm raises the bow at the target as soon as the draw starts; the right upper arm swings out
/// and back as the charge builds while the forearm folds to bring the string hand to the cheek
pub fn bow_draw(charge: f32, pitch: f32) -> Pose {
    let c = charge.clamp(0.0, 1.0);
    let aim = -std::f32::consts::FRAC_PI_2 - pitch * 0.85;
    let mut q = stance("longbow");
    q.al = v(aim, 0.0, 0.0); q.el = 0.0; q.hl = Vec3::ZERO;
    q.ar = v(-pitch * 0.85, -1.35 + c * 1.6, std::f32::consts::FRAC_PI_2);
    q.er = -0.25 - c * 2.3;
    q.hr = Vec3::ZERO;
    q.torso = v(0.0, -0.15 - c * 0.2, 0.0);
    q
}
/// the moment after the release: the string hand flies back open, the bow arm kicks, then back to stance
pub fn bow_release(t: f32, pitch: f32) -> Pose {
    let full = bow_draw(1.0, pitch);
    let mut kick = full; kick.ar.y += 0.5; kick.er = -1.3; kick.al.x -= 0.18; kick.torso.y -= 0.08;
    let s = stance("longbow");
    if t < 0.18 { full.lerp(&kick, ease(t / 0.18)) } else if t < 0.5 { kick } else { kick.lerp(&s, ease((t - 0.5) / 0.5)) }
}
