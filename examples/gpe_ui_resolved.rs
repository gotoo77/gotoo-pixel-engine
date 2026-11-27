//! Free-rectangle controls: arrows/pad navigate, Enter/South confirm,
//! Escape/East cancel, R/Start explicitly suspend, D/West toggle availability.
//! Focus is a border, hover a fill, capture a stronger fill. Confirmation uses
//! a short flash; keyboard/pad have no invented held/active visual state.
use std::time::Duration;

use gotoo_pixel_engine::{
    ActionId, ControlMap, EngineConfig, EngineError, Frame, Game, GameResult, GamepadButton, Key,
    MouseButton, Pixel, Rect, run,
    ui::{
        resolved::{
            self, ResolvedState, ResolvedTarget, UiId, UiInput, UiNavInput, UiPointerInput,
        },
        standard_menu_controls,
    },
};

const UP: ActionId = ActionId::new("up");
const DOWN: ActionId = ActionId::new("down");
const LEFT: ActionId = ActionId::new("left");
const RIGHT: ActionId = ActionId::new("right");
const CONFIRM: ActionId = ActionId::new("confirm");
const CANCEL: ActionId = ActionId::new("cancel");
const SUSPEND: ActionId = ActionId::new("suspend");
const TOGGLE: ActionId = ActionId::new("toggle");
const WIDTH: u32 = 480;
const HEIGHT: u32 = 300;

struct Probe {
    state: ResolvedState,
    controls: ControlMap,
    targets: [ResolvedTarget; 5],
    flash: Option<(UiId, Duration)>,
    #[cfg(feature = "outline-fonts")]
    font: gotoo_pixel_engine::outline_text::OutlineFont,
}

impl Probe {
    fn new() -> Self {
        let mut controls = standard_menu_controls(UP, DOWN, CONFIRM);
        for (action, key, button) in [
            (LEFT, Key::Left, GamepadButton::DPadLeft),
            (RIGHT, Key::Right, GamepadButton::DPadRight),
            (CANCEL, Key::Escape, GamepadButton::East),
            (SUSPEND, Key::R, GamepadButton::Start),
            (TOGGLE, Key::D, GamepadButton::West),
        ] {
            controls.bind_key(action, key).bind_gamepad(action, button);
        }
        let rects = [
            Rect {
                x: 165,
                y: 110,
                width: 100,
                height: 45,
            },
            Rect {
                x: 180,
                y: 45,
                width: 75,
                height: 30,
            },
            Rect {
                x: 330,
                y: 100,
                width: 125,
                height: 55,
            },
            Rect {
                x: 150,
                y: 195,
                width: 130,
                height: 60,
            },
            Rect {
                x: 25,
                y: 115,
                width: 80,
                height: 35,
            },
        ];
        Self {
            state: ResolvedState::default(),
            controls,
            targets: std::array::from_fn(|index| ResolvedTarget {
                id: UiId::from_stable(index as u64 + 1).unwrap(),
                rect: rects[index],
                action: ActionId::new(["center", "up", "right", "down", "left"][index]),
                enabled: index != 3,
            }),
            flash: None,
            #[cfg(feature = "outline-fonts")]
            font: gotoo_pixel_engine::outline_text::OutlineFont::from_bytes(include_bytes!(
                "../assets/fonts/p4/figtree/font.ttf"
            ))
            .expect("bundled font"),
        }
    }
}

impl Game for Probe {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        self.controls.update(frame.input);
        if self.controls.action(SUSPEND).pressed() {
            self.state.suspend();
            self.flash = None;
        }
        if self.controls.action(TOGGLE).pressed() {
            self.targets[3].enabled = !self.targets[3].enabled;
        }
        let output = resolved::update(
            &mut self.state,
            UiInput {
                nav: UiNavInput {
                    up: self.controls.action(UP).pressed(),
                    down: self.controls.action(DOWN).pressed(),
                    left: self.controls.action(LEFT).pressed(),
                    right: self.controls.action(RIGHT).pressed(),
                    confirm: self.controls.action(CONFIRM).pressed(),
                    cancel: self.controls.action(CANCEL).pressed(),
                },
                pointer: UiPointerInput {
                    position: frame.input.mouse_position(),
                    pressed: frame.input.mouse_button(MouseButton::Left).pressed(),
                    released: frame.input.mouse_button(MouseButton::Left).released(),
                },
                touches: frame.input.touches(),
            },
            &self.targets,
        )
        .expect("fixed disjoint rectangles and stable identities");
        if let Some((_, remaining)) = &mut self.flash {
            *remaining = remaining.saturating_sub(frame.delta_time);
        }
        if let Some(activation) = output.activation {
            self.flash = Some((activation.id, Duration::from_millis(180)));
        }
        if output.cancelled {
            self.flash = None;
        }
        let fb = &mut *frame.framebuffer;
        fb.clear(Pixel::rgb(12, 16, 22));
        fb.draw_text(12, 12, "RESOLVED CONTROLS / FREE GEOMETRY", Pixel::WHITE);
        fb.draw_text(12, 272, "R/START RESET  D/WEST AVAILABILITY", Pixel::WHITE);
        for target in &self.targets {
            let fill = if !target.enabled {
                Pixel::rgb(42, 42, 42)
            } else if output.active(target.id) {
                Pixel::rgb(50, 145, 120)
            } else if output.hovered == Some(target.id) {
                Pixel::rgb(45, 75, 110)
            } else {
                Pixel::rgb(30, 43, 60)
            };
            let rect = target.rect; // Exactly the rectangle passed to interaction.
            fb.fill_rect(rect.x, rect.y, rect.width, rect.height, fill);
            let flash = self
                .flash
                .is_some_and(|(id, time)| id == target.id && !time.is_zero());
            let border = if flash {
                Pixel::rgb(255, 180, 70)
            } else if output.focused == Some(target.id) {
                Pixel::rgb(120, 235, 255)
            } else {
                Pixel::rgb(80, 85, 95)
            };
            fb.draw_rect(rect.x, rect.y, rect.width, rect.height, border);
            #[cfg(feature = "outline-fonts")]
            self.font
                .draw(fb, target.action.as_str(), 17.0, rect, Pixel::WHITE);
            #[cfg(not(feature = "outline-fonts"))]
            fb.draw_text(
                rect.x + 5,
                rect.y + 10,
                target.action.as_str(),
                Pixel::WHITE,
            );
        }
        GameResult::Continue
    }
}

pub fn launch() -> Result<(), EngineError> {
    run(
        EngineConfig {
            title: "GPE #117 resolved controls".into(),
            framebuffer_width: WIDTH,
            framebuffer_height: HEIGHT,
            window_width: WIDTH * 2,
            window_height: HEIGHT * 2,
        },
        Probe::new(),
    )
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), EngineError> {
    launch()
}

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() -> Result<(), wasm_bindgen::JsValue> {
    launch().map_err(|error| wasm_bindgen::JsValue::from_str(&error.to_string()))
}
