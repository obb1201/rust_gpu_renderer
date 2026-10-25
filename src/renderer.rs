//! All the wgpu (GPU) code lives here.
//!
//! The main wgpu objects, in the order we create them:
//! - `Instance`: the entry point to wgpu. It finds the GPU APIs (Vulkan, DX12, Metal...).
//! - `Surface`: the part of the window we draw into.
//! - `Adapter`: one physical GPU, reached through one API (for example "RTX 4070 via Vulkan").
//! - `Device`: our open connection to the adapter. We use it to create GPU resources.
//! - `Queue`: where we send finished work for the GPU to run.
//! - `RenderPipeline`: the shaders plus all the fixed settings for drawing.
//! - `Buffer`: a block of GPU memory. We use one for vertices, one for indices
//!   and one for the matrix (the uniform buffer).
//! - `BindGroup`: tells the shader which buffers (and later, textures) to use.
//! - `Texture`: an image on the GPU. We use one as the depth buffer.

use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

// `DeviceExt` is a trait that adds `create_buffer_init` to `Device`.
// In Rust, a trait's methods can only be called when the trait is imported.
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::math::{Mat4, Vec3};
use crate::obj;
use crate::vertex::Vertex;

const TITLE: &str = "Rust GPU Renderer";

// `env!` reads a value when the program compiles. CARGO_MANIFEST_DIR is the folder
// that holds Cargo.toml, so this path works whichever folder we run from.
const MODEL_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/teapot.obj");

// 32-bit float depth: one number per pixel, from 0 (near) to 1 (far).
const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

/// What we copy into the uniform buffer each frame. Must match `Uniforms` in shader.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    mvp: Mat4,
    model: Mat4,
}

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
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    // How many indices to draw (3 per triangle).
    index_count: u32,
    // Moves and scales the model so it sits at the origin and fits the view.
    fit_model: Mat4,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    // We only keep the view. The view keeps its texture alive.
    depth_view: wgpu::TextureView,
    // When we started. The model's angle comes from the time since then.
    start_time: Instant,
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

        let mesh = obj::load(Path::new(MODEL_PATH)).unwrap_or_else(|e| panic!("{e}"));
        println!(
            "Loaded {MODEL_PATH}: {} vertices, {} triangles",
            mesh.vertices.len(),
            mesh.indices.len() / 3
        );

        // Models come in any size and position. Move the centre of the model's box to
        // the origin, then scale it so its longest side is 3 units.
        let (min, max) = mesh.bounds();
        let center = (min + max) * 0.5;
        let size = max - min;
        let longest = size.x.max(size.y).max(size.z);
        let fit_model = Mat4::scale(3.0 / longest) * Mat4::translation(center * -1.0);

        // Copy the model's data into GPU memory. `bytemuck::cast_slice` views our
        // `&[Vertex]` as `&[u8]` (raw bytes) without copying anything on the CPU.
        // `usage` says what the buffer is for. Using it any other way is an error.
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex buffer"),
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index buffer"),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        // The uniform buffer holds two matrices. COPY_DST means we're allowed to
        // write new data into it later, which we do every frame.
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("uniform buffer"),
            contents: bytemuck::bytes_of(&Uniforms {
                mvp: Mat4::IDENTITY,
                model: Mat4::IDENTITY,
            }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // A bind group layout describes the shape: "binding 0 is a uniform buffer,
        // used by the vertex shader". The bind group fills that shape with our buffer.
        let uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("uniform layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("uniform bind group"),
            layout: &uniform_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline = create_pipeline(&device, config.format, &uniform_layout);
        let depth_view = create_depth_view(&device, &config);

        Renderer {
            window,
            surface,
            device,
            queue,
            config,
            pipeline,
            vertex_buffer,
            index_buffer,
            index_count: mesh.indices.len() as u32,
            fit_model,
            uniform_buffer,
            uniform_bind_group,
            depth_view,
            start_time: Instant::now(),
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
        // The depth buffer must be the same size as the window, so make a new one.
        self.depth_view = create_depth_view(&self.device, &self.config);
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

        // Work out this frame's matrices and copy them into the uniform buffer.
        // `write_buffer` is queued, so it happens before the draw we submit below.
        let uniforms = self.uniforms();
        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

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
                label: Some("main pass"),
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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        // Start every pixel at 1.0, the furthest possible depth.
                        load: wgpu::LoadOp::Clear(1.0),
                        // We don't need the depth values after this frame.
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            // Group 0 matches `@group(0)` in the shader.
            pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            // Slot 0 matches the first entry in the pipeline's `buffers` list.
            // `slice(..)` means "the whole buffer".
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            // Uint32 must match the `u32` type of `Mesh::indices`.
            pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            // Draw all the indices, starting at vertex 0, with 1 instance.
            pass.draw_indexed(0..self.index_count, 0, 0..1);
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

    /// Builds this frame's matrices: the model matrix, and projection * view * model.
    fn uniforms(&self) -> Uniforms {
        // The angle comes from real time, not from the frame count,
        // so the model spins at the same speed at any frame rate.
        let t = self.start_time.elapsed().as_secs_f32();
        // First fit the model to the view, then turn it.
        let model = Mat4::rotation_y(t * 0.6) * self.fit_model;

        // The camera sits a little above and in front of the model, looking at its centre.
        let view = Mat4::look_at(
            Vec3::new(0.0, 2.0, 6.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        );

        let aspect = self.config.width as f32 / self.config.height as f32;
        let proj = Mat4::perspective(45f32.to_radians(), aspect, 0.1, 100.0);

        Uniforms {
            mvp: proj * view * model,
            model,
        }
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

/// Makes a depth texture the same size as the window, and returns a view of it.
///
/// The depth buffer stores how far away the nearest thing drawn so far is, for each pixel.
/// A new pixel is only drawn if it is closer, so the front of the model hides the back.
fn create_depth_view(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
) -> wgpu::TextureView {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("depth texture"),
        size: wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}

fn create_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    uniform_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    // `include_wgsl!` copies shader.wgsl into the program when it compiles.
    let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

    // The layout lists the bind groups the shaders use. Entry 0 is `@group(0)`.
    // Each entry is an `Option` because wgpu 30 allows empty groups.
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("main layout"),
        bind_group_layouts: &[Some(uniform_layout)],
        immediate_size: 0,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("main pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            // One vertex buffer (slot 0), laid out as described by `Vertex::layout()`.
            // Each slot is an `Option` because wgpu 30 allows empty slots.
            buffers: &[Some(Vertex::layout())],
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
        primitive: wgpu::PrimitiveState {
            // Counter-clockwise triangles are the front.
            front_face: wgpu::FrontFace::Ccw,
            // Skip triangles that face away from the camera (the back of the model).
            // We can't see them anyway, so this saves the GPU work.
            cull_mode: Some(wgpu::Face::Back),
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            depth_write_enabled: Some(true),
            // Draw a pixel only if it is closer than what is already there.
            depth_compare: Some(wgpu::CompareFunction::Less),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}
