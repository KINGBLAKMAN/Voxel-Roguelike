// Foliage vertex shader: the standard PBR material, but each vertex sways in the wind by its colour alpha (0 at the root, 1 at the tip).
#import bevy_pbr::{mesh_functions, forward_io::{Vertex, VertexOutput}, view_transformations::position_world_to_clip, mesh_view_bindings::globals}

@vertex fn vertex(vertex: Vertex) -> VertexOutput {
  var out: VertexOutput;
  let world_from_local = mesh_functions::get_world_from_local(vertex.instance_index);
  var wp = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(vertex.position, 1.0));
#ifdef VERTEX_COLORS
  let sway = vertex.color.a;
  let t = globals.time;
  let gust = 0.7 + 0.3 * sin(t * 0.37 + wp.x * 0.05);
  wp.x += sin(t * 1.8 + wp.x * 0.7 + wp.z * 0.4) * 0.13 * sway * gust;
  wp.z += cos(t * 1.4 + wp.x * 0.3 + wp.z * 0.6) * 0.09 * sway * gust;
  out.color = vec4<f32>(vertex.color.rgb, 1.0);
#endif
  out.world_position = wp;
  out.position = position_world_to_clip(wp.xyz);
#ifdef VERTEX_NORMALS
  out.world_normal = mesh_functions::mesh_normal_local_to_world(vertex.normal, vertex.instance_index);
#endif
#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
  out.instance_index = vertex.instance_index;
#endif
  return out;
}
