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
  wp.y += sin(wp.x * 0.35 + t * 1.3) * 0.06 + cos(wp.z * 0.4 + t * 1.1) * 0.06;
  o.wp = wp;
  o.clip = position_world_to_clip(wp);
  return o;
}
@fragment fn fragment(i: VOut) -> @location(0) vec4<f32> {
  let t = globals.time;
  let p = i.wp;
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
