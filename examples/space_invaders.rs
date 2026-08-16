#[path = "space_invaders/enhanced.rs"]
mod game;
#[path = "space_invaders/menu.rs"]
mod menu;

use game::{EnhancedSpaceInvadersGame, FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH};
use gotoo_pixel_engine::{
    EngineConfig, EngineError, Frame, Game, GameResult, GamepadProfile, Key, Size, run,
    ui::{PauseConfig, PauseGame},
};
use menu::{MenuAction, SpaceInvadersMenu};

struct SpaceInvadersApp {
    game: Option<PauseGame<EnhancedSpaceInvadersGame>>,
    menu: SpaceInvadersMenu,
    gamepad_profile: GamepadProfile,
}

impl SpaceInvadersApp {
    fn new() -> Self {
        Self {
            game: None,
            menu: SpaceInvadersMenu::new(),
            gamepad_profile: GamepadProfile::standard(),
        }
    }

    fn apply_gamepad_profile(&self, frame: &mut Frame<'_>) {
        let gamepad_ids = frame.input.gamepad_ids().collect::<Vec<_>>();
        for id in gamepad_ids {
            frame.set_gamepad_profile(id, self.gamepad_profile);
        }
    }

    fn start_game(&mut self) {
        let mut game = EnhancedSpaceInvadersGame::new();
        game.controls_mut().clear_virtual();
        self.game = Some(PauseGame::new(
            game,
            PauseConfig::new(Size {
                width: FRAMEBUFFER_WIDTH,
                height: FRAMEBUFFER_HEIGHT,
            }),
        ));
    }
}

impl Game for SpaceInvadersApp {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        self.apply_gamepad_profile(frame);

        if let Some(game) = &mut self.game {
            return game.update(frame);
        }

        if frame.input.key(Key::Escape).pressed() {
            return GameResult::Exit;
        }

        match self.menu.update(frame.input) {
            Some(MenuAction::Play) => self.start_game(),
            Some(MenuAction::Quit) => return GameResult::Exit,
            Some(MenuAction::DecreaseGamepadThreshold) => {
                self.gamepad_profile = self
                    .gamepad_profile
                    .with_digital_threshold(self.gamepad_profile.digital_threshold - 0.05);
            }
            Some(MenuAction::IncreaseGamepadThreshold) => {
                self.gamepad_profile = self
                    .gamepad_profile
                    .with_digital_threshold(self.gamepad_profile.digital_threshold + 0.05);
            }
            Some(MenuAction::ResetGamepadProfile) => {
                self.gamepad_profile = GamepadProfile::standard();
            }
            None => {}
        }

        self.menu.render(frame.framebuffer, self.gamepad_profile);
        GameResult::Continue
    }
}

fn window_size() -> (u32, u32) {
    if std::env::var_os("WSL_DISTRO_NAME").is_some() {
        // WSLg/Weston is known to crash for some surface sizes. 960x612 is
        // documented as stable in docs/investigations/wslg-surface-present-stall.md.
        (960, 612)
    } else {
        (768, 672)
    }
}

fn main() -> Result<(), EngineError> {
    let (window_width, window_height) = window_size();

    run(
        EngineConfig {
            title: "Space Invaders - Gotoo Pixel Engine".into(),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width,
            window_height,
        },
        SpaceInvadersApp::new(),
    )
}
