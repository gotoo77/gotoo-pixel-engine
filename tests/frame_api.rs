use std::time::Duration;

use gotoo_pixel_engine::{
    Frame, Framebuffer, Input, NoopAudio, NoopStorage, Size, Viewport,
};

#[test]
fn frame_remains_constructible_without_optional_platform_services() {
    let mut framebuffer = Framebuffer::new(16, 16);
    let input = Input::default();
    let mut storage = NoopStorage;
    let mut audio = NoopAudio::default();
    let size = Size {
        width: 16,
        height: 16,
    };

    let frame = Frame {
        framebuffer: &mut framebuffer,
        input: &input,
        delta_time: Duration::ZERO,
        storage: &mut storage,
        audio: &mut audio,
        surface_size: size,
        viewport: Viewport::new(size, size),
    };

    assert_eq!(frame.surface_size, size);
}
