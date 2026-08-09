mod demo;
mod framebuffer;
mod renderer;

use std::sync::Arc;
use std::time::Instant;

use demo::Demo;
use framebuffer::Framebuffer;
use renderer::{RenderOutcome, Renderer};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

const FRAMEBUFFER_WIDTH: u32 = 320;
const FRAMEBUFFER_HEIGHT: u32 = 180;

fn main() {
    let event_loop = EventLoop::new().expect("failed to create event loop");
    let mut app = App::new();
    event_loop.run_app(&mut app).expect("event loop failed");
}

struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    framebuffer: Framebuffer,
    demo: Demo,
    last_frame_at: Instant,
    fps_timer: Instant,
    fps_frames: u32,
}

impl App {
    fn new() -> Self {
        let now = Instant::now();

        Self {
            window: None,
            renderer: None,
            framebuffer: Framebuffer::new(FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT),
            demo: Demo::new(),
            last_frame_at: now,
            fps_timer: now,
            fps_frames: 0,
        }
    }

    fn render_frame(&mut self) {
        let Some(window) = self.window.as_ref() else {
            return;
        };
        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };

        let now = Instant::now();
        let dt = now.duration_since(self.last_frame_at);
        self.last_frame_at = now;

        self.demo.update(dt, &mut self.framebuffer);

        match renderer.render(&self.framebuffer) {
            RenderOutcome::Presented => {
                self.fps_frames += 1;
                let elapsed = self.fps_timer.elapsed();
                if elapsed.as_secs_f32() >= 0.5 {
                    let fps = self.fps_frames as f32 / elapsed.as_secs_f32();
                    window.set_title(&format!(
                        "gotoo-pixel-engine M0 | {:.1} ms | {:.0} FPS",
                        dt.as_secs_f64() * 1_000.0,
                        fps
                    ));
                    self.fps_timer = now;
                    self.fps_frames = 0;
                }
            }
            RenderOutcome::SurfaceChanged => renderer.resize(window.inner_size()),
            RenderOutcome::Skipped => {}
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attributes = Window::default_attributes()
            .with_title("gotoo-pixel-engine M0")
            .with_inner_size(LogicalSize::new(960.0, 540.0))
            .with_min_inner_size(LogicalSize::new(320.0, 180.0));

        let window = Arc::new(
            event_loop
                .create_window(attributes)
                .expect("failed to create window"),
        );

        let renderer = pollster::block_on(Renderer::new(
            Arc::clone(&window),
            FRAMEBUFFER_WIDTH,
            FRAMEBUFFER_HEIGHT,
        ))
        .expect("failed to initialize renderer");

        self.last_frame_at = Instant::now();
        self.fps_timer = self.last_frame_at;
        self.window = Some(window);
        self.renderer = Some(renderer);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.window.as_ref() else {
            return;
        };

        if window.id() != window_id {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize(size);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    match event.physical_key {
                        PhysicalKey::Code(KeyCode::Escape) => event_loop.exit(),
                        PhysicalKey::Code(KeyCode::Space) => self.demo.toggle_palette(),
                        _ => {}
                    }
                }
            }
            WindowEvent::RedrawRequested => self.render_frame(),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}
