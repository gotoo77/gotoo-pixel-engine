use crate::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiGridSpec {
    pub min_cell_width: u32,
    pub preferred_cell_height: u32,
    pub gap: u32,
    pub padding: u32,
}

impl Default for UiGridSpec {
    fn default() -> Self {
        Self {
            min_cell_width: 96,
            preferred_cell_height: 72,
            gap: 8,
            padding: 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UiGridLayout {
    pub(crate) columns: usize,
    pub(crate) rows: usize,
    pub(crate) rects: Vec<Rect>,
}

pub(crate) fn inset_rect(rect: Rect, inset: u32) -> Rect {
    Rect {
        x: rect.x.saturating_add(u32_to_i32(inset)),
        y: rect.y.saturating_add(u32_to_i32(inset)),
        width: rect.width.saturating_sub(inset.saturating_mul(2)),
        height: rect.height.saturating_sub(inset.saturating_mul(2)),
    }
}

pub(crate) fn layout_vertical_children(
    bounds: Rect,
    padding: u32,
    gap: u32,
    child_heights: &[u32],
) -> Vec<Rect> {
    let x = bounds.x.saturating_add(u32_to_i32(padding));
    let width = bounds.width.saturating_sub(padding.saturating_mul(2));
    let mut y = bounds.y.saturating_add(u32_to_i32(padding));
    let mut rects = Vec::with_capacity(child_heights.len());

    for &height in child_heights {
        rects.push(Rect {
            x,
            y,
            width,
            height,
        });
        y = y
            .saturating_add(u32_to_i32(height))
            .saturating_add(u32_to_i32(gap));
    }

    rects
}

pub(crate) fn layout_responsive_grid(
    bounds: Rect,
    spec: UiGridSpec,
    child_count: usize,
) -> UiGridLayout {
    if child_count == 0 || bounds.width == 0 || bounds.height == 0 {
        return UiGridLayout {
            columns: 0,
            rows: 0,
            rects: Vec::new(),
        };
    }

    let inner_width = bounds.width.saturating_sub(spec.padding.saturating_mul(2));
    let inner_height = bounds.height.saturating_sub(spec.padding.saturating_mul(2));
    if inner_width == 0 || inner_height == 0 {
        return UiGridLayout {
            columns: 0,
            rows: 0,
            rects: Vec::new(),
        };
    }

    let denominator = spec.min_cell_width.max(1).saturating_add(spec.gap).max(1);
    let columns = inner_width
        .saturating_add(spec.gap)
        .checked_div(denominator)
        .unwrap_or(1)
        .max(1)
        .min(child_count as u32) as usize;
    let rows = child_count.div_ceil(columns);

    let horizontal_gap_total = spec.gap.saturating_mul(columns.saturating_sub(1) as u32);
    let distributable_width = inner_width.saturating_sub(horizontal_gap_total);
    let base_width = distributable_width / columns as u32;
    let width_remainder = distributable_width % columns as u32;

    let vertical_gap_total = spec.gap.saturating_mul(rows.saturating_sub(1) as u32);
    let distributable_height = inner_height.saturating_sub(vertical_gap_total);
    let fit_height = distributable_height / rows as u32;
    let cell_height = spec.preferred_cell_height.min(fit_height).max(1);

    let mut x_positions = Vec::with_capacity(columns);
    let mut widths = Vec::with_capacity(columns);
    let mut x = bounds.x.saturating_add(u32_to_i32(spec.padding));
    for column in 0..columns {
        let width = base_width + u32::from((column as u32) < width_remainder);
        x_positions.push(x);
        widths.push(width);
        x = x
            .saturating_add(u32_to_i32(width))
            .saturating_add(u32_to_i32(spec.gap));
    }

    let mut rects = Vec::with_capacity(child_count);
    for index in 0..child_count {
        let row = index / columns;
        let column = index % columns;
        let y = bounds
            .y
            .saturating_add(u32_to_i32(spec.padding))
            .saturating_add(u32_to_i32(
                (cell_height.saturating_add(spec.gap)).saturating_mul(row as u32),
            ));
        rects.push(Rect {
            x: x_positions[column],
            y,
            width: widths[column],
            height: cell_height,
        });
    }

    UiGridLayout {
        columns,
        rows,
        rects,
    }
}

fn u32_to_i32(value: u32) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds(width: u32) -> Rect {
        Rect {
            x: 0,
            y: 0,
            width,
            height: 220,
        }
    }

    fn mfe_grid_spec() -> UiGridSpec {
        UiGridSpec {
            min_cell_width: 100,
            preferred_cell_height: 70,
            gap: 8,
            padding: 8,
        }
    }

    #[test]
    fn responsive_grid_preserves_one_two_three_column_mfe_behavior() {
        let spec = mfe_grid_spec();

        assert_eq!(layout_responsive_grid(bounds(120), spec, 6).columns, 1);
        assert_eq!(layout_responsive_grid(bounds(240), spec, 6).columns, 2);
        let wide = layout_responsive_grid(bounds(360), spec, 6);
        assert_eq!(wide.columns, 3);
        assert_eq!(wide.rows, 2);
    }

    #[test]
    fn responsive_grid_distributes_remainder_pixels_from_first_track() {
        let layout = layout_responsive_grid(bounds(358), mfe_grid_spec(), 3);
        let widths = layout
            .rects
            .iter()
            .map(|rect| rect.width)
            .collect::<Vec<_>>();

        assert_eq!(widths, vec![109, 109, 108]);
    }

    #[test]
    fn responsive_grid_is_deterministic_for_zero_or_unusable_space() {
        assert_eq!(
            layout_responsive_grid(bounds(0), mfe_grid_spec(), 6).columns,
            0
        );
        assert_eq!(
            layout_responsive_grid(bounds(360), mfe_grid_spec(), 0).rows,
            0
        );

        let unusable = layout_responsive_grid(
            Rect {
                x: 0,
                y: 0,
                width: 8,
                height: 8,
            },
            UiGridSpec {
                padding: 8,
                ..mfe_grid_spec()
            },
            1,
        );
        assert!(unusable.rects.is_empty());
    }

    #[test]
    fn vertical_layout_preserves_padding_gap_and_child_heights() {
        let rects = layout_vertical_children(
            Rect {
                x: 10,
                y: 20,
                width: 100,
                height: 200,
            },
            8,
            4,
            &[10, 20],
        );

        assert_eq!(
            rects,
            vec![
                Rect {
                    x: 18,
                    y: 28,
                    width: 84,
                    height: 10,
                },
                Rect {
                    x: 18,
                    y: 42,
                    width: 84,
                    height: 20,
                },
            ]
        );
    }

    #[test]
    fn inset_uses_saturating_integer_geometry() {
        assert_eq!(
            inset_rect(
                Rect {
                    x: i32::MAX - 1,
                    y: i32::MAX - 1,
                    width: 5,
                    height: 5,
                },
                4,
            ),
            Rect {
                x: i32::MAX,
                y: i32::MAX,
                width: 0,
                height: 0,
            }
        );
    }
}
