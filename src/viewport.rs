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
    use super::{Rect, Size, Viewport};

    const FRAMEBUFFER: Size = Size {
        width: 480,
        height: 204,
    };

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
