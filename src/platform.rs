use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::Framebuffer;
use crate::input::{Input, Key, MouseButton};
use crate::renderer::{RenderOutcome, Renderer, RendererInitError};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineConfig {
    pub title: String,
    pub framebuffer_width: u32,
    pub framebuffer_height: u32,
    pub window_width: u32,
    pub window_height: u32,
}

pub trait Game {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameResult {
    Continue,
    Exit,
}

pub struct Frame<'a> {
    pub framebuffer: &'a mut Framebuffer,
    pub input: &'a Input,
    pub delta_time: Duration,
}

#[derive(Debug)]
pub struct EngineError {
    message: String,
}

impl EngineError {
    fn config(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    fn event_loop(err: winit::error::EventLoopError) -> Self {
        Self {
            message: format!("event loop failed: {err}"),
        }
    }

    fn window(err: winit::error::OsError) -> Self {
        Self {
            message: format!("window creation failed: {err}"),
        }
    }

    fn renderer(err: RendererInitError) -> Self {
        Self {
            message: format!("renderer failed: {err}"),
        }
    }
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for EngineError {}

pub fn run<G: Game + 'static>(config: EngineConfig, game: G) -> Result<(), EngineError> {
    validate_config(&config)?;

    let event_loop = EventLoop::new().map_err(EngineError::event_loop)?;
    let mut app = PlatformApp::new(config, game);
    event_loop
        .run_app(&mut app)
        .map_err(EngineError::event_loop)?;

    if let Some(err) = app.pending_error {
        Err(err)
    } else {
        Ok(())
    }
}

struct PlatformApp<G> {
    config: EngineConfig,
    game: G,
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    framebuffer: Framebuffer,
    input: Input,
    last_frame_at: Instant,
    fps_timer: Instant,
    fps_frames: u32,
    pending_error: Option<EngineError>,
}

impl<G: Game> PlatformApp<G> {
    fn new(config: EngineConfig, game: G) -> Self {
        let now = Instant::now();
        let framebuffer = Framebuffer::new(config.framebuffer_width, config.framebuffer_height);

        Self {
            config,
            game,
            window: None,
            renderer: None,
            framebuffer,
            input: Input::default(),
            last_frame_at: now,
            fps_timer: now,
            fps_frames: 0,
            pending_error: None,
        }
    }

    fn render_frame(&mut self, event_loop: &ActiveEventLoop) {
        let Some(window) = self.window.as_ref() else {
            return;
        };
        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };

        let now = Instant::now();
        let dt = now.duration_since(self.last_frame_at);
        self.last_frame_at = now;

        let mut frame = Frame {
            framebuffer: &mut self.framebuffer,
            input: &self.input,
            delta_time: dt,
        };

        if self.game.update(&mut frame) == GameResult::Exit {
            event_loop.exit();
            return;
        }

        match renderer.render(&self.framebuffer) {
            RenderOutcome::Presented => {
                self.fps_frames += 1;
                let elapsed = self.fps_timer.elapsed();
                if elapsed.as_secs_f32() >= 0.5 {
                    let fps = self.fps_frames as f32 / elapsed.as_secs_f32();
                    window.set_title(&format!(
                        "{} | {:.1} ms | {:.0} FPS",
                        self.config.title,
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

        self.input.advance_frame();
    }

    fn update_mouse_position(&mut self, position: PhysicalPosition<f64>) {
        let Some(window) = self.window.as_ref() else {
            return;
        };

        self.input
            .set_mouse_position(window_to_framebuffer_position(
                position,
                window.inner_size(),
                self.config.framebuffer_width,
                self.config.framebuffer_height,
            ));
    }

    fn create_window_and_renderer(&mut self, event_loop: &ActiveEventLoop) {
        let attributes = Window::default_attributes()
            .with_title(self.config.title.clone())
            .with_inner_size(LogicalSize::new(
                f64::from(self.config.window_width),
                f64::from(self.config.window_height),
            ))
            .with_min_inner_size(LogicalSize::new(1.0, 1.0));

        let window = match event_loop.create_window(attributes) {
            Ok(window) => Arc::new(window),
            Err(err) => {
                self.pending_error = Some(EngineError::window(err));
                event_loop.exit();
                return;
            }
        };

        let renderer = match pollster::block_on(Renderer::new(
            Arc::clone(&window),
            self.config.framebuffer_width,
            self.config.framebuffer_height,
        )) {
            Ok(renderer) => renderer,
            Err(err) => {
                self.pending_error = Some(EngineError::renderer(err));
                event_loop.exit();
                return;
            }
        };

        self.last_frame_at = Instant::now();
        self.fps_timer = self.last_frame_at;
        self.window = Some(window);
        self.renderer = Some(renderer);
    }
}

impl<G: Game> ApplicationHandler for PlatformApp<G> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        self.create_window_and_renderer(event_loop);
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
                let Some(key) = key_from_winit(event.physical_key) else {
                    return;
                };

                match event.state {
                    ElementState::Pressed => self.input.press_key(key),
                    ElementState::Released => self.input.release_key(key),
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let Some(button) = mouse_button_from_winit(button) else {
                    return;
                };

                match state {
                    ElementState::Pressed => self.input.press_mouse_button(button),
                    ElementState::Released => self.input.release_mouse_button(button),
                }
            }
            WindowEvent::CursorMoved { position, .. } => self.update_mouse_position(position),
            WindowEvent::CursorLeft { .. } => self.input.set_mouse_position(None),
            WindowEvent::Focused(false) => self.input.reset(),
            WindowEvent::RedrawRequested => self.render_frame(event_loop),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}

fn key_from_winit(key: PhysicalKey) -> Option<Key> {
    match key {
        PhysicalKey::Code(KeyCode::Escape) => Some(Key::Escape),
        PhysicalKey::Code(KeyCode::Space) => Some(Key::Space),
        PhysicalKey::Code(KeyCode::ArrowUp) => Some(Key::Up),
        PhysicalKey::Code(KeyCode::ArrowDown) => Some(Key::Down),
        PhysicalKey::Code(KeyCode::ArrowLeft) => Some(Key::Left),
        PhysicalKey::Code(KeyCode::ArrowRight) => Some(Key::Right),
        PhysicalKey::Code(KeyCode::KeyA) => Some(Key::A),
        PhysicalKey::Code(KeyCode::KeyD) => Some(Key::D),
        PhysicalKey::Code(KeyCode::KeyS) => Some(Key::S),
        PhysicalKey::Code(KeyCode::KeyW) => Some(Key::W),
        _ => None,
    }
}

fn mouse_button_from_winit(button: winit::event::MouseButton) -> Option<MouseButton> {
    match button {
        winit::event::MouseButton::Left => Some(MouseButton::Left),
        winit::event::MouseButton::Right => Some(MouseButton::Right),
        winit::event::MouseButton::Middle => Some(MouseButton::Middle),
        _ => None,
    }
}

fn window_to_framebuffer_position(
    position: PhysicalPosition<f64>,
    window_size: PhysicalSize<u32>,
    framebuffer_width: u32,
    framebuffer_height: u32,
) -> Option<(i32, i32)> {
    if window_size.width == 0 || window_size.height == 0 {
        return None;
    }

    let x = (position.x * f64::from(framebuffer_width) / f64::from(window_size.width)).floor();
    let y = (position.y * f64::from(framebuffer_height) / f64::from(window_size.height)).floor();

    if x < 0.0 || y < 0.0 || x >= f64::from(framebuffer_width) || y >= f64::from(framebuffer_height)
    {
        return None;
    }

    Some((x as i32, y as i32))
}

fn validate_config(config: &EngineConfig) -> Result<(), EngineError> {
    if config.framebuffer_width == 0 {
        return Err(EngineError::config(
            "framebuffer_width must be greater than 0",
        ));
    }
    if config.framebuffer_height == 0 {
        return Err(EngineError::config(
            "framebuffer_height must be greater than 0",
        ));
    }
    if config.window_width == 0 {
        return Err(EngineError::config("window_width must be greater than 0"));
    }
    if config.window_height == 0 {
        return Err(EngineError::config("window_height must be greater than 0"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        EngineConfig, Key, MouseButton, key_from_winit, mouse_button_from_winit, validate_config,
        window_to_framebuffer_position,
    };
    use winit::dpi::{PhysicalPosition, PhysicalSize};
    use winit::keyboard::{KeyCode, PhysicalKey};

    #[test]
    fn maps_minimal_keyboard_keys() {
        assert_eq!(
            key_from_winit(PhysicalKey::Code(KeyCode::Escape)),
            Some(Key::Escape)
        );
        assert_eq!(
            key_from_winit(PhysicalKey::Code(KeyCode::ArrowLeft)),
            Some(Key::Left)
        );
        assert_eq!(
            key_from_winit(PhysicalKey::Code(KeyCode::KeyW)),
            Some(Key::W)
        );
        assert_eq!(key_from_winit(PhysicalKey::Code(KeyCode::F1)), None);
    }

    #[test]
    fn maps_minimal_mouse_buttons() {
        assert_eq!(
            mouse_button_from_winit(winit::event::MouseButton::Left),
            Some(MouseButton::Left)
        );
        assert_eq!(
            mouse_button_from_winit(winit::event::MouseButton::Other(4)),
            None
        );
    }

    #[test]
    fn maps_window_position_to_framebuffer_position() {
        assert_eq!(
            window_to_framebuffer_position(
                PhysicalPosition::new(480.0, 270.0),
                PhysicalSize::new(960, 540),
                320,
                180,
            ),
            Some((160, 90))
        );
    }

    #[test]
    fn rejects_mouse_position_outside_framebuffer() {
        assert_eq!(
            window_to_framebuffer_position(
                PhysicalPosition::new(960.0, 10.0),
                PhysicalSize::new(960, 540),
                320,
                180,
            ),
            None
        );
    }

    #[test]
    fn config_validation_accepts_positive_dimensions() {
        let config = EngineConfig {
            title: "test".into(),
            framebuffer_width: 320,
            framebuffer_height: 180,
            window_width: 960,
            window_height: 540,
        };

        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn config_validation_rejects_zero_framebuffer_width() {
        let config = EngineConfig {
            title: "test".into(),
            framebuffer_width: 0,
            framebuffer_height: 180,
            window_width: 960,
            window_height: 540,
        };

        let err = validate_config(&config).expect_err("config should be rejected");
        assert_eq!(err.to_string(), "framebuffer_width must be greater than 0");
    }

    #[test]
    fn config_validation_rejects_zero_framebuffer_height() {
        let config = EngineConfig {
            title: "test".into(),
            framebuffer_width: 320,
            framebuffer_height: 0,
            window_width: 960,
            window_height: 540,
        };

        let err = validate_config(&config).expect_err("config should be rejected");
        assert_eq!(err.to_string(), "framebuffer_height must be greater than 0");
    }

    #[test]
    fn config_validation_rejects_zero_window_width() {
        let config = EngineConfig {
            title: "test".into(),
            framebuffer_width: 320,
            framebuffer_height: 180,
            window_width: 0,
            window_height: 540,
        };

        let err = validate_config(&config).expect_err("config should be rejected");
        assert_eq!(err.to_string(), "window_width must be greater than 0");
    }

    #[test]
    fn config_validation_rejects_zero_window_height() {
        let config = EngineConfig {
            title: "test".into(),
            framebuffer_width: 320,
            framebuffer_height: 180,
            window_width: 960,
            window_height: 0,
        };

        let err = validate_config(&config).expect_err("config should be rejected");
        assert_eq!(err.to_string(), "window_height must be greater than 0");
    }
}
