use std::{io, path::PathBuf};

use gotoo_pixel_engine::{
    EngineConfig, Frame, Framebuffer, Game, GameResult, Key, OutlineFontStack, Pixel, Rect,
    TextRenderer,
    outline_text::OutlineFont,
    run,
    ui::{
        CHINESE_SIMPLIFIED, CHINESE_TRADITIONAL, ENGLISH, FRENCH, GERMAN, ITALIAN, JAPANESE,
        KOREAN, PORTUGUESE_BRAZIL, PORTUGUESE_PORTUGAL, RUSSIAN, SPANISH, FlagIcon,
        LanguageOption, classify_text_script, recommended_raster_scale,
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
    font_stack: OutlineFontStack,
    cjk_source: Option<String>,
}

impl Gallery {
    fn new(cjk_font_path: Option<PathBuf>) -> io::Result<Self> {
        let primary_bytes = gotoo_pixel_engine::ui::fonts::EXO_2;
        let ui_font = outline_font(primary_bytes)?;
        let mut font_stack = OutlineFontStack::new(outline_font(primary_bytes)?);

        let cjk_source = if let Some(path) = cjk_font_path {
            let bytes = std::fs::read(&path)?;
            font_stack.push_fallback(outline_font(&bytes)?);
            Some(path.display().to_string())
        } else {
            None
        };

        Ok(Self {
            ui_font,
            font_stack,
            cjk_source,
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

        let subtitle = match &self.cjk_source {
            Some(source) => format!("Outline fallback active: {source}"),
            None => "No CJK outline fallback loaded. Set GPE_CJK_FONT or use --cjk-font <path>."
                .to_owned(),
        };
        self.ui_font.draw(
            framebuffer,
            &subtitle,
            15.0,
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

            let preferred_label = format!("{}   {}", language.locale, language.native_name);
            let resolved = self.font_stack.resolve_font_index(&preferred_label);
            let raster_scale = recommended_raster_scale(language.native_name);
            let label = if resolved.is_some() {
                preferred_label
            } else {
                format!("{}   [fallback font required]", language.locale)
            };
            let label_bounds = Rect {
                x: x + 56,
                y: y + 8,
                width: row_w.saturating_sub(72),
                height: 30,
            };

            if resolved.is_some() {
                let _ = self.font_stack.draw_supersampled(
                    framebuffer,
                    &label,
                    19.0,
                    label_bounds,
                    TEXT,
                    raster_scale,
                );
            } else {
                self.ui_font
                    .draw(framebuffer, &label, 19.0, label_bounds, ACCENT);
            }

            let script = classify_text_script(language.native_name);
            let face = match resolved {
                Some(0) => "primary",
                Some(index) => {
                    if index == 1 {
                        "fallback#1"
                    } else {
                        "fallback"
                    }
                }
                None => "missing",
            };
            let detail = format!("script={script:?}   raster={raster_scale}x   face={face}");
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

        let cjk_sample = "日本語";
        if self.font_stack.supports_text(cjk_sample) {
            for (index, scale) in [1_u32, 2, 3, 4].into_iter().enumerate() {
                let x = 48 + index as i32 * 205;
                let bounds = Rect {
                    x,
                    y: sample_y + 44,
                    width: 180,
                    height: 42,
                };
                let _ = self.font_stack.draw_supersampled(
                    framebuffer,
                    cjk_sample,
                    22.0,
                    bounds,
                    if scale == 3 { ACCENT } else { TEXT },
                    scale,
                );
            }
        } else {
            for (index, scale) in [1_u32, 2, 3, 4].into_iter().enumerate() {
                let x = 48 + index as i32 * 205;
                let _ = self.ui_font.draw_supersampled(
                    framebuffer,
                    &format!("GPE {scale}x"),
                    22.0,
                    Rect {
                        x,
                        y: sample_y + 44,
                        width: 180,
                        height: 42,
                    },
                    if scale == 3 { ACCENT } else { TEXT },
                    scale,
                );
            }
        }

        TextRenderer::default().draw(
            framebuffer,
            32,
            HEIGHT as i32 - 24,
            "ESC = quit   /   optional CJK font remains consumer-provided",
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

fn outline_font(bytes: &[u8]) -> io::Result<OutlineFont> {
    OutlineFont::from_bytes(bytes).map_err(io::Error::other)
}

fn cjk_font_path_from_args() -> io::Result<Option<PathBuf>> {
    let mut args = std::env::args_os().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--cjk-font" {
            return args.next().map(PathBuf::from).map(Some).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "--cjk-font requires a path")
            });
        }
        if let Some(value) = arg.to_str().and_then(|arg| arg.strip_prefix("--cjk-font=")) {
            return Ok(Some(PathBuf::from(value)));
        }
    }

    Ok(std::env::var_os("GPE_CJK_FONT").map(PathBuf::from))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cjk_font_path = cjk_font_path_from_args()?;
    run(
        EngineConfig {
            title: "GPE.UI / Language + CJK Gallery".into(),
            framebuffer_width: WIDTH,
            framebuffer_height: HEIGHT,
            window_width: WIDTH,
            window_height: HEIGHT,
        },
        Gallery::new(cjk_font_path)?,
    )?;
    Ok(())
}
