use std::time::Duration;

use gotoo_pixel_engine::{
    Frame, Framebuffer, GamepadProfiles, Input, NoopAudio, NoopStorage, Rect, Size, Viewport,
};

#[test]
fn frame_remains_constructible_without_optional_platform_services() {
    let mut framebuffer = Framebuffer::new(16, 16);
    let input = Input::default();
    let mut gamepad_profiles = GamepadProfiles::default();
    let mut storage = NoopStorage;
    let mut audio = NoopAudio::default();
    let size = Size {
        width: 16,
        height: 16,
    };

    let frame = Frame {
        framebuffer: &mut framebuffer,
        input: &input,
        gamepad_profiles: &mut gamepad_profiles,
        delta_time: Duration::ZERO,
        storage: &mut storage,
        audio: &mut audio,
        surface_size: size,
        viewport: Viewport::new(size, size),
    };

    assert_eq!(frame.surface_size, size);
}

#[test]
fn rect_intersects_exposes_aabb_semantics_to_games() {
    let paddle = Rect {
        x: 10,
        y: 10,
        width: 20,
        height: 10,
    };

    assert!(
        Rect {
            x: 12,
            y: 12,
            width: 4,
            height: 4,
        }
        .intersects(paddle)
    );

    assert!(
        !Rect {
            x: 30,
            y: 12,
            width: 4,
            height: 4,
        }
        .intersects(paddle)
    );

    assert!(
        !Rect {
            x: 10,
            y: 10,
            width: 0,
            height: 4,
        }
        .intersects(paddle)
    );
}
