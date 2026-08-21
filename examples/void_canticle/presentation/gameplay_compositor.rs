const GAMEPLAY_PRESENTATION_VERSION: &str = "VC3.3";

struct HdGameplayCompositeApp {
    inner: VoidCanticleApp,
    composition_framebuffer: Framebuffer,
}

impl HdGameplayCompositeApp {
    fn new() -> Self {
        Self {
            inner: VoidCanticleApp::new(),
            composition_framebuffer: Framebuffer::new(FRONT_WIDTH, FRONT_HEIGHT),
        }
    }
}

impl Game for HdGameplayCompositeApp {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        self.composition_framebuffer.clear(Pixel::TRANSPARENT);

        let result = {
            let mut inner_frame = Frame {
                framebuffer: &mut self.composition_framebuffer,
                input: frame.input,
                delta_time: frame.delta_time,
                storage: &mut *frame.storage,
                audio: &mut *frame.audio,
                surface_size: frame.surface_size,
                viewport: gotoo_pixel_engine::Viewport::new(
                    frame.surface_size,
                    gotoo_pixel_engine::Size {
                        width: FRONT_WIDTH,
                        height: FRONT_HEIGHT,
                    },
                ),
            };
            self.inner.update(&mut inner_frame)
        };

        render_abyssal_void_background(frame.framebuffer, self.inner.presentation_time);
        composite_front_frame(&self.composition_framebuffer, frame.framebuffer);
        result
    }
}

fn composite_front_frame(source: &Framebuffer, destination: &mut Framebuffer) {
    debug_assert_eq!(source.width(), FRONT_WIDTH);
    debug_assert_eq!(source.height(), FRONT_HEIGHT);
    debug_assert_eq!(destination.width(), FRONT_WIDTH);
    debug_assert_eq!(destination.height(), FRONT_HEIGHT);

    let source_rgba = source.as_rgba8();
    for y in 0..FRONT_HEIGHT {
        for x in 0..FRONT_WIDTH {
            let index = ((y * FRONT_WIDTH + x) * 4) as usize;
            let source_pixel = Pixel::rgba(
                source_rgba[index],
                source_rgba[index + 1],
                source_rgba[index + 2],
                source_rgba[index + 3],
            );

            match source_pixel.a {
                0 => {}
                255 => destination.set_pixel_in_bounds(x, y, source_pixel),
                _ => {
                    let background = destination
                        .pixel(x as i32, y as i32)
                        .unwrap_or(Pixel::TRANSPARENT);
                    destination.set_pixel_in_bounds(x, y, composite_over(background, source_pixel));
                }
            }
        }
    }
}

pub fn run_void_canticle_composited_with_obs_mirror() -> Result<(), EngineError> {
    run(
        EngineConfig {
            title: format!(
                "Void Canticle {GAMEPLAY_PRESENTATION_VERSION} - Gotoo Pixel Engine"
            ),
            framebuffer_width: FRONT_WIDTH,
            framebuffer_height: FRONT_HEIGHT,
            window_width: FRONT_WIDTH,
            window_height: FRONT_HEIGHT,
        },
        gotoo_pixel_engine::ObsMirrorGame::from_env(
            HdGameplayCompositeApp::new(),
            FRONT_WIDTH,
            FRONT_HEIGHT,
        ),
    )
}

#[cfg(test)]
mod gameplay_compositor_tests {
    use super::*;

    #[test]
    fn transparent_gameplay_pixels_leave_hd_background_untouched() {
        let mut source = Framebuffer::new(FRONT_WIDTH, FRONT_HEIGHT);
        source.set_pixel_in_bounds(10, 12, Pixel::RED);

        let background = Pixel::rgb(3, 7, 19);
        let mut destination = Framebuffer::new(FRONT_WIDTH, FRONT_HEIGHT);
        destination.clear(background);

        composite_front_frame(&source, &mut destination);

        assert_eq!(destination.pixel(0, 0), Some(background));
        assert_eq!(destination.pixel(10, 12), Some(Pixel::RED));
    }

    #[test]
    fn semitransparent_gameplay_pixels_are_composited_over_hd_background() {
        let mut source = Framebuffer::new(FRONT_WIDTH, FRONT_HEIGHT);
        source.set_pixel_in_bounds(5, 7, Pixel::rgba(255, 0, 0, 128));

        let background = Pixel::rgb(0, 0, 255);
        let mut destination = Framebuffer::new(FRONT_WIDTH, FRONT_HEIGHT);
        destination.clear(background);

        composite_front_frame(&source, &mut destination);

        let pixel = destination.pixel(5, 7).unwrap();
        assert_eq!(pixel.a, 255);
        assert!(pixel.r > 120);
        assert!(pixel.b > 120);
    }
}