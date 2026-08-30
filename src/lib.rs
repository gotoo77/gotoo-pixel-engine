mod audio;
mod audio_wav;
mod bitmap_font;
#[cfg(not(target_arch = "wasm32"))]
mod capture_mirror;
#[cfg(target_arch = "wasm32")]
mod capture_mirror_stub;
mod control;
#[cfg(feature = "diagnostics")]
mod diagnostics;
pub mod framebuffer;
mod gamepad;
mod gamepad_profile;
mod image;
mod image_fit;
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

pub use audio::{
    Audio, AudioBus, AudioError, NoopAudio, PlaybackId, PlaybackState, SoundBank, SoundId,
};
pub use audio_wav::pcm16_mono_wav;
pub use bitmap_font::{BitmapFont, BitmapGlyph, BitmapTextRenderer};
#[cfg(not(target_arch = "wasm32"))]
pub use capture_mirror::ObsMirrorGame;
#[cfg(target_arch = "wasm32")]
pub use capture_mirror_stub::ObsMirrorGame;
pub use control::{ActionId, ControlBinding, ControlMap};
#[cfg(all(feature = "diagnostics-fault-injection", not(target_arch = "wasm32")))]
#[doc(hidden)]
pub use diagnostics::fault_injection as diagnostics_fault_probe;
#[cfg(feature = "diagnostics")]
pub use diagnostics::{
    AdapterBackend, AdapterDeviceType, AdapterFacts, AudioBackend, AudioBackendError,
    AudioErrorCategory, AudioInitializationOutcome, AudioSummary, Availability, BuildProfile,
    BuildProvenance, CollectionMode, DeviceLostObservation, DeviceLostReason, DiagnosticEvent,
    DiagnosticEventKind, DiagnosticField, DiagnosticObservation, DiagnosticProducer,
    DiagnosticReadKind, DiagnosticSection, EngineDiagnostics, EngineDiagnosticsHandle,
    EngineDiagnosticsRegistration, EventHistory, Freshness, ObservationStamp, OpaqueText,
    PanicStrategy, PresentObservation, RendererIncarnation, RendererLifecycle,
    RendererObservations, RendererRecord, RendererRole, RendererSource, RepresentationKind,
    RuntimeLifecycle, RuntimeOutcome, RuntimeState, SaturatingCounter, StaleReason,
    SurfaceAlphaMode, SurfaceConfiguration, SurfaceFailure, SurfaceFormat, SurfacePresentMode,
    TargetFamily, WgpuErrorCategory,
};
pub use framebuffer::{Font, Framebuffer};
pub use gamepad_profile::{AxisCalibration, GamepadProfile, TriggerCalibration};
pub use image::{Image, ImageError, ImageRegion};
pub use image_fit::{ImageFilter, ImageFit};
pub use input::{
    ButtonState, GamepadAxis, GamepadButton, GamepadCapabilities, GamepadCapability,
    GamepadConnectionEvent, GamepadDeviceInfo, GamepadId, GamepadMappingSource, Input, Key,
    MouseButton, Touch, TouchPhase,
};
pub use pixel::Pixel;
#[cfg(feature = "diagnostics")]
pub use platform::run_with_diagnostics;
pub use platform::{
    EngineConfig, EngineError, Frame, Game, GameResult, ToolFrame, ToolWindowConfig,
    ToolWindowMode, run, tool_window_supported,
};
pub use sfx_manifest::SfxManifest;
pub use sprite::{Sprite, SpriteError};
pub use storage::{LocalStorage, NoopStorage, StorageError};
pub use text::TextRenderer;
pub use viewport::{MAX_SPLIT_VIEWS, Rect, Size, Viewport, split_view_layout};
pub use window_icon::{set_window_icon, set_window_icon_png};
