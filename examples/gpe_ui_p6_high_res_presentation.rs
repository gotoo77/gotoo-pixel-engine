#[cfg(feature = "outline-fonts")]
use gotoo_pixel_engine::outline_text::OutlineFont;
use gotoo_pixel_engine::{
    EngineConfig, Frame, Framebuffer, Game, GameResult, Pixel, Rect, present_pixel_surface, run,
    ui::draw_text_centered,
};

const HOST_WIDTH: u32 = 1280;
const HOST_HEIGHT: u32 = 720;
const GAME_WIDTH: u32 = 320;
const GAME_HEIGHT: u32 = 180;

struct HighResPresentationMfe {
    game: Framebuffer,
    #[cfg(feature = "outline-fonts")]
    font: Option<OutlineFont>,
}

impl HighResPresentationMfe {
    fn new() -> Self {
        Self {
            game: Framebuffer::new(GAME_WIDTH, GAME_HEIGHT),
            #[cfg(feature = "outline-fonts")]
            font: OutlineFont::from_bytes(include_bytes!(
                "../assets/fonts/p4/unbounded/font.ttf"
            ))
            .ok(),
        }
    }

    fn render_game_surface(&mut self) {
        self.game.clear(Pixel::rgb(9, 14, 22));
        let tile = 12_u32;
        for y in 0..GAME_HEIGHT.div_ceil(tile) {
            for x in 0..GAME_WIDTH.div_ceil(tile) {
                let pixel = if (x + y) % 2 == 0 {
                    Pixel::rgb(36, 76, 92)
                } else {
                    Pixel::rgb(15, 35, 48)
                };
                self.game.fill_rect(
                    i32::try_from(x * tile).unwrap_or(i32::MAX),
                    i32::try_from(y * tile).unwrap_or(i32::MAX),
                    tile.min(GAME_WIDTH.saturating_sub(x * tile)),
                    tile.min(GAME_HEIGHT.saturating_sub(y * tile)),
                    pixel,
                );
            }
        }

        self.game.fill_rect(42, 54, 74, 52, Pixel::rgb(255, 205, 76));
        self.game.fill_rect(204, 84, 48, 48, Pixel::rgb(255, 103, 145));
        self.game.draw_rect(12, 12, 296, 156, Pixel::rgb(111, 238, 184));
    }

    fn render_high_res_ui(&mut self, frame: &mut Frame<'_>) {
        let header = Rect {
            x: 56,
            y: 42,
            width: 1168,
            height: 128,
        };
        frame
            .framebuffer
            .fill_rect(header.x, header.y, header.width, header.height, Pixel::rgb(4, 10, 17));
        frame.framebuffer.draw_rect(
            header.x,
            header.y,
            header.width,
            header.height,
            Pixel::rgb(52, 91, 112),
        );

        #[cfg(feature = "outline-fonts")]
        if let Some(font) = self.font.as_mut() {
            let _ = font.draw(
                frame.framebuffer,
                "GPE.UI HIGH-RES PRESENTATION",
                46.0,
                Rect {
                    x: 86,
                    y: 60,
                    width: 930,
                    height: 72,
                },
                Pixel::rgb(111, 238, 184),
            );
            let _ = font.draw(
                frame.framebuffer,
                "320x180 pixel surface @ integer nearest + 1280x720 outline UI",
                20.0,
                Rect {
                    x: 88,
                    y: 126,
                    width: 980,
                    height: 34,
                },
                Pixel::rgb(205, 218, 224),
            );
        }

        #[cfg(not(feature = "outline-fonts"))]
        {
            draw_text_centered(
                frame.framebuffer,
                header,
                "GPE.UI HIGH-RES PRESENTATION (enable outline-fonts)",
                2,
                Pixel::rgb(111, 238, 184),
            );
        }

        let button = Rect {
            x: 920,
            y: 616,
            width: 260,
            height: 62,
        };
        let hovered = frame.input.mouse_position().is_some_and(|(x, y)| {
            x >= button.x
                && y >= button.y
                && x < button.x + i32::try_from(button.width).unwrap_or(i32::MAX)
                && y < button.y + i32::try_from(button.height).unwrap_or(i32::MAX)
        });
        let button_bg = if hovered {
            Pixel::rgb(34, 103, 92)
        } else {
            Pixel::rgb(13, 38, 48)
        };
        frame.framebuffer.fill_rect(
            button.x,
            button.y,
            button.width,
            button.height,
            button_bg,
        );
        frame.framebuffer.draw_rect(
            button.x,
            button.y,
            button.width,
            button.height,
            Pixel::rgb(111, 238, 184),
        );

        #[cfg(feature = "outline-fonts")]
        if let Some(font) = self.font.as_mut() {
            let _ = font.draw(
                frame.framebuffer,
                if hovered { "POINTER: HIT" } else { "POINTER: MOVE HERE" },
                22.0,
                Rect {
                    x: button.x + 22,
                    y: button.y + 16,
                    width: button.width - 44,
                    height: 36,
                },
                Pixel::rgb(236, 244, 242),
            );
        }

        draw_text_centered(
            frame.framebuffer,
            Rect {
                x: 76,
                y: 630,
                width: 760,
                height: 30,
            },
            "PIXEL GAME: NEAREST INTEGER SCALE | UI: HOST RESOLUTION",
            2,
            Pixel::rgb(180, 194, 202),
        );
    }
}

impl Game for HighResPresentationMfe {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        frame.framebuffer.clear(Pixel::rgb(2, 6, 10));
        self.render_game_surface();

        let presentation = present_pixel_surface(
            frame.framebuffer,
            &self.game,
            Rect {
                x: 0,
                y: 0,
                width: HOST_WIDTH,
                height: HOST_HEIGHT,
            },
        )
        .expect("320x180 must fit at integer scale in 1280x720");
        debug_assert_eq!(presentation.scale, 4);

        self.render_high_res_ui(frame);
        GameResult::Continue
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run(
        EngineConfig {
            title: "GPE.UI P6 High-Res Presentation MFE".into(),
            framebuffer_width: HOST_WIDTH,
            framebuffer_height: HOST_HEIGHT,
            window_width: HOST_WIDTH,
            window_height: HOST_HEIGHT,
        },
        HighResPresentationMfe::new(),
    )?;
    Ok(())
}
