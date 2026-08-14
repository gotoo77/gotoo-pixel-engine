from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one match, found {count}")
    return text.replace(old, new, 1)


# Tetris game: add the touch consumer directly around the existing ControlMap.
path = Path("examples/tetris/game.rs")
text = path.read_text()
text = replace_once(
    text,
    '''use gotoo_pixel_engine::{
    ActionId, ControlMap, Frame, Game, GameResult, GamepadButton, Key, Pixel, SoundBank, SoundId,
    pcm16_mono_wav,
};
''',
    '''use gotoo_pixel_engine::{
    ActionId, ControlMap, Frame, Framebuffer, Game, GameResult, GamepadButton, Key, Pixel, Rect,
    SoundBank, SoundId, pcm16_mono_wav,
    ui::{VirtualButton, VirtualPad, draw_panel, draw_text_centered},
};
''',
    "imports",
)
text = replace_once(
    text,
    '''pub const FRAMEBUFFER_WIDTH: u32 = 220;
pub const FRAMEBUFFER_HEIGHT: u32 = 224;
''',
    '''pub const FRAMEBUFFER_WIDTH: u32 = 220;
pub const TOUCH_FRAMEBUFFER_WIDTH: u32 = 360;
pub const FRAMEBUFFER_HEIGHT: u32 = 224;
''',
    "framebuffer constants",
)
text = replace_once(
    text,
    '''const GAME_OVER: Pixel = Pixel::rgb(245, 76, 76);
const COLORS: [Pixel; 7] = [
''',
    '''const GAME_OVER: Pixel = Pixel::rgb(245, 76, 76);
const TOUCH_FILL: Pixel = Pixel::rgb(18, 28, 34);
const TOUCH_ACCENT: Pixel = Pixel::rgb(80, 220, 230);
const TOUCH_PANEL: Rect = Rect {
    x: FRAMEBUFFER_WIDTH as i32,
    y: 0,
    width: TOUCH_FRAMEBUFFER_WIDTH - FRAMEBUFFER_WIDTH,
    height: FRAMEBUFFER_HEIGHT,
};
const TOUCH_ROTATE: Rect = Rect {
    x: 244,
    y: 20,
    width: 92,
    height: 42,
};
const TOUCH_LEFT: Rect = Rect {
    x: 226,
    y: 82,
    width: 52,
    height: 44,
};
const TOUCH_RIGHT: Rect = Rect {
    x: 302,
    y: 82,
    width: 52,
    height: 44,
};
const TOUCH_SOFT_DROP: Rect = Rect {
    x: 264,
    y: 136,
    width: 52,
    height: 40,
};
const TOUCH_HARD_DROP: Rect = Rect {
    x: 244,
    y: 184,
    width: 92,
    height: 32,
};
const COLORS: [Pixel; 7] = [
''',
    "touch geometry",
)
text = replace_once(
    text,
    '''pub struct TetrisGame {
    world: TetrisWorld,
    accumulator: Duration,
    controls: ControlMap,
    sounds: SoundBank,
}

impl TetrisGame {
    pub fn new() -> Self {
        Self {
            world: TetrisWorld::new(),
            accumulator: Duration::ZERO,
            controls: default_controls(),
            sounds: tetris_sound_bank(),
        }
    }

    fn input(&mut self, frame: &Frame<'_>) -> GameResult {
        self.controls.update(frame.input);
''',
    '''pub struct TetrisGame {
    world: TetrisWorld,
    accumulator: Duration,
    controls: ControlMap,
    virtual_pad: Option<VirtualPad>,
    sounds: SoundBank,
}

impl TetrisGame {
    pub fn new() -> Self {
        Self::new_with_touch(false)
    }

    pub fn new_touch() -> Self {
        Self::new_with_touch(true)
    }

    fn new_with_touch(touch: bool) -> Self {
        let virtual_pad = touch.then(|| {
            VirtualPad::new([
                VirtualButton::new(CONTROL_LEFT, TOUCH_LEFT),
                VirtualButton::new(CONTROL_RIGHT, TOUCH_RIGHT),
                VirtualButton::new(CONTROL_ROTATE, TOUCH_ROTATE),
                VirtualButton::new(CONTROL_SOFT_DROP, TOUCH_SOFT_DROP),
                VirtualButton::new(CONTROL_HARD_DROP, TOUCH_HARD_DROP),
            ])
        });

        Self {
            world: TetrisWorld::new(),
            accumulator: Duration::ZERO,
            controls: default_controls(),
            virtual_pad,
            sounds: tetris_sound_bank(),
        }
    }

    fn input(&mut self, frame: &Frame<'_>) -> GameResult {
        if let Some(virtual_pad) = &mut self.virtual_pad {
            virtual_pad.update(frame.input, &mut self.controls);
        }
        self.controls.update(frame.input);
''',
    "TetrisGame touch state",
)
text = replace_once(
    text,
    '''        if self.world.game_over {
            fb.fill_rect(18, 92, 80, 42, BG);
            fb.draw_rect(18, 92, 80, 42, GAME_OVER);
            fb.draw_text(25, 101, "GAME OVER", GAME_OVER);
            fb.draw_text(24, 117, "DROP TO REPLAY", TEXT);
        }
    }
}
''',
    '''        if self.world.game_over {
            fb.fill_rect(18, 92, 80, 42, BG);
            fb.draw_rect(18, 92, 80, 42, GAME_OVER);
            fb.draw_text(25, 101, "GAME OVER", GAME_OVER);
            fb.draw_text(24, 117, "DROP TO REPLAY", TEXT);
        }
        if self.virtual_pad.is_some() {
            draw_touch_controls(fb);
        }
    }
}
''',
    "touch rendering hook",
)
text = replace_once(
    text,
    '''fn default_controls() -> ControlMap {
''',
    '''fn draw_touch_controls(framebuffer: &mut Framebuffer) {
    draw_panel(framebuffer, TOUCH_PANEL, BG, BORDER);
    draw_text_centered(
        framebuffer,
        Rect {
            x: TOUCH_PANEL.x,
            y: 4,
            width: TOUCH_PANEL.width,
            height: 12,
        },
        "TOUCH",
        1,
        TEXT,
    );

    for (rect, label) in [
        (TOUCH_ROTATE, "ROTATE"),
        (TOUCH_LEFT, "LEFT"),
        (TOUCH_RIGHT, "RIGHT"),
        (TOUCH_SOFT_DROP, "DOWN"),
        (TOUCH_HARD_DROP, "DROP"),
    ] {
        framebuffer.fill_rect(rect.x, rect.y, rect.width, rect.height, TOUCH_FILL);
        framebuffer.draw_rect(rect.x, rect.y, rect.width, rect.height, BORDER);
        draw_text_centered(framebuffer, rect, label, 1, TOUCH_ACCENT);
    }
}

fn default_controls() -> ControlMap {
''',
    "draw touch controls",
)
text = replace_once(
    text,
    '''    #[test]
    fn bag_contains_every_piece_once() {
''',
    '''    #[test]
    fn touch_mode_exposes_five_virtual_actions() {
        let game = TetrisGame::new_touch();
        assert_eq!(game.virtual_pad.as_ref().map(|pad| pad.buttons().len()), Some(5));
    }

    #[test]
    fn bag_contains_every_piece_once() {
''',
    "touch test",
)
path.write_text(text)

# Web entry point.
Path("examples/tetris_web.rs").write_text('''#[allow(dead_code)]
#[path = "tetris/game.rs"]
mod game;

use game::{FRAMEBUFFER_HEIGHT, TOUCH_FRAMEBUFFER_WIDTH, TetrisGame};
use gotoo_pixel_engine::{EngineConfig, run};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    run(
        EngineConfig {
            title: "Tetris".into(),
            framebuffer_width: TOUCH_FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width: TOUCH_FRAMEBUFFER_WIDTH * 3,
            window_height: FRAMEBUFFER_HEIGHT * 3,
        },
        TetrisGame::new_touch(),
    )
    .map_err(|err| JsValue::from_str(&err.to_string()))
}
''')

# Cargo example metadata.
path = Path("Cargo.toml")
text = path.read_text()
text = replace_once(
    text,
    '''[[example]]
name = "breakout_web"
path = "examples/breakout_web.rs"
crate-type = ["cdylib"]

[[bench]]
''',
    '''[[example]]
name = "breakout_web"
path = "examples/breakout_web.rs"
crate-type = ["cdylib"]

[[example]]
name = "tetris_web"
path = "examples/tetris_web.rs"
crate-type = ["cdylib"]

[[bench]]
''',
    "Cargo example",
)
path.write_text(text)

# Local web builder.
path = Path("scripts/build-web.sh")
text = path.read_text()
text = replace_once(
    text,
    '''build_web_example snake_web
build_web_example breakout_web
''',
    '''build_web_example snake_web
build_web_example breakout_web
build_web_example tetris_web
''',
    "build web examples",
)
path.write_text(text)

# GitHub Pages build/deploy.
path = Path(".github/workflows/pages.yml")
text = path.read_text()
text = replace_once(
    text,
    '''          cargo build --release --target wasm32-unknown-unknown --example snake_web
          cargo build --release --target wasm32-unknown-unknown --example breakout_web
''',
    '''          cargo build --release --target wasm32-unknown-unknown --example snake_web
          cargo build --release --target wasm32-unknown-unknown --example breakout_web
          cargo build --release --target wasm32-unknown-unknown --example tetris_web
''',
    "Pages build",
)
text = replace_once(
    text,
    '''          cp web/snake.html dist/snake.html
          cp web/breakout.html dist/breakout.html
''',
    '''          cp web/snake.html dist/snake.html
          cp web/breakout.html dist/breakout.html
          cp web/tetris.html dist/tetris.html
''',
    "Pages HTML copy",
)
text = replace_once(
    text,
    '''          wasm-bindgen \\
            --target web \\
            --out-dir dist/pkg \\
            target/wasm32-unknown-unknown/release/examples/breakout_web.wasm
''',
    '''          wasm-bindgen \\
            --target web \\
            --out-dir dist/pkg \\
            target/wasm32-unknown-unknown/release/examples/breakout_web.wasm
          wasm-bindgen \\
            --target web \\
            --out-dir dist/pkg \\
            target/wasm32-unknown-unknown/release/examples/tetris_web.wasm
''',
    "Pages wasm-bindgen",
)
path.write_text(text)

# Browser page.
Path("web/tetris.html").write_text('''<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Tetris</title>
    <style>
      html,
      body {
        width: 100%;
        height: 100%;
        margin: 0;
        background: #111;
      }

      body {
        display: grid;
        place-items: center;
      }

      canvas {
        display: block;
        width: 1080px;
        height: 672px;
        max-width: 100vw;
        max-height: 100vh;
        touch-action: none;
        image-rendering: pixelated;
      }
    </style>
  </head>
  <body>
    <script type="module">
      import init from "./pkg/tetris_web.js";

      init();
    </script>
  </body>
</html>
''')
