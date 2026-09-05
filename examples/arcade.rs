#[path = "arcade/game.rs"]
mod arcade;

use arcade::{ArcadeApp, ArcadeInteractionMode};
use gotoo_pixel_engine::{EngineConfig, Frame, Game, GameResult, GamepadAxis, GamepadButton, run};

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
    last_probe: Vec<String>,
}

impl ArcadeWithGamepadProbe {
    fn new() -> Self {
        Self {
            inner: ArcadeApp::new(ArcadeInteractionMode::Native),
            last_probe: Vec::new(),
        }
    }
}

impl Game for ArcadeWithGamepadProbe {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let result = self.inner.update(frame);
        let probe = gamepad_probe_lines(frame);
        if probe != self.last_probe {
            for line in &probe {
                println!("[gpe][gamepad] {line}");
            }
            self.last_probe = probe;
        }
        result
    }
}

fn gamepad_probe_lines(frame: &Frame<'_>) -> Vec<String> {
    let mut ids = frame.input.gamepad_ids().collect::<Vec<_>>();
    ids.sort_by_key(|id| id.as_usize());
    if ids.is_empty() {
        return vec!["no controller detected".to_owned()];
    }

    let mut lines = Vec::with_capacity(ids.len() * 2);
    for id in ids {
        if let Some(info) = frame.input.gamepad_info(id) {
            lines.push(format!(
                "PAD {} {} MAP:{:?} CAPS A:{:?} D:{:?} LSX:{:?}",
                id.as_usize(),
                info.name,
                info.mapping_source,
                info.capabilities.button(GamepadButton::South),
                info.capabilities.button(GamepadButton::DPadUp),
                info.capabilities.axis(GamepadAxis::LeftStickX),
            ));
        } else {
            lines.push(format!("PAD {} (NO DEVICE INFO)", id.as_usize()));
        }

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
        lines.push(format!(
            "PAD {} STATE D:{}{}{}{} LS:{:+.2},{:+.2} A:{}",
            id.as_usize(),
            if up { 'U' } else { '-' },
            if down { 'D' } else { '-' },
            if left { 'L' } else { '-' },
            if right { 'R' } else { '-' },
            x,
            y,
            if south { '1' } else { '0' },
        ));
    }
    lines
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
