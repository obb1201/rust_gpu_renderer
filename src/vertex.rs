//! The `Vertex` type: one corner of a mesh, as the GPU reads it.

/// One corner of a shape, as the GPU will read it from the vertex buffer.
///
/// `#[repr(C)]` makes Rust lay the fields out in memory in the order we wrote them,
/// with no reordering. Normally Rust may reorder fields, but the GPU needs to know
/// exactly where each field is.
///
/// `Pod` ("plain old data") and `Zeroable` come from the `bytemuck` crate. They promise
/// the struct is just plain numbers, so it's safe to view it as raw bytes for uploading.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    /// The direction the surface faces at this corner, with length 1.
    /// The shader uses it to work out how much light the surface gets.
    pub normal: [f32; 3],
}

impl Vertex {
    // Tells the shader which field goes to which `@location`, and its type.
    // Location 0 is `position` (3 floats), location 1 is `normal` (3 floats).
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
