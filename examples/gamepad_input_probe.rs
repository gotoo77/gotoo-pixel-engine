use gotoo_pixel_engine::{
    EngineConfig, Frame, Game, GameResult, GamepadButton, GamepadConnectionEvent, Key, Pixel, run,
};

const WIDTH: u32 = 320;
const HEIGHT: u32 = 180;

struct GamepadInputProbe;

impl Game for GamepadInputProbe {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if frame.input.key(Key::Escape).pressed() {
            return GameResult::Exit;
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
        frame
            .framebuffer
            .draw_text(8, 24, "WATCH TERMINAL OUTPUT", Pixel::rgb(160, 200, 180));
        frame
            .framebuffer
            .draw_text(8, 40, "ESC TO QUIT", Pixel::rgb(160, 200, 180));

        GameResult::Continue
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
        GamepadInputProbe,
    )
}
