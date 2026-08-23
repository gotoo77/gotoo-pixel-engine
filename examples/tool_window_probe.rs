#![deny(warnings)]

use gotoo_pixel_engine::{
    EngineConfig, Frame, Game, GameResult, Input, Key, Pixel, TextRenderer, ToolFrame,
    ToolWindowConfig, ToolWindowMode, run,
    ui::{Ui, UiState, UiTheme},
};

const MAIN_WIDTH: u32 = 320;
const MAIN_HEIGHT: u32 = 180;
const TOOL_WIDTH: u32 = 240;
const TOOL_HEIGHT: u32 = 220;
const TOOL_TABS: [&str; 2] = ["CONTROLS", "OTHER"];

struct ToolWindowProbe {
    tool_open: bool,
    tool_mode: ToolWindowMode,
    escape_close_armed: bool,
    ui_state: UiState,
    selected_tab: usize,
    bar_enabled: bool,
    bar_width: f32,
    bar_gain: f32,
    heartbeat: f32,
}

impl ToolWindowProbe {
    fn open_tool(&mut self, mode: ToolWindowMode) {
        self.tool_mode = mode;
        self.tool_open = true;
        self.escape_close_armed = false;
        self.ui_state = UiState::default();
    }
}

fn control_shift_pressed(input: &Input, key: Key) -> bool {
    let control = input.key(Key::LeftControl).held() || input.key(Key::RightControl).held();
    let shift = input.key(Key::LeftShift).held() || input.key(Key::RightShift).held();
    control && shift && input.key(key).pressed()
}

impl Game for ToolWindowProbe {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if frame.input.key(Key::Escape).pressed() {
            return GameResult::Exit;
        }

        if !self.tool_open && control_shift_pressed(frame.input, Key::L) {
            self.open_tool(ToolWindowMode::Modeless);
        } else if !self.tool_open && control_shift_pressed(frame.input, Key::M) {
            self.open_tool(ToolWindowMode::Modal);
        } else if !self.tool_open && control_shift_pressed(frame.input, Key::H) {
            self.open_tool(ToolWindowMode::ModalWhenFocused);
        }

        // This moving marker is intentionally driven only by the primary game
        // frame. Live keeps it moving, Modal always stops it, and Hybrid stops
        // it only while the tool owns focus.
        self.heartbeat = (self.heartbeat + frame.delta_time.as_secs_f32() * 90.0) % 200.0;

        frame.framebuffer.clear(Pixel::rgb(5, 8, 14));
        let text = TextRenderer::default();
        text.draw(
            frame.framebuffer,
            20,
            10,
            "CTRL SHIFT L: LIVE",
            Pixel::rgb(170, 205, 235),
        );
        text.draw(
            frame.framebuffer,
            20,
            22,
            "CTRL SHIFT M: MODAL",
            Pixel::rgb(220, 170, 235),
        );
        text.draw(
            frame.framebuffer,
            20,
            34,
            "CTRL SHIFT H: HYBRID",
            Pixel::rgb(200, 195, 235),
        );
        frame
            .framebuffer
            .draw_rect(20, 60, 220, 40, Pixel::rgb(90, 120, 180));
        frame.framebuffer.fill_rect(
            30,
            70,
            self.bar_width.round() as u32,
            20,
            Pixel::rgb(100, 220, 180),
        );
        frame
            .framebuffer
            .draw_rect(30, 125, 208, 12, Pixel::rgb(70, 90, 130));
        frame.framebuffer.fill_rect(
            34 + self.heartbeat as i32,
            127,
            6,
            8,
            Pixel::rgb(240, 210, 90),
        );

        if self.tool_open {
            frame
                .framebuffer
                .draw_rect(270, 10, 30, 30, Pixel::rgb(220, 120, 255));
        }

        GameResult::Continue
    }

    fn tool_window_config(&self) -> Option<ToolWindowConfig> {
        self.tool_open.then(|| ToolWindowConfig {
            title: "GPE Auxiliary Tool Window Probe".into(),
            framebuffer_width: TOOL_WIDTH,
            framebuffer_height: TOOL_HEIGHT,
            window_width: 480,
            window_height: 440,
            mode: self.tool_mode,
        })
    }

    fn update_tool_window(&mut self, frame: &mut ToolFrame<'_>) {
        // Close on Escape release rather than press. On desktop the main window
        // regains focus immediately after the tool closes; waiting for release
        // prevents that same physical Escape from leaking into the parent.
        if frame.input.key(Key::Escape).pressed() {
            self.escape_close_armed = true;
        }
        if self.escape_close_armed && frame.input.key(Key::Escape).released() {
            self.escape_close_armed = false;
            self.tool_open = false;
            return;
        }

        frame.framebuffer.clear(Pixel::rgb(12, 7, 18));
        let mode_label = match self.tool_mode {
            ToolWindowMode::Modal => "MODAL: MAIN BLOCKED",
            ToolWindowMode::Modeless => "LIVE: MAIN ALWAYS RUNS",
            ToolWindowMode::ModalWhenFocused => "HYBRID: PAUSE ON TOOL FOCUS",
        };

        let requested_tab = {
            let mut ui = Ui::new(
                frame.framebuffer,
                frame.input,
                frame.delta_time,
                &mut self.ui_state,
                UiTheme::default(),
            );
            ui.label(mode_label);
            let requested_tab = ui.tabs(self.selected_tab, &TOOL_TABS);

            match self.selected_tab {
                0 => {
                    ui.toggle("BAR ENABLED", &mut self.bar_enabled);
                    ui.slider_f32("BAR WIDTH", &mut self.bar_width, 8.0..=200.0, 4.0);
                    ui.slider_f32("BAR GAIN", &mut self.bar_gain, 0.0..=1.0, 0.05);
                }
                1 => {
                    if ui.button("RESET VALUES").clicked {
                        self.bar_enabled = true;
                        self.bar_width = 80.0;
                        self.bar_gain = 0.65;
                    }
                }
                _ => ui.label("NORMALIZING TAB"),
            }

            requested_tab
        };

        if let Some(next_tab) = requested_tab {
            self.ui_state.reset_interaction();
            self.selected_tab = next_tab;
        }

        frame
            .framebuffer
            .draw_rect(12, 164, 216, 32, Pixel::rgb(90, 75, 120));
        if self.bar_enabled {
            let height = (6.0 + self.bar_gain * 18.0).round() as u32;
            let y = 180_i32.saturating_sub((height / 2) as i32);
            frame.framebuffer.fill_rect(
                20,
                y,
                self.bar_width.round() as u32,
                height,
                Pixel::rgb(100, 220, 180),
            );
        }
    }

    fn tool_window_closed(&mut self) {
        self.tool_open = false;
        self.escape_close_armed = false;
        self.ui_state = UiState::default();
    }
}

fn main() -> Result<(), gotoo_pixel_engine::EngineError> {
    run(
        EngineConfig {
            title: "GPE Tool Window Probe - L live / M modal / H hybrid".into(),
            framebuffer_width: MAIN_WIDTH,
            framebuffer_height: MAIN_HEIGHT,
            window_width: 960,
            window_height: 540,
        },
        ToolWindowProbe {
            tool_open: false,
            tool_mode: ToolWindowMode::Modeless,
            escape_close_armed: false,
            ui_state: UiState::default(),
            selected_tab: 0,
            bar_enabled: true,
            bar_width: 80.0,
            bar_gain: 0.65,
            heartbeat: 0.0,
        },
    )
}
