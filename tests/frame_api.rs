use std::time::Duration;

use gotoo_pixel_engine::{Frame, Framebuffer, Input, NoopAudio, NoopStorage, Rect, Size, Viewport};

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

#[test]
fn rect_intersects_remains_safe_at_public_coordinate_limits() {
    let near_max = Rect {
        x: i32::MAX - 1,
        y: i32::MIN,
        width: 2,
        height: 2,
    };
    let max_edge = Rect {
        x: i32::MAX,
        y: i32::MIN,
        width: 1,
        height: 1,
    };

    assert!(near_max.intersects(max_edge));
}
