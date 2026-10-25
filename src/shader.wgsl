// A shader is a small program that runs on the GPU.
// This file has two of them:
// - the vertex shader runs once per corner (vertex) and says where it goes on screen.
// - the fragment shader runs once per pixel inside a triangle and says its colour.

// Data that is the same for every vertex in a draw call.
// Must match the `Uniforms` struct in renderer.rs.
struct Uniforms {
    // projection * view * model: from the model's own space to the screen.
    mvp: mat4x4<f32>,
    // model only: from the model's own space to the world. Used to turn the normals.
    model: mat4x4<f32>,
};

// `@group(0) @binding(0)` matches the bind group set in renderer.rs.
@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

// One vertex, read from the vertex buffer.
// The `@location` numbers match `Vertex::ATTRIBUTES` in vertex.rs.
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

// Data passed from the vertex shader to the fragment shader.
// The GPU blends `normal` smoothly between the corners.
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
};

// The direction the light comes from, in world space: above, right and in front.
const LIGHT_DIR: vec3<f32> = vec3<f32>(0.4, 1.0, 0.7);
const BASE_COLOR: vec3<f32> = vec3<f32>(0.9, 0.55, 0.25);
// A little light everywhere, so the side facing away isn't pure black.
const AMBIENT: f32 = 0.12;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // w = 1 means "a point". The matrix moves it from the model's own space to the screen.
    out.position = uniforms.mvp * vec4<f32>(in.position, 1.0);
    // w = 0 means "a direction", so the model's translation does not move it.
    // Note: this is only right when the model is scaled the same amount on every axis.
    // Uneven scaling would need the "inverse transpose" of the model matrix instead.
    out.normal = (uniforms.model * vec4<f32>(in.normal, 0.0)).xyz;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Blending between corners makes the normal a bit shorter, so make it length 1 again.
    let n = normalize(in.normal);
    // Diffuse (Lambert) lighting: a surface facing the light gets full light (dot = 1),
    // one side-on gets none (dot = 0). `max` stops faces turned away going negative.
    let diffuse = max(dot(n, normalize(LIGHT_DIR)), 0.0);
    let light = AMBIENT + (1.0 - AMBIENT) * diffuse;
    return vec4<f32>(BASE_COLOR * light, 1.0);
}
