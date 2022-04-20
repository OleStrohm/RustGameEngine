// Vertex Shader

struct Camera {
    view_pos: vec4<f32>;
    view_proj: mat4x4<f32>;
};
[[group(0), binding(0)]]
var<uniform> camera: Camera;

struct Light {
    position: vec3<f32>;
    color: vec3<f32>;
};
struct Lights {
    data: array<Light>;
};
[[group(1), binding(0)]]
var<storage, read> lights: Lights;
struct LightCount {
    data: u32;
};

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] color: vec3<f32>;
    [[location(1)]] instance_index: u32;
    [[location(2)]] model_pos: vec3<f32>;
};

[[stage(vertex)]]
fn vs_main(
    model: VertexInput,
    [[builtin(instance_index)]] instance_index: u32,
) -> VertexOutput {
    let light = lights.data[instance_index];

    let scale = 0.25;
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(scale * model.position + light.position, 1.0);
    out.color = light.color;
    out.instance_index = instance_index;
    out.model_pos = scale * model.position;

    return out;
}

// Fragment shader

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let light = lights.data[in.instance_index];
    let radius = length(in.model_pos);
    let scale = 0.25;
    let mask = smoothStep(0.48, 0.5, radius / scale);

    return vec4<f32>(in.color, 1.0 - mask);
}

