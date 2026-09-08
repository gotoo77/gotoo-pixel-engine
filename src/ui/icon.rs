use crate::{Framebuffer, Rect, Size};

use super::FlagIcon;

/// Small paintable visual that can be composed with regular UI controls.
///
/// Icons do not own interaction semantics: focus, click and keyboard behavior
/// stay with the containing control. This keeps icon rendering reusable beyond
/// language selectors.
pub trait UiIcon {
    /// Natural logical size of the icon before the containing control applies
    /// its own fitting policy.
    fn preferred_size(&self) -> Size;

    /// Paints the icon into the supplied logical rectangle.
    fn draw(&self, framebuffer: &mut Framebuffer, rect: Rect);
}

impl UiIcon for FlagIcon {
    fn preferred_size(&self) -> Size {
        Size {
            width: FlagIcon::PREFERRED_WIDTH,
            height: FlagIcon::PREFERRED_HEIGHT,
        }
    }

    fn draw(&self, framebuffer: &mut Framebuffer, rect: Rect) {
        (*self).draw(framebuffer, rect);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Pixel;

    #[test]
    fn flag_icons_satisfy_generic_icon_contract() {
        let icon = FlagIcon::Japan;
        assert_eq!(
            icon.preferred_size(),
            Size {
                width: 24,
                height: 16
            }
        );

        let mut framebuffer = Framebuffer::new(24, 16);
        icon.draw(
            &mut framebuffer,
            Rect {
                x: 0,
                y: 0,
                width: 24,
                height: 16,
            },
        );
        assert_eq!(framebuffer.pixel(12, 8), Some(Pixel::rgb(188, 0, 45)));
    }
}
