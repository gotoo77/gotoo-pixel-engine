//! Experimental CPU outline text; enabled only by `outline-fonts`.
use std::collections::HashMap;

use crate::{Framebuffer, Pixel, Rect, Size};
use fontdue::{
    Font, FontSettings, Metrics,
    layout::{CoordinateSystem, GlyphRasterConfig, Layout, LayoutSettings, TextStyle},
};

/// One owned font and its bounded glyph cache. Multiple instances can coexist.
/// Uses fontdue's basic layout, not complex-script shaping or automatic fallback.
pub struct OutlineFont {
    font: Font,
    layout: Layout,
    cache: HashMap<GlyphRasterConfig, (Metrics, Vec<u8>)>,
}

impl OutlineFont {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        Ok(Self {
            font: Font::from_bytes(bytes, FontSettings::default())?,
            layout: Layout::new(CoordinateSystem::PositiveYDown),
            cache: HashMap::new(),
        })
    }

    pub fn has_glyph(&self, character: char) -> bool {
        self.font.has_glyph(character)
    }

    fn arrange(&mut self, text: &str, px: f32, bounds: Rect) -> Size {
        self.layout.reset(&LayoutSettings {
            x: bounds.x as f32,
            y: bounds.y as f32,
            max_width: Some(bounds.width as f32),
            ..LayoutSettings::default()
        });
        if !px.is_finite() || px <= 0.0 || bounds.width == 0 || bounds.height == 0 {
            return Size {
                width: 0,
                height: 0,
            };
        }
        self.layout
            .append(&[&self.font], &TextStyle::new(text, px.min(256.0), 0));
        let width = self
            .layout
            .glyphs()
            .iter()
            .filter(|g| !g.parent.is_control())
            .map(|g| (g.x - bounds.x as f32 + g.width as f32).ceil() as u32)
            .max()
            .unwrap_or(0);
        Size {
            width,
            height: self.layout.height().ceil() as u32,
        }
    }

    /// Measures with the same wrapped layout used for painting.
    pub fn measure(&mut self, text: &str, px: f32, bounds: Rect) -> Size {
        self.arrange(text, px, bounds)
    }

    /// Paints antialiased glyphs, clipped to bounds and the framebuffer.
    pub fn draw(
        &mut self,
        framebuffer: &mut Framebuffer,
        text: &str,
        px: f32,
        bounds: Rect,
        color: Pixel,
    ) -> Size {
        let measured = self.arrange(text, px, bounds);
        let right =
            (i64::from(bounds.x) + i64::from(bounds.width)).min(i64::from(framebuffer.width()));
        let bottom =
            (i64::from(bounds.y) + i64::from(bounds.height)).min(i64::from(framebuffer.height()));
        for glyph in self.layout.glyphs() {
            if glyph.parent.is_control() {
                continue;
            }
            if self.cache.len() >= 1024 {
                self.cache.clear();
            }
            let (metrics, bitmap) = self
                .cache
                .entry(glyph.key)
                .or_insert_with(|| self.font.rasterize_config(glyph.key));
            for row in 0..metrics.height {
                let y = glyph.y.floor() as i64 + row as i64;
                if y < i64::from(bounds.y).max(0) || y >= bottom {
                    continue;
                }
                for column in 0..metrics.width {
                    let x = glyph.x.floor() as i64 + column as i64;
                    if x < i64::from(bounds.x).max(0) || x >= right {
                        continue;
                    }
                    let alpha = (u16::from(bitmap[row * metrics.width + column])
                        * u16::from(color.a)
                        / 255) as u8;
                    framebuffer.blend_rgba8(
                        x as u32,
                        y as u32,
                        &[color.r, color.g, color.b, alpha],
                    );
                }
            }
        }
        measured
    }
}
