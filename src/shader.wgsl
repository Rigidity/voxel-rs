struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@group(2) @binding(0)
var<uniform> chunk_position: vec3<f32>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) ao: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) ao: f32,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.normal = model.normal;
    out.ao = model.ao;
    out.clip_position = camera.view_proj * vec4<f32>(chunk_position + model.position, 1.0);
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample the texture
    let texture_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    
    // Define a directional light (like the sun)
    let light_dir = normalize(vec3<f32>(0.0, 1.0, 0.0));
    
    // Calculate diffuse lighting
    let diffuse = max(dot(in.normal, light_dir), 0.0);
    
    // Base ambient lighting (darker than before since AO will brighten open areas)
    let ambient = 0.15;
    
    // Combine directional lighting
    let directional_lighting = ambient + diffuse * 0.6;
    
    // Apply ambient occlusion (AO values range from dark in corners to bright in open areas)
    let lighting = directional_lighting * in.ao;
    
    return vec4<f32>(texture_color.rgb * lighting, texture_color.a);
}
