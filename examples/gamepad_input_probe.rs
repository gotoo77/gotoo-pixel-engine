use gotoo_pixel_engine::{
    EngineConfig, Frame, Framebuffer, Game, GameResult, GamepadButton, GamepadConnectionEvent,
    GamepadId, GamepadProfile, Input, Key, Pixel, run,
};

const WIDTH: u32 = 320;
const HEIGHT: u32 = 180;
const BACKGROUND: Pixel = Pixel::rgb(8, 12, 16);
const FOREGROUND: Pixel = Pixel::rgb(160, 200, 180);
const BRIGHT: Pixel = Pixel::rgb(220, 235, 220);
const ACTIVE: Pixel = Pixel::rgb(120, 255, 120);

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

        let gamepad_ids = frame.input.gamepad_ids().collect::<Vec<_>>();
        for id in gamepad_ids.iter().copied() {
            frame.gamepad_profiles.set_profile(id, self.profile);
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

        for id in gamepad_ids {
            for (button, label) in BUTTONS {
                let state = frame.input.gamepad_button(id, button);
                if state.pressed() {
                    println!("{:?}: {label} pressed", id);
                }
                if state.released() {
                    println!("{:?}: {label} released", id);
                }
            }
        }

        render_probe(frame.framebuffer, frame.input, self.profile);
        GameResult::Continue
    }
}

const BUTTONS: [(GamepadButton, &str); 16] = [
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
];

fn render_probe(framebuffer: &mut Framebuffer, input: &Input, profile: GamepadProfile) {
    framebuffer.clear(BACKGROUND);
    framebuffer.draw_text(8, 6, "GPE GAMEPAD PROBE", BRIGHT);

    let active_gamepad = input.gamepad_ids().min_by_key(|id| id.as_usize());
    let device = active_gamepad
        .map(|id| {
            format!(
                "DEVICE {} {}",
                id.as_usize(),
                gamepad_display_name(input, id)
            )
        })
        .unwrap_or_else(|| "DEVICE NONE".to_owned());
    framebuffer.draw_text(8, 18, &device, FOREGROUND);
    framebuffer.draw_text(
        8,
        30,
        &format!(
            "DIGITAL THRESHOLD {:02} PCT",
            (profile.digital_threshold * 100.0).round() as u32
        ),
        FOREGROUND,
    );

    if let Some(id) = active_gamepad {
        for (row, (button, label)) in BUTTONS[..8].iter().copied().enumerate() {
            draw_button_status(
                framebuffer,
                input,
                id,
                8,
                48 + row as i32 * 12,
                label,
                button,
            );
        }
        for (row, (button, label)) in BUTTONS[8..].iter().copied().enumerate() {
            draw_button_status(
                framebuffer,
                input,
                id,
                164,
                48 + row as i32 * 12,
                label,
                button,
            );
        }
    } else {
        framebuffer.draw_text(8, 48, "WAITING FOR GAMEPAD", FOREGROUND);
    }

    framebuffer.draw_text(8, 150, "L/R SHOULDER OR A/D ADJUST", FOREGROUND);
    framebuffer.draw_text(8, 162, "START OR SPACE RESET   ESC QUIT", FOREGROUND);
}

fn draw_button_status(
    framebuffer: &mut Framebuffer,
    input: &Input,
    id: GamepadId,
    x: i32,
    y: i32,
    label: &str,
    button: GamepadButton,
) {
    let held = input.gamepad_button(id, button).held();
    framebuffer.draw_text(
        x,
        y,
        &format!("{label} {}", if held { "ON" } else { "OFF" }),
        if held { ACTIVE } else { FOREGROUND },
    );
}

fn gamepad_display_name(input: &Input, id: GamepadId) -> String {
    input
        .gamepad_info(id)
        .map(|info| {
            info.name
                .chars()
                .map(|character| {
                    if character.is_ascii_alphanumeric() || character == ' ' {
                        character.to_ascii_uppercase()
                    } else {
                        ' '
                    }
                })
                .take(32)
                .collect()
        })
        .unwrap_or_else(|| "UNKNOWN".to_owned())
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
