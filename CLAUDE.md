# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project goal

The owner is learning Rust and building a GPU renderer in it. Treat this as a learning project: when introducing Rust concepts (ownership, lifetimes, traits, etc.) or GPU concepts, briefly explain the why, and prefer simple, idiomatic code over clever abstractions.

## Working rules

- Write in clear, simple English, in code comments, docs and replies.
- If a requirement is unclear, ask the user key questions before proceeding.
- Record changes in the change log in the `docs/` folder (`docs/CHANGELOG.md`). Add an entry whenever you make a notable change.

## Current state

A Cargo binary project named `rust_gpu_renderer` (edition 2024). It opens a window and draws a spinning, coloured 3D cube with wgpu (vertex and index buffers, a uniform buffer for the matrix, a depth buffer, back-face culling). Tested on Windows 11 (MSVC toolchain, RTX 3090) with both the Vulkan and DX12 backends.

Dependencies: `wgpu` 30, `winit` 0.30, `pollster` (runs async setup code), `bytemuck` (turns vertex data into bytes), `env_logger` and `log`. wgpu 30 changed several APIs compared with older tutorials. For example, frames are shown with `queue.present(frame)`, `get_current_texture()` returns a `CurrentSurfaceTexture` enum, `VertexState::buffers` takes `Option<VertexBufferLayout>` items, `PipelineLayoutDescriptor::bind_group_layouts` takes `Option<&BindGroupLayout>` items, and `DepthStencilState`'s `depth_write_enabled` and `depth_compare` are `Option`s.

No math library: the owner chose a small hand-written `Mat4` in `src/math.rs` to learn how the matrices work. Extend it when new math is needed rather than adding `glam` or `cgmath`. Check the crate source in `~/.cargo/registry/src` when unsure.

## Commands

- `cargo build`: compile
- `cargo run`: compile and run
- `$env:WGPU_BACKEND="vulkan"; cargo run` (PowerShell): force a backend (`vulkan`, `dx12`, `metal`, `gl`). The chosen GPU and backend are printed at startup.
- `cargo test`: run all tests. `cargo test <name>` runs tests whose name contains `<name>`.
- `cargo clippy`: lint
- `cargo fmt`: format code

## Architecture

Decided:

- Must be cross-platform.
- The owner wants to learn three GPU APIs: wgpu, Vulkan and DX12.
- Start with **wgpu**. wgpu runs on top of Vulkan, DX12 and Metal, so one codebase works everywhere, and the backend can be picked at startup (for example Vulkan or DX12 on Windows).
- Plan (option B): wgpu first, then a raw Vulkan backend (`ash`), then a raw DX12 backend (`windows` crate). Later these will sit behind one shared Rust trait. Don't add that trait until the second backend exists.
- Windowing: `winit` 0.30, using the `ApplicationHandler` trait. Shaders: WGSL.

Code layout:

- `src/main.rs`: creates the window and handles winit events (resize, redraw, close).
- `src/renderer.rs`: all wgpu code. `Renderer::new` sets up the instance, surface, adapter, device, queue, pipeline and buffers. `render()` draws one frame.
- `src/vertex.rs`: the `Vertex` struct (`#[repr(C)]`, bytemuck `Pod`), its buffer layout, and the cube's vertices and indices (counter-clockwise from outside, because back faces are culled).
- `src/math.rs`: `Vec3` and `Mat4`. Right-handed, depth 0 to 1 (wgpu/Vulkan/DX12, not OpenGL's -1 to 1), column-major (`cols[column][row]`, same as WGSL). `a * b` applies `b` first. Has unit tests.
- `src/shader.wgsl`: vertex and fragment shaders. `@location` numbers must match `Vertex::ATTRIBUTES`. `@group(0) @binding(0)` is the uniform buffer holding the `mvp` matrix.

Roadmap: triangle (done) → vertex buffer (done) → spinning 3D cube (done) → load models → raw Vulkan backend → raw DX12 backend.
