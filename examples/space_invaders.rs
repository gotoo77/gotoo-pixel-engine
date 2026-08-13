#[path = "space_invaders/enhanced.rs"]
mod game;
#[path = "space_invaders/menu.rs"]
mod menu;

use game::{EnhancedSpaceInvadersGame, FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH};
use gotoo_pixel_engine::{EngineConfig, EngineError, Frame, Game, GameResult, Key, run};
use menu::{MenuAction, SpaceInvadersMenu};

struct SpaceInvadersApp {
    game: EnhancedSpaceInvadersGame,
    menu: SpaceInvadersMenu,
    playing: bool,
}

impl SpaceInvadersApp {
    fn new() -> Self {
        let mut game = EnhancedSpaceInvadersGame::new();
        game.controls_mut().clear_virtual();

        Self {
            game,
            menu: SpaceInvadersMenu::new(),
            playing: false,
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
            None => {}
        }

        self.menu.render(frame.framebuffer);
        GameResult::Continue
    }
}

fn main() -> Result<(), EngineError> {
    run(
        EngineConfig {
            title: "Space Invaders - Gotoo Pixel Engine".into(),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width: 768,
            window_height: 672,
        },
        SpaceInvadersApp::new(),
    )
}
