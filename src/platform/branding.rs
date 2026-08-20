use winit::window::Icon;

const ICON_SIZE: u32 = 32;
const TRANSPARENT: [u8; 4] = [0, 0, 0, 0];
const PANEL: [u8; 4] = [7, 10, 18, 255];
const CYAN: [u8; 4] = [104, 222, 255, 255];
const GOLD: [u8; 4] = [246, 196, 83, 255];
const VIOLET: [u8; 4] = [185, 104, 255, 255];
const LIGHT: [u8; 4] = [232, 240, 255, 255];

pub(super) fn default_window_icon() -> Option<Icon> {
    Icon::from_rgba(gpe_icon_rgba(), ICON_SIZE, ICON_SIZE).ok()
}

fn gpe_icon_rgba() -> Vec<u8> {
    let mut rgba = vec![0; (ICON_SIZE * ICON_SIZE * 4) as usize];
    for pixel in rgba.chunks_exact_mut(4) {
        pixel.copy_from_slice(&TRANSPARENT);
    }

    paint_rect(&mut rgba, 2, 2, 28, 28, PANEL);
    paint_rect(&mut rgba, 2, 2, 28, 2, CYAN);
    paint_rect(&mut rgba, 2, 28, 28, 2, CYAN);
    paint_rect(&mut rgba, 2, 4, 2, 24, CYAN);
    paint_rect(&mut rgba, 28, 4, 2, 24, CYAN);

    // Small asymmetric accents keep the mark readable as pixel art at 16/32 px.
    paint_rect(&mut rgba, 5, 6, 4, 2, GOLD);
    paint_rect(&mut rgba, 23, 24, 4, 2, VIOLET);
    paint_rect(&mut rgba, 15, 6, 2, 2, LIGHT);
    paint_rect(&mut rgba, 15, 24, 2, 2, LIGHT);

    draw_glyph(&mut rgba, 4, 11, &["111", "100", "101", "101", "111"], GOLD);
    draw_glyph(
        &mut rgba,
        13,
        11,
        &["111", "101", "111", "100", "100"],
        VIOLET,
    );
    draw_glyph(&mut rgba, 22, 11, &["111", "100", "110", "100", "111"], CYAN);

    rgba
}

fn draw_glyph(rgba: &mut [u8], x: u32, y: u32, rows: &[&str; 5], color: [u8; 4]) {
    const SCALE: u32 = 2;
    for (row, pattern) in rows.iter().enumerate() {
        for (column, cell) in pattern.bytes().enumerate() {
            if cell == b'1' {
                paint_rect(
                    rgba,
                    x + column as u32 * SCALE,
                    y + row as u32 * SCALE,
                    SCALE,
                    SCALE,
                    color,
                );
            }
        }
    }
}

fn paint_rect(rgba: &mut [u8], x: u32, y: u32, width: u32, height: u32, color: [u8; 4]) {
    for py in y..y.saturating_add(height).min(ICON_SIZE) {
        for px in x..x.saturating_add(width).min(ICON_SIZE) {
            let offset = ((py * ICON_SIZE + px) * 4) as usize;
            rgba[offset..offset + 4].copy_from_slice(&color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_icon_has_expected_rgba_shape() {
        let rgba = gpe_icon_rgba();
        assert_eq!(rgba.len(), (ICON_SIZE * ICON_SIZE * 4) as usize);
        assert_eq!(&rgba[0..4], &TRANSPARENT);
    }

    #[test]
    fn default_icon_is_accepted_by_winit() {
        assert!(default_window_icon().is_some());
    }
}
