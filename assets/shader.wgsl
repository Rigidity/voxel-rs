#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}

#ifdef OIT_ENABLED
#import bevy_core_pipeline::oit::oit_draw
#endif // OIT_ENABLED

struct ModelVertex {
    position: vec3<f32>,
    uv: vec2<f32>,
    normal: vec3<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var my_array_texture: texture_2d_array<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var my_array_texture_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var<storage, read> model_buffer: array<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var<uniform> sun_direction: vec3<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var<uniform> sun_strength: f32;
@group(#{MATERIAL_BIND_GROUP}) @binding(5) var<uniform> ambient: f32;
@group(#{MATERIAL_BIND_GROUP}) @binding(6) var<uniform> sky_brightness: f32;

struct VertexInput {
    @builtin(instance_index) instance_index: u32,
    @location(0) data: u32,
    @location(1) texture_index: u32,
    @location(2) light_data: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) @interpolate(flat) texture_index: u32,
    @location(2) world_normal: vec3<f32>,
    @location(3) sky_light: f32,
    @location(4) block_light: f32,
}

@vertex
fn vs_main(
    input: VertexInput,
) -> VertexOutput {
    // Unpack bitpacked data
    let pos_x = (input.data >> 25) & 0x1F;  // 5 bits
    let pos_y = (input.data >> 20) & 0x1F;  // 5 bits
    let pos_z = (input.data >> 15) & 0x1F;  // 5 bits
    let vertex_idx = (input.data >> 1) & 0x3FFF;  // 14 bits
    let is_transparent = input.data & 0x01;  // 1 bit

    // Calculate offset into model buffer
    // Each vertex is 8 floats: position(3) + uv(2) + normal(3)
    let buffer_idx = vertex_idx * 8u;

    // Read model vertex data from buffer
    let model_position = vec3<f32>(
        model_buffer[buffer_idx],
        model_buffer[buffer_idx + 1u],
        model_buffer[buffer_idx + 2u]
    );
    let model_uv = vec2<f32>(
        model_buffer[buffer_idx + 3u],
        model_buffer[buffer_idx + 4u]
    );
    let model_normal = vec3<f32>(
        model_buffer[buffer_idx + 5u],
        model_buffer[buffer_idx + 6u],
        model_buffer[buffer_idx + 7u]
    );

    // Combine chunk position with model position
    let block_pos = vec3<f32>(f32(pos_x), f32(pos_y), f32(pos_z));
    let final_position = vec4<f32>(block_pos + model_position, 1.0);

    var out: VertexOutput;
    out.tex_coords = model_uv;
    out.clip_position = mesh_position_local_to_clip(get_world_from_local(input.instance_index), final_position);

    // Apply depth offset for transparent faces to prevent z-fighting
    if (is_transparent != 0u) {
        out.clip_position.z += 0.0001;
    }

    // Chunks only have translation transforms, so local normal = world normal
    out.world_normal = model_normal;

    // Unpack light data
    let sky_l = (input.light_data >> 4u) & 0xFu;
    let block_l = input.light_data & 0xFu;
    out.sky_light = f32(sky_l) / 15.0;
    out.block_light = f32(block_l) / 15.0;

    out.texture_index = input.texture_index;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texture_color = textureSampleLevel(my_array_texture, my_array_texture_sampler, in.tex_coords, in.texture_index, 0.0);

    // Exponential light curve for smooth falloff
    let sky = pow(0.8, 15.0 * (1.0 - in.sky_light)) * sky_brightness;
    let block_l = pow(0.8, 15.0 * (1.0 - in.block_light));

    // Sky light is modulated by sun direction + ambient
    let directional = max(dot(in.world_normal, sun_direction), 0.0) * sun_strength;
    let sky_contribution = sky * (directional + ambient);

    // Block light is omnidirectional — not affected by sun direction
    let light = max(sky_contribution, block_l);

    let color = vec4<f32>(texture_color.rgb * light, texture_color.a);

    #ifdef OIT_ENABLED
        oit_draw(in.clip_position, color);
        discard;
    #endif // OIT_ENABLED

    return color;
}
