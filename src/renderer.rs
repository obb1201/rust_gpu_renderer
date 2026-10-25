//! All the wgpu (GPU) code lives here.
//!
//! The main wgpu objects, in the order we create them:
//! - `Instance`: the entry point to wgpu. It finds the GPU APIs (Vulkan, DX12, Metal...).
//! - `Surface`: the part of the window we draw into.
//! - `Adapter`: one physical GPU, reached through one API (for example "RTX 4070 via Vulkan").
//! - `Device`: our open connection to the adapter. We use it to create GPU resources.
//! - `Queue`: where we send finished work for the GPU to run.
//! - `RenderPipeline`: the shaders plus all the fixed settings for drawing.

use std::sync::Arc;
use std::time::Instant;

use winit::window::Window;

const TITLE: &str = "Rust GPU Renderer";

pub struct Renderer {
    // `Arc` (atomic reference counted pointer) lets the window have two owners:
    // the surface and us. The window is freed only when both are done with it.
    window: Arc<Window>,
    // `'static` means the surface does not borrow anything. It holds its own
    // `Arc` to the window, so it can never outlive it.
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    // Shown in the window title, for example "NVIDIA GeForce RTX 3090 | Dx12 | driver 581.57".
    gpu_label: String,
    // Frame rate counter: we count frames and update the title once per second.
    frame_count: u32,
    fps_timer: Instant,
}

impl Renderer {
    /// Sets up the GPU. This is `async` because finding a GPU and opening a device
    /// can take time. `main.rs` waits for it with `pollster::block_on`.
    pub async fn new(window: Arc<Window>) -> Renderer {
        // `_from_env` lets you choose the backend with the WGPU_BACKEND
        // environment variable, for example `vulkan` or `dx12`.
        // With no variable set, wgpu picks the best one for this platform.
        let instance =
            wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());

        // `window.clone()` copies the `Arc` pointer, not the window itself.
        let surface = instance
            .create_surface(window.clone())
            .expect("failed to create surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .expect("no GPU adapter found");

        let info = adapter.get_info();
        let mut gpu_label = format!("{} | {:?}", info.name, info.backend);
        // Some backends (DX12, for example) don't report a driver version.
        if !info.driver_info.is_empty() {
            gpu_label.push_str(&format!(" | driver {}", info.driver_info));
        }
        println!("Using GPU: {gpu_label}");
        window.set_title(&format!("{TITLE} | {gpu_label}"));

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .expect("failed to create device");

        let size = window.inner_size();
        let mut config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .expect("surface is not supported by this adapter");
        // Prefer an sRGB format so colours look correct on screen.
        let caps = surface.get_capabilities(&adapter);
        if let Some(format) = caps.formats.iter().find(|f| f.is_srgb()) {
            config.format = *format;
        }
        // Fifo means "wait for vsync". Every platform supports it.
        config.present_mode = wgpu::PresentMode::Fifo;
        surface.configure(&device, &config);

        let pipeline = create_pipeline(&device, config.format);

        Renderer {
            window,
            surface,
            device,
            queue,
            config,
            pipeline,
            gpu_label,
            frame_count: 0,
            fps_timer: Instant::now(),
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        // A minimised window has size 0, and wgpu does not allow a 0-sized surface.
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn render(&mut self) {
        // Ask the surface for the next image to draw into.
        let (frame, suboptimal) = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => (frame, false),
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => (frame, true),
            // The window is hidden or busy. Skip this frame.
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                self.window.request_redraw();
                return;
            }
            // The surface changed (for example, after a resize). Set it up again.
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                self.window.request_redraw();
                return;
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                log::error!("validation error while getting the surface texture");
                return;
            }
        };

        // A "view" describes how to read or write a texture. The default is fine.
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // We record GPU commands into an encoder. Nothing runs until we submit.
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame encoder"),
            });

        // The extra `{ }` block ends the render pass before `encoder.finish()`.
        // The pass borrows the encoder, and Rust won't let us use the encoder
        // again until that borrow is over.
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("triangle pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        // Fill the screen with dark grey first.
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.05,
                            g: 0.05,
                            b: 0.08,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            // Draw 3 vertices (indices 0, 1, 2) and 1 instance.
            pass.draw(0..3, 0..1);
        }

        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        self.queue.present(frame);

        // The surface can't be reconfigured while we hold a frame, so we do it now.
        if suboptimal {
            self.surface.configure(&self.device, &self.config);
        }

        self.update_fps();

        // Ask for the next frame straight away, so we keep drawing.
        self.window.request_redraw();
    }

    /// Counts frames, and once a second shows the frame rate in the window title.
    /// Note: with `PresentMode::Fifo` (vsync) the rate is capped at the monitor's refresh rate.
    fn update_fps(&mut self) {
        self.frame_count += 1;
        let elapsed = self.fps_timer.elapsed().as_secs_f64();
        if elapsed >= 1.0 {
            let fps = self.frame_count as f64 / elapsed;
            let frame_ms = 1000.0 / fps;
            self.window.set_title(&format!(
                "{TITLE} | {} | {fps:.0} FPS ({frame_ms:.2} ms)",
                self.gpu_label
            ));
            self.frame_count = 0;
            self.fps_timer = Instant::now();
        }
    }
}

fn create_pipeline(device: &wgpu::Device, format: wgpu::TextureFormat) -> wgpu::RenderPipeline {
    // `include_wgsl!` copies shader.wgsl into the program when it compiles.
    let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

    // The layout lists the resources (buffers, textures) the shaders use.
    // Our triangle uses none yet, so it is empty.
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("triangle layout"),
        bind_group_layouts: &[],
        immediate_size: 0,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("triangle pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        // Default: draw triangles from every 3 vertices, counter-clockwise is the front.
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}
