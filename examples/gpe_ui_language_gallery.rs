use gotoo_pixel_engine::{
    EngineConfig, Frame, Framebuffer, Game, GameResult, Key, Pixel, Rect, TextRenderer,
    outline_text::OutlineFont,
    run,
    ui::{
        CHINESE_SIMPLIFIED, CHINESE_TRADITIONAL, ENGLISH, FRENCH, GERMAN, ITALIAN, JAPANESE,
        KOREAN, PORTUGUESE_BRAZIL, PORTUGUESE_PORTUGAL, RUSSIAN, SPANISH, FlagIcon,
        LanguageOption, UiIcon, classify_text_script, recommended_raster_scale,
    },
};

const WIDTH: u32 = 920;
const HEIGHT: u32 = 720;
const BG: Pixel = Pixel::rgb(10, 12, 18);
const PANEL: Pixel = Pixel::rgb(18, 22, 31);
const BORDER: Pixel = Pixel::rgb(67, 79, 98);
const TEXT: Pixel = Pixel::rgb(232, 236, 244);
const MUTED: Pixel = Pixel::rgb(151, 161, 177);
const ACCENT: Pixel = Pixel::rgb(225, 132, 255);

const LANGUAGES: [LanguageOption; 12] = [
    ENGLISH,
    FRENCH,
    GERMAN,
    SPANISH,
    ITALIAN,
    PORTUGUESE_PORTUGAL,
    PORTUGUESE_BRAZIL,
    RUSSIAN,
    JAPANESE,
    KOREAN,
    CHINESE_SIMPLIFIED,
    CHINESE_TRADITIONAL,
];

struct Gallery {
    ui_font: OutlineFont,
}

impl Gallery {
    fn new() -> Result<Self, &'static str> {
        Ok(Self {
            ui_font: OutlineFont::from_bytes(gotoo_pixel_engine::ui::fonts::EXO_2)?,
        })
    }

    fn paint(&mut self, framebuffer: &mut Framebuffer) {
        framebuffer.clear(BG);
        self.ui_font.draw(
            framebuffer,
            "GPE.UI / Language + CJK validation",
            30.0,
            Rect {
                x: 32,
                y: 24,
                width: WIDTH - 64,
                height: 50,
            },
            TEXT,
        );
        self.ui_font.draw(
            framebuffer,
            "Flags are secondary cues. Locale/name remain authoritative.",
            17.0,
            Rect {
                x: 32,
                y: 70,
                width: WIDTH - 64,
                height: 32,
            },
            MUTED,
        );

        let row_w = 410_u32;
        let row_h = 58_u32;
        let gap_x = 24_i32;
        let gap_y = 12_i32;
        let start_x = 32_i32;
        let start_y = 120_i32;

        for (index, language) in LANGUAGES.iter().copied().enumerate() {
            let column = (index % 2) as i32;
            let row = (index / 2) as i32;
            let x = start_x + column * (row_w as i32 + gap_x);
            let y = start_y + row * (row_h as i32 + gap_y);

            framebuffer.fill_rect(x, y, row_w, row_h, PANEL);
            framebuffer.draw_rect(x, y, row_w, row_h, BORDER);

            if let Some(flag) = language.flag {
                flag.draw(
                    framebuffer,
                    Rect {
                        x: x + 16,
                        y: y + 19,
                        width: FlagIcon::PREFERRED_WIDTH,
                        height: FlagIcon::PREFERRED_HEIGHT,
                    },
                );
            }

            let supported = self.ui_font.supports_text(language.native_name);
            let label = if supported {
                format!("{}   {}", language.locale, language.native_name)
            } else {
                format!("{}   [fallback font required]", language.locale)
            };
            self.ui_font.draw(
                framebuffer,
                &label,
                19.0,
                Rect {
                    x: x + 56,
                    y: y + 10,
                    width: row_w.saturating_sub(72),
                    height: 28,
                },
                if supported { TEXT } else { ACCENT },
            );

            let script = classify_text_script(language.native_name);
            let policy = recommended_raster_scale(language.native_name);
            let detail = format!("script={script:?}   raster={policy}x   outline={}", if supported { "yes" } else { "no" });
            self.ui_font.draw(
                framebuffer,
                &detail,
                13.0,
                Rect {
                    x: x + 56,
                    y: y + 34,
                    width: row_w.saturating_sub(72),
                    height: 18,
                },
                MUTED,
            );
        }

        let sample_y = 570_i32;
        framebuffer.fill_rect(32, sample_y, WIDTH - 64, 104, PANEL);
        framebuffer.draw_rect(32, sample_y, WIDTH - 64, 104, BORDER);
        self.ui_font.draw(
            framebuffer,
            "Raster precision comparison (same logical size)",
            17.0,
            Rect {
                x: 48,
                y: sample_y + 12,
                width: WIDTH - 96,
                height: 24,
            },
            TEXT,
        );
        for (index, scale) in [1_u32, 2, 3, 4].into_iter().enumerate() {
            let x = 48 + index as i32 * 205;
            let bounds = Rect {
                x,
                y: sample_y + 44,
                width: 180,
                height: 42,
            };
            let _ = self.ui_font.draw_supersampled(
                framebuffer,
                &format!("GPE {scale}x"),
                22.0,
                bounds,
                if scale == 3 { ACCENT } else { TEXT },
                scale,
            );
        }

        TextRenderer::default().draw(
            framebuffer,
            32,
            HEIGHT as i32 - 24,
            "ESC = quit   /   CJK rows intentionally expose fallback coverage",
            MUTED,
        );
    }
}

impl Game for Gallery {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if frame.input.key(Key::Escape).pressed() {
            return GameResult::Exit;
        }
        self.paint(frame.framebuffer);
        GameResult::Continue
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run(
        EngineConfig {
            title: "GPE.UI / Language + CJK Gallery".into(),
            framebuffer_width: WIDTH,
            framebuffer_height: HEIGHT,
            window_width: WIDTH,
            window_height: HEIGHT,
        },
        Gallery::new()?,
    )?;
    Ok(())
}
