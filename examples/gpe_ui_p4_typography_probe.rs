use gotoo_pixel_engine::{
    EngineConfig, Font, Frame, Framebuffer, Game, GameResult, Key, Pixel, TextRenderer, run,
};

const WIDTH: u32 = 640;
const HEIGHT: u32 = 400;
const SAMPLES: [(&str, &str); 5] = [
    ("SETTINGS", "Volume: 0.65  Enabled: ON"),
    ("FRENCH", "\u{c9}t\u{e9}  Fran\u{e7}ais  Co\u{fb}t  O\u{f9}"),
    (
        "PUNCTUATION",
        "A\u{2013}B A\u{2014}B  \u{2022}  \u{2190} \u{2192} \u{2191} \u{2193}",
    ),
    ("LITERAL QUESTION", "?"),
    ("MISSING U+10FFFF", "\u{10ffff}"),
];

fn paint(framebuffer: &mut Framebuffer) {
    framebuffer.clear(Pixel::rgb(15, 18, 22));
    let heading = TextRenderer::default();
    heading.draw(
        framebuffer,
        20,
        18,
        "GPE.UI P4 / TYPOGRAPHY",
        Pixel::rgb(133, 217, 188),
    );
    for (column, font) in [Font::Pixel5x7, Font::Mini3x5].into_iter().enumerate() {
        let x = 20 + column as i32 * 320;
        let text = TextRenderer::new(font);
        heading.draw(framebuffer, x, 46, font.name(), Pixel::rgb(240, 187, 85));
        for (row, (label, sample)) in SAMPLES.iter().enumerate() {
            let y = 76 + row as i32 * 58;
            heading.draw(framebuffer, x, y, label, Pixel::rgb(145, 155, 166));
            text.draw(framebuffer, x, y + 14, sample, Pixel::rgb(232, 237, 240));
            text.draw_scaled(framebuffer, x, y + 28, sample, 2, Pixel::rgb(232, 237, 240));
        }
    }
}

struct TypographyProbe;

impl Game for TypographyProbe {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if frame.input.key(Key::Escape).pressed() {
            return GameResult::Exit;
        }
        paint(frame.framebuffer);
        GameResult::Continue
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Some(path) = std::env::args().nth(1) {
        let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
        paint(&mut framebuffer);
        let file = std::io::BufWriter::new(std::fs::File::create(path)?);
        let mut encoder = png::Encoder::new(file, WIDTH, HEIGHT);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()?
            .write_image_data(framebuffer.as_rgba8())?;
        return Ok(());
    }
    run(
        EngineConfig {
            title: "GPE.UI P4 Typography Probe".into(),
            framebuffer_width: WIDTH,
            framebuffer_height: HEIGHT,
            window_width: WIDTH * 2,
            window_height: HEIGHT * 2,
        },
        TypographyProbe,
    )?;
    Ok(())
}
