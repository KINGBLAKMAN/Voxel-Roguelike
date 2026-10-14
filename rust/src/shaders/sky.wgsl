// Procedural sky: horizon-to-zenith gradient, haze band, sun disk and glow, moon, and hashed twinkling stars.
#import bevy_pbr::{mesh_functions, view_transformations::position_world_to_clip, mesh_view_bindings::{view, globals}}

struct Sky { top: vec4<f32>, hor: vec4<f32>, sun_dir: vec4<f32>, sun_col: vec4<f32>, moon_col: vec4<f32>, params: vec4<f32> };
@group(2) @binding(0) var<uniform> sky: Sky;

struct VIn { @builtin(instance_index) ii: u32, @location(0) pos: vec3<f32> };
struct VOut { @builtin(position) clip: vec4<f32>, @location(0) wp: vec3<f32> };

@vertex fn vertex(v: VIn) -> VOut {
  var o: VOut;
  let m = mesh_functions::get_world_from_local(v.ii);
  let wp = (m * vec4<f32>(v.pos, 1.0)).xyz;
  o.wp = wp;
  o.clip = position_world_to_clip(wp);
  return o;
}
fn hash3(p: vec3<f32>) -> f32 { return fract(sin(dot(p, vec3<f32>(127.1, 311.7, 74.7))) * 43758.5453); }
fn hash2(p: vec2<f32>) -> f32 { let q = fract(p * vec2<f32>(0.1031, 0.1030)); let r = q + dot(q, q.yx + 33.33); return fract((r.x + r.y) * r.x); }
fn vnoise(p: vec2<f32>) -> f32 {
  let i = floor(p); let f = fract(p); let u = f * f * (3.0 - 2.0 * f);
  return mix(mix(hash2(i), hash2(i + vec2<f32>(1.0, 0.0)), u.x), mix(hash2(i + vec2<f32>(0.0, 1.0)), hash2(i + vec2<f32>(1.0, 1.0)), u.x), u.y);
}
fn fbm(p0: vec2<f32>) -> f32 {
  var p = p0; var a = 0.5; var v = 0.0;
  for (var k = 0; k < 6; k++) { v += a * vnoise(p); p = mat2x2<f32>(1.6, 1.2, -1.2, 1.6) * p + vec2<f32>(7.1, 3.3); a *= 0.5; }
  return v;
}

@fragment fn fragment(i: VOut) -> @location(0) vec4<f32> {
  let d = normalize(i.wp - view.world_position);
  let h = clamp(d.y, 0.0, 1.0);
  var c = mix(sky.hor.rgb, sky.top.rgb, pow(h, 0.45));
  c = mix(c, sky.hor.rgb * 1.1 + vec3<f32>(0.02), exp(-h * 14.0) * 0.5);
  if (d.y < 0.0) { c = mix(sky.hor.rgb, sky.hor.rgb * 0.6, clamp(-d.y * 4.0, 0.0, 1.0)); }
  let night = sky.params.x;
  let s = max(dot(d, sky.sun_dir.xyz), 0.0);
  c += sky.sun_col.rgb * (smoothstep(0.9986, 0.9993, s) * 30.0 + pow(s, 18.0) * 0.5 + pow(s, 4.0) * 0.2 * exp(-h * 3.0));
  let m = max(dot(d, -sky.sun_dir.xyz), 0.0);
  c += sky.moon_col.rgb * (smoothstep(0.9990, 0.9994, m) * 6.0 + pow(m, 40.0) * 0.25) * night;
  // stars: hash a grid of directions, twinkle with time
  if (night > 0.01 && d.y > 0.0) {
    let g = floor(d * 220.0);
    let r = hash3(g);
    if (r > 0.9965) {
      let tw = 0.6 + 0.4 * sin(globals.time * (2.0 + r * 30.0) + r * 100.0);
      let center = (g + 0.5) / 220.0;
      let fall = smoothstep(0.0045, 0.0, length(d - normalize(center)));
      c += vec3<f32>(0.9, 0.95, 1.0) * fall * tw * night * 4.0 * h;
    }
  }
  // painted cumulus: fbm on a plane above the world, lit from the sun side with bright rims
  if (d.y > 0.0) {
    let t = globals.time;
    let uv = d.xz / (d.y + 0.12) * 1.1 + vec2<f32>(t * 0.004, t * 0.0015);
    let n = fbm(uv * 1.4);
    let cov = sky.params.y;
    let dens = smoothstep(cov, cov + 0.22, n) * smoothstep(0.0, 0.22, d.y);
    if (dens > 0.0) {
      let sun2 = normalize(sky.sun_dir.xz + vec2<f32>(0.0001));
      let n2 = fbm(uv * 1.4 + sun2 * 0.12);
      let lit = clamp((n - n2) * 4.0 + 0.55, 0.0, 1.0);
      let day = 1.0 - night;
      let shadow = mix(sky.hor.rgb * 0.55, sky.top.rgb * 0.7, 0.4);
      let bright = mix(vec3<f32>(0.16, 0.18, 0.26), vec3<f32>(1.0, 0.98, 0.95) * 1.4, day) + sky.sun_col.rgb * 0.25 * day;
      var cc = mix(shadow, bright, lit);
      cc += sky.sun_col.rgb * pow(s, 10.0) * (1.0 - dens) * 2.5 * day;
      cc += sky.moon_col.rgb * pow(m, 12.0) * 0.3 * night;
      c = mix(c, cc, dens * 0.92);
    }
  }
  return vec4<f32>(c, 1.0);
}
