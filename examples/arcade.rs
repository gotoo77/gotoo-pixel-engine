#[path = "arcade/game.rs"]
mod arcade;

use arcade::{ArcadeApp, ArcadeInteractionMode};
use gotoo_pixel_engine::{
    EngineConfig, Frame, Game, GameResult, GamepadAxis, GamepadButton, Pixel, Rect, run,
    ui::draw_text_centered,
};

fn window_size(framebuffer_width: u32, framebuffer_height: u32) -> (u32, u32) {
    if std::env::var_os("WSL_DISTRO_NAME").is_some() {
        // WSLg/Weston is known to crash for some surface sizes. 960x612 is
        // documented as stable in docs/investigations/wslg-surface-present-stall.md.
        (960, 612)
    } else {
        (framebuffer_width * 3, framebuffer_height * 3)
    }
}

struct ArcadeWithGamepadProbe {
    inner: ArcadeApp,
}

impl ArcadeWithGamepadProbe {
    fn new() -> Self {
        Self {
            inner: ArcadeApp::new(ArcadeInteractionMode::Native),
        }
    }
}

impl Game for ArcadeWithGamepadProbe {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let result = self.inner.update(frame);
        draw_gamepad_probe(frame);
        result
    }
}

fn draw_gamepad_probe(frame: &mut Frame<'_>) {
    let first = frame.input.gamepad_ids().next();
    let label = if let Some(id) = first {
        let up = frame.input.gamepad_button(id, GamepadButton::DPadUp).held();
        let down = frame
            .input
            .gamepad_button(id, GamepadButton::DPadDown)
            .held();
        let left = frame
            .input
            .gamepad_button(id, GamepadButton::DPadLeft)
            .held();
        let right = frame
            .input
            .gamepad_button(id, GamepadButton::DPadRight)
            .held();
        let south = frame.input.gamepad_button(id, GamepadButton::South).held();
        let x = frame.input.gamepad_axis(id, GamepadAxis::LeftStickX);
        let y = frame.input.gamepad_axis(id, GamepadAxis::LeftStickY);
        format!(
            "PAD {}  D:{}{}{}{}  LS:{:+.2},{:+.2}  A:{}",
            id.as_usize(),
            if up { 'U' } else { '-' },
            if down { 'D' } else { '-' },
            if left { 'L' } else { '-' },
            if right { 'R' } else { '-' },
            x,
            y,
            if south { '1' } else { '0' },
        )
    } else {
        "PAD: NONE (GILRS DID NOT DETECT A CONTROLLER)".to_owned()
    };

    let height = 14_u32.min(frame.framebuffer.height());
    let y = i32::try_from(frame.framebuffer.height().saturating_sub(height)).unwrap_or(0);
    let bounds = Rect {
        x: 0,
        y,
        width: frame.framebuffer.width(),
        height,
    };
    frame.framebuffer.fill_rect(
        bounds.x,
        bounds.y,
        bounds.width,
        bounds.height,
        Pixel::rgb(3, 7, 11),
    );
    draw_text_centered(
        frame.framebuffer,
        bounds,
        &label,
        1,
        Pixel::rgb(111, 238, 184),
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let size = ArcadeInteractionMode::Native.framebuffer_size();
    let (window_width, window_height) = window_size(size.width, size.height);
    run(
        EngineConfig {
            title: "GPE Arcade".into(),
            framebuffer_width: size.width,
            framebuffer_height: size.height,
            window_width,
            window_height,
        },
        ArcadeWithGamepadProbe::new(),
    )?;
    Ok(())
}
