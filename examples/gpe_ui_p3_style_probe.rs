#![deny(warnings)]

use std::time::Duration;

use gotoo_pixel_engine::{
    ActionId, EngineConfig, Frame, Framebuffer, Game, GameResult, GamepadButton, Key, MouseButton,
    Pixel, Rect, Size, TextRenderer, run,
    ui::{
        UiComponentStyle, UiStyleOverride, UiStyleSheet, UiTheme,
        experimental::{self, UiInput, UiNavInput, UiPointerInput, UiStateStore},
        experimental_spatial::{
            GridSpec, SpatialCard, SpatialInput, SpatialState, run_default_card_grid_styled,
        },
    },
};

const WIDTH: u32 = 640;
const HEIGHT: u32 = 360;
const TRANSACTIONAL_X: i32 = 20;
const TRANSACTIONAL_Y: i32 = 58;
const TRANSACTIONAL_WIDTH: u32 = 292;
const TRANSACTIONAL_HEIGHT: u32 = 214;
const SPATIAL_BOUNDS: Rect = Rect {
    x: 348,
    y: 76,
    width: 260,
    height: 170,
};

const RESET: ActionId = ActionId::new("p3.probe.reset");
const CARD_A: ActionId = ActionId::new("p3.probe.card.a");
const CARD_B: ActionId = ActionId::new("p3.probe.card.b");
const CARD_C: ActionId = ActionId::new("p3.probe.card.c");
const CARD_ACTIONS: [ActionId; 3] = [CARD_A, CARD_B, CARD_C];
const CARD_TITLES: [&str; 3] = ["ALPHA", "BRAVO", "CHARLIE"];

#[derive(Debug, Clone, Copy)]
struct Settings {
    enabled: bool,
    volume: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: true,
            volume: 0.55,
        }
    }
}

struct P3StyleProbe {
    ui_state: UiStateStore,
    spatial_state: SpatialState,
    settings: Settings,
    card_ids: Vec<experimental::UiId>,
    last_action: &'static str,
    left_repeat: RepeatPulse,
    right_repeat: RepeatPulse,
}

impl P3StyleProbe {
    fn new() -> Self {
        Self {
            ui_state: UiStateStore::default(),
            spatial_state: SpatialState::default(),
            settings: Settings::default(),
            card_ids: stable_card_ids(),
            last_action: "NONE",
            left_repeat: RepeatPulse::default(),
            right_repeat: RepeatPulse::default(),
        }
    }

    fn nav(frame: &Frame<'_>) -> UiNavInput {
        UiNavInput {
            up: frame.input.key(Key::Up).pressed()
                || frame.input.key(Key::W).pressed()
                || frame
                    .input
                    .gamepad_button_any(GamepadButton::DPadUp)
                    .pressed(),
            down: frame.input.key(Key::Down).pressed()
                || frame.input.key(Key::S).pressed()
                || frame
                    .input
                    .gamepad_button_any(GamepadButton::DPadDown)
                    .pressed(),
            left: false,
            right: false,
            confirm: frame.input.key(Key::Space).pressed()
                || frame
                    .input
                    .gamepad_button_any(GamepadButton::South)
                    .pressed(),
            cancel: frame.input.key(Key::Escape).pressed()
                || frame
                    .input
                    .gamepad_button_any(GamepadButton::East)
                    .pressed(),
        }
    }

    fn pointer(frame: &Frame<'_>) -> UiPointerInput {
        UiPointerInput {
            position: frame.input.mouse_position(),
            pressed: frame.input.mouse_button(MouseButton::Left).pressed(),
            released: frame.input.mouse_button(MouseButton::Left).released(),
        }
    }
}

impl Game for P3StyleProbe {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let mut nav = Self::nav(frame);
        nav.left = self.left_repeat.update(
            horizontal_left_pressed(frame),
            horizontal_left_held(frame),
            frame.delta_time,
        );
        nav.right = self.right_repeat.update(
            horizontal_right_pressed(frame),
            horizontal_right_held(frame),
            frame.delta_time,
        );
        if nav.cancel {
            return GameResult::Exit;
        }

        frame.framebuffer.clear(Pixel::rgb(8, 10, 16));

        let theme = UiTheme {
            padding: 10,
            row_spacing: 6,
            row_height: 28,
            text: Pixel::rgb(224, 232, 238),
            muted_text: Pixel::rgb(145, 156, 168),
            control_background: Pixel::rgb(18, 23, 31),
            border: Pixel::rgb(68, 80, 92),
            accent: Pixel::rgb(238, 176, 72),
            ..UiTheme::default()
        };
        let stylesheet = p3_stylesheet();
        let pointer = Self::pointer(frame);
        let mut transactional_framebuffer =
            Framebuffer::new(TRANSACTIONAL_WIDTH, TRANSACTIONAL_HEIGHT);
        transactional_framebuffer.clear(Pixel::rgb(13, 16, 23));

        let snapshot = self.settings;
        let (output, controls) = experimental::run_with_input_styled(
            &mut transactional_framebuffer,
            &mut self.ui_state,
            UiInput {
                nav,
                pointer: translate_pointer(pointer, -TRANSACTIONAL_X, -TRANSACTIONAL_Y),
                touches: &[],
            },
            theme,
            stylesheet,
            |ui| {
                ui.panel_styled(
                    UiStyleOverride {
                        background: Some(Pixel::rgb(22, 27, 36)),
                        border: Some(Pixel::rgb(91, 106, 118)),
                        padding: Some(14),
                        vertical_gap: Some(8),
                        ..UiStyleOverride::default()
                    },
                    |ui| {
                        ui.text_styled(
                            "GPE.UI P3 STYLE PROBE",
                            UiStyleOverride {
                                text: Some(Pixel::rgb(140, 218, 196)),
                                ..UiStyleOverride::default()
                            },
                        );
                        let enabled =
                            ui.keyed("enabled", |ui| ui.toggle("ENABLED", snapshot.enabled));
                        let volume = ui.keyed("volume", |ui| {
                            ui.slider_f32("VOLUME", snapshot.volume, 0.0..=1.0, 0.05)
                        });
                        ui.keyed("reset", |ui| {
                            ui.button_action_styled(
                                "RESET LOCAL OVERRIDE",
                                RESET,
                                UiStyleOverride {
                                    background: Some(Pixel::rgb(70, 32, 48)),
                                    border: Some(Pixel::rgb(205, 92, 104)),
                                    text: Some(Pixel::rgb(255, 220, 220)),
                                    ..UiStyleOverride::default()
                                },
                            )
                        });
                        (enabled, volume)
                    },
                )
            },
        );

        if let Some(value) = output.changed(controls.0) {
            self.settings.enabled = value;
        }
        if let Some(value) = output.changed(controls.1) {
            self.settings.volume = value;
        }
        if output.action_pressed(RESET) {
            self.settings = Settings::default();
            self.last_action = "RESET";
        }
        blit_framebuffer(
            frame.framebuffer,
            &transactional_framebuffer,
            TRANSACTIONAL_X,
            TRANSACTIONAL_Y,
        );

        let cards = self
            .card_ids
            .iter()
            .enumerate()
            .map(|(index, id)| SpatialCard {
                id: *id,
                title: CARD_TITLES[index],
                subtitle: "SPATIAL CARD",
                image: None,
                action: CARD_ACTIONS[index],
            })
            .collect::<Vec<_>>();

        let spatial = run_default_card_grid_styled(
            frame.framebuffer,
            SPATIAL_BOUNDS,
            &mut self.spatial_state,
            SpatialInput {
                nav,
                pointer,
                touches: frame.input.touches(),
            },
            GridSpec {
                min_cell_width: 118,
                preferred_cell_height: 72,
                gap: 10,
                padding: 8,
            },
            theme,
            stylesheet,
            &cards,
        );

        for (index, action) in CARD_ACTIONS.iter().copied().enumerate() {
            if spatial.action_pressed(action) {
                self.last_action = CARD_TITLES[index];
            }
        }

        let text = TextRenderer::default();
        text.draw(
            frame.framebuffer,
            TRANSACTIONAL_X,
            26,
            "TRANSACTIONAL",
            Pixel::rgb(140, 218, 196),
        );
        text.draw(
            frame.framebuffer,
            SPATIAL_BOUNDS.x,
            26,
            "SPATIAL CARDS",
            Pixel::rgb(140, 218, 196),
        );
        text.draw(
            frame.framebuffer,
            16,
            318,
            "ARROWS/WASD: FOCUS   HOLD LEFT/RIGHT: REPEAT SLIDER   MOUSE: HOVER/PRESS   ESC: EXIT",
            Pixel::rgb(160, 172, 186),
        );
        text.draw(
            frame.framebuffer,
            SPATIAL_BOUNDS.x,
            264,
            &format!(
                "CARD FOCUS={:?} HOVER={:?}",
                spatial.focused_id().map(experimental::UiId::raw),
                spatial.hovered_id().map(experimental::UiId::raw)
            ),
            Pixel::rgb(160, 172, 186),
        );
        text.draw(
            frame.framebuffer,
            SPATIAL_BOUNDS.x,
            282,
            &format!("LAST ACTION={}", self.last_action),
            Pixel::rgb(238, 176, 72),
        );

        GameResult::Continue
    }
}

fn translate_pointer(pointer: UiPointerInput, dx: i32, dy: i32) -> UiPointerInput {
    UiPointerInput {
        position: pointer
            .position
            .map(|(x, y)| (x.saturating_add(dx), y.saturating_add(dy))),
        pressed: pointer.pressed,
        released: pointer.released,
    }
}

fn blit_framebuffer(target: &mut Framebuffer, source: &Framebuffer, x: i32, y: i32) {
    for row in 0..source.height() {
        for column in 0..source.width() {
            let index = ((row * source.width() + column) * 4) as usize;
            let rgba = &source.as_rgba8()[index..index + 4];
            target.draw(
                x.saturating_add(column as i32),
                y.saturating_add(row as i32),
                Pixel::rgba(rgba[0], rgba[1], rgba[2], rgba[3]),
            );
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct RepeatPulse {
    held_last_frame: bool,
    elapsed: Duration,
    next_repeat: Duration,
}

impl Default for RepeatPulse {
    fn default() -> Self {
        Self {
            held_last_frame: false,
            elapsed: Duration::ZERO,
            next_repeat: INITIAL_REPEAT_DELAY,
        }
    }
}

impl RepeatPulse {
    fn update(&mut self, pressed: bool, held: bool, delta_time: Duration) -> bool {
        if pressed || (held && !self.held_last_frame) {
            self.held_last_frame = held;
            self.elapsed = Duration::ZERO;
            self.next_repeat = INITIAL_REPEAT_DELAY;
            return true;
        }

        if !held {
            *self = Self::default();
            return false;
        }

        self.held_last_frame = true;
        self.elapsed = self.elapsed.saturating_add(delta_time);
        if self.elapsed < self.next_repeat {
            return false;
        }

        self.next_repeat = self.next_repeat.saturating_add(REPEAT_INTERVAL);
        true
    }
}

const INITIAL_REPEAT_DELAY: Duration = Duration::from_millis(260);
const REPEAT_INTERVAL: Duration = Duration::from_millis(55);

fn horizontal_left_pressed(frame: &Frame<'_>) -> bool {
    frame.input.key(Key::Left).pressed()
        || frame.input.key(Key::A).pressed()
        || frame
            .input
            .gamepad_button_any(GamepadButton::DPadLeft)
            .pressed()
}

fn horizontal_left_held(frame: &Frame<'_>) -> bool {
    frame.input.key(Key::Left).held()
        || frame.input.key(Key::A).held()
        || frame
            .input
            .gamepad_button_any(GamepadButton::DPadLeft)
            .held()
}

fn horizontal_right_pressed(frame: &Frame<'_>) -> bool {
    frame.input.key(Key::Right).pressed()
        || frame.input.key(Key::D).pressed()
        || frame
            .input
            .gamepad_button_any(GamepadButton::DPadRight)
            .pressed()
}

fn horizontal_right_held(frame: &Frame<'_>) -> bool {
    frame.input.key(Key::Right).held()
        || frame.input.key(Key::D).held()
        || frame
            .input
            .gamepad_button_any(GamepadButton::DPadRight)
            .held()
}

fn p3_stylesheet() -> UiStyleSheet {
    UiStyleSheet {
        panel: UiComponentStyle {
            base: UiStyleOverride {
                background: Some(Pixel::rgb(18, 23, 31)),
                border: Some(Pixel::rgb(84, 96, 110)),
                border_width: Some(2),
                padding: Some(12),
                vertical_gap: Some(7),
                ..UiStyleOverride::default()
            },
            ..UiComponentStyle::default()
        },
        text: UiComponentStyle {
            base: UiStyleOverride {
                text: Some(Pixel::rgb(218, 228, 235)),
                ..UiStyleOverride::default()
            },
            ..UiComponentStyle::default()
        },
        button: UiComponentStyle {
            base: UiStyleOverride {
                background: Some(Pixel::rgb(29, 36, 45)),
                border: Some(Pixel::rgb(82, 98, 116)),
                text: Some(Pixel::rgb(226, 232, 238)),
                muted_text: Some(Pixel::rgb(152, 164, 176)),
                accent: Some(Pixel::rgb(238, 176, 72)),
                border_width: Some(2),
                ..UiStyleOverride::default()
            },
            focused: UiStyleOverride {
                border: Some(Pixel::rgb(238, 176, 72)),
                background: Some(Pixel::rgb(38, 45, 54)),
                ..UiStyleOverride::default()
            },
            hovered: UiStyleOverride {
                border: Some(Pixel::rgb(122, 211, 190)),
                background: Some(Pixel::rgb(34, 54, 54)),
                ..UiStyleOverride::default()
            },
            active: UiStyleOverride {
                border: Some(Pixel::rgb(255, 224, 128)),
                background: Some(Pixel::rgb(74, 62, 34)),
                accent: Some(Pixel::rgb(255, 224, 128)),
                ..UiStyleOverride::default()
            },
        },
        toggle_bool: UiComponentStyle {
            base: UiStyleOverride {
                background: Some(Pixel::rgb(24, 42, 38)),
                border: Some(Pixel::rgb(82, 126, 112)),
                accent: Some(Pixel::rgb(122, 211, 190)),
                border_width: Some(2),
                ..UiStyleOverride::default()
            },
            focused: UiStyleOverride {
                border: Some(Pixel::rgb(238, 176, 72)),
                ..UiStyleOverride::default()
            },
            hovered: UiStyleOverride {
                background: Some(Pixel::rgb(30, 58, 52)),
                ..UiStyleOverride::default()
            },
            active: UiStyleOverride {
                background: Some(Pixel::rgb(62, 74, 42)),
                ..UiStyleOverride::default()
            },
        },
        slider_f32: UiComponentStyle {
            base: UiStyleOverride {
                background: Some(Pixel::rgb(28, 34, 52)),
                border: Some(Pixel::rgb(92, 106, 148)),
                accent: Some(Pixel::rgb(118, 152, 232)),
                border_width: Some(2),
                ..UiStyleOverride::default()
            },
            focused: UiStyleOverride {
                border: Some(Pixel::rgb(238, 176, 72)),
                ..UiStyleOverride::default()
            },
            hovered: UiStyleOverride {
                background: Some(Pixel::rgb(34, 44, 68)),
                ..UiStyleOverride::default()
            },
            active: UiStyleOverride {
                accent: Some(Pixel::rgb(255, 224, 128)),
                ..UiStyleOverride::default()
            },
        },
    }
}

fn stable_card_ids() -> Vec<experimental::UiId> {
    let mut state = UiStateStore::default();
    let (_, ids) = experimental::run_headless(
        Size {
            width: WIDTH,
            height: HEIGHT,
        },
        &mut state,
        UiNavInput::default(),
        UiTheme::default(),
        |ui| {
            CARD_TITLES
                .iter()
                .map(|title| ui.keyed(title, |ui| ui.button(*title).id()))
                .collect::<Vec<_>>()
        },
    );
    ids
}

fn main() -> Result<(), gotoo_pixel_engine::EngineError> {
    run(
        EngineConfig {
            title: "GPE.UI P3 Style Probe".into(),
            framebuffer_width: WIDTH,
            framebuffer_height: HEIGHT,
            window_width: WIDTH * 2,
            window_height: HEIGHT * 2,
        },
        P3StyleProbe::new(),
    )
}
