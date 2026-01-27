struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@group(2) @binding(0)
var<uniform> chunk_position: vec3<f32>;

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

    let position = vec3<f32>(f32(x), f32(y), f32(z));
    let tex_coords = vec2<f32>(f32(u), f32(v));

    var out: VertexOutput;
    out.tex_coords = tex_coords;
    out.ao = (f32(ao) / 1.5 + 1.0) / 3.0;
    out.clip_position = camera.view_proj * vec4<f32>(chunk_position + position, 1.0);
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
