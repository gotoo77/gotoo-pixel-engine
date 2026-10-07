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
const DIRECTIONS: [&str; 2] = ["LEFT TO RIGHT", "RIGHT TO LEFT"];
const VISUAL_STYLES: [&str; 3] = ["SOLID", "STRIPED", "GHOST"];

struct ToolWindowProbe {
    tool_open: bool,
    tool_mode: ToolWindowMode,
    escape_close_armed: bool,
    ui_state: UiState,
    selected_tab: usize,
    bar_enabled: bool,
    direction: usize,
    bar_width: f32,
    bar_gain: f32,
    visual_enabled: bool,
    visual_style: usize,
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
        if self.bar_enabled {
            let bar_width = self.bar_width.round() as u32;
            let bar_x = if self.direction == 0 {
                30
            } else {
                230 - bar_width as i32
            };
            let gain = self.bar_gain.clamp(0.0, 1.0);
            let green = (80.0 + gain * 175.0).round() as u8;
            let blue = (70.0 + gain * 150.0).round() as u8;

            frame
                .framebuffer
                .fill_rect(bar_x, 70, bar_width, 20, Pixel::rgb(70, green, blue));
        }
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

        // Visual controls in the sidecar intentionally affect this preview so
        // the probe verifies that tool-window edits propagate into the primary
        // game state immediately.
        frame
            .framebuffer
            .draw_rect(255, 70, 45, 45, Pixel::rgb(80, 95, 130));
        if self.visual_enabled {
            match self.visual_style {
                0 => {
                    frame
                        .framebuffer
                        .fill_rect(260, 75, 35, 35, Pixel::rgb(205, 95, 235));
                }
                1 => {
                    for x in (260..295).step_by(8) {
                        frame
                            .framebuffer
                            .fill_rect(x, 75, 4, 35, Pixel::rgb(205, 95, 235));
                    }
                }
                _ => {
                    frame
                        .framebuffer
                        .draw_rect(260, 75, 35, 35, Pixel::rgb(205, 95, 235));
                    frame
                        .framebuffer
                        .draw_rect(265, 80, 25, 25, Pixel::rgb(125, 75, 150));
                }
            }
        }

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
                    ui.section("INPUT");
                    ui.toggle("BAR ENABLED", &mut self.bar_enabled);
                    ui.select("DIRECTION", &mut self.direction, &DIRECTIONS);
                    ui.slider_f32("BAR WIDTH", &mut self.bar_width, 8.0..=200.0, 4.0);
                    ui.slider_f32("BAR GAIN", &mut self.bar_gain, 0.0..=1.0, 0.05);
                    ui.section("OVERFLOW");
                    ui.toggle("OVERLAY", &mut self.visual_enabled);
                    ui.select("STYLE", &mut self.visual_style, &VISUAL_STYLES);
                    ui.slider_f32("GAIN MIRROR", &mut self.bar_gain, 0.0..=1.0, 0.05);
                }
                1 => {
                    ui.section("VISUAL");
                    ui.toggle("OVERLAY", &mut self.visual_enabled);
                    ui.select("STYLE", &mut self.visual_style, &VISUAL_STYLES);
                    ui.section("DEBUG");
                    if ui.button("RESET VALUES").clicked {
                        self.bar_enabled = true;
                        self.direction = 0;
                        self.bar_width = 80.0;
                        self.bar_gain = 0.65;
                        self.visual_enabled = true;
                        self.visual_style = 0;
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
            direction: 0,
            bar_width: 80.0,
            bar_gain: 0.65,
            visual_enabled: true,
            visual_style: 0,
            heartbeat: 0.0,
        },
    )
}
