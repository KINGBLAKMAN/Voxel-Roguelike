//! Procedural sound: every effect is synthesised into an in-memory WAV at startup (oscillators with pitch
//! sweeps and envelopes, filtered noise), so the game ships as a single exe with no audio files.
use bevy::audio::{AudioPlayer, AudioSink, AudioSinkPlayback, AudioSource, PlaybackSettings, Volume};
use bevy::prelude::*;
use std::collections::HashMap;

const SR: f32 = 22050.0;

#[derive(Clone, Copy)]
enum Wave { Sine, Tri, Saw, Square }
struct Synth { buf: Vec<f32>, seed: u32, fmul: f32 }
impl Synth {
    fn new(len: f32) -> Self { Synth { buf: vec![0.0; (len * SR) as usize + 1], seed: 0x1234567, fmul: 1.0 } }
    fn noise(&mut self) -> f32 { self.seed ^= self.seed << 13; self.seed ^= self.seed >> 17; self.seed ^= self.seed << 5; (self.seed as f32 / u32::MAX as f32) * 2.0 - 1.0 }
    /// oscillator with an exponential pitch sweep and a quick-attack exponential decay
    fn tone(&mut self, f0: f32, f1: f32, dur: f32, w: Wave, vol: f32, delay: f32) {
        let (s0, n) = ((delay * SR) as usize, (dur * SR) as usize);
        let mut ph = 0.0f32;
        let (f0, f1) = (f0 * self.fmul, f1 * self.fmul);
        for i in 0..n {
            let t = i as f32 / n as f32;
            let f = f0 * (f1 / f0).powf(t);
            ph += f / SR;
            let x = ph.fract();
            let v = match w { Wave::Sine => (x * std::f32::consts::TAU).sin(), Wave::Tri => 4.0 * (x - 0.5).abs() - 1.0, Wave::Saw => 2.0 * x - 1.0, Wave::Square => if x < 0.5 { 1.0 } else { -1.0 } };
            let env = (i as f32 / (0.01 * SR)).min(1.0) * (1.0 - t).powf(1.6);
            if let Some(b) = self.buf.get_mut(s0 + i) { *b += v * env * vol; }
        }
    }
    /// noise through a swept one-pole filter: kind 0 lowpass, 1 highpass, 2 band
    fn hiss(&mut self, dur: f32, f0: f32, f1: f32, vol: f32, kind: u8, delay: f32) {
        let (s0, n) = ((delay * SR) as usize, (dur * SR) as usize);
        let (mut lp, mut lp2) = (0.0f32, 0.0f32);
        let (f0, f1) = (f0 * self.fmul, f1 * self.fmul);
        for i in 0..n {
            let t = i as f32 / n as f32;
            let f = f0 * (f1 / f0).powf(t);
            let a = (1.0 - (-std::f32::consts::TAU * f / SR).exp()).clamp(0.0, 1.0);
            let x = self.noise();
            lp += a * (x - lp); lp2 += a * (lp - lp2);
            let v = match kind { 0 => lp2 * 1.8, 1 => x - lp, _ => (lp - lp2) * 3.0 };
            let env = (i as f32 / (0.004 * SR)).min(1.0) * (1.0 - t).powf(1.4);
            if let Some(b) = self.buf.get_mut(s0 + i) { *b += v * env * vol; }
        }
    }
    fn wav(self) -> Vec<u8> {
        let peak = self.buf.iter().fold(0.0f32, |m, v| m.max(v.abs())).max(1.0);
        let data: Vec<i16> = self.buf.iter().map(|v| ((v / peak) * 0.9 * 32767.0) as i16).collect();
        let mut out = Vec::with_capacity(44 + data.len() * 2);
        let dl = (data.len() * 2) as u32;
        out.extend_from_slice(b"RIFF"); out.extend_from_slice(&(36 + dl).to_le_bytes()); out.extend_from_slice(b"WAVEfmt ");
        out.extend_from_slice(&16u32.to_le_bytes()); out.extend_from_slice(&1u16.to_le_bytes()); out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&(SR as u32).to_le_bytes()); out.extend_from_slice(&((SR as u32) * 2).to_le_bytes()); out.extend_from_slice(&2u16.to_le_bytes()); out.extend_from_slice(&16u16.to_le_bytes());
        out.extend_from_slice(b"data"); out.extend_from_slice(&dl.to_le_bytes());
        for s in data { out.extend_from_slice(&s.to_le_bytes()); }
        out
    }
}

/// synthesise one sound; `var` makes slightly different takes (pitch and noise) of the frequent ones
fn make(name: &str, var: u32) -> (Vec<u8>, f32) {
    use Wave::*;
    let mut s = Synth::new(1.6);
    s.fmul = [1.0, 1.07, 0.94][var as usize % 3]; s.seed = 0x1234567 + var * 7919;
    let vol = match name {
        "swing" => { s.hiss(0.18, 1800.0, 600.0, 1.0, 2, 0.0); 0.35 }
        "heavy" => { s.hiss(0.32, 900.0, 250.0, 1.0, 2, 0.0); s.tone(90.0, 50.0, 0.2, Sine, 0.4, 0.0); 0.45 }
        // soft, short impacts: a lowpassed thud, never a buzz
        "hit" => { s.tone(160.0, 75.0, 0.07, Tri, 0.6, 0.0); s.hiss(0.06, 1400.0, 700.0, 0.6, 0, 0.0); 0.24 }
        "crit" => { s.tone(700.0, 1100.0, 0.07, Tri, 0.35, 0.0); s.hiss(0.08, 3000.0, 1800.0, 0.5, 2, 0.0); s.tone(140.0, 70.0, 0.08, Sine, 0.5, 0.0); 0.28 }
        // the ranger's bow: a soft thwip of the string
        "bow" => { s.tone(260.0, 120.0, 0.1, Tri, 0.6, 0.0); s.hiss(0.08, 2500.0, 1500.0, 0.3, 2, 0.0); 0.3 }
        // casters' primaries: a glassy wand tick, a warm staff puff, a single soft scepter bell
        "wand" => { s.tone(1600.0, 1200.0, 0.05, Sine, 0.4, 0.0); s.hiss(0.05, 6000.0, 4000.0, 0.15, 1, 0.0); 0.14 }
        "staff" => { s.hiss(0.18, 500.0, 1400.0, 0.8, 0, 0.0); s.tone(180.0, 120.0, 0.12, Sine, 0.4, 0.0); 0.22 }
        "scepter" => { s.tone(988.0, 988.0, 0.22, Sine, 0.35, 0.0); s.tone(1976.0, 1976.0, 0.12, Sine, 0.08, 0.0); 0.16 }
        "fire" => { s.hiss(0.35, 700.0, 2400.0, 1.0, 0, 0.0); 0.4 }
        "boom" => { s.hiss(0.9, 1200.0, 80.0, 1.0, 0, 0.0); s.tone(90.0, 35.0, 0.6, Sine, 0.8, 0.0); 0.75 }
        "ice" => { s.tone(1100.0, 1500.0, 0.12, Sine, 0.45, 0.0); s.hiss(0.1, 4000.0, 2500.0, 0.25, 2, 0.02); 0.24 }
        "arcane" => { s.tone(440.0, 660.0, 0.22, Tri, 0.3, 0.0); s.tone(660.0, 440.0, 0.25, Sine, 0.4, 0.0); 0.3 }
        "holy" => { s.tone(880.0, 880.0, 0.35, Sine, 0.45, 0.0); s.tone(1320.0, 1320.0, 0.25, Sine, 0.15, 0.0); 0.28 }
        "heal" => { for (i, f) in [523.0, 659.0, 784.0, 1046.0].iter().enumerate() { s.tone(*f, *f, 0.35, Sine, 0.4, i as f32 * 0.06); } 0.35 }
        "level" => { for (i, f) in [392.0, 523.0, 659.0, 784.0, 1046.0].iter().enumerate() { s.tone(*f, *f, 0.45, Tri, 0.5, i as f32 * 0.08); } 0.45 }
        "hurt" => { s.tone(140.0, 60.0, 0.22, Square, 0.6, 0.0); 0.4 }
        "pick" => { s.tone(880.0, 1320.0, 0.08, Sine, 0.5, 0.0); 0.25 }
        "gem" => { s.tone(1250.0, 1900.0, 0.07, Sine, 0.5, 0.0); 0.18 }
        "dash" => { s.hiss(0.25, 1200.0, 3000.0, 1.0, 2, 0.0); 0.35 }
        "die" => { s.tone(220.0, 55.0, 0.5, Saw, 0.4, 0.0); 0.3 }
        "chest" => { for (i, f) in [330.0, 440.0, 554.0, 660.0, 880.0].iter().enumerate() { s.tone(*f, *f, 0.3, Tri, 0.4, i as f32 * 0.05); } 0.4 }
        "shout" => { s.tone(110.0, 90.0, 0.6, Saw, 0.6, 0.0); s.tone(165.0, 140.0, 0.6, Saw, 0.35, 0.0); 0.45 }
        "zap" => { s.hiss(0.22, 4000.0, 1500.0, 0.8, 2, 0.0); s.tone(900.0, 150.0, 0.1, Tri, 0.25, 0.0); s.tone(70.0, 40.0, 0.3, Sine, 0.7, 0.02); 0.42 }
        "horn" => { s.tone(98.0, 94.0, 1.4, Saw, 0.5, 0.0); s.tone(147.0, 141.0, 1.3, Saw, 0.3, 0.12); 0.4 }
        "glass" => { s.tone(2400.0, 1300.0, 0.09, Tri, 0.4, 0.0); s.hiss(0.18, 6000.0, 6000.0, 0.8, 1, 0.0); 0.35 }
        "swell" => { s.tone(110.0, 330.0, 0.7, Saw, 0.3, 0.0); 0.3 }
        "squelch" => { s.hiss(0.4, 500.0, 90.0, 1.0, 0, 0.0); s.tone(160.0, 50.0, 0.3, Sine, 0.7, 0.0); 0.5 }
        "whoosh" => { s.hiss(0.3, 600.0, 2400.0, 1.0, 2, 0.0); 0.35 }
        "rise" => { s.hiss(0.35, 260.0, 120.0, 1.0, 0, 0.0); 0.3 }
        "relic" => { for (i, f) in [523.0, 784.0, 1046.0, 1568.0].iter().enumerate() { s.tone(*f, *f, 0.5, Tri, 0.4, i as f32 * 0.07); } s.tone(262.0, 262.0, 0.8, Sine, 0.4, 0.0); 0.4 }
        "bomb" => { s.hiss(1.2, 2400.0, 60.0, 1.0, 0, 0.0); s.tone(60.0, 28.0, 0.9, Sine, 1.0, 0.0); s.tone(880.0, 220.0, 0.5, Tri, 0.2, 0.0); 0.8 }
        "thud" => { s.tone(80.0, 40.0, 0.25, Sine, 1.0, 0.0); s.hiss(0.15, 600.0, 200.0, 0.6, 0, 0.0); 0.45 }
        "snap" => { s.tone(1600.0, 400.0, 0.06, Square, 0.5, 0.0); s.hiss(0.12, 4000.0, 1500.0, 0.8, 2, 0.0); s.tone(120.0, 60.0, 0.15, Sine, 0.6, 0.02); 0.5 }
        "eat" => { s.hiss(0.1, 900.0, 900.0, 0.8, 2, 0.0); s.hiss(0.1, 700.0, 700.0, 0.8, 2, 0.12); 0.3 }
        "spawn" => { s.tone(70.0, 40.0, 0.6, Sine, 0.6, 0.0); 0.3 }
        "swap" => { s.hiss(0.14, 1500.0, 3200.0, 0.8, 2, 0.0); s.tone(660.0, 990.0, 0.06, Tri, 0.3, 0.05); 0.25 }
        "rumble" => { s.hiss(1.5, 300.0, 60.0, 1.0, 0, 0.0); s.tone(45.0, 30.0, 1.4, Sine, 0.8, 0.0); 0.6 }
        "splash" => { s.hiss(0.5, 2600.0, 500.0, 1.0, 0, 0.0); s.hiss(0.25, 5000.0, 2000.0, 0.4, 2, 0.04); s.tone(300.0, 120.0, 0.12, Sine, 0.4, 0.0); 0.35 }
        "stroke" => { s.hiss(0.22, 1400.0, 500.0, 1.0, 0, 0.0); s.hiss(0.12, 3000.0, 1600.0, 0.3, 2, 0.08); 0.12 }
        "sizzle" => { s.hiss(0.6, 6000.0, 2500.0, 1.0, 1, 0.0); s.hiss(0.4, 900.0, 300.0, 0.6, 0, 0.0); 0.3 }
        "gun" => { s.hiss(0.09, 5000.0, 1200.0, 1.0, 1, 0.0); s.tone(220.0, 70.0, 0.08, Tri, 0.7, 0.0); s.hiss(0.25, 900.0, 200.0, 0.5, 0, 0.02); 0.32 }
        "rifle" => { s.hiss(0.14, 4000.0, 600.0, 1.0, 1, 0.0); s.tone(150.0, 45.0, 0.2, Sine, 0.9, 0.0); s.hiss(0.6, 700.0, 120.0, 0.6, 0, 0.03); s.tone(1800.0, 1600.0, 0.03, Square, 0.15, 0.32); s.tone(1400.0, 1200.0, 0.03, Square, 0.15, 0.4); 0.45 }
        "reload" => { s.tone(2200.0, 2000.0, 0.03, Square, 0.3, 0.0); s.tone(1600.0, 1500.0, 0.03, Square, 0.3, 0.07); s.hiss(0.06, 5000.0, 4000.0, 0.4, 2, 0.12); 0.25 }
        "bone" => { for i in 0..4 { s.tone(900.0 - i as f32 * 120.0, 500.0, 0.04, Tri, 0.4, i as f32 * 0.05); s.hiss(0.03, 3000.0, 2000.0, 0.4, 2, i as f32 * 0.05); } 0.3 }
        "soul" => { s.tone(330.0, 220.0, 0.35, Sine, 0.5, 0.0); s.tone(494.0, 330.0, 0.3, Sine, 0.25, 0.04); s.hiss(0.3, 600.0, 1500.0, 0.25, 2, 0.0); 0.22 }
        "knife" => { s.hiss(0.09, 5000.0, 2500.0, 0.8, 2, 0.0); s.tone(2400.0, 1800.0, 0.03, Sine, 0.15, 0.0); 0.18 }
        "reap" => { s.hiss(0.3, 400.0, 1600.0, 1.0, 2, 0.0); s.hiss(0.2, 2400.0, 900.0, 0.3, 2, 0.08); s.tone(220.0, 150.0, 0.2, Sine, 0.15, 0.05); 0.28 }
        "ui" => { s.tone(420.0, 300.0, 0.05, Tri, 0.6, 0.0); s.hiss(0.04, 1800.0, 900.0, 0.4, 2, 0.0); 0.12 }
        "portal" => { s.tone(110.0, 440.0, 1.1, Saw, 0.25, 0.0); s.tone(220.0, 880.0, 1.0, Sine, 0.4, 0.1); s.hiss(1.2, 400.0, 3000.0, 0.6, 2, 0.0); 0.45 }
        _ => { s.tone(440.0, 440.0, 0.1, Sine, 0.2, 0.0); 0.1 }
    };
    // trim trailing silence
    let last = s.buf.iter().rposition(|v| v.abs() > 1e-4).unwrap_or(0) + 64;
    s.buf.truncate(last.min(s.buf.len()));
    (s.wav(), vol)
}

pub const NAMES: &[&str] = &["swing", "heavy", "hit", "crit", "bow", "fire", "boom", "ice", "arcane", "holy", "heal", "level", "hurt", "pick", "gem", "dash", "die", "chest", "shout",
    "zap", "horn", "glass", "swell", "squelch", "whoosh", "rise", "relic", "bomb", "thud", "snap", "eat", "spawn", "rumble", "swap", "splash", "stroke", "sizzle", "portal", "gun", "rifle", "reload", "bone", "soul", "ui", "wand", "staff", "scepter", "knife", "reap"];

/// how a sound behaves in the mix: the shortest gap between plays, how many may sound at once, a volume trim,
/// a priority (0 chatter .. 3 moments) and whether it ducks everything else
struct Rule { gap: f64, voices: usize, vol: f32, prio: u8, duck: bool }
fn rule(n: &str) -> Rule {
    let r = |gap: f64, voices: usize, vol: f32, prio: u8| Rule { gap, voices, vol, prio, duck: false };
    match n {
        "hit" => r(0.045, 3, 1.0, 0), "gem" => r(0.03, 3, 1.0, 0), "die" => r(0.06, 3, 0.85, 0), "stroke" => r(0.2, 1, 1.0, 0), "ui" => r(0.04, 2, 1.0, 1),
        "crit" => r(0.06, 2, 1.0, 1), "swing" | "heavy" | "knife" | "reap" => r(0.06, 2, 1.0, 1), "bow" | "gun" => r(0.05, 3, 1.0, 1), "wand" | "staff" | "scepter" | "soul" => r(0.08, 2, 1.0, 1),
        "fire" | "ice" | "zap" | "snap" | "squelch" | "bone" => r(0.11, 2, 1.0, 1), "arcane" => r(0.18, 2, 1.0, 1), "holy" => r(0.25, 1, 1.0, 1), "rise" => r(0.3, 1, 1.0, 1),
        "hurt" => r(0.15, 1, 1.0, 2), "boom" | "bomb" => Rule { gap: 0.08, voices: 3, vol: 1.0, prio: 2, duck: true },
        "level" | "relic" | "chest" | "horn" | "rumble" | "portal" => Rule { gap: 0.3, voices: 1, vol: 1.0, prio: 3, duck: true },
        _ => r(0.06, 3, 1.0, 1),
    }
}
/// the frequent sounds get three slightly different takes so repeats don't sound mechanical
fn variants(n: &str) -> u32 { if matches!(n, "knife" | "reap" | "hit" | "crit" | "swing" | "heavy" | "bow" | "gun" | "gem" | "die" | "wand" | "staff" | "scepter" | "soul" | "bone" | "stroke" | "ui" | "splash" | "rifle") { 3 } else { 1 } }

/// a sound that is playing (so the mixer can count voices)
#[derive(Component)]
pub struct Voice { name: &'static str, end: f64 }

#[derive(Resource, Default)]
pub struct Sfx { pub queue: Vec<&'static str>, handles: HashMap<&'static str, (Vec<Handle<AudioSource>>, f32, f32)>, last: HashMap<&'static str, f64>, fatigue: HashMap<&'static str, f32>, pub duck: f32, rng: u32, pub muted: bool, pub master: f32, pub music: f32 }
impl Sfx {
    pub fn play(&mut self, n: &'static str) { if self.queue.len() < 96 { self.queue.push(n); } }
    fn r(&mut self) -> f32 { let mut x = self.rng.max(1); x ^= x << 13; x ^= x >> 17; x ^= x << 5; self.rng = x; (x % 10000) as f32 / 10000.0 }
}

pub fn setup_audio(mut sfx: ResMut<Sfx>, mut sources: ResMut<Assets<AudioSource>>) {
    sfx.master = 0.7; sfx.music = 0.6; sfx.rng = 0x2545f491;
    for &n in NAMES {
        let mut hs = vec![]; let mut vol = 0.3; let mut dur = 0.2;
        for v in 0..variants(n) { let (bytes, vo) = make(n, v); vol = vo; dur = (bytes.len().saturating_sub(44) / 2) as f32 / SR; hs.push(sources.add(AudioSource { bytes: bytes.into() })); }
        sfx.handles.insert(n, (hs, vol, dur));
    }
}
/// the mixer: every request this frame is grouped by name, then each sound obeys its rule. Rapid repeats get
/// quieter (fatigue), every play is pitched slightly differently, a crowd of the same sound in one frame plays once
/// a little louder (log scale) instead of stacking, and big moments duck everything small for an instant.
pub fn play_sfx(mut c: Commands, mut sfx: ResMut<Sfx>, time: Res<Time>, keys: Res<ButtonInput<KeyCode>>, voices: Query<(Entity, &Voice)>) {
    if keys.just_pressed(KeyCode::KeyM) { sfx.muted = !sfx.muted; }
    let now = time.elapsed_secs_f64(); let dt = time.delta_secs();
    for v in sfx.fatigue.values_mut() { *v = (*v - dt * 2.5).max(0.0); }
    sfx.duck = (sfx.duck - dt).max(0.0);
    let q = std::mem::take(&mut sfx.queue);
    if sfx.muted { return; }
    let mut groups: Vec<(&'static str, u32)> = vec![];
    for n in q { if let Some(g) = groups.iter_mut().find(|g| g.0 == n) { g.1 += 1; } else { groups.push((n, 1)); } }
    let log = cfg!(feature = "dev") && std::env::var("EW_SFXLOG").is_ok();
    let mut live: HashMap<&'static str, usize> = HashMap::new(); let mut total = 0usize;
    // a voice counts until its clip has finished (and is cleared up even if no audio device ever played it)
    for (e, v) in voices.iter() { if now < v.end { *live.entry(v.name).or_insert(0) += 1; total += 1; } else if now > v.end + 1.0 { c.entity(e).despawn(); } }
    for (n, cnt) in groups {
        let r = rule(n);
        if sfx.last.get(n).map(|t| now - t < r.gap).unwrap_or(false) { continue; }
        if live.get(n).copied().unwrap_or(0) >= r.voices { continue; }
        if total >= 22 && r.prio < 2 { continue; }
        let Some((hs, base, dur)) = sfx.handles.get(n).cloned() else { continue };
        let fat = sfx.fatigue.get(n).copied().unwrap_or(0.0);
        let fk = (1.0 / (1.0 + 0.22 * fat)).max(0.35);
        let salvo = (1.0 + 0.25 * (cnt as f32).ln()).min(1.8);
        let dk = if sfx.duck > 0.0 && r.prio < 2 { 0.55 } else { 1.0 };
        let vol = base * r.vol * fk * salvo * dk * sfx.master;
        let pitch = 1.0 + (sfx.r() - 0.5) * if r.prio >= 2 { 0.06 } else { 0.14 };
        let h = hs[(sfx.r() * hs.len() as f32) as usize % hs.len()].clone();
        c.spawn((AudioPlayer(h), PlaybackSettings::DESPAWN.with_volume(Volume::new(vol)).with_speed(pitch), Voice { name: n, end: now + (dur / pitch) as f64 }));
        *sfx.fatigue.entry(n).or_insert(0.0) += 1.0; sfx.last.insert(n, now);
        *live.entry(n).or_insert(0) += 1; total += 1;
        if r.duck { sfx.duck = 0.4; }
        if log { *LOG.lock().unwrap().entry(n).or_insert(0) += 1; }
    }
    // dev: print how many of each sound actually played in the last second, and the most voices at once
    if log {
        let mut l = LOG.lock().unwrap(); let peak = l.entry("_peak").or_insert(0); *peak = (*peak).max(total as u32);
        let _ = &voices;
        let sec = now as u64; let last = l.entry("_sec").or_insert(sec as u32);
        if *last != sec as u32 { *last = sec as u32; let mut v: Vec<_> = l.iter().filter(|(k, _)| !k.starts_with('_')).map(|(k, n)| format!("{}:{}", k, n)).collect(); v.sort(); println!("[sfx] t={} peak={} {}", sec, l["_peak"], v.join(" ")); l.clear(); }
    }
}
static LOG: std::sync::LazyLock<std::sync::Mutex<HashMap<&'static str, u32>>> = std::sync::LazyLock::new(|| std::sync::Mutex::new(HashMap::new()));

// ---------------- music ----------------
// A small generative score rendered once at startup: a minor pad, bass and a pentatonic pluck line in one loop,
// and a drum loop of the same length layered on top. The drums swell with how hot the fight is.
const BEAT: f32 = 0.17;
const STEPS: usize = 16 * 8;
fn mf(semi: f32) -> f32 { 110.0 * 2f32.powf(semi / 12.0) }
fn music_loop(drums: bool) -> Vec<u8> {
    use Wave::*;
    let len = STEPS as f32 * BEAT;
    let mut s = Synth::new(len + 2.0);
    let scale = [0.0, 3.0, 5.0, 7.0, 10.0, 12.0, 15.0, 17.0, 19.0];
    let mut seed = 0x51f2u32;
    let mut rnd = move || { seed ^= seed << 13; seed ^= seed >> 17; seed ^= seed << 5; (seed % 10000) as f32 / 10000.0 };
    for step in 0..STEPS {
        let (bar, k) = (step / 16, step % 16); let t = step as f32 * BEAT;
        let root = [0.0, -4.0, -2.0, -5.0][bar % 4];
        if !drums {
            if k == 0 { for iv in [0.0, 7.0, 10.0, 15.0] { s.tone(mf(root + iv - 12.0), mf(root + iv - 12.0), BEAT * 17.0, Tri, 0.05, t); } s.tone(mf(root - 24.0), mf(root - 24.0), BEAT * 16.0, Sine, 0.14, t); }
            if k % 2 == 0 && rnd() < 0.45 { let n = scale[(rnd() * scale.len() as f32) as usize % scale.len()] + if rnd() < 0.3 { 12.0 } else { 0.0 }; s.tone(mf(root + n), mf(root + n), 0.4, Sine, 0.07, t); }
        } else {
            if k % 4 == 0 { s.tone(130.0, 38.0, 0.25, Sine, 0.5, t); }
            if k % 4 == 2 { s.hiss(0.05, 7000.0, 7000.0, 0.12, 1, t); }
            if k % 2 == 1 && rnd() < 0.6 { s.hiss(0.03, 8000.0, 8000.0, 0.07, 1, t); }
            if k % 8 == 4 { s.tone(mf(root - 12.0), mf(root - 12.0), 0.25, Saw, 0.08, t); s.hiss(0.12, 1800.0, 900.0, 0.18, 2, t); }
        }
    }
    // fold the tail back over the start so the loop is seamless
    let n = (len * SR) as usize;
    let tail: Vec<f32> = s.buf[n..].to_vec();
    for (i, v) in tail.iter().enumerate() { if i < n { s.buf[i] += v; } }
    s.buf.truncate(n);
    s.wav()
}
#[derive(Component)]
pub struct MusicLayer(pub u8);
pub fn setup_music(mut c: Commands, mut sources: ResMut<Assets<AudioSource>>) {
    for k in 0..2u8 {
        let h = sources.add(AudioSource { bytes: music_loop(k == 1).into() });
        c.spawn((AudioPlayer(h), PlaybackSettings::LOOP.with_volume(Volume::new(0.0)), MusicLayer(k)));
    }
}
pub fn update_music(q: Query<(&MusicLayer, &AudioSink)>, sfx: Res<Sfx>, g: Res<crate::game::Game>, ui: Res<crate::hud::UiState>) {
    let playing = ui.screen != crate::hud::Screen::Menu;
    for (l, sink) in q.iter() {
        let v = sfx.music * if sfx.duck > 0.0 { 0.6 } else { 1.0 } * if sfx.muted { 0.0 } else if l.0 == 0 { 0.55 } else if playing { 0.65 * ((g.dir.heat - 0.15) / 0.6).clamp(0.0, 1.0) } else { 0.0 };
        let cur = sink.volume();
        sink.set_volume(cur + (v - cur) * 0.05);
    }
}
