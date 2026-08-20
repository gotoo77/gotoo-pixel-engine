mod audio;
mod audio_wav;
#[cfg(not(target_arch = "wasm32"))]
mod capture_mirror;
mod control;
pub mod framebuffer;
mod gamepad;
mod gamepad_profile;
mod image;
mod input;
mod pixel;
mod platform;
mod renderer;
mod sprite;
mod storage;
pub mod ui;
mod viewport;

pub use audio::{Audio, AudioError, NoopAudio, PlaybackId, SoundBank, SoundId};
pub use audio_wav::pcm16_mono_wav;
#[cfg(not(target_arch = "wasm32"))]
pub use capture_mirror::ObsMirrorGame;
pub use control::{ActionId, ControlBinding, ControlMap};
pub use framebuffer::{Font, Framebuffer};
pub use gamepad_profile::{AxisCalibration, GamepadProfile};
pub use image::{Image, ImageError, ImageRegion};
pub use input::{
    ButtonState, GamepadButton, GamepadConnectionEvent, GamepadDeviceInfo, GamepadId, Input, Key,
    MouseButton, Touch, TouchPhase,
};
pub use pixel::Pixel;
pub use platform::{EngineConfig, EngineError, Frame, Game, GameResult, run};
pub use sprite::{Sprite, SpriteError};
pub use storage::{LocalStorage, NoopStorage, StorageError};
pub use viewport::{Rect, Size, Viewport};
