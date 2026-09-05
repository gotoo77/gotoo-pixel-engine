use crate::{Framebuffer, Pixel, Rect, Size};

/// Result of presenting one low-resolution framebuffer into a high-resolution host.
///
/// The source is scaled by one integer factor with nearest-neighbour semantics and
/// centered inside the requested bounds. The returned mapping can translate host
/// pointer/touch coordinates back into source framebuffer coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelPresentation {
    pub rect: Rect,
    pub source_size: Size,
    pub scale: u32,
}

impl PixelPresentation {
    /// Maps a host-space point into the low-resolution source framebuffer.
    /// Returns `None` when the point is outside the presented pixel surface.
    pub fn map_point(self, point: (i32, i32)) -> Option<(i32, i32)> {
        if self.scale == 0 || self.rect.width == 0 || self.rect.height == 0 {
            return None;
        }
        let local_x = point.0.checked_sub(self.rect.x)?;
        let local_y = point.1.checked_sub(self.rect.y)?;
        if local_x < 0
            || local_y < 0
            || u32::try_from(local_x).ok()? >= self.rect.width
            || u32::try_from(local_y).ok()? >= self.rect.height
        {
            return None;
        }
        Some((
            local_x / i32::try_from(self.scale).ok()?,
            local_y / i32::try_from(self.scale).ok()?,
        ))
    }
}

/// Presents `source` into `host` using the largest integer nearest-neighbour scale
/// that fits inside `bounds`.
///
/// This is deliberately small in scope: it is a pixel-surface composition primitive,
/// not a render graph or arbitrary layer system. Existing single-framebuffer games pay
/// no cost unless they call it.
pub fn present_pixel_surface(
    host: &mut Framebuffer,
    source: &Framebuffer,
    bounds: Rect,
) -> Option<PixelPresentation> {
    if source.width() == 0
        || source.height() == 0
        || bounds.width == 0
        || bounds.height == 0
    {
        return None;
    }

    let scale_x = bounds.width / source.width();
    let scale_y = bounds.height / source.height();
    let scale = scale_x.min(scale_y);
    if scale == 0 {
        return None;
    }

    let width = source.width().checked_mul(scale)?;
    let height = source.height().checked_mul(scale)?;
    let x = bounds
        .x
        .checked_add(i32::try_from((bounds.width - width) / 2).ok()?)?;
    let y = bounds
        .y
        .checked_add(i32::try_from((bounds.height - height) / 2).ok()?)?;
    let rect = Rect {
        x,
        y,
        width,
        height,
    };

    let bytes = source.as_rgba8();
    for source_y in 0..source.height() {
        for source_x in 0..source.width() {
            let index = (usize::try_from(source_y).ok()?
                * usize::try_from(source.width()).ok()?
                + usize::try_from(source_x).ok()?)
                * 4;
            let rgba = bytes.get(index..index + 4)?;
            let pixel = Pixel::rgba(rgba[0], rgba[1], rgba[2], rgba[3]);
            let destination_x = x.checked_add(
                i32::try_from(source_x.checked_mul(scale)?).ok()?,
            )?;
            let destination_y = y.checked_add(
                i32::try_from(source_y.checked_mul(scale)?).ok()?,
            )?;
            host.fill_rect(destination_x, destination_y, scale, scale, pixel);
        }
    }

    Some(PixelPresentation {
        rect,
        source_size: Size {
            width: source.width(),
            height: source.height(),
        },
        scale,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_scale_is_centered_and_maps_points_back_to_source() {
        let mut host = Framebuffer::new(1280, 720);
        let source = Framebuffer::new(320, 180);
        let presentation = present_pixel_surface(
            &mut host,
            &source,
            Rect {
                x: 0,
                y: 0,
                width: 1280,
                height: 720,
            },
        )
        .expect("4x presentation should fit");

        assert_eq!(presentation.scale, 4);
        assert_eq!(presentation.rect.x, 0);
        assert_eq!(presentation.rect.y, 0);
        assert_eq!(presentation.rect.width, 1280);
        assert_eq!(presentation.rect.height, 720);
        assert_eq!(presentation.map_point((0, 0)), Some((0, 0)));
        assert_eq!(presentation.map_point((1279, 719)), Some((319, 179)));
        assert_eq!(presentation.map_point((1280, 719)), None);
    }

    #[test]
    fn contain_centers_integer_scaled_surface_inside_non_matching_bounds() {
        let mut host = Framebuffer::new(1000, 700);
        let source = Framebuffer::new(320, 180);
        let presentation = present_pixel_surface(
            &mut host,
            &source,
            Rect {
                x: 0,
                y: 0,
                width: 1000,
                height: 700,
            },
        )
        .expect("3x presentation should fit");

        assert_eq!(presentation.scale, 3);
        assert_eq!(presentation.rect.width, 960);
        assert_eq!(presentation.rect.height, 540);
        assert_eq!(presentation.rect.x, 20);
        assert_eq!(presentation.rect.y, 80);
        assert_eq!(presentation.map_point((20, 80)), Some((0, 0)));
        assert_eq!(presentation.map_point((979, 619)), Some((319, 179)));
        assert_eq!(presentation.map_point((19, 80)), None);
    }
}
