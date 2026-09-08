use crate::{Rect, Size};

/// Geometry for a horizontally composed icon + text control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IconTextLayout {
    pub icon: Rect,
    pub text: Rect,
}

/// Computes centered icon + text geometry inside `bounds`.
///
/// The icon keeps its aspect ratio and is reduced as needed to fit the row.
/// Text receives the remaining width. Oversized inputs are handled with
/// saturating arithmetic so layout remains deterministic and panic-free.
pub fn icon_text_layout(
    bounds: Rect,
    icon_preferred: Size,
    text_size: Size,
    gap: u32,
) -> IconTextLayout {
    if bounds.width == 0 || bounds.height == 0 {
        return IconTextLayout {
            icon: Rect {
                x: bounds.x,
                y: bounds.y,
                width: 0,
                height: 0,
            },
            text: Rect {
                x: bounds.x,
                y: bounds.y,
                width: 0,
                height: 0,
            },
        };
    }

    let icon = fit_icon(icon_preferred, bounds.width, bounds.height);
    let effective_gap = if icon.width == 0 || text_size.width == 0 {
        0
    } else {
        gap.min(bounds.width.saturating_sub(icon.width))
    };
    let requested_width = icon
        .width
        .saturating_add(effective_gap)
        .saturating_add(text_size.width);
    let group_width = requested_width.min(bounds.width);
    let group_x = centered_coordinate(bounds.x, bounds.width, group_width);

    let icon_rect = Rect {
        x: group_x,
        y: centered_coordinate(bounds.y, bounds.height, icon.height),
        width: icon.width,
        height: icon.height,
    };
    let text_x = group_x
        .saturating_add(to_i32(icon.width))
        .saturating_add(to_i32(effective_gap));
    let text_width = group_width.saturating_sub(icon.width.saturating_add(effective_gap));
    let text_height = text_size.height.min(bounds.height);
    let text_rect = Rect {
        x: text_x,
        y: centered_coordinate(bounds.y, bounds.height, text_height),
        width: text_width,
        height: text_height,
    };

    IconTextLayout {
        icon: icon_rect,
        text: text_rect,
    }
}

fn fit_icon(preferred: Size, max_width: u32, max_height: u32) -> Size {
    if preferred.width == 0 || preferred.height == 0 || max_width == 0 || max_height == 0 {
        return Size {
            width: 0,
            height: 0,
        };
    }

    if preferred.width <= max_width && preferred.height <= max_height {
        return preferred;
    }

    let width_limited_height =
        (u64::from(preferred.height) * u64::from(max_width) / u64::from(preferred.width)) as u32;
    if width_limited_height <= max_height {
        return Size {
            width: max_width,
            height: width_limited_height.max(1),
        };
    }

    let height_limited_width =
        (u64::from(preferred.width) * u64::from(max_height) / u64::from(preferred.height)) as u32;
    Size {
        width: height_limited_width.max(1).min(max_width),
        height: max_height,
    }
}

fn centered_coordinate(origin: i32, extent: u32, content_extent: u32) -> i32 {
    let coordinate = i64::from(origin) + (i64::from(extent) - i64::from(content_extent)) / 2;
    coordinate.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

fn to_i32(value: u32) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn common_flag_and_label_are_centered_as_one_group() {
        let layout = icon_text_layout(
            Rect {
                x: 0,
                y: 0,
                width: 200,
                height: 32,
            },
            Size {
                width: 24,
                height: 16,
            },
            Size {
                width: 72,
                height: 14,
            },
            8,
        );

        assert_eq!(layout.icon.x, 48);
        assert_eq!(layout.icon.y, 8);
        assert_eq!(layout.text.x, 80);
        assert_eq!(layout.text.width, 72);
        assert_eq!(layout.text.y, 9);
    }

    #[test]
    fn icon_is_aspect_fitted_when_row_is_too_short() {
        let layout = icon_text_layout(
            Rect {
                x: 10,
                y: 20,
                width: 100,
                height: 8,
            },
            Size {
                width: 24,
                height: 16,
            },
            Size {
                width: 40,
                height: 7,
            },
            4,
        );

        assert_eq!(layout.icon.width, 12);
        assert_eq!(layout.icon.height, 8);
        assert!(layout.text.width <= 40);
    }

    #[test]
    fn oversized_content_saturates_without_leaving_bounds() {
        let bounds = Rect {
            x: 3,
            y: 5,
            width: 20,
            height: 10,
        };
        let layout = icon_text_layout(
            bounds,
            Size {
                width: 100,
                height: 50,
            },
            Size {
                width: u32::MAX,
                height: 50,
            },
            u32::MAX,
        );

        assert!(layout.icon.width <= bounds.width);
        assert!(layout.icon.height <= bounds.height);
        assert!(layout.text.width <= bounds.width);
        assert!(layout.text.height <= bounds.height);
    }

    #[test]
    fn empty_bounds_produce_empty_layout() {
        let layout = icon_text_layout(
            Rect {
                x: 7,
                y: 9,
                width: 0,
                height: 0,
            },
            Size {
                width: 24,
                height: 16,
            },
            Size {
                width: 10,
                height: 7,
            },
            4,
        );
        assert_eq!(layout.icon.width, 0);
        assert_eq!(layout.text.width, 0);
    }
}
