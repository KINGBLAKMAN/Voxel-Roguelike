// Foliage vertex shaders (main pass and prepass/shadows share the same sway so depth, AO and shadows line up):
// the standard PBR material, but each vertex sways in the wind by its colour alpha (0 at the root, 1 at the tip).
#import bevy_pbr::{mesh_functions, view_transformations::position_world_to_clip}
#ifdef PREPASS_PIPELINE
#import bevy_pbr::prepass_io::{Vertex, VertexOutput}
#else
#import bevy_pbr::forward_io::{Vertex, VertexOutput}
#endif

struct Wind { v: vec4<f32> };
@group(2) @binding(100) var<uniform> wind: Wind;

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
