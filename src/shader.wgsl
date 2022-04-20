// Vertex shader

struct InstanceInput {
    [[location(5)]] model_matrix_0: vec4<f32>;
    [[location(6)]] model_matrix_1: vec4<f32>;
    [[location(7)]] model_matrix_2: vec4<f32>;
    [[location(8)]] model_matrix_3: vec4<f32>;
};

struct Light {
    position: vec3<f32>;
    color: vec3<f32>;
};
struct Lights {
    data: array<Light>;
};
[[group(2), binding(0)]]
var<storage, read> lights: Lights;
struct LightCount {
    data: u32;
};
[[group(3), binding(0)]]
var<uniform> num_lights: LightCount;

struct CameraUniform {
    view_pos: vec4<f32>;
    view_proj: mat4x4<f32>;
};

[[group(1), binding(0)]]
var<uniform> camera: CameraUniform;

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] tex_coords: vec2<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] tex_coords: vec2<f32>;
    [[location(2)]] world_position: vec3<f32>;
};

[[stage(vertex)]]
fn vs_main(model: VertexInput, instance: InstanceInput) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    var world_position: vec4<f32> = model_matrix * vec4<f32>(model.position, 1.0);
    out.world_position = world_position.xyz;
    out.clip_position = camera.view_proj * world_position;
    return out;
}

// Fragment shader

[[group(0), binding(0)]]
var t_diffuse: texture_2d<f32>;
[[group(0), binding(1)]]
var s_diffuse: sampler;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    var diffuse_color = vec3<f32>(0.1, 0.1, 0.1);

    //let num_lights: i32 = bitcast<i32>(num_lights.data);

    for (var i: u32 = 0u; i < num_lights.data; i = i + 1u) {
        let light = lights.data[i];
        let distance = length(in.world_position - light.position);
        let strength = 1.0 / distance / distance;

        diffuse_color = diffuse_color + light.color * strength;
    }

    return vec4<f32>(diffuse_color * color.xyz, color.a);

}
