// A shader is a small program that runs on the GPU.
// This file has two of them:
// - the vertex shader runs once per corner (vertex) and says where it goes on screen.
// - the fragment shader runs once per pixel inside the triangle and says its colour.

// Data passed from the vertex shader to the fragment shader.
// The GPU blends `color` smoothly between the three corners.
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VertexOutput {
    // For now the corners live inside the shader, so we don't need a vertex buffer yet.
    // Screen space goes from -1 to 1 on both axes, with (0, 0) in the middle.
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 0.5),   // top
        vec2<f32>(-0.5, -0.5), // bottom left
        vec2<f32>(0.5, -0.5),  // bottom right
    );
    var colors = array<vec3<f32>, 3>(
        vec3<f32>(1.0, 0.0, 0.0), // red
        vec3<f32>(0.0, 1.0, 0.0), // green
        vec3<f32>(0.0, 0.0, 1.0), // blue
    );

    var out: VertexOutput;
    out.position = vec4<f32>(positions[index], 0.0, 1.0);
    out.color = colors[index];
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
