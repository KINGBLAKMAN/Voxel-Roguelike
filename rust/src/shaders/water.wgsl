// Water: rolling vertex waves, normal ripples, Fresnel blend toward the sky, sun glints, and matching distance fog.
#import bevy_pbr::{mesh_functions, view_transformations::position_world_to_clip, mesh_view_bindings::{view, globals}}

struct Water { deep: vec4<f32>, shallow: vec4<f32>, sky: vec4<f32>, sun: vec4<f32>, fog: vec4<f32>, params: vec4<f32> };
@group(2) @binding(0) var<uniform> wm: Water;

struct VIn { @builtin(instance_index) ii: u32, @location(0) pos: vec3<f32> };
struct VOut { @builtin(position) clip: vec4<f32>, @location(0) wp: vec3<f32> };

@vertex fn vertex(v: VIn) -> VOut {
  var o: VOut;
  let t = globals.time;
  let m = mesh_functions::get_world_from_local(v.ii);
  var wp = (m * vec4<f32>(v.pos, 1.0)).xyz;
  let calm = select(1.0, 0.35, wm.params.z > 0.5);
  wp.y += (sin(wp.x * 0.35 + t * 1.3 * calm) * 0.06 + cos(wp.z * 0.4 + t * 1.1 * calm) * 0.06) * calm;
  o.wp = wp;
  o.clip = position_world_to_clip(wp);
  return o;
}
@fragment fn fragment(i: VOut) -> @location(0) vec4<f32> {
  let t = globals.time;
  let p = i.wp;
  if (wm.params.z > 0.5) {
    // lava: a slow churning crust over a white-hot core that feeds the bloom
    let q = p.xz * 0.32;
    let n1 = sin(q.x * 1.3 + t * 0.35 + sin(q.y * 0.9 - t * 0.27) * 1.6) * sin(q.y * 1.1 - t * 0.3 + sin(q.x * 0.7 + t * 0.2) * 1.3);
    let n2 = sin(q.x * 3.4 - t * 0.6 + n1 * 2.2) * sin(q.y * 2.9 + t * 0.45 - n1);
    let n3 = sin((q.x + q.y) * 7.0 + n2 * 3.0 - t * 1.1) * 0.5 + 0.5;
    let crust = smoothstep(-0.15, 0.45, n1 * 0.6 + n2 * 0.35 + n3 * 0.12);
    var lc = mix(wm.shallow.rgb * 2.6, vec3<f32>(0.05, 0.018, 0.012), crust);
    lc += vec3<f32>(1.0, 0.62, 0.22) * pow(1.0 - crust, 5.0) * 3.2;
    lc += wm.deep.rgb * 0.6 * (1.0 - crust) * (0.6 + 0.4 * sin(t * 2.0 + q.x * 2.0));
    let ldist = length(view.world_position - p);
    let lf = clamp((ldist - wm.fog.w) / (wm.params.y - wm.fog.w), 0.0, 1.0) * 0.7;
    return vec4<f32>(mix(lc, wm.fog.rgb, lf), 1.0);
  }
  let r = sin(p.x * 0.9 + t * 1.7) * sin(p.z * 0.8 - t * 1.3) + sin((p.x + p.z) * 0.45 + t);
  let r2 = sin(p.x * 1.7 - t * 2.1 + sin(p.z * 0.6)) * 0.5 + sin(p.z * 1.9 + t * 1.6 + sin(p.x * 0.5)) * 0.5;
  let n = normalize(vec3<f32>(cos(p.x * 0.9 + t * 1.7) * 0.1 + r2 * 0.07, 1.0, cos(p.z * 0.8 - t * 1.3) * 0.1 - r2 * 0.06));
  let to_cam = view.world_position - p;
  let dist = length(to_cam);
  let v = to_cam / dist;
  let fres = pow(1.0 - clamp(dot(n, v), 0.0, 1.0), 3.0);
  var c = mix(wm.deep.rgb, wm.shallow.rgb, 0.45 + 0.16 * r + 0.06 * r2);
  c = mix(c, wm.sky.rgb, fres * 0.6);
  let day = wm.params.x;
  let hv = normalize(v + wm.sun.xyz);
  let sp = pow(max(dot(n, hv), 0.0), 160.0) * step(0.0, wm.sun.y);
  c += vec3<f32>(1.0, 0.95, 0.82) * (sp * 6.0 + smoothstep(1.45, 1.95, r) * 0.12) * day;
  c *= 0.35 + 0.65 * day;
  let f = clamp((dist - wm.fog.w) / (wm.params.y - wm.fog.w), 0.0, 1.0);
  c = mix(c, wm.fog.rgb, f);
  return vec4<f32>(c, mix(0.74, 0.94, fres));
}
