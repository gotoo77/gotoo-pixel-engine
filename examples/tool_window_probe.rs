#![deny(warnings)]

use std::time::Duration;

use gotoo_pixel_engine::{
    EngineConfig, Frame, Game, GameResult, Input, Key, Pixel, ToolFrame, ToolWindowConfig,
    ToolWindowMode, run,
};

const MAIN_WIDTH: u32 = 320;
const MAIN_HEIGHT: u32 = 180;
const TOOL_WIDTH: u32 = 240;
const TOOL_HEIGHT: u32 = 180;
const REPEAT_DELAY: Duration = Duration::from_millis(300);
const REPEAT_INTERVAL: Duration = Duration::from_millis(60);

#[derive(Default)]
struct HeldRepeat {
    elapsed: Duration,
    next_repeat: Duration,
}

impl HeldRepeat {
    fn update(&mut self, state: gotoo_pixel_engine::ButtonState, delta_time: Duration) -> bool {
        if state.pressed() {
            self.elapsed = Duration::ZERO;
            self.next_repeat = REPEAT_DELAY;
            return true;
        }
        if !state.held() {
            self.elapsed = Duration::ZERO;
            self.next_repeat = REPEAT_DELAY;
            return false;
        }

        self.elapsed += delta_time;
        if self.elapsed < self.next_repeat {
            return false;
        }

        self.next_repeat += REPEAT_INTERVAL;
        true
    }
}

struct ToolWindowProbe {
    tool_open: bool,
    tool_mode: ToolWindowMode,
    escape_close_armed: bool,
    left_repeat: HeldRepeat,
    right_repeat: HeldRepeat,
    bar_width: u32,
    heartbeat: f32,
}

impl ToolWindowProbe {
    fn adjust(&mut self, direction: i32) {
        if direction < 0 {
            self.bar_width = self.bar_width.saturating_sub(4).max(8);
        } else {
            self.bar_width = (self.bar_width + 4).min(200);
        }
    }

    fn open_tool(&mut self, mode: ToolWindowMode) {
        self.tool_mode = mode;
        self.tool_open = true;
        self.escape_close_armed = false;
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
        frame.framebuffer.draw_text_scaled(
            20,
            10,
            "CTRL SHIFT L: LIVE",
            1,
            Pixel::rgb(170, 205, 235),
        );
        frame.framebuffer.draw_text_scaled(
            20,
            22,
            "CTRL SHIFT M: MODAL",
            1,
            Pixel::rgb(220, 170, 235),
        );
        frame.framebuffer.draw_text_scaled(
            20,
            34,
            "CTRL SHIFT H: HYBRID",
            1,
            Pixel::rgb(200, 195, 235),
        );
        frame
            .framebuffer
            .draw_rect(20, 60, 220, 40, Pixel::rgb(90, 120, 180));
        frame
            .framebuffer
            .fill_rect(30, 70, self.bar_width, 20, Pixel::rgb(100, 220, 180));
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
            window_height: 360,
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

        if self
            .left_repeat
            .update(frame.input.key(Key::Left), frame.delta_time)
        {
            self.adjust(-1);
        }
        if self
            .right_repeat
            .update(frame.input.key(Key::Right), frame.delta_time)
        {
            self.adjust(1);
        }

        frame.framebuffer.clear(Pixel::rgb(12, 7, 18));
        let mode_label = match self.tool_mode {
            ToolWindowMode::Modal => "MODAL: MAIN BLOCKED",
            ToolWindowMode::Modeless => "LIVE: MAIN ALWAYS RUNS",
            ToolWindowMode::ModalWhenFocused => "HYBRID: PAUSE ON TOOL FOCUS",
        };
        frame
            .framebuffer
            .draw_text_scaled(12, 18, mode_label, 1, Pixel::rgb(230, 205, 245));
        frame.framebuffer.draw_text_scaled(
            12,
            34,
            "HOLD LEFT / RIGHT TO REPEAT",
            1,
            Pixel::rgb(185, 180, 205),
        );
        frame
            .framebuffer
            .draw_rect(12, 58, 216, 80, Pixel::rgb(170, 90, 230));
        frame
            .framebuffer
            .fill_rect(20, 88, self.bar_width, 20, Pixel::rgb(100, 220, 180));
    }

    fn tool_window_closed(&mut self) {
        self.tool_open = false;
        self.escape_close_armed = false;
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
            left_repeat: HeldRepeat::default(),
            right_repeat: HeldRepeat::default(),
            bar_width: 80,
            heartbeat: 0.0,
        },
    )
}
