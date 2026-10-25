# Change Log

All notable changes to this project are recorded here. Newest entries go first.

## 2026-10-25

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
