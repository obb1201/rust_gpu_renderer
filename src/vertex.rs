//! The `Vertex` type, and the shape we draw (a square made of two triangles).

/// One corner of a shape, as the GPU will read it from the vertex buffer.
///
/// `#[repr(C)]` makes Rust lay the fields out in memory in the order we wrote them,
/// with no reordering. Normally Rust may reorder fields, but the GPU needs to know
/// exactly where each field is.
///
/// `Pod` ("plain old data") and `Zeroable` come from the `bytemuck` crate. They promise
/// the struct is just plain numbers, so it's safe to view it as raw bytes for uploading.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 3],
}

impl Vertex {
    // Tells the shader which field goes to which `@location`, and its type.
    // Location 0 is `position` (2 floats), location 1 is `color` (3 floats).
    // The macro works out each field's byte offset for us.
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x3];

    /// Describes how the bytes in the vertex buffer split into vertices.
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            // How many bytes to step forward to reach the next vertex: 5 floats = 20 bytes.
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            // Move to the next vertex for each vertex (not for each instance).
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

// The four corners of the square. Screen space goes from -1 to 1 on both axes.
pub const VERTICES: &[Vertex] = &[
    // 0: top left, red
    Vertex {
        position: [-0.5, 0.5],
        color: [1.0, 0.0, 0.0],
    },
    // 1: bottom left, green
    Vertex {
        position: [-0.5, -0.5],
        color: [0.0, 1.0, 0.0],
    },
    // 2: bottom right, blue
    Vertex {
        position: [0.5, -0.5],
        color: [0.0, 0.0, 1.0],
    },
    // 3: top right, yellow
    Vertex {
        position: [0.5, 0.5],
        color: [1.0, 1.0, 0.0],
    },
];

// Which corners make each triangle. A square is two triangles, but they share
// two corners, so with indices we store 4 vertices instead of 6.
// Each triangle is listed counter-clockwise, which wgpu treats as the front.
pub const INDICES: &[u16] = &[
    0, 1, 2, // first triangle
    0, 2, 3, // second triangle
];
