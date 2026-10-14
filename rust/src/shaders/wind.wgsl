// Foliage vertex shaders (main pass and prepass/shadows share the same sway so depth, AO and shadows line up):
// the standard PBR material, but each vertex sways in the wind by its colour alpha (0 at the root, 1 at the tip).
// Fragments sample the foliage card texture array (layer in uv_b.x) and cut out by alpha in every pass, so grass,
// leaves and bushes cast leaf-shaped shadows. Normals bend towards the sky so cards shade like a soft volume.
#import bevy_pbr::{mesh_functions, view_transformations::position_world_to_clip}
#ifdef PREPASS_PIPELINE
#import bevy_pbr::prepass_io::{Vertex, VertexOutput, FragmentOutput}
#else
#import bevy_pbr::{forward_io::{Vertex, VertexOutput, FragmentOutput}, pbr_fragment::pbr_input_from_standard_material, pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing}}
#endif

struct Wind { v: vec4<f32> };
@group(2) @binding(100) var<uniform> wind: Wind;
@group(2) @binding(101) var card_tex: texture_2d_array<f32>;
@group(2) @binding(102) var card_samp: sampler;

fn card(in: VertexOutput) -> vec4<f32> {
#ifdef VERTEX_UVS_B
  return textureSample(card_tex, card_samp, in.uv, i32(in.uv_b.x + 0.5));
#else
  return vec4<f32>(1.0);
#endif
}

fn sway(p: vec3<f32>, w: f32) -> vec3<f32> {
  let t = wind.v.y;
  let k = wind.v.x * w;
  let gust = 0.65 + 0.35 * sin(t * 0.37 + p.x * 0.05 + p.z * 0.03);
  return p + vec3<f32>(sin(t * 1.8 + p.x * 0.7 + p.z * 0.4) * 0.13, 0.0, cos(t * 1.4 + p.x * 0.3 + p.z * 0.6) * 0.09) * k * gust;
}

@vertex fn vertex(vertex: Vertex) -> VertexOutput {
  var out: VertexOutput;
  let world_from_local = mesh_functions::get_world_from_local(vertex.instance_index);
  var wp = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(vertex.position, 1.0));
#ifdef VERTEX_COLORS
  wp = vec4<f32>(sway(wp.xyz, vertex.color.a), 1.0);
  out.color = vec4<f32>(vertex.color.rgb, 1.0);
#endif
  out.world_position = wp;
  out.position = position_world_to_clip(wp.xyz);
#ifdef VERTEX_UVS_A
  out.uv = vertex.uv;
#endif
#ifdef VERTEX_UVS_B
  out.uv_b = vertex.uv_b;
#endif
#ifdef PREPASS_PIPELINE
#ifdef NORMAL_PREPASS_OR_DEFERRED_PREPASS
  out.world_normal = mesh_functions::mesh_normal_local_to_world(vertex.normal, vertex.instance_index);
#endif
#ifdef MOTION_VECTOR_PREPASS
  out.previous_world_position = wp;
#endif
#ifdef DEPTH_CLAMP_ORTHO
  out.clip_position_unclamped = out.position;
  out.position.z = min(out.position.z, 1.0);
#endif
#else
#ifdef VERTEX_NORMALS
  out.world_normal = mesh_functions::mesh_normal_local_to_world(vertex.normal, vertex.instance_index);
#endif
#endif
#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
  out.instance_index = vertex.instance_index;
#endif
  return out;
}

#ifdef PREPASS_PIPELINE
#ifdef PREPASS_FRAGMENT
@fragment fn fragment(in: VertexOutput) -> FragmentOutput {
  let a = card(in);
  if (a.a < 0.5) { discard; }
  var out: FragmentOutput;
#ifdef NORMAL_PREPASS
  out.normal = vec4<f32>(normalize(mix(in.world_normal, vec3<f32>(0.0, 1.0, 0.0), 0.6)) * 0.5 + vec3<f32>(0.5), 1.0);
#endif
#ifdef DEPTH_CLAMP_ORTHO
  out.frag_depth = in.clip_position_unclamped.z;
#endif
#ifdef MOTION_VECTOR_PREPASS
  out.motion_vector = vec2<f32>(0.0);
#endif
  return out;
}
#else
@fragment fn fragment(in: VertexOutput) {
  let a = card(in);
  if (a.a < 0.5) { discard; }
}
#endif
#else
@fragment fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
  let a = card(in);
  if (a.a < 0.5) { discard; }
  var pbr_input = pbr_input_from_standard_material(in, is_front);
  pbr_input.material.base_color = vec4<f32>(pbr_input.material.base_color.rgb * a.rgb, 1.0);
  pbr_input.N = normalize(mix(pbr_input.N, vec3<f32>(0.0, 1.0, 0.0), 0.6));
  var out: FragmentOutput;
  out.color = apply_pbr_lighting(pbr_input);
  out.color = main_pass_post_lighting_processing(pbr_input, out.color);
  return out;
}
#endif
