# Change Log

All notable changes to this project are recorded here. Newest entries go first.

## 2026-10-25

- The renderer now loads and draws a model from a file: the Utah teapot (`assets/teapot.obj`), spinning around y. The model is moved and scaled to fit the view, using its bounding box. Tested with Vulkan and DX12.
- Added `src/obj.rs`: a hand-written OBJ reader. It reads `v` and `f` lines (including `1/2/3` and negative indices), splits faces with more than 3 corners into triangles, and works out smooth normals. Has unit tests (`cargo test obj`).
- Added simple lighting: `Vertex.color` became `Vertex.normal`, and the fragment shader does diffuse (Lambert) lighting plus a little ambient light. The uniform buffer now holds the model matrix as well as `mvp`, so normals can be turned into world space.
- Indices are now `u32` instead of `u16`, so models can have more than 65,535 vertices. Removed the cube data.
- `math.rs`: added `Mat4::translation`, `Mat4::scale`, and `Vec3` add, scale, length, min and max.
- The renderer now draws a spinning 3D cube. Added a uniform buffer and bind group for the matrix, a depth buffer (recreated on resize) and back-face culling. The projection uses the window's aspect ratio, so shapes no longer stretch.
- Added `src/math.rs`: a small hand-written `Vec3` and `Mat4` (rotation, look-at camera, perspective), with unit tests (`cargo test math`). We chose this over a math library so the matrix math is visible.
- `Vertex.position` is now 3D (`[f32; 3]`). The vertex data is now a cube: 8 corners and 36 indices.
- Vertex data now lives in Rust instead of the shader. Added a `Vertex` type (`src/vertex.rs`), a vertex buffer and an index buffer. The renderer now draws a 4-coloured square from two triangles with `draw_indexed`.
- Added the `bytemuck` dependency, to turn vertex data into raw bytes for the GPU.
- The window title now shows the GPU, backend, driver version (when known) and the frame rate (FPS and ms per frame), updated once per second.

- Recorded the GPU API decision in `CLAUDE.md`: cross-platform, start with wgpu, and learn Vulkan and DX12 too.
- Updated the "Current state" and "Commands" sections in `CLAUDE.md`.
- Added dependencies: `wgpu` 30, `winit` 0.30, `pollster`, `env_logger`, `log`.
- First renderer: opens a window and draws a red/green/blue triangle (`src/main.rs`, `src/renderer.rs`, `src/shader.wgsl`).
- The backend can be chosen with the `WGPU_BACKEND` environment variable. Tested with Vulkan and DX12.
- Recorded the learning plan in `CLAUDE.md`: wgpu, then raw Vulkan (`ash`), then raw DX12.

## 2026-10-24

- Created the Cargo project and set the package name to `rust_gpu_renderer` in `Cargo.toml`.

- Created `CLAUDE.md` with the project goal and working rules.
- Created this change log.
