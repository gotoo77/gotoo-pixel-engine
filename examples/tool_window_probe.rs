#![deny(warnings)]

use gotoo_pixel_engine::{
    EngineConfig, Frame, Game, GameResult, Input, Key, Pixel, ToolFrame, ToolWindowConfig, run,
};

const MAIN_WIDTH: u32 = 320;
const MAIN_HEIGHT: u32 = 180;
const TOOL_WIDTH: u32 = 240;
const TOOL_HEIGHT: u32 = 180;

struct ToolWindowProbe {
    tool_open: bool,
    escape_close_armed: bool,
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
        if control_shift_pressed(frame.input, Key::F) {
            self.tool_open = !self.tool_open;
            self.escape_close_armed = false;
        }

        // This moving marker is intentionally driven only by the primary game
        // frame. It makes it obvious whether the main simulation keeps ticking
        // while the auxiliary window owns keyboard focus.
        self.heartbeat = (self.heartbeat + frame.delta_time.as_secs_f32() * 90.0) % 200.0;

        frame.framebuffer.clear(Pixel::rgb(5, 8, 14));
        frame
            .framebuffer
            .draw_rect(20, 60, 220, 40, Pixel::rgb(90, 120, 180));
        frame
            .framebuffer
            .fill_rect(30, 70, self.bar_width, 20, Pixel::rgb(100, 220, 180));
        frame.framebuffer.draw_rect(
            30,
            125,
            208,
            12,
            Pixel::rgb(70, 90, 130),
        );
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
        })
    }

    fn update_tool_window(&mut self, frame: &mut ToolFrame<'_>) {
        // Close on Escape release rather than press. On desktop the main window
        // regains focus immediately after the tool closes; waiting for release
        // prevents that same physical Escape from leaking into the parent and
        // triggering its own exit binding.
        if frame.input.key(Key::Escape).pressed() {
            self.escape_close_armed = true;
        }
        if self.escape_close_armed && frame.input.key(Key::Escape).released() {
            self.escape_close_armed = false;
            self.tool_open = false;
            return;
        }
        if frame.input.key(Key::Left).pressed() {
            self.adjust(-1);
        }
        if frame.input.key(Key::Right).pressed() {
            self.adjust(1);
        }

        frame.framebuffer.clear(Pixel::rgb(12, 7, 18));
        frame
            .framebuffer
            .draw_rect(12, 50, 216, 80, Pixel::rgb(170, 90, 230));
        frame
            .framebuffer
            .fill_rect(20, 80, self.bar_width, 20, Pixel::rgb(100, 220, 180));
    }

    fn tool_window_closed(&mut self) {
        self.tool_open = false;
        self.escape_close_armed = false;
    }
}

fn main() -> Result<(), gotoo_pixel_engine::EngineError> {
    run(
        EngineConfig {
            title: "GPE Tool Window Probe - Ctrl+Shift+F opens tool".into(),
            framebuffer_width: MAIN_WIDTH,
            framebuffer_height: MAIN_HEIGHT,
            window_width: 960,
            window_height: 540,
        },
        ToolWindowProbe {
            tool_open: false,
            escape_close_armed: false,
            bar_width: 80,
            heartbeat: 0.0,
        },
    )
}
