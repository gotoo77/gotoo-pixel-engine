use gotoo_pixel_engine::{
    EngineConfig, Frame, Game, GameResult, GamepadButton, GamepadConnectionEvent, GamepadId,
    GamepadProfile, Key, Pixel, run,
};

const WIDTH: u32 = 320;
const HEIGHT: u32 = 180;

struct GamepadInputProbe {
    profile: GamepadProfile,
}

impl Default for GamepadInputProbe {
    fn default() -> Self {
        Self {
            profile: GamepadProfile::standard(),
        }
    }
}

impl Game for GamepadInputProbe {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if frame.input.key(Key::Escape).pressed() {
            return GameResult::Exit;
        }

        let decrease = frame.input.key(Key::A).pressed()
            || frame
                .input
                .gamepad_button_any(GamepadButton::LeftShoulder)
                .pressed();
        let increase = frame.input.key(Key::D).pressed()
            || frame
                .input
                .gamepad_button_any(GamepadButton::RightShoulder)
                .pressed();
        let reset = frame.input.key(Key::Space).pressed()
            || frame
                .input
                .gamepad_button_any(GamepadButton::Start)
                .pressed();

        if reset {
            self.profile = GamepadProfile::standard();
        } else if decrease {
            self.profile = self
                .profile
                .with_digital_threshold(self.profile.digital_threshold - 0.05);
        } else if increase {
            self.profile = self
                .profile
                .with_digital_threshold(self.profile.digital_threshold + 0.05);
        }

        for event in frame.input.gamepad_connection_events() {
            match event {
                GamepadConnectionEvent::Connected(info) => {
                    println!("CONNECTED {:?}: {}", info.id, info.name);
                }
                GamepadConnectionEvent::Disconnected(info) => {
                    println!("DISCONNECTED {:?}: {}", info.id, info.name);
                }
            }
        }

        for id in frame.input.gamepad_ids() {
            for (button, label) in [
                (GamepadButton::South, "SOUTH"),
                (GamepadButton::East, "EAST"),
                (GamepadButton::North, "NORTH"),
                (GamepadButton::West, "WEST"),
                (GamepadButton::LeftShoulder, "LEFT SHOULDER"),
                (GamepadButton::RightShoulder, "RIGHT SHOULDER"),
                (GamepadButton::Start, "START"),
                (GamepadButton::Select, "SELECT"),
                (GamepadButton::DPadUp, "DPAD UP"),
                (GamepadButton::DPadDown, "DPAD DOWN"),
                (GamepadButton::DPadLeft, "DPAD LEFT"),
                (GamepadButton::DPadRight, "DPAD RIGHT"),
                (GamepadButton::LeftStickUp, "STICK UP"),
                (GamepadButton::LeftStickDown, "STICK DOWN"),
                (GamepadButton::LeftStickLeft, "STICK LEFT"),
                (GamepadButton::LeftStickRight, "STICK RIGHT"),
            ] {
                let state = frame.input.gamepad_button(id, button);
                if state.pressed() {
                    println!("{:?}: {label} pressed", id);
                }
                if state.released() {
                    println!("{:?}: {label} released", id);
                }
            }
        }

        frame.framebuffer.clear(Pixel::rgb(8, 12, 16));
        frame
            .framebuffer
            .draw_text(8, 8, "GPE GAMEPAD PROBE", Pixel::rgb(220, 235, 220));
        frame.framebuffer.draw_text(
            8,
            24,
            &format!(
                "DIGITAL THRESHOLD {:02}%",
                (self.profile.digital_threshold * 100.0).round() as u32
            ),
            Pixel::rgb(160, 200, 180),
        );
        frame.framebuffer.draw_text(
            8,
            40,
            "L/R SHOULDER OR A/D: ADJUST",
            Pixel::rgb(160, 200, 180),
        );
        frame.framebuffer.draw_text(
            8,
            56,
            "START OR SPACE: RESET",
            Pixel::rgb(160, 200, 180),
        );
        frame
            .framebuffer
            .draw_text(8, 72, "ESC TO QUIT", Pixel::rgb(160, 200, 180));

        GameResult::Continue
    }

    fn gamepad_profile(&self, _id: GamepadId) -> Option<GamepadProfile> {
        Some(self.profile)
    }
}

fn main() -> Result<(), gotoo_pixel_engine::EngineError> {
    run(
        EngineConfig {
            title: "GPE Gamepad Input Probe".into(),
            framebuffer_width: WIDTH,
            framebuffer_height: HEIGHT,
            window_width: WIDTH * 3,
            window_height: HEIGHT * 3,
        },
        GamepadInputProbe::default(),
    )
}
