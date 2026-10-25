# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project goal

The owner is learning Rust and building a GPU renderer in it. Treat this as a learning project: when introducing Rust concepts (ownership, lifetimes, traits, etc.) or GPU concepts, briefly explain the why, and prefer simple, idiomatic code over clever abstractions.

## Working rules

- Write in clear, simple English, in code comments, docs and replies.
- If a requirement is unclear, ask the user key questions before proceeding.
- Record changes in the change log in the `docs/` folder (`docs/CHANGELOG.md`). Add an entry whenever you make a notable change.

## Current state

A Cargo binary project named `rust_gpu_renderer` (edition 2024). It opens a window and draws a coloured square (two triangles, from a vertex buffer and an index buffer) with wgpu. Tested on Windows 11 (MSVC toolchain, RTX 3090) with both the Vulkan and DX12 backends.

Dependencies: `wgpu` 30, `winit` 0.30, `pollster` (runs async setup code), `bytemuck` (turns vertex data into bytes), `env_logger` and `log`. wgpu 30 changed several APIs compared with older tutorials. For example, frames are shown with `queue.present(frame)`, `get_current_texture()` returns a `CurrentSurfaceTexture` enum, and `VertexState::buffers` takes `Option<VertexBufferLayout>` items. Check the crate source in `~/.cargo/registry/src` when unsure.

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
- `src/vertex.rs`: the `Vertex` struct (`#[repr(C)]`, bytemuck `Pod`), its buffer layout, and the square's vertices and indices.
- `src/shader.wgsl`: vertex and fragment shaders. `@location` numbers must match `Vertex::ATTRIBUTES`.

Roadmap: triangle (done) → vertex buffer (done) → spinning 3D cube (uniforms, depth buffer) → load models → raw Vulkan backend → raw DX12 backend.
