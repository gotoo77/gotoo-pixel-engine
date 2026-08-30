#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Maximum number of panels produced by [`split_view_layout`].
pub const MAX_SPLIT_VIEWS: usize = 4;

/// Splits `bounds` into at most four deterministic, non-overlapping view rectangles.
///
/// The function is deliberately geometry-only: callers own view identity and decide when to
/// recompute. Two views split along the bounds' longest axis. Three views give half of the long
/// axis to the first view and divide the other half between the remaining views. Four views use
/// a 2-by-2 grid. Counts above four are clamped to [`MAX_SPLIT_VIEWS`].
pub fn split_view_layout(bounds: Rect, view_count: usize) -> Vec<Rect> {
    let count = view_count.min(MAX_SPLIT_VIEWS);
    if count == 0 || bounds.width == 0 || bounds.height == 0 {
        return Vec::new();
    }

    let vertical = bounds.width >= bounds.height;
    match count {
        1 => vec![bounds],
        2 if vertical => split_columns(bounds).to_vec(),
        2 => split_rows(bounds).to_vec(),
        3 if vertical => {
            let [left, right] = split_columns(bounds);
            let [top_right, bottom_right] = split_rows(right);
            vec![left, top_right, bottom_right]
        }
        3 => {
            let [top, bottom] = split_rows(bounds);
            let [bottom_left, bottom_right] = split_columns(bottom);
            vec![top, bottom_left, bottom_right]
        }
        _ => {
            let [top, bottom] = split_rows(bounds);
            let [top_left, top_right] = split_columns(top);
            let [bottom_left, bottom_right] = split_columns(bottom);
            vec![top_left, top_right, bottom_left, bottom_right]
        }
    }
}

fn split_columns(bounds: Rect) -> [Rect; 2] {
    let first_width = bounds.width / 2;
    [
        Rect {
            width: first_width,
            ..bounds
        },
        Rect {
            x: bounds.x.saturating_add(u32_to_i32(first_width)),
            width: bounds.width - first_width,
            ..bounds
        },
    ]
}

fn split_rows(bounds: Rect) -> [Rect; 2] {
    let first_height = bounds.height / 2;
    [
        Rect {
            height: first_height,
            ..bounds
        },
        Rect {
            y: bounds.y.saturating_add(u32_to_i32(first_height)),
            height: bounds.height - first_height,
            ..bounds
        },
    ]
}

fn u32_to_i32(value: u32) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

impl Rect {
    pub fn contains(self, position: (i32, i32)) -> bool {
        if self.width == 0 || self.height == 0 {
            return false;
        }

        let min_x = i64::from(self.x);
        let min_y = i64::from(self.y);
        let max_x = min_x + i64::from(self.width) - 1;
        let max_y = min_y + i64::from(self.height) - 1;
        let x = i64::from(position.0);
        let y = i64::from(position.1);

        x >= min_x && x <= max_x && y >= min_y && y <= max_y
    }

    pub fn intersects(self, other: Self) -> bool {
        if self.width == 0 || self.height == 0 || other.width == 0 || other.height == 0 {
            return false;
        }

        let self_min_x = i64::from(self.x);
        let self_min_y = i64::from(self.y);
        let self_max_x = self_min_x + i64::from(self.width);
        let self_max_y = self_min_y + i64::from(self.height);
        let other_min_x = i64::from(other.x);
        let other_min_y = i64::from(other.y);
        let other_max_x = other_min_x + i64::from(other.width);
        let other_max_y = other_min_y + i64::from(other.height);

        self_min_x < other_max_x
            && self_max_x > other_min_x
            && self_min_y < other_max_y
            && self_max_y > other_min_y
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    pub surface_size: Size,
    pub framebuffer_size: Size,
    pub rect: Rect,
    pub scale: f64,
}

impl Viewport {
    pub fn new(surface_size: Size, framebuffer_size: Size) -> Self {
        if surface_size.width == 0
            || surface_size.height == 0
            || framebuffer_size.width == 0
            || framebuffer_size.height == 0
        {
            return Self {
                surface_size,
                framebuffer_size,
                rect: Rect {
                    x: 0,
                    y: 0,
                    width: 0,
                    height: 0,
                },
                scale: 0.0,
            };
        }

        let raw_scale = (f64::from(surface_size.width) / f64::from(framebuffer_size.width))
            .min(f64::from(surface_size.height) / f64::from(framebuffer_size.height));
        let scale = pixel_art_scale(raw_scale);
        let width = (f64::from(framebuffer_size.width) * scale).round() as u32;
        let height = (f64::from(framebuffer_size.height) * scale).round() as u32;
        let x = ((i64::from(surface_size.width) - i64::from(width)) / 2) as i32;
        let y = ((i64::from(surface_size.height) - i64::from(height)) / 2) as i32;

        Self {
            surface_size,
            framebuffer_size,
            rect: Rect {
                x,
                y,
                width,
                height,
            },
            scale,
        }
    }

    pub fn map_surface_position(self, x: f64, y: f64) -> Option<(i32, i32)> {
        if self.rect.width == 0 || self.rect.height == 0 {
            return None;
        }

        let min_x = f64::from(self.rect.x);
        let min_y = f64::from(self.rect.y);
        let max_x = min_x + f64::from(self.rect.width);
        let max_y = min_y + f64::from(self.rect.height);

        if x < min_x || y < min_y || x >= max_x || y >= max_y {
            return None;
        }

        let framebuffer_x = ((x - min_x) * f64::from(self.framebuffer_size.width)
            / f64::from(self.rect.width))
        .floor();
        let framebuffer_y = ((y - min_y) * f64::from(self.framebuffer_size.height)
            / f64::from(self.rect.height))
        .floor();

        if framebuffer_x < 0.0
            || framebuffer_y < 0.0
            || framebuffer_x >= f64::from(self.framebuffer_size.width)
            || framebuffer_y >= f64::from(self.framebuffer_size.height)
        {
            return None;
        }

        Some((framebuffer_x as i32, framebuffer_y as i32))
    }
}

fn pixel_art_scale(raw_scale: f64) -> f64 {
    if raw_scale >= 1.0 {
        let integer_scale = raw_scale.floor();
        if (raw_scale - integer_scale).abs() < f64::EPSILON {
            return integer_scale;
        }
    }

    raw_scale
}

#[cfg(test)]
mod tests {
    use super::{MAX_SPLIT_VIEWS, Rect, Size, Viewport, split_view_layout};

    const FRAMEBUFFER: Size = Size {
        width: 480,
        height: 204,
    };

    #[test]
    fn split_layout_covers_zero_through_four_views() {
        let bounds = Rect {
            x: 7,
            y: 11,
            width: 101,
            height: 61,
        };
        assert!(split_view_layout(bounds, 0).is_empty());
        assert_eq!(split_view_layout(bounds, 1), vec![bounds]);
        for count in 2..=4 {
            assert_eq!(split_view_layout(bounds, count).len(), count);
        }
        assert_eq!(split_view_layout(bounds, 99).len(), MAX_SPLIT_VIEWS);
    }

    #[test]
    fn split_layout_uses_long_axis_for_landscape_and_portrait() {
        let landscape = Rect {
            x: 0,
            y: 0,
            width: 120,
            height: 60,
        };
        let portrait = Rect {
            x: 0,
            y: 0,
            width: 60,
            height: 120,
        };

        assert_eq!(split_view_layout(landscape, 2)[1].x, 60);
        assert_eq!(split_view_layout(landscape, 2)[1].y, 0);
        assert_eq!(split_view_layout(portrait, 2)[1].x, 0);
        assert_eq!(split_view_layout(portrait, 2)[1].y, 60);
    }

    #[test]
    fn split_layout_is_deterministic_inside_bounds_and_non_overlapping() {
        for bounds in [
            Rect {
                x: -13,
                y: 5,
                width: 121,
                height: 70,
            },
            Rect {
                x: 4,
                y: -9,
                width: 71,
                height: 123,
            },
        ] {
            for count in 1..=4 {
                let first = split_view_layout(bounds, count);
                assert_eq!(first, split_view_layout(bounds, count));
                for (index, rect) in first.iter().copied().enumerate() {
                    assert!(rect.width > 0 && rect.height > 0);
                    assert!(bounds.contains((rect.x, rect.y)));
                    let last_x = i64::from(rect.x) + i64::from(rect.width) - 1;
                    let last_y = i64::from(rect.y) + i64::from(rect.height) - 1;
                    assert!(bounds.contains((last_x as i32, last_y as i32)));
                    for other in first.iter().copied().skip(index + 1) {
                        assert!(!rect.intersects(other));
                    }
                }
            }
        }
    }

    #[test]
    fn empty_bounds_produce_no_views() {
        assert!(
            split_view_layout(
                Rect {
                    x: 0,
                    y: 0,
                    width: 0,
                    height: 10
                },
                1
            )
            .is_empty()
        );
        assert!(
            split_view_layout(
                Rect {
                    x: 0,
                    y: 0,
                    width: 10,
                    height: 0
                },
                4
            )
            .is_empty()
        );
    }

    #[test]
    fn same_ratio_surface_uses_full_surface() {
        let viewport = Viewport::new(
            Size {
                width: 960,
                height: 408,
            },
            FRAMEBUFFER,
        );

        assert_eq!(
            viewport.rect,
            Rect {
                x: 0,
                y: 0,
                width: 960,
                height: 408
            }
        );
        assert_eq!(viewport.scale, 2.0);
    }

    #[test]
    fn wider_surface_uses_pillarboxing() {
        let viewport = Viewport::new(
            Size {
                width: 1200,
                height: 408,
            },
            FRAMEBUFFER,
        );

        assert_eq!(
            viewport.rect,
            Rect {
                x: 120,
                y: 0,
                width: 960,
                height: 408
            }
        );
    }

    #[test]
    fn taller_surface_uses_letterboxing() {
        let viewport = Viewport::new(
            Size {
                width: 960,
                height: 600,
            },
            FRAMEBUFFER,
        );

        assert_eq!(
            viewport.rect,
            Rect {
                x: 0,
                y: 96,
                width: 960,
                height: 408
            }
        );
    }

    #[test]
    fn exact_integer_scale_is_retained() {
        let viewport = Viewport::new(
            Size {
                width: 1440,
                height: 612,
            },
            FRAMEBUFFER,
        );

        assert_eq!(viewport.scale, 3.0);
        assert_eq!(viewport.rect.width, 1440);
        assert_eq!(viewport.rect.height, 612);
    }

    #[test]
    fn fractional_scale_is_used_when_integer_scale_would_shrink_too_much() {
        let viewport = Viewport::new(
            Size {
                width: 800,
                height: 408,
            },
            FRAMEBUFFER,
        );

        assert!((viewport.scale - 1.666_666_666_666_666_7).abs() < 0.000_001);
        assert_eq!(viewport.rect.width, 800);
        assert_eq!(viewport.rect.height, 340);
        assert_eq!(viewport.rect.y, 34);
    }

    #[test]
    fn zero_surface_produces_empty_viewport() {
        let viewport = Viewport::new(
            Size {
                width: 0,
                height: 408,
            },
            FRAMEBUFFER,
        );

        assert_eq!(viewport.rect.width, 0);
        assert_eq!(viewport.rect.height, 0);
        assert_eq!(viewport.scale, 0.0);
    }

    #[test]
    fn maps_surface_center_to_framebuffer_center() {
        let viewport = Viewport::new(
            Size {
                width: 1200,
                height: 408,
            },
            FRAMEBUFFER,
        );

        assert_eq!(
            viewport.map_surface_position(600.0, 204.0),
            Some((240, 102))
        );
    }

    #[test]
    fn maps_four_viewport_edges() {
        let viewport = Viewport::new(
            Size {
                width: 1200,
                height: 408,
            },
            FRAMEBUFFER,
        );

        assert_eq!(viewport.map_surface_position(120.0, 0.0), Some((0, 0)));
        assert_eq!(viewport.map_surface_position(1079.999, 0.0), Some((479, 0)));
        assert_eq!(
            viewport.map_surface_position(120.0, 407.999),
            Some((0, 203))
        );
        assert_eq!(
            viewport.map_surface_position(1079.999, 407.999),
            Some((479, 203))
        );
    }

    #[test]
    fn input_outside_viewport_returns_none() {
        let viewport = Viewport::new(
            Size {
                width: 1200,
                height: 408,
            },
            FRAMEBUFFER,
        );

        assert_eq!(viewport.map_surface_position(119.999, 20.0), None);
        assert_eq!(viewport.map_surface_position(1080.0, 20.0), None);
        assert_eq!(viewport.map_surface_position(600.0, -0.001), None);
        assert_eq!(viewport.map_surface_position(600.0, 408.0), None);
    }

    #[test]
    fn resize_changes_viewport() {
        let first = Viewport::new(
            Size {
                width: 960,
                height: 408,
            },
            FRAMEBUFFER,
        );
        let second = Viewport::new(
            Size {
                width: 1200,
                height: 408,
            },
            FRAMEBUFFER,
        );

        assert_ne!(first.rect, second.rect);
        assert_eq!(second.rect.x, 120);
    }

    #[test]
    fn rect_contains_edges_inclusively() {
        let rect = Rect {
            x: 10,
            y: 20,
            width: 30,
            height: 40,
        };

        assert!(rect.contains((10, 20)));
        assert!(rect.contains((39, 59)));
        assert!(!rect.contains((40, 59)));
        assert!(!rect.contains((39, 60)));
    }

    #[test]
    fn rect_intersection_handles_positive_i32_edge_without_overflow() {
        let left = Rect {
            x: i32::MAX - 1,
            y: 0,
            width: 2,
            height: 2,
        };
        let right = Rect {
            x: i32::MAX,
            y: 0,
            width: 1,
            height: 2,
        };

        assert!(left.intersects(right));
    }

    #[test]
    fn rect_intersection_handles_large_unsigned_extents_without_overflow() {
        let wide = Rect {
            x: i32::MIN,
            y: -1,
            width: u32::MAX,
            height: 2,
        };
        let edge = Rect {
            x: i32::MAX - 1,
            y: -1,
            width: 1,
            height: 2,
        };

        assert!(wide.intersects(edge));
    }
}
