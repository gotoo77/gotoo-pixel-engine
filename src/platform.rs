use std::fmt;
use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};
#[cfg(target_arch = "wasm32")]
use web_time::{Duration, Instant};

use crate::Framebuffer;
use crate::audio::{Audio, PlatformAudio, platform_audio};
use crate::gamepad::GamepadInputBackend;
use crate::input::{Input, Key, MouseButton, Touch, TouchPhase};
use crate::renderer::{RenderOutcome, Renderer, RendererInitError};
use crate::storage::{LocalStorage, platform_storage};
use crate::{GamepadId, GamepadProfile, Size, Viewport};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, WindowEvent};
#[cfg(target_arch = "wasm32")]
use winit::event_loop::EventLoopProxy;
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

    fn gamepad_profile(&self, _id: GamepadId) -> Option<GamepadProfile> {
        None
    }
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
    pub storage: &'a mut dyn LocalStorage,
    pub audio: &'a mut dyn Audio,
    pub surface_size: Size,
    pub viewport: Viewport,
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

    let event_loop = EventLoop::<PlatformEvent>::with_user_event()
        .build()
        .map_err(EngineError::event_loop)?;
    #[cfg(target_arch = "wasm32")]
    let mut app = PlatformApp::new(config, game, event_loop.create_proxy());
    #[cfg(not(target_arch = "wasm32"))]
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

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
enum PlatformEvent {
    RendererReady(Result<Renderer, RendererInitError>),
}

struct PlatformApp<G> {
    config: EngineConfig,
    game: G,
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    framebuffer: Framebuffer,
    input: Input,
    gamepads: GamepadInputBackend,
    storage: Box<dyn LocalStorage>,
    audio: Box<dyn PlatformAudio>,
    last_frame_at: Instant,
    fps_timer: Instant,
    fps_frames: u32,
    last_non_zero_window_size: Option<PhysicalSize<u32>>,
    pending_error: Option<EngineError>,
    #[cfg(target_arch = "wasm32")]
    event_loop_proxy: EventLoopProxy<PlatformEvent>,
}

impl<G: Game> PlatformApp<G> {
    #[cfg(not(target_arch = "wasm32"))]
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
            gamepads: GamepadInputBackend::default(),
            storage: platform_storage(),
            audio: platform_audio(),
            last_frame_at: now,
            fps_timer: now,
            fps_frames: 0,
            last_non_zero_window_size: None,
            pending_error: None,
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn new(config: EngineConfig, game: G, event_loop_proxy: EventLoopProxy<PlatformEvent>) -> Self {
        let now = Instant::now();
        let framebuffer = Framebuffer::new(config.framebuffer_width, config.framebuffer_height);

        Self {
            config,
            game,
            window: None,
            renderer: None,
            framebuffer,
            input: Input::default(),
            gamepads: GamepadInputBackend::default(),
            storage: platform_storage(),
            audio: platform_audio(),
            last_frame_at: now,
            fps_timer: now,
            fps_frames: 0,
            last_non_zero_window_size: None,
            pending_error: None,
            event_loop_proxy,
        }
    }

    fn render_frame(&mut self, event_loop: &ActiveEventLoop) {
        let Some(window) = self.window.as_ref() else {
            return;
        };
        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };

        {
            let game = &self.game;
            self.gamepads
                .poll(&mut self.input, |id| game.gamepad_profile(id));
        }

        let now = Instant::now();
        let dt = now.duration_since(self.last_frame_at);
        self.last_frame_at = now;
        let surface_size = size_from_physical(window.inner_size());
        let viewport = renderer.viewport();

        let mut frame = Frame {
            framebuffer: &mut self.framebuffer,
            input: &self.input,
            delta_time: dt,
            storage: self.storage.as_mut(),
            audio: self.audio.as_mut(),
            surface_size,
            viewport,
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
            RenderOutcome::SurfaceChanged => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize(window.inner_size());
                }
            }
            RenderOutcome::Skipped => {}
        }

        self.input.advance_frame();
    }

    fn update_mouse_position(&mut self, position: PhysicalPosition<f64>) {
        let position = self.framebuffer_position(position);
        self.input.set_mouse_position(position);
    }

    fn update_touch(&mut self, touch: winit::event::Touch) {
        let Some(window) = self.window.as_ref() else {
            return;
        };

        if touch.phase == winit::event::TouchPhase::Started {
            self.audio.activate();
        }

        self.input.push_touch(touch_from_winit(
            touch.id,
            touch.phase,
            touch.location,
            current_viewport(
                window.inner_size(),
                self.config.framebuffer_width,
                self.config.framebuffer_height,
            ),
        ));
    }

    fn framebuffer_position(&self, position: PhysicalPosition<f64>) -> Option<(i32, i32)> {
        let window = self.window.as_ref()?;

        surface_to_framebuffer_position(
            position,
            current_viewport(
                window.inner_size(),
                self.config.framebuffer_width,
                self.config.framebuffer_height,
            ),
        )
    }

    fn create_window_and_renderer(&mut self, event_loop: &ActiveEventLoop) {
        let attributes = Window::default_attributes()
            .with_title(self.config.title.clone())
            .with_inner_size(LogicalSize::new(
                f64::from(self.config.window_width),
                f64::from(self.config.window_height),
            ))
            .with_min_inner_size(LogicalSize::new(1.0, 1.0));

        #[cfg(target_arch = "wasm32")]
        let attributes = {
            use winit::platform::web::WindowAttributesExtWebSys;

            attributes.with_append(true)
        };

        let window = match event_loop.create_window(attributes) {
            Ok(window) => Arc::new(window),
            Err(err) => {
                self.pending_error = Some(EngineError::window(err));
                event_loop.exit();
                return;
            }
        };

        #[cfg(target_arch = "wasm32")]
        {
            self.last_frame_at = Instant::now();
            self.fps_timer = self.last_frame_at;
            self.window = Some(Arc::clone(&window));

            let proxy = self.event_loop_proxy.clone();
            let framebuffer_width = self.config.framebuffer_width;
            let framebuffer_height = self.config.framebuffer_height;

            wasm_bindgen_futures::spawn_local(async move {
                let result = Renderer::new(window, framebuffer_width, framebuffer_height).await;
                let _ = proxy.send_event(PlatformEvent::RendererReady(result));
            });

            return;
        }

        #[cfg(not(target_arch = "wasm32"))]
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

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.last_frame_at = Instant::now();
            self.fps_timer = self.last_frame_at;
            self.window = Some(window);
            self.renderer = Some(renderer);
        }
    }

    fn finish_renderer_init(
        &mut self,
        event_loop: &ActiveEventLoop,
        result: Result<Renderer, RendererInitError>,
    ) {
        let mut renderer = match result {
            Ok(renderer) => renderer,
            Err(err) => {
                self.pending_error = Some(EngineError::renderer(err));
                event_loop.exit();
                return;
            }
        };

        self.last_frame_at = Instant::now();
        self.fps_timer = self.last_frame_at;
        if let Some(size) = self.last_non_zero_window_size {
            renderer.resize(size);
        }
        self.renderer = Some(renderer);

        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}

impl<G: Game> ApplicationHandler<PlatformEvent> for PlatformApp<G> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        self.create_window_and_renderer(event_loop);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: PlatformEvent) {
        match event {
            PlatformEvent::RendererReady(result) => self.finish_renderer_init(event_loop, result),
        }
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
                remember_non_zero_size(&mut self.last_non_zero_window_size, size);
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize(size);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let Some(key) = key_from_winit(event.physical_key) else {
                    return;
                };

                if event.state == ElementState::Pressed {
                    self.audio.activate();
                }

                match event.state {
                    ElementState::Pressed => self.input.press_key(key),
                    ElementState::Released => self.input.release_key(key),
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let Some(button) = mouse_button_from_winit(button) else {
                    return;
                };

                if state == ElementState::Pressed {
                    self.audio.activate();
                }

                match state {
                    ElementState::Pressed => self.input.press_mouse_button(button),
                    ElementState::Released => self.input.release_mouse_button(button),
                }
            }
            WindowEvent::CursorMoved { position, .. } => self.update_mouse_position(position),
            WindowEvent::CursorLeft { .. } => self.input.set_mouse_position(None),
            WindowEvent::Touch(touch) => self.update_touch(touch),
            WindowEvent::Focused(false) => self.input.reset_window_devices(),
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

fn remember_non_zero_size(last_size: &mut Option<PhysicalSize<u32>>, size: PhysicalSize<u32>) {
    if size.width != 0 && size.height != 0 {
        *last_size = Some(size);
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

fn touch_phase_from_winit(phase: winit::event::TouchPhase) -> TouchPhase {
    match phase {
        winit::event::TouchPhase::Started => TouchPhase::Started,
        winit::event::TouchPhase::Moved => TouchPhase::Moved,
        winit::event::TouchPhase::Ended => TouchPhase::Ended,
        winit::event::TouchPhase::Cancelled => TouchPhase::Cancelled,
    }
}

fn touch_from_winit(
    id: u64,
    phase: winit::event::TouchPhase,
    location: PhysicalPosition<f64>,
    viewport: Viewport,
) -> Touch {
    Touch {
        id,
        phase: touch_phase_from_winit(phase),
        position: surface_to_framebuffer_position(location, viewport),
    }
}

fn surface_to_framebuffer_position(
    position: PhysicalPosition<f64>,
    viewport: Viewport,
) -> Option<(i32, i32)> {
    viewport.map_surface_position(position.x, position.y)
}

fn current_viewport(
    window_size: PhysicalSize<u32>,
    framebuffer_width: u32,
    framebuffer_height: u32,
) -> Viewport {
    Viewport::new(
        size_from_physical(window_size),
        Size {
            width: framebuffer_width,
            height: framebuffer_height,
        },
    )
}

fn size_from_physical(size: PhysicalSize<u32>) -> Size {
    Size {
        width: size.width,
        height: size.height,
    }
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
        EngineConfig, Key, MouseButton, TouchPhase, current_viewport, key_from_winit,
        mouse_button_from_winit, remember_non_zero_size, surface_to_framebuffer_position,
        touch_from_winit, touch_phase_from_winit, validate_config,
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
    fn maps_touch_phases() {
        assert_eq!(
            touch_phase_from_winit(winit::event::TouchPhase::Started),
            TouchPhase::Started
        );
        assert_eq!(
            touch_phase_from_winit(winit::event::TouchPhase::Moved),
            TouchPhase::Moved
        );
        assert_eq!(
            touch_phase_from_winit(winit::event::TouchPhase::Ended),
            TouchPhase::Ended
        );
        assert_eq!(
            touch_phase_from_winit(winit::event::TouchPhase::Cancelled),
            TouchPhase::Cancelled
        );
    }

    #[test]
    fn maps_surface_position_to_framebuffer_position_through_viewport() {
        let viewport = current_viewport(PhysicalSize::new(960, 408), 480, 204);

        assert_eq!(
            surface_to_framebuffer_position(PhysicalPosition::new(480.0, 204.0), viewport,),
            Some((240, 102))
        );
    }

    #[test]
    fn maps_touch_position_with_same_viewport_transform() {
        let viewport = current_viewport(PhysicalSize::new(1200, 408), 480, 204);
        let touch = touch_from_winit(
            42,
            winit::event::TouchPhase::Moved,
            PhysicalPosition::new(600.0, 204.0),
            viewport,
        );

        assert_eq!(touch.id, 42);
        assert_eq!(touch.phase, TouchPhase::Moved);
        assert_eq!(touch.position, Some((240, 102)));
        assert_eq!(
            surface_to_framebuffer_position(PhysicalPosition::new(600.0, 204.0), viewport),
            touch.position
        );
    }

    #[test]
    fn preserves_touch_events_outside_viewport() {
        let viewport = current_viewport(PhysicalSize::new(1200, 408), 480, 204);
        let touch = touch_from_winit(
            42,
            winit::event::TouchPhase::Ended,
            PhysicalPosition::new(119.0, 10.0),
            viewport,
        );

        assert_eq!(touch.id, 42);
        assert_eq!(touch.phase, TouchPhase::Ended);
        assert_eq!(touch.position, None);
    }

    #[test]
    fn preserves_touch_events_when_window_size_is_invalid() {
        let viewport = current_viewport(PhysicalSize::new(0, 540), 480, 204);
        let touch = touch_from_winit(
            42,
            winit::event::TouchPhase::Cancelled,
            PhysicalPosition::new(10.0, 10.0),
            viewport,
        );

        assert_eq!(touch.id, 42);
        assert_eq!(touch.phase, TouchPhase::Cancelled);
        assert_eq!(touch.position, None);
    }

    #[test]
    fn rejects_mouse_position_outside_viewport() {
        let viewport = current_viewport(PhysicalSize::new(1200, 408), 480, 204);

        assert_eq!(
            surface_to_framebuffer_position(PhysicalPosition::new(1080.0, 10.0), viewport),
            None
        );
    }

    #[test]
    fn current_viewport_recalculates_after_resize() {
        let first = current_viewport(PhysicalSize::new(960, 408), 480, 204);
        let second = current_viewport(PhysicalSize::new(1200, 408), 480, 204);

        assert_ne!(first.rect, second.rect);
        assert_eq!(second.rect.x, 120);
    }

    #[test]
    fn remember_non_zero_size_keeps_latest_non_zero_size() {
        let mut size = None;

        remember_non_zero_size(&mut size, PhysicalSize::new(960, 540));
        remember_non_zero_size(&mut size, PhysicalSize::new(1280, 720));

        assert_eq!(size, Some(PhysicalSize::new(1280, 720)));
    }

    #[test]
    fn remember_non_zero_size_ignores_zero_dimensions() {
        let mut size = Some(PhysicalSize::new(960, 540));

        remember_non_zero_size(&mut size, PhysicalSize::new(0, 540));
        remember_non_zero_size(&mut size, PhysicalSize::new(960, 0));
        remember_non_zero_size(&mut size, PhysicalSize::new(0, 0));

        assert_eq!(size, Some(PhysicalSize::new(960, 540)));
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
