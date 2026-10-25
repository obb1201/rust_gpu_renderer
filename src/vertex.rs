//! The `Vertex` type, and the shape we draw (a cube).

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
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl Vertex {
    // Tells the shader which field goes to which `@location`, and its type.
    // Location 0 is `position` (3 floats), location 1 is `color` (3 floats).
    // The macro works out each field's byte offset for us.
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

    /// Describes how the bytes in the vertex buffer split into vertices.
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            // How many bytes to step forward to reach the next vertex: 6 floats = 24 bytes.
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            // Move to the next vertex for each vertex (not for each instance).
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

// The 8 corners of a cube, from -1 to 1 on each axis. Each corner has its own colour.
// "Front" is +z (towards the camera at the start), "top" is +y, "right" is +x.
pub const VERTICES: &[Vertex] = &[
    // 0: front bottom left, red
    Vertex {
        position: [-1.0, -1.0, 1.0],
        color: [1.0, 0.0, 0.0],
    },
    // 1: front bottom right, green
    Vertex {
        position: [1.0, -1.0, 1.0],
        color: [0.0, 1.0, 0.0],
    },
    // 2: front top right, blue
    Vertex {
        position: [1.0, 1.0, 1.0],
        color: [0.0, 0.0, 1.0],
    },
    // 3: front top left, yellow
    Vertex {
        position: [-1.0, 1.0, 1.0],
        color: [1.0, 1.0, 0.0],
    },
    // 4: back bottom left, magenta
    Vertex {
        position: [-1.0, -1.0, -1.0],
        color: [1.0, 0.0, 1.0],
    },
    // 5: back bottom right, cyan
    Vertex {
        position: [1.0, -1.0, -1.0],
        color: [0.0, 1.0, 1.0],
    },
    // 6: back top right, white
    Vertex {
        position: [1.0, 1.0, -1.0],
        color: [1.0, 1.0, 1.0],
    },
    // 7: back top left, black
    Vertex {
        position: [-1.0, 1.0, -1.0],
        color: [0.0, 0.0, 0.0],
    },
];

// 6 faces, 2 triangles per face, 3 corners per triangle = 36 indices.
// Each triangle is counter-clockwise when you look at that face from outside the cube.
// That matters now: the pipeline skips triangles that face away from the camera.
pub const INDICES: &[u16] = &[
    0, 1, 2, 0, 2, 3, // front (+z)
    5, 4, 7, 5, 7, 6, // back (-z)
    1, 5, 6, 1, 6, 2, // right (+x)
    4, 0, 3, 4, 3, 7, // left (-x)
    3, 2, 6, 3, 6, 7, // top (+y)
    4, 5, 1, 4, 1, 0, // bottom (-y)
];
