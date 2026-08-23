#![deny(warnings)]

use gotoo_pixel_engine::{
    EngineConfig, Frame, Game, GameResult, Key, Pixel, ToolFrame, ToolWindowConfig, run,
};

const MAIN_WIDTH: u32 = 320;
const MAIN_HEIGHT: u32 = 180;
const TOOL_WIDTH: u32 = 240;
const TOOL_HEIGHT: u32 = 180;

struct ToolWindowProbe {
    tool_open: bool,
    bar_width: u32,
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

impl Game for ToolWindowProbe {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if frame.input.key(Key::Escape).pressed() {
            return GameResult::Exit;
        }
        if frame.input.key(Key::F).pressed() {
            self.tool_open = !self.tool_open;
        }

        frame.framebuffer.clear(Pixel::rgb(5, 8, 14));
        frame
            .framebuffer
            .draw_rect(20, 60, 220, 40, Pixel::rgb(90, 120, 180));
        frame
            .framebuffer
            .fill_rect(30, 70, self.bar_width, 20, Pixel::rgb(100, 220, 180));

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
        if frame.input.key(Key::Escape).pressed() || frame.input.key(Key::F).pressed() {
            self.tool_open = false;
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
    }
}

fn main() -> Result<(), gotoo_pixel_engine::EngineError> {
    run(
        EngineConfig {
            title: "GPE Tool Window Probe - F opens tool".into(),
            framebuffer_width: MAIN_WIDTH,
            framebuffer_height: MAIN_HEIGHT,
            window_width: 960,
            window_height: 540,
        },
        ToolWindowProbe {
            tool_open: false,
            bar_width: 80,
        },
    )
}
