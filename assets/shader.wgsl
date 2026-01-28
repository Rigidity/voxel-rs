#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var my_array_texture: texture_2d_array<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var my_array_texture_sampler: sampler;

struct VertexInput {
    @location(0) data: u32,
    @location(1) texture_index: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) ao: f32,
    @location(2) @interpolate(flat) texture_index: u32,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    let x = (model.data >> 26) & 0x3F;
    let y = (model.data >> 20) & 0x3F;
    let z = (model.data >> 14) & 0x3F;
    let u = (model.data >> 13) & 0x01;
    let v = (model.data >> 12) & 0x01;
    let ao = (model.data >> 10) & 0x03;

    let position = vec4<f32>(f32(x), f32(y), f32(z), 1.0);
    let tex_coords = vec2<f32>(f32(u), f32(v));

    let world_from_local = get_world_from_local(0);
    let world_position = world_from_local * position;

    var out: VertexOutput;
    out.tex_coords = tex_coords;
    out.ao = 0.7 + f32(ao) * 0.1;
    out.clip_position = mesh_position_local_to_clip(world_position);
    out.texture_index = model.texture_index;
    return out;
}

@group(0) @binding(0)
var texture_array: binding_array<texture_2d<f32>>;
@group(0) @binding(1)
var texture_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texture_color = textureSampleLevel(texture_array[in.texture_index], texture_sampler, in.tex_coords, 0.0);
    return vec4<f32>(texture_color.rgb * in.ao, texture_color.a);
}
