use std::time::Duration;

use gotoo_pixel_engine::{Framebuffer, Pixel};

#[derive(Debug)]
pub struct Demo {
    time: f32,
    alternate_palette: bool,
}

impl Demo {
    pub fn new() -> Self {
        Self {
            time: 0.0,
            alternate_palette: false,
        }
    }

    pub fn toggle_palette(&mut self) {
        self.alternate_palette = !self.alternate_palette;
    }

    pub fn update(&mut self, dt: Duration, framebuffer: &mut Framebuffer) {
        self.time += dt.as_secs_f32();

        let width = framebuffer.width();
        let height = framebuffer.height();
        let phase = (self.time * 96.0) as u32;

        for y in 0..height {
            for x in 0..width {
                let checker = (((x + phase) / 16) ^ ((y + phase / 2) / 16)) & 1;
                let color = if self.alternate_palette {
                    alternate_color(x, y, width, height, checker)
                } else {
                    default_color(x, y, width, height, checker)
                };
                framebuffer.set_pixel_in_bounds(x, y, color);
            }
        }
    }
}

fn default_color(x: u32, y: u32, width: u32, height: u32, checker: u32) -> Pixel {
    let r = scale_u8(x, width);
    let g = scale_u8(y, height);
    let b = if checker == 0 { 48 } else { 220 };

    Pixel::rgb(r, g, b)
}

fn alternate_color(x: u32, y: u32, width: u32, height: u32, checker: u32) -> Pixel {
    let r = if checker == 0 { 24 } else { 245 };
    let g = scale_u8(width - 1 - x, width);
    let b = scale_u8(height - 1 - y, height);

    Pixel::rgb(r, g, b)
}

fn scale_u8(value: u32, max: u32) -> u8 {
    if max <= 1 {
        return 0;
    }

    ((value as f32 / (max - 1) as f32) * 255.0).round() as u8
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::Demo;
    use gotoo_pixel_engine::Framebuffer;

    #[test]
    fn update_changes_cpu_framebuffer() {
        let mut demo = Demo::new();
        let mut framebuffer = Framebuffer::new(4, 4);

        demo.update(Duration::from_millis(16), &mut framebuffer);

        assert!(framebuffer.as_rgba8().iter().any(|channel| *channel != 0));
    }

    #[test]
    fn keyboard_palette_toggle_changes_output() {
        let mut demo = Demo::new();
        let mut first = Framebuffer::new(4, 4);
        let mut second = Framebuffer::new(4, 4);

        demo.update(Duration::ZERO, &mut first);
        demo.toggle_palette();
        demo.update(Duration::ZERO, &mut second);

        assert_ne!(first.as_rgba8(), second.as_rgba8());
    }
}
