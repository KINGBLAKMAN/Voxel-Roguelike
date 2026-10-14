// Terrain: the standard PBR material with a per-face texture layer (uv_b.x) from the block texture arrays.
// The albedo multiplies the vertex colour (baked AO, face shading, tint); the normal map is applied in a tangent
// frame built from the face's axis-aligned normal, matching how the mesher lays out UVs.
#import bevy_pbr::{pbr_fragment::pbr_input_from_standard_material, pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT,
    forward_io::{VertexOutput, FragmentOutput}, pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing}}

@group(2) @binding(100) var albedo_tex: texture_2d_array<f32>;
@group(2) @binding(101) var albedo_samp: sampler;
@group(2) @binding(102) var normal_tex: texture_2d_array<f32>;
@group(2) @binding(103) var normal_samp: sampler;

@fragment fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
  var pbr_input = pbr_input_from_standard_material(in, is_front);
  var out: FragmentOutput;
#ifdef VERTEX_UVS_B
  let layer = i32(in.uv_b.x + 0.5);
  let a = textureSample(albedo_tex, albedo_samp, in.uv, layer);
  let nm = textureSample(normal_tex, normal_samp, in.uv, layer);
  pbr_input.material.base_color = vec4<f32>(pbr_input.material.base_color.rgb * a.rgb, 1.0);
  let n = pbr_input.world_normal;
  let an = abs(n);
  var t = vec3<f32>(1.0, 0.0, 0.0);
  var b = vec3<f32>(0.0, -1.0, 0.0);
  if (an.x > 0.5) { t = vec3<f32>(0.0, 0.0, 1.0); }
  else if (an.y > 0.5) { b = vec3<f32>(0.0, 0.0, 1.0); }
  let tn = nm.xyz * 2.0 - 1.0;
  pbr_input.N = normalize(t * tn.x + b * tn.y + n * tn.z);
  pbr_input.material.perceptual_roughness = nm.a;
#endif
  if ((pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 0u) {
    out.color = apply_pbr_lighting(pbr_input);
  } else {
    out.color = pbr_input.material.base_color;
  }
  out.color = main_pass_post_lighting_processing(pbr_input, out.color);
  return out;
}
