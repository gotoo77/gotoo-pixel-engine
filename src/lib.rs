mod audio;
mod audio_wav;
mod bitmap_font;
#[cfg(not(target_arch = "wasm32"))]
mod capture_mirror;
#[cfg(target_arch = "wasm32")]
mod capture_mirror_stub;
mod control;
pub mod framebuffer;
mod gamepad;
mod gamepad_profile;
mod image;
mod input;
mod pixel;
mod platform;
mod renderer;
mod sfx_manifest;
mod sprite;
mod storage;
mod text;
pub mod ui;
mod viewport;
mod window_icon;

pub use audio::{Audio, AudioBus, AudioError, NoopAudio, PlaybackId, SoundBank, SoundId};
pub use audio_wav::pcm16_mono_wav;
pub use bitmap_font::{BitmapFont, BitmapGlyph, BitmapTextRenderer};
#[cfg(not(target_arch = "wasm32"))]
pub use capture_mirror::ObsMirrorGame;
#[cfg(target_arch = "wasm32")]
pub use capture_mirror_stub::ObsMirrorGame;
pub use control::{ActionId, ControlBinding, ControlMap};
pub use framebuffer::{Font, Framebuffer};
pub use gamepad_profile::{AxisCalibration, GamepadProfile};
pub use image::{Image, ImageError, ImageRegion};
pub use input::{
    ButtonState, GamepadButton, GamepadConnectionEvent, GamepadDeviceInfo, GamepadId, Input, Key,
    MouseButton, Touch, TouchPhase,
};
pub use pixel::Pixel;
pub use platform::{
    EngineConfig, EngineError, Frame, Game, GameResult, ToolFrame, ToolWindowConfig,
    ToolWindowMode, run, tool_window_supported,
};
pub use sfx_manifest::SfxManifest;
pub use sprite::{Sprite, SpriteError};
pub use storage::{LocalStorage, NoopStorage, StorageError};
pub use text::TextRenderer;
pub use viewport::{Rect, Size, Viewport};
pub use window_icon::{set_window_icon, set_window_icon_png};
