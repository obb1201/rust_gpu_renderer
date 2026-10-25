// A shader is a small program that runs on the GPU.
// This file has two of them:
// - the vertex shader runs once per corner (vertex) and says where it goes on screen.
// - the fragment shader runs once per pixel inside a triangle and says its colour.

// Data that is the same for every vertex in a draw call.
// `mvp` is projection * view * model, built on the CPU in renderer.rs.
struct Uniforms {
    mvp: mat4x4<f32>,
};

// `@group(0) @binding(0)` matches the bind group set in renderer.rs.
@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

// One vertex, read from the vertex buffer.
// The `@location` numbers match `Vertex::ATTRIBUTES` in vertex.rs.
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

// Data passed from the vertex shader to the fragment shader.
// The GPU blends `color` smoothly between the corners.
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // w = 1 means "a point". The matrix moves it from the cube's own space to the screen.
    out.position = uniforms.mvp * vec4<f32>(in.position, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
