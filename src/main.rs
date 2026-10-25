mod math;
mod obj;
mod renderer;
mod vertex;

use std::sync::Arc;

use renderer::Renderer;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

/// Our app state. winit calls the methods below when something happens
/// (the app starts, the window resizes, the window needs redrawing, ...).
///
/// `renderer` is an `Option` because we can't create the window until winit
/// calls `resumed`, so it starts out as `None`.
#[derive(Default)]
struct App {
    renderer: Option<Renderer>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // `resumed` can be called more than once (on mobile, for example), so only set up once.
        if self.renderer.is_some() {
            return;
        }

        let attributes = Window::default_attributes().with_title("Rust GPU Renderer");
        let window = Arc::new(
            event_loop
                .create_window(attributes)
                .expect("failed to create window"),
        );

        // `block_on` runs the async setup and waits for it to finish.
        let renderer = pollster::block_on(Renderer::new(window.clone()));
        self.renderer = Some(renderer);
        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        // `let ... else` returns early if there is no renderer yet.
        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => renderer.resize(size.width, size.height),
            WindowEvent::RedrawRequested => renderer.render(),
            _ => {}
        }
    }
}

fn main() {
    // Prints wgpu warnings and errors. Set RUST_LOG=info for more detail.
    env_logger::init();

    let event_loop = EventLoop::new().expect("failed to create event loop");
    let mut app = App::default();
    event_loop.run_app(&mut app).expect("event loop failed");
}
