use crate::platform::{Frame, Game, GameResult};

/// Web-compatible OBS mirror facade.
///
/// The native implementation publishes framebuffer pixels over a local TCP
/// server. Browsers cannot expose that server, but game/example code should
/// not need platform-specific runner forks merely because it opts into the
/// mirror wrapper. On WASM this type therefore preserves the same construction
/// and `Game` contract while forwarding frames directly to the wrapped game.
pub struct ObsMirrorGame<G> {
    game: G,
}

impl<G> ObsMirrorGame<G> {
    pub fn from_env(game: G, _width: u32, _height: u32) -> Self {
        Self { game }
    }
}

impl<G: Game> Game for ObsMirrorGame<G> {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        self.game.update(frame)
    }
}
