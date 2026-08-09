use gotoo_pixel_engine::{Frame, Game, GameResult, Key, MouseButton, Pixel};

#[derive(Debug)]
pub struct DemoGame {
    x: f32,
    y: f32,
    time: f32,
    alternate_palette: bool,
}

impl DemoGame {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            time: 0.0,
            alternate_palette: false,
        }
    }

    fn toggle_palette(&mut self) {
        self.alternate_palette = !self.alternate_palette;
    }

    fn update_position(&mut self, frame: &Frame<'_>) {
        let dt = frame.delta_time.as_secs_f32();
        let speed = 120.0 * dt;

        if frame.input.key(Key::Left).held() || frame.input.key(Key::A).held() {
            self.x -= speed;
        }
        if frame.input.key(Key::Right).held() || frame.input.key(Key::D).held() {
            self.x += speed;
        }
        if frame.input.key(Key::Up).held() || frame.input.key(Key::W).held() {
            self.y -= speed;
        }
        if frame.input.key(Key::Down).held() || frame.input.key(Key::S).held() {
            self.y += speed;
        }

        self.x = self.x.clamp(0.0, frame.framebuffer.width() as f32 - 1.0);
        self.y = self.y.clamp(0.0, frame.framebuffer.height() as f32 - 1.0);
    }

    fn draw_background(&self, frame: &mut Frame<'_>) {
        let framebuffer = &mut frame.framebuffer;

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

    fn draw_actor(&self, frame: &mut Frame<'_>) {
        let color = if frame.input.mouse_button(MouseButton::Left).held() {
            Pixel::RED
        } else {
            Pixel::WHITE
        };

        frame
            .framebuffer
            .fill_circle(self.x.round() as i32, self.y.round() as i32, 8, color);

        if let Some((x, y)) = frame.input.mouse_position() {
            frame.framebuffer.draw_circle(x, y, 5, Pixel::GREEN);
        }
    }
}

impl Game for DemoGame {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if frame.input.key(Key::Escape).pressed() {
            return GameResult::Exit;
        }

        if frame.input.key(Key::Space).pressed() {
            self.toggle_palette();
        }

        self.time += frame.delta_time.as_secs_f32();
        self.update_position(frame);
        self.draw_background(frame);
        self.draw_actor(frame);

        GameResult::Continue
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

    use super::DemoGame;
    use gotoo_pixel_engine::{Frame, Framebuffer, Game, GameResult, Input};

    #[test]
    fn update_changes_cpu_framebuffer() {
        let mut demo = DemoGame::new(2.0, 2.0);
        let mut framebuffer = Framebuffer::new(4, 4);
        let input = Input::default();
        let mut frame = Frame {
            framebuffer: &mut framebuffer,
            input: &input,
            delta_time: Duration::from_millis(16),
        };

        assert_eq!(demo.update(&mut frame), GameResult::Continue);

        assert!(
            frame
                .framebuffer
                .as_rgba8()
                .iter()
                .any(|channel| *channel != 0)
        );
    }

    #[test]
    fn different_palettes_change_output() {
        let mut demo = DemoGame::new(2.0, 2.0);
        let mut first = Framebuffer::new(20, 20);
        let mut second = Framebuffer::new(20, 20);
        let input = Input::default();

        {
            let mut frame = Frame {
                framebuffer: &mut first,
                input: &input,
                delta_time: Duration::ZERO,
            };
            demo.update(&mut frame);
        }

        demo.toggle_palette();
        {
            let mut frame = Frame {
                framebuffer: &mut second,
                input: &input,
                delta_time: Duration::ZERO,
            };
            demo.update(&mut frame);
        }

        assert_ne!(first.as_rgba8(), second.as_rgba8());
    }
}
