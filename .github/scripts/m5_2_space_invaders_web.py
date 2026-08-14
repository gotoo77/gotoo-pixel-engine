from pathlib import Path


def replace(path: str, old: str, new: str) -> None:
    file = Path(path)
    text = file.read_text()
    if old not in text:
        raise SystemExit(f"expected snippet not found in {path}: {old[:80]!r}")
    file.write_text(text.replace(old, new, 1))


# Core game: add a touch-only presentation mode while preserving the 256x224 world.
replace(
    "examples/space_invaders/game.rs",
    "use gotoo_pixel_engine::{\n    ActionId, ControlMap, Frame, Framebuffer, Game, GameResult, GamepadButton, Key, Pixel,\n};",
    "use gotoo_pixel_engine::{\n    ActionId, ControlMap, Frame, Framebuffer, Game, GameResult, GamepadButton, Key, Pixel, Rect,\n    ui::{VirtualButton, VirtualPad, draw_panel, draw_text_centered},\n};",
)
replace(
    "examples/space_invaders/game.rs",
    "pub const FRAMEBUFFER_WIDTH: u32 = 256;\npub const FRAMEBUFFER_HEIGHT: u32 = 224;",
    "pub const FRAMEBUFFER_WIDTH: u32 = 256;\npub const TOUCH_FRAMEBUFFER_WIDTH: u32 = 376;\npub const FRAMEBUFFER_HEIGHT: u32 = 224;",
)
replace(
    "examples/space_invaders/game.rs",
    "const ALIEN_BOTTOM_COLOR: Pixel = Pixel::rgb(120, 255, 120);",
    "const ALIEN_BOTTOM_COLOR: Pixel = Pixel::rgb(120, 255, 120);\nconst TOUCH_FILL: Pixel = Pixel::rgb(12, 24, 28);\nconst TOUCH_ACCENT: Pixel = Pixel::rgb(120, 255, 120);\nconst TOUCH_PANEL: Rect = Rect {\n    x: FRAMEBUFFER_WIDTH as i32,\n    y: 0,\n    width: TOUCH_FRAMEBUFFER_WIDTH - FRAMEBUFFER_WIDTH,\n    height: FRAMEBUFFER_HEIGHT,\n};\nconst TOUCH_FIRE: Rect = Rect {\n    x: 270,\n    y: 34,\n    width: 92,\n    height: 86,\n};\nconst TOUCH_LEFT: Rect = Rect {\n    x: 262,\n    y: 150,\n    width: 50,\n    height: 50,\n};\nconst TOUCH_RIGHT: Rect = Rect {\n    x: 320,\n    y: 150,\n    width: 50,\n    height: 50,\n};",
)
replace(
    "examples/space_invaders/game.rs",
    "pub struct SpaceInvadersGame {\n    world: SpaceInvadersWorld,\n    controls: ControlMap,\n}\n\nimpl SpaceInvadersGame {\n    pub fn new() -> Self {\n        Self {\n            world: SpaceInvadersWorld::new(),\n            controls: default_controls(),\n        }\n    }",
    "pub struct SpaceInvadersGame {\n    world: SpaceInvadersWorld,\n    controls: ControlMap,\n    virtual_pad: Option<VirtualPad>,\n}\n\nimpl SpaceInvadersGame {\n    pub fn new() -> Self {\n        Self::new_with_touch(false)\n    }\n\n    pub fn new_touch() -> Self {\n        Self::new_with_touch(true)\n    }\n\n    fn new_with_touch(touch: bool) -> Self {\n        let virtual_pad = touch.then(|| {\n            VirtualPad::new([\n                VirtualButton::new(CONTROL_LEFT, TOUCH_LEFT),\n                VirtualButton::new(CONTROL_RIGHT, TOUCH_RIGHT),\n                VirtualButton::new(CONTROL_FIRE, TOUCH_FIRE),\n            ])\n        });\n\n        Self {\n            world: SpaceInvadersWorld::new(),\n            controls: default_controls(),\n            virtual_pad,\n        }\n    }",
)
replace(
    "examples/space_invaders/game.rs",
    "    fn input(&mut self, frame: &Frame<'_>) -> GameResult {\n        self.controls.update(frame.input);",
    "    fn input(&mut self, frame: &Frame<'_>) -> GameResult {\n        if let Some(virtual_pad) = &mut self.virtual_pad {\n            virtual_pad.update(frame.input, &mut self.controls);\n        }\n        self.controls.update(frame.input);",
)
replace(
    "examples/space_invaders/game.rs",
    "        match self.world.state {\n            RoundState::Playing => {\n                fb.draw_text(66, 216, \"ARROWS/PAD MOVE  FIRE\", TEXT);\n            }\n            RoundState::Victory => {\n                fb.fill_rect(65, 91, 126, 39, BG);\n                fb.draw_rect(65, 91, 126, 39, FOREGROUND);\n                fb.draw_text(94, 100, \"YOU WIN\", FOREGROUND);\n                fb.draw_text(81, 116, \"FIRE TO REPLAY\", TEXT);\n            }\n            RoundState::GameOver => {\n                fb.fill_rect(65, 91, 126, 39, BG);\n                fb.draw_rect(65, 91, 126, 39, DANGER);\n                fb.draw_text(88, 100, \"GAME OVER\", DANGER);\n                fb.draw_text(81, 116, \"FIRE TO REPLAY\", TEXT);\n            }\n        }\n    }\n}",
    "        match self.world.state {\n            RoundState::Playing => {\n                fb.draw_text(66, 216, \"ARROWS/PAD MOVE  FIRE\", TEXT);\n            }\n            RoundState::Victory => {\n                fb.fill_rect(65, 91, 126, 39, BG);\n                fb.draw_rect(65, 91, 126, 39, FOREGROUND);\n                fb.draw_text(94, 100, \"YOU WIN\", FOREGROUND);\n                fb.draw_text(81, 116, \"FIRE TO REPLAY\", TEXT);\n            }\n            RoundState::GameOver => {\n                fb.fill_rect(65, 91, 126, 39, BG);\n                fb.draw_rect(65, 91, 126, 39, DANGER);\n                fb.draw_text(88, 100, \"GAME OVER\", DANGER);\n                fb.draw_text(81, 116, \"FIRE TO REPLAY\", TEXT);\n            }\n        }\n\n        if self.virtual_pad.is_some() {\n            draw_touch_controls(fb);\n        }\n    }\n}",
)
replace(
    "examples/space_invaders/game.rs",
    "fn default_controls() -> ControlMap {",
    "fn draw_touch_controls(framebuffer: &mut Framebuffer) {\n    draw_panel(framebuffer, TOUCH_PANEL, BG, FOREGROUND);\n    draw_text_centered(\n        framebuffer,\n        Rect {\n            x: TOUCH_PANEL.x,\n            y: 6,\n            width: TOUCH_PANEL.width,\n            height: 12,\n        },\n        \"TOUCH\",\n        1,\n        TEXT,\n    );\n\n    for (rect, label) in [(TOUCH_FIRE, \"FIRE\"), (TOUCH_LEFT, \"LEFT\"), (TOUCH_RIGHT, \"RIGHT\")] {\n        framebuffer.fill_rect(rect.x, rect.y, rect.width, rect.height, TOUCH_FILL);\n        framebuffer.draw_rect(rect.x, rect.y, rect.width, rect.height, FOREGROUND);\n        draw_text_centered(framebuffer, rect, label, 1, TOUCH_ACCENT);\n    }\n}\n\nfn default_controls() -> ControlMap {",
)
replace(
    "examples/space_invaders/game.rs",
    "    #[test]\n    fn starts_with_classic_fifty_five_invaders() {",
    "    #[test]\n    fn touch_mode_exposes_three_virtual_actions() {\n        let game = SpaceInvadersGame::new_touch();\n        let pad = game.virtual_pad.as_ref().expect(\"touch mode should own a virtual pad\");\n        let actions = pad.buttons().iter().map(|button| button.action).collect::<Vec<_>>();\n        assert_eq!(actions, vec![CONTROL_LEFT, CONTROL_RIGHT, CONTROL_FIRE]);\n        assert_eq!(TOUCH_PANEL.x, FRAMEBUFFER_WIDTH as i32);\n        assert_eq!(TOUCH_FRAMEBUFFER_WIDTH, 376);\n    }\n\n    #[test]\n    fn starts_with_classic_fifty_five_invaders() {",
)

# Enhanced wrapper: same audiovisual layer, different core constructor for web touch mode.
replace(
    "examples/space_invaders/enhanced.rs",
    "impl EnhancedSpaceInvadersGame {\n    pub fn new() -> Self {\n        let mut sounds = SoundBank::new();",
    "impl EnhancedSpaceInvadersGame {\n    pub fn new() -> Self {\n        Self::new_with_touch(false)\n    }\n\n    // Consumed by the separate `space_invaders_web` entrypoint.\n    #[allow(dead_code)]\n    pub fn new_touch() -> Self {\n        Self::new_with_touch(true)\n    }\n\n    fn new_with_touch(touch: bool) -> Self {\n        let mut sounds = SoundBank::new();",
)
replace(
    "examples/space_invaders/enhanced.rs",
    "            core: SpaceInvadersGame::new(),",
    "            core: if touch {\n                SpaceInvadersGame::new_touch()\n            } else {\n                SpaceInvadersGame::new()\n            },",
)

# WASM entrypoint starts directly in-game; Arcade will own the shared launcher later.
Path("examples/space_invaders_web.rs").write_text('''#[allow(dead_code)]\n#[path = "space_invaders/enhanced.rs"]\nmod game;\n\nuse game::{EnhancedSpaceInvadersGame, FRAMEBUFFER_HEIGHT, TOUCH_FRAMEBUFFER_WIDTH};\nuse gotoo_pixel_engine::{EngineConfig, run};\nuse wasm_bindgen::prelude::*;\n\n#[wasm_bindgen(start)]\npub fn start() -> Result<(), JsValue> {\n    run(\n        EngineConfig {\n            title: "Space Invaders".into(),\n            framebuffer_width: TOUCH_FRAMEBUFFER_WIDTH,\n            framebuffer_height: FRAMEBUFFER_HEIGHT,\n            window_width: TOUCH_FRAMEBUFFER_WIDTH * 3,\n            window_height: FRAMEBUFFER_HEIGHT * 3,\n        },\n        EnhancedSpaceInvadersGame::new_touch(),\n    )\n    .map_err(|err| JsValue::from_str(&err.to_string()))\n}\n''')

replace(
    "Cargo.toml",
    "[[example]]\nname = \"tetris_web\"\npath = \"examples/tetris_web.rs\"\ncrate-type = [\"cdylib\"]",
    "[[example]]\nname = \"tetris_web\"\npath = \"examples/tetris_web.rs\"\ncrate-type = [\"cdylib\"]\n\n[[example]]\nname = \"space_invaders_web\"\npath = \"examples/space_invaders_web.rs\"\ncrate-type = [\"cdylib\"]",
)
replace(
    "scripts/build-web.sh",
    "build_web_example tetris_web\n",
    "build_web_example tetris_web\nbuild_web_example space_invaders_web\n",
)

Path("web/space_invaders.html").write_text('''<!doctype html>\n<html lang="en">\n  <head>\n    <meta charset="utf-8">\n    <meta name="viewport" content="width=device-width, initial-scale=1">\n    <title>Space Invaders</title>\n    <style>\n      html,\n      body {\n        width: 100%;\n        height: 100%;\n        margin: 0;\n        background: #080c0c;\n      }\n\n      body {\n        display: grid;\n        place-items: center;\n      }\n\n      canvas {\n        display: block;\n        width: 1128px;\n        height: 672px;\n        max-width: 100vw;\n        max-height: 100vh;\n        touch-action: none;\n        image-rendering: pixelated;\n      }\n    </style>\n  </head>\n  <body>\n    <script type="module">\n      import init from "./pkg/space_invaders_web.js";\n\n      init();\n    </script>\n  </body>\n</html>\n''')
