#[path = "space_invaders/enhanced.rs"]
mod game;
#[path = "space_invaders/menu.rs"]
mod menu;

use game::{EnhancedSpaceInvadersGame, FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH};
use gotoo_pixel_engine::{
    EngineConfig, EngineError, Frame, Game, GameResult, GamepadId, GamepadProfile, Key, run,
};
use menu::{MenuAction, SpaceInvadersMenu};

struct SpaceInvadersApp {
    game: EnhancedSpaceInvadersGame,
    menu: SpaceInvadersMenu,
    playing: bool,
    gamepad_profile: GamepadProfile,
}

impl SpaceInvadersApp {
    fn new() -> Self {
        let mut game = EnhancedSpaceInvadersGame::new();
        game.controls_mut().clear_virtual();

        Self {
            game,
            menu: SpaceInvadersMenu::new(),
            playing: false,
            gamepad_profile: GamepadProfile::standard(),
        }
    }
}

impl Game for SpaceInvadersApp {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if self.playing {
            return self.game.update(frame);
        }

        if frame.input.key(Key::Escape).pressed() {
            return GameResult::Exit;
        }

        match self.menu.update(frame.input) {
            Some(MenuAction::Play) => self.playing = true,
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

    fn gamepad_profile(&self, _id: GamepadId) -> Option<GamepadProfile> {
        Some(self.gamepad_profile)
    }
}

fn main() -> Result<(), EngineError> {
    run(
        EngineConfig {
            title: "Space Invaders - Gotoo Pixel Engine".into(),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width: 960,
            window_height: 612,
        },
        SpaceInvadersApp::new(),
    )
}
