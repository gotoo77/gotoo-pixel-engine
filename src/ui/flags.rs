use crate::{Framebuffer, Pixel, Rect};

/// Small deterministic country/region flags intended as secondary visual cues
/// in language selectors. Text remains the authoritative language label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlagIcon {
    UnitedKingdom,
    France,
    Germany,
    Spain,
    Italy,
    Portugal,
    Brazil,
    Russia,
    Japan,
    SouthKorea,
    China,
    Taiwan,
}

impl FlagIcon {
    pub const PREFERRED_WIDTH: u32 = 24;
    pub const PREFERRED_HEIGHT: u32 = 16;

    /// Draws the flag into `rect`. The renderer is intentionally procedural so
    /// GPE does not depend on OS emoji fonts or platform-specific flag glyphs.
    pub fn draw(self, framebuffer: &mut Framebuffer, rect: Rect) {
        if rect.width == 0 || rect.height == 0 {
            return;
        }

        match self {
            Self::France => vertical_tricolor(
                framebuffer,
                rect,
                Pixel::rgb(0, 35, 149),
                Pixel::WHITE,
                Pixel::rgb(237, 41, 57),
            ),
            Self::Italy => vertical_tricolor(
                framebuffer,
                rect,
                Pixel::rgb(0, 146, 70),
                Pixel::WHITE,
                Pixel::rgb(206, 43, 55),
            ),
            Self::Germany => horizontal_tricolor(
                framebuffer,
                rect,
                Pixel::rgb(20, 20, 20),
                Pixel::rgb(221, 0, 0),
                Pixel::rgb(255, 206, 0),
            ),
            Self::Russia => horizontal_tricolor(
                framebuffer,
                rect,
                Pixel::WHITE,
                Pixel::rgb(0, 57, 166),
                Pixel::rgb(213, 43, 30),
            ),
            Self::Spain => draw_spain(framebuffer, rect),
            Self::Portugal => draw_portugal(framebuffer, rect),
            Self::Brazil => draw_brazil(framebuffer, rect),
            Self::Japan => draw_japan(framebuffer, rect),
            Self::SouthKorea => draw_south_korea(framebuffer, rect),
            Self::China => draw_china(framebuffer, rect),
            Self::Taiwan => draw_taiwan(framebuffer, rect),
            Self::UnitedKingdom => draw_united_kingdom(framebuffer, rect),
        }
    }
}

fn vertical_tricolor(
    framebuffer: &mut Framebuffer,
    rect: Rect,
    left: Pixel,
    center: Pixel,
    right: Pixel,
) {
    let first = rect.width / 3;
    let second = rect.width.saturating_mul(2) / 3;
    framebuffer.fill_rect(rect.x, rect.y, first, rect.height, left);
    framebuffer.fill_rect(
        rect.x.saturating_add(to_i32(first)),
        rect.y,
        second.saturating_sub(first),
        rect.height,
        center,
    );
    framebuffer.fill_rect(
        rect.x.saturating_add(to_i32(second)),
        rect.y,
        rect.width.saturating_sub(second),
        rect.height,
        right,
    );
}

fn horizontal_tricolor(
    framebuffer: &mut Framebuffer,
    rect: Rect,
    top: Pixel,
    middle: Pixel,
    bottom: Pixel,
) {
    let first = rect.height / 3;
    let second = rect.height.saturating_mul(2) / 3;
    framebuffer.fill_rect(rect.x, rect.y, rect.width, first, top);
    framebuffer.fill_rect(
        rect.x,
        rect.y.saturating_add(to_i32(first)),
        rect.width,
        second.saturating_sub(first),
        middle,
    );
    framebuffer.fill_rect(
        rect.x,
        rect.y.saturating_add(to_i32(second)),
        rect.width,
        rect.height.saturating_sub(second),
        bottom,
    );
}

fn draw_spain(framebuffer: &mut Framebuffer, rect: Rect) {
    let red = Pixel::rgb(170, 21, 27);
    let yellow = Pixel::rgb(241, 191, 0);
    let band = (rect.height / 4).max(1).min(rect.height);
    framebuffer.fill_rect(rect.x, rect.y, rect.width, rect.height, yellow);
    framebuffer.fill_rect(rect.x, rect.y, rect.width, band, red);
    framebuffer.fill_rect(
        rect.x,
        rect.y
            .saturating_add(to_i32(rect.height.saturating_sub(band))),
        rect.width,
        band,
        red,
    );
}

fn draw_portugal(framebuffer: &mut Framebuffer, rect: Rect) {
    let green_width = rect.width.saturating_mul(2) / 5;
    framebuffer.fill_rect(
        rect.x,
        rect.y,
        green_width,
        rect.height,
        Pixel::rgb(4, 106, 56),
    );
    framebuffer.fill_rect(
        rect.x.saturating_add(to_i32(green_width)),
        rect.y,
        rect.width.saturating_sub(green_width),
        rect.height,
        Pixel::rgb(218, 41, 28),
    );
    let radius = (rect.height / 6).max(1);
    framebuffer.fill_circle(
        rect.x.saturating_add(to_i32(green_width)),
        centered(rect.y, rect.height),
        radius,
        Pixel::rgb(255, 205, 0),
    );
}

fn draw_brazil(framebuffer: &mut Framebuffer, rect: Rect) {
    framebuffer.fill_rect(
        rect.x,
        rect.y,
        rect.width,
        rect.height,
        Pixel::rgb(0, 151, 57),
    );
    let yellow = Pixel::rgb(255, 223, 0);
    let cx = rect.x.saturating_add(to_i32(rect.width / 2));
    let cy = centered(rect.y, rect.height);
    let half_w = rect.width.saturating_mul(3) / 8;
    let half_h = rect.height / 3;
    for dy in 0..=half_h {
        let row_half = half_w.saturating_mul(half_h.saturating_sub(dy)) / half_h.max(1);
        framebuffer.fill_rect(
            cx.saturating_sub(to_i32(row_half)),
            cy.saturating_add(to_i32(dy)),
            row_half.saturating_mul(2).saturating_add(1),
            1,
            yellow,
        );
        if dy != 0 {
            framebuffer.fill_rect(
                cx.saturating_sub(to_i32(row_half)),
                cy.saturating_sub(to_i32(dy)),
                row_half.saturating_mul(2).saturating_add(1),
                1,
                yellow,
            );
        }
    }
    framebuffer.fill_circle(cx, cy, (rect.height / 5).max(1), Pixel::rgb(0, 39, 118));
}

fn draw_japan(framebuffer: &mut Framebuffer, rect: Rect) {
    framebuffer.fill_rect(rect.x, rect.y, rect.width, rect.height, Pixel::WHITE);
    framebuffer.fill_circle(
        rect.x.saturating_add(to_i32(rect.width / 2)),
        centered(rect.y, rect.height),
        (rect.height.saturating_mul(3) / 10).max(1),
        Pixel::rgb(188, 0, 45),
    );
}

fn draw_south_korea(framebuffer: &mut Framebuffer, rect: Rect) {
    framebuffer.fill_rect(rect.x, rect.y, rect.width, rect.height, Pixel::WHITE);
    let cx = rect.x.saturating_add(to_i32(rect.width / 2));
    let cy = centered(rect.y, rect.height);
    let radius = (rect.height / 4).max(1);
    framebuffer.fill_circle(cx, cy, radius, Pixel::rgb(205, 46, 58));
    framebuffer.fill_rect(
        cx.saturating_sub(to_i32(radius)),
        cy,
        radius.saturating_mul(2).saturating_add(1),
        radius.saturating_add(1),
        Pixel::rgb(0, 71, 160),
    );
    // Compact trigram cues; recognizable at the intended 24x16 size without
    // relying on font glyphs.
    let ink = Pixel::rgb(20, 20, 20);
    let mark_w = (rect.width / 6).max(2);
    framebuffer.fill_rect(
        rect.x.saturating_add(2),
        rect.y.saturating_add(2),
        mark_w,
        1,
        ink,
    );
    framebuffer.fill_rect(
        rect.x
            .saturating_add(to_i32(rect.width.saturating_sub(mark_w + 2))),
        rect.y.saturating_add(2),
        mark_w,
        1,
        ink,
    );
    framebuffer.fill_rect(
        rect.x.saturating_add(2),
        rect.y.saturating_add(to_i32(rect.height.saturating_sub(3))),
        mark_w,
        1,
        ink,
    );
    framebuffer.fill_rect(
        rect.x
            .saturating_add(to_i32(rect.width.saturating_sub(mark_w + 2))),
        rect.y.saturating_add(to_i32(rect.height.saturating_sub(3))),
        mark_w,
        1,
        ink,
    );
}

fn draw_china(framebuffer: &mut Framebuffer, rect: Rect) {
    let red = Pixel::rgb(222, 41, 16);
    let yellow = Pixel::rgb(255, 222, 0);
    framebuffer.fill_rect(rect.x, rect.y, rect.width, rect.height, red);
    let size = (rect.height / 4).max(2);
    let x = rect.x.saturating_add(2);
    let y = rect.y.saturating_add(2);
    framebuffer.fill_rect(x, y.saturating_add(to_i32(size / 2)), size, 1, yellow);
    framebuffer.fill_rect(x.saturating_add(to_i32(size / 2)), y, 1, size, yellow);
}

fn draw_taiwan(framebuffer: &mut Framebuffer, rect: Rect) {
    framebuffer.fill_rect(
        rect.x,
        rect.y,
        rect.width,
        rect.height,
        Pixel::rgb(254, 0, 0),
    );
    let canton_w = rect.width / 2;
    let canton_h = rect.height / 2;
    framebuffer.fill_rect(rect.x, rect.y, canton_w, canton_h, Pixel::rgb(0, 0, 149));
    framebuffer.fill_circle(
        rect.x.saturating_add(to_i32(canton_w / 2)),
        rect.y.saturating_add(to_i32(canton_h / 2)),
        (canton_h / 5).max(1),
        Pixel::WHITE,
    );
}

fn draw_united_kingdom(framebuffer: &mut Framebuffer, rect: Rect) {
    let blue = Pixel::rgb(1, 33, 105);
    let white = Pixel::WHITE;
    let red = Pixel::rgb(200, 16, 46);
    framebuffer.fill_rect(rect.x, rect.y, rect.width, rect.height, blue);

    // Diagonal saltire approximated by one-pixel stepped lines at tiny sizes.
    let steps = rect.width.max(rect.height);
    for i in 0..steps {
        let x = rect
            .x
            .saturating_add(to_i32(i.saturating_mul(rect.width) / steps.max(1)));
        let y1 = rect
            .y
            .saturating_add(to_i32(i.saturating_mul(rect.height) / steps.max(1)));
        let y2 = rect
            .y
            .saturating_add(to_i32(rect.height.saturating_sub(1)))
            .saturating_sub(to_i32(i.saturating_mul(rect.height) / steps.max(1)));
        framebuffer.draw(x, y1, white);
        framebuffer.draw(x, y2, white);
    }

    let white_vertical = (rect.width / 5).max(1);
    let white_horizontal = (rect.height / 4).max(1);
    let red_vertical = (rect.width / 9).max(1);
    let red_horizontal = (rect.height / 7).max(1);
    framebuffer.fill_rect(
        rect.x
            .saturating_add(to_i32((rect.width - white_vertical) / 2)),
        rect.y,
        white_vertical,
        rect.height,
        white,
    );
    framebuffer.fill_rect(
        rect.x,
        rect.y
            .saturating_add(to_i32((rect.height - white_horizontal) / 2)),
        rect.width,
        white_horizontal,
        white,
    );
    framebuffer.fill_rect(
        rect.x
            .saturating_add(to_i32((rect.width - red_vertical) / 2)),
        rect.y,
        red_vertical,
        rect.height,
        red,
    );
    framebuffer.fill_rect(
        rect.x,
        rect.y
            .saturating_add(to_i32((rect.height - red_horizontal) / 2)),
        rect.width,
        red_horizontal,
        red,
    );
}

fn centered(origin: i32, extent: u32) -> i32 {
    origin.saturating_add(to_i32(extent / 2))
}

fn to_i32(value: u32) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(flag: FlagIcon) -> Framebuffer {
        let mut framebuffer =
            Framebuffer::new(FlagIcon::PREFERRED_WIDTH, FlagIcon::PREFERRED_HEIGHT);
        flag.draw(
            &mut framebuffer,
            Rect {
                x: 0,
                y: 0,
                width: FlagIcon::PREFERRED_WIDTH,
                height: FlagIcon::PREFERRED_HEIGHT,
            },
        );
        framebuffer
    }

    #[test]
    fn japan_has_white_corner_and_red_center() {
        let framebuffer = render(FlagIcon::Japan);
        assert_eq!(framebuffer.pixel(0, 0), Some(Pixel::WHITE));
        assert_eq!(framebuffer.pixel(12, 8), Some(Pixel::rgb(188, 0, 45)));
    }

    #[test]
    fn french_columns_are_distinct() {
        let framebuffer = render(FlagIcon::France);
        assert_ne!(framebuffer.pixel(1, 8), framebuffer.pixel(12, 8));
        assert_ne!(framebuffer.pixel(12, 8), framebuffer.pixel(22, 8));
    }

    #[test]
    fn all_catalog_flags_render_non_empty_pixels() {
        for flag in [
            FlagIcon::UnitedKingdom,
            FlagIcon::France,
            FlagIcon::Germany,
            FlagIcon::Spain,
            FlagIcon::Italy,
            FlagIcon::Portugal,
            FlagIcon::Brazil,
            FlagIcon::Russia,
            FlagIcon::Japan,
            FlagIcon::SouthKorea,
            FlagIcon::China,
            FlagIcon::Taiwan,
        ] {
            let framebuffer = render(flag);
            assert!(
                framebuffer
                    .as_rgba8()
                    .chunks_exact(4)
                    .any(|pixel| pixel[3] != 0)
            );
        }
    }
}
