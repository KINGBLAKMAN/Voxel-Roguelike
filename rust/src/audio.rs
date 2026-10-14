//! Procedural sound: every effect is synthesised into an in-memory WAV at startup (oscillators with pitch
//! sweeps and envelopes, filtered noise), so the game ships as a single exe with no audio files.
use bevy::audio::{AudioPlayer, AudioSource, PlaybackSettings, Volume};
use bevy::prelude::*;
use std::collections::HashMap;

const SR: f32 = 22050.0;

#[derive(Clone, Copy)]
enum Wave { Sine, Tri, Saw, Square }
struct Synth { buf: Vec<f32>, seed: u32 }
impl Synth {
    fn new(len: f32) -> Self { Synth { buf: vec![0.0; (len * SR) as usize + 1], seed: 0x1234567 } }
    fn noise(&mut self) -> f32 { self.seed ^= self.seed << 13; self.seed ^= self.seed >> 17; self.seed ^= self.seed << 5; (self.seed as f32 / u32::MAX as f32) * 2.0 - 1.0 }
    /// oscillator with an exponential pitch sweep and a quick-attack exponential decay
    fn tone(&mut self, f0: f32, f1: f32, dur: f32, w: Wave, vol: f32, delay: f32) {
        let (s0, n) = ((delay * SR) as usize, (dur * SR) as usize);
        let mut ph = 0.0f32;
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

fn make(name: &str) -> (Vec<u8>, f32) {
    use Wave::*;
    let mut s = Synth::new(1.6);
    let vol = match name {
        "swing" => { s.hiss(0.18, 1800.0, 600.0, 1.0, 2, 0.0); 0.35 }
        "heavy" => { s.hiss(0.32, 900.0, 250.0, 1.0, 2, 0.0); s.tone(90.0, 50.0, 0.2, Sine, 0.4, 0.0); 0.45 }
        "hit" => { s.tone(180.0, 70.0, 0.12, Square, 0.5, 0.0); s.hiss(0.08, 2500.0, 2000.0, 0.6, 2, 0.0); 0.3 }
        "crit" => { s.tone(520.0, 900.0, 0.12, Square, 0.4, 0.0); s.hiss(0.1, 3000.0, 3000.0, 0.7, 2, 0.0); 0.35 }
        "bow" => { s.tone(330.0, 140.0, 0.15, Tri, 0.8, 0.0); s.hiss(0.12, 3500.0, 3500.0, 0.4, 2, 0.0); 0.4 }
        "fire" => { s.hiss(0.35, 700.0, 2400.0, 1.0, 0, 0.0); 0.4 }
        "boom" => { s.hiss(0.9, 1200.0, 80.0, 1.0, 0, 0.0); s.tone(90.0, 35.0, 0.6, Sine, 0.8, 0.0); 0.75 }
        "ice" => { s.tone(1400.0, 2400.0, 0.25, Sine, 0.6, 0.0); s.tone(2100.0, 1300.0, 0.2, Tri, 0.3, 0.03); 0.3 }
        "arcane" => { s.tone(440.0, 880.0, 0.4, Saw, 0.25, 0.0); s.tone(660.0, 330.0, 0.4, Sine, 0.5, 0.0); 0.35 }
        "holy" => { s.tone(784.0, 784.0, 0.6, Sine, 0.5, 0.0); s.tone(988.0, 988.0, 0.6, Sine, 0.4, 0.06); s.tone(1175.0, 1175.0, 0.7, Sine, 0.35, 0.12); 0.35 }
        "heal" => { for (i, f) in [523.0, 659.0, 784.0, 1046.0].iter().enumerate() { s.tone(*f, *f, 0.35, Sine, 0.4, i as f32 * 0.06); } 0.35 }
        "level" => { for (i, f) in [392.0, 523.0, 659.0, 784.0, 1046.0].iter().enumerate() { s.tone(*f, *f, 0.45, Tri, 0.5, i as f32 * 0.08); } 0.45 }
        "hurt" => { s.tone(140.0, 60.0, 0.22, Square, 0.6, 0.0); 0.4 }
        "pick" => { s.tone(880.0, 1320.0, 0.08, Sine, 0.5, 0.0); 0.25 }
        "gem" => { s.tone(1250.0, 1900.0, 0.07, Sine, 0.5, 0.0); 0.18 }
        "dash" => { s.hiss(0.25, 1200.0, 3000.0, 1.0, 2, 0.0); 0.35 }
        "die" => { s.tone(220.0, 55.0, 0.5, Saw, 0.4, 0.0); 0.3 }
        "chest" => { for (i, f) in [330.0, 440.0, 554.0, 660.0, 880.0].iter().enumerate() { s.tone(*f, *f, 0.3, Tri, 0.4, i as f32 * 0.05); } 0.4 }
        "shout" => { s.tone(110.0, 90.0, 0.6, Saw, 0.6, 0.0); s.tone(165.0, 140.0, 0.6, Saw, 0.35, 0.0); 0.45 }
        "zap" => { s.hiss(0.3, 5200.0, 1800.0, 1.0, 1, 0.0); s.tone(1800.0, 180.0, 0.14, Square, 0.2, 0.0); s.tone(70.0, 40.0, 0.35, Sine, 0.8, 0.02); 0.5 }
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
        "rumble" => { s.hiss(1.5, 300.0, 60.0, 1.0, 0, 0.0); s.tone(45.0, 30.0, 1.4, Sine, 0.8, 0.0); 0.6 }
        _ => { s.tone(440.0, 440.0, 0.1, Sine, 0.2, 0.0); 0.1 }
    };
    // trim trailing silence
    let last = s.buf.iter().rposition(|v| v.abs() > 1e-4).unwrap_or(0) + 64;
    s.buf.truncate(last.min(s.buf.len()));
    (s.wav(), vol)
}

pub const NAMES: &[&str] = &["swing", "heavy", "hit", "crit", "bow", "fire", "boom", "ice", "arcane", "holy", "heal", "level", "hurt", "pick", "gem", "dash", "die", "chest", "shout",
    "zap", "horn", "glass", "swell", "squelch", "whoosh", "rise", "relic", "bomb", "thud", "snap", "eat", "spawn", "rumble"];

#[derive(Resource, Default)]
pub struct Sfx { pub queue: Vec<&'static str>, handles: HashMap<&'static str, (Handle<AudioSource>, f32)>, gate: HashMap<&'static str, f64>, pub muted: bool, pub master: f32 }
impl Sfx { pub fn play(&mut self, n: &'static str) { if self.queue.len() < 24 { self.queue.push(n); } } }

pub fn setup_audio(mut sfx: ResMut<Sfx>, mut sources: ResMut<Assets<AudioSource>>) {
    sfx.master = 0.7;
    for &n in NAMES { let (bytes, vol) = make(n); let h = sources.add(AudioSource { bytes: bytes.into() }); sfx.handles.insert(n, (h, vol)); }
}
pub fn play_sfx(mut c: Commands, mut sfx: ResMut<Sfx>, time: Res<Time>, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::KeyM) { sfx.muted = !sfx.muted; }
    let now = time.elapsed_secs_f64();
    let q = std::mem::take(&mut sfx.queue);
    if sfx.muted { return; }
    for n in q {
        if let Some(t) = sfx.gate.get(n) { if now - t < 0.05 { continue; } }
        sfx.gate.insert(n, now);
        if let Some((h, v)) = sfx.handles.get(n) { c.spawn((AudioPlayer(h.clone()), PlaybackSettings::DESPAWN.with_volume(Volume::new(v * sfx.master)))); }
    }
}
