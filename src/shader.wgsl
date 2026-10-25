// A shader is a small program that runs on the GPU.
// This file has two of them:
// - the vertex shader runs once per corner (vertex) and says where it goes on screen.
// - the fragment shader runs once per pixel inside a triangle and says its colour.

// One vertex, read from the vertex buffer.
// The `@location` numbers match `Vertex::ATTRIBUTES` in vertex.rs.
struct VertexInput {
    @location(0) position: vec2<f32>,
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
    // z = 0 (no depth yet), w = 1 (a normal point).
    out.position = vec4<f32>(in.position, 0.0, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
