from pathlib import Path


def replace_once(text: str, old: str, new: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"expected one match, found {count}: {old[:80]!r}")
    return text.replace(old, new, 1)


pong_path = Path("examples/pong.rs")
pong = pong_path.read_text()

pong = replace_once(
    pong,
    "        MenuState, draw_menu_item, draw_panel, draw_text_centered, menu_confirm_pressed,\n        menu_down_pressed, menu_up_pressed,\n",
    "        MenuState, VirtualButton, VirtualPad, draw_menu_item, draw_panel, draw_text_centered,\n        menu_confirm_pressed, menu_down_pressed, menu_up_pressed,\n",
)
pong = replace_once(
    pong,
    "const FRAMEBUFFER_WIDTH: u32 = 320;\nconst FRAMEBUFFER_HEIGHT: u32 = 180;",
    "pub const FRAMEBUFFER_WIDTH: u32 = 320;\npub const FRAMEBUFFER_HEIGHT: u32 = 180;",
)
pong = replace_once(
    pong,
    "const P2_DOWN: ActionId = ActionId::new(\"pong.p2.down\");\n",
    "const P2_DOWN: ActionId = ActionId::new(\"pong.p2.down\");\nconst TOUCH_ACTION: ActionId = ActionId::new(\"pong.touch.action\");\n",
)
pong = replace_once(
    pong,
    "const BORDER: Pixel = Pixel::rgb(80, 150, 220);\n",
    """const BORDER: Pixel = Pixel::rgb(80, 150, 220);
const TOUCH_ACCENT: Pixel = Pixel::rgb(245, 190, 90);
const TOUCH_P1_UP: Rect = Rect { x: 4, y: 32, width: 56, height: 52 };
const TOUCH_P1_DOWN: Rect = Rect { x: 4, y: 112, width: 56, height: 52 };
const TOUCH_P2_UP: Rect = Rect { x: 260, y: 32, width: 56, height: 52 };
const TOUCH_P2_DOWN: Rect = Rect { x: 260, y: 112, width: 56, height: 52 };
const TOUCH_ACTION_RECT: Rect = Rect { x: 120, y: 154, width: 80, height: 22 };
""",
)
pong = replace_once(pong, "struct PongApp {", "pub struct PongApp {")
pong = replace_once(
    pong,
    "    p2_controls: ControlMap,\n    sounds: SoundBank,",
    "    p2_controls: ControlMap,\n    virtual_pad: Option<VirtualPad>,\n    touch_controls: ControlMap,\n    sounds: SoundBank,",
)
pong = replace_once(
    pong,
    "impl PongApp {\n    fn new() -> Self {\n        let mut sounds = SoundBank::new();",
    """impl PongApp {
    pub fn new() -> Self {
        Self::new_with_touch(false)
    }

    #[allow(dead_code)]
    pub fn new_touch() -> Self {
        Self::new_with_touch(true)
    }

    fn new_with_touch(touch: bool) -> Self {
        let mut sounds = SoundBank::new();""",
)
pong = replace_once(
    pong,
    "            p1_controls: ControlMap::new(),\n            p2_controls: ControlMap::new(),\n            sounds,",
    """            p1_controls: ControlMap::new(),
            p2_controls: ControlMap::new(),
            virtual_pad: touch.then(|| {
                VirtualPad::new([
                    VirtualButton::new(P1_UP, TOUCH_P1_UP),
                    VirtualButton::new(P1_DOWN, TOUCH_P1_DOWN),
                    VirtualButton::new(P2_UP, TOUCH_P2_UP),
                    VirtualButton::new(P2_DOWN, TOUCH_P2_DOWN),
                    VirtualButton::new(TOUCH_ACTION, TOUCH_ACTION_RECT),
                ])
            }),
            touch_controls: ControlMap::new(),
            sounds,""",
)
pong = replace_once(
    pong,
    "        app.rebuild_controls();\n        app\n",
    """        app.rebuild_controls();
        if touch {
            app.reset_match();
            app.playing = true;
        }
        app
""",
)
pong = replace_once(
    pong,
    "    fn update_game(&mut self, frame: &mut Frame<'_>) -> GameResult {\n        if frame.input.key(Key::Escape).pressed() {",
    """    fn update_game(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if let Some(virtual_pad) = &mut self.virtual_pad {
            virtual_pad.update(frame.input, &mut self.touch_controls);
        }
        self.touch_controls.update(frame.input);
        let touch_action_pressed = self.touch_controls.action(TOUCH_ACTION).pressed();

        if frame.input.key(Key::Escape).pressed() {""",
)
pong = replace_once(
    pong,
    "        if self.match_state == MatchState::MatchOver {\n            if menu_up_pressed(frame.input) {",
    """        if self.match_state == MatchState::MatchOver {
            if touch_action_pressed {
                self.reset_match();
                self.render_game(frame.framebuffer);
                return GameResult::Continue;
            }
            if menu_up_pressed(frame.input) {""",
)
pong = replace_once(
    pong,
    """        let dt = frame.delta_time.as_secs_f32().min(0.05);
        self.p1_y = move_paddle(
            self.p1_y,
            self.p1_controls.action(P1_UP).held(),
            self.p1_controls.action(P1_DOWN).held(),
            dt,
        );
        self.p2_y = move_paddle(
            self.p2_y,
            self.p2_controls.action(P2_UP).held(),
            self.p2_controls.action(P2_DOWN).held(),
            dt,
        );

        if self.match_state == MatchState::WaitingServe && menu_confirm_pressed(frame.input) {
""",
    """        let dt = frame.delta_time.as_secs_f32().min(0.05);
        let p1_up = self.p1_controls.action(P1_UP).held()
            || self.touch_controls.action(P1_UP).held();
        let p1_down = self.p1_controls.action(P1_DOWN).held()
            || self.touch_controls.action(P1_DOWN).held();
        let p2_up = self.p2_controls.action(P2_UP).held()
            || self.touch_controls.action(P2_UP).held();
        let p2_down = self.p2_controls.action(P2_DOWN).held()
            || self.touch_controls.action(P2_DOWN).held();
        self.p1_y = move_paddle(self.p1_y, p1_up, p1_down, dt);
        self.p2_y = move_paddle(self.p2_y, p2_up, p2_down, dt);

        if self.match_state == MatchState::WaitingServe
            && (menu_confirm_pressed(frame.input) || touch_action_pressed)
        {
""",
)
pong = replace_once(
    pong,
    """            MatchState::WaitingServe => {
                draw_text_centered(
                    framebuffer,
                    Rect {
                        x: 72,
                        y: 158,
                        width: 176,
                        height: 12,
                    },
                    "SPACE/SOUTH SERVE",
                    1,
                    FG,
                );
            }
            MatchState::Playing => {}
            MatchState::MatchOver => self.render_end_menu(framebuffer),
        }
    }
""",
    """            MatchState::WaitingServe => {
                if self.virtual_pad.is_none() {
                    draw_text_centered(
                        framebuffer,
                        Rect {
                            x: 72,
                            y: 158,
                            width: 176,
                            height: 12,
                        },
                        "SPACE/SOUTH SERVE",
                        1,
                        FG,
                    );
                }
            }
            MatchState::Playing => {}
            MatchState::MatchOver => self.render_end_menu(framebuffer),
        }

        if self.virtual_pad.is_some() {
            draw_touch_controls(framebuffer, self.match_state);
        }
    }
""",
)
pong = replace_once(
    pong,
    "}\n\nimpl Game for PongApp {",
    """}

fn draw_touch_controls(framebuffer: &mut Framebuffer, state: MatchState) {
    for (rect, label) in [
        (TOUCH_P1_UP, "P1 UP"),
        (TOUCH_P1_DOWN, "P1 DOWN"),
        (TOUCH_P2_UP, "P2 UP"),
        (TOUCH_P2_DOWN, "P2 DOWN"),
    ] {
        framebuffer.draw_rect(rect.x, rect.y, rect.width, rect.height, BORDER);
        draw_text_centered(framebuffer, rect, label, 1, TOUCH_ACCENT);
    }

    let action_label = match state {
        MatchState::WaitingServe => Some("SERVE"),
        MatchState::MatchOver => Some("REPLAY"),
        MatchState::Playing => None,
    };
    if let Some(label) = action_label {
        framebuffer.draw_rect(
            TOUCH_ACTION_RECT.x,
            TOUCH_ACTION_RECT.y,
            TOUCH_ACTION_RECT.width,
            TOUCH_ACTION_RECT.height,
            TOUCH_ACCENT,
        );
        draw_text_centered(framebuffer, TOUCH_ACTION_RECT, label, 1, TOUCH_ACCENT);
    }
}

impl Game for PongApp {""",
)
pong = replace_once(
    pong,
    """    #[test]
    fn sound_bank_owns_pong_feedback_assets() {
""",
    """    #[test]
    fn touch_mode_starts_match_with_two_player_virtual_controls() {
        let app = PongApp::new_touch();
        assert!(app.playing);
        let pad = app.virtual_pad.as_ref().expect("touch mode owns a virtual pad");
        let actions = pad
            .buttons()
            .iter()
            .map(|button| button.action)
            .collect::<Vec<_>>();
        assert_eq!(actions.len(), 5);
        for action in [P1_UP, P1_DOWN, P2_UP, P2_DOWN, TOUCH_ACTION] {
            assert!(actions.contains(&action));
        }
    }

    #[test]
    fn sound_bank_owns_pong_feedback_assets() {
""",
)

pong_path.write_text(pong)

Path("examples/pong_web.rs").write_text(
    '''#[allow(dead_code)]
#[path = "pong.rs"]
mod pong;

use gotoo_pixel_engine::{EngineConfig, run};
use pong::{FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH, PongApp};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    run(
        EngineConfig {
            title: "Pong".into(),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width: FRAMEBUFFER_WIDTH * 3,
            window_height: FRAMEBUFFER_HEIGHT * 3,
        },
        PongApp::new_touch(),
    )
    .map_err(|err| JsValue::from_str(&err.to_string()))
}
'''
)

cargo_path = Path("Cargo.toml")
cargo = cargo_path.read_text()
cargo = replace_once(
    cargo,
    '[[example]]\nname = "tetris_web"\npath = "examples/tetris_web.rs"\ncrate-type = ["cdylib"]\n',
    '[[example]]\nname = "tetris_web"\npath = "examples/tetris_web.rs"\ncrate-type = ["cdylib"]\n\n[[example]]\nname = "pong_web"\npath = "examples/pong_web.rs"\ncrate-type = ["cdylib"]\n',
)
cargo_path.write_text(cargo)

build_path = Path("scripts/build-web.sh")
build = build_path.read_text()
build = replace_once(
    build,
    "build_web_example tetris_web\n",
    "build_web_example tetris_web\nbuild_web_example pong_web\n",
)
build_path.write_text(build)

Path("web/pong.html").write_text(
    '''<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Pong</title>
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
        width: 960px;
        height: 540px;
        max-width: 100vw;
        max-height: 100vh;
        touch-action: none;
        image-rendering: pixelated;
      }
    </style>
  </head>
  <body>
    <script type="module">
      import init from "./pkg/pong_web.js";
      init();
    </script>
  </body>
</html>
'''
)
