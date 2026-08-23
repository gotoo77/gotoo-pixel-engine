use crate::platform::{Frame, Game, GameResult, ToolFrame, ToolWindowConfig};

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

    fn tool_window_config(&self) -> Option<ToolWindowConfig> {
        self.game.tool_window_config()
    }

    fn update_tool_window(&mut self, frame: &mut ToolFrame<'_>) {
        self.game.update_tool_window(frame);
    }

    fn tool_window_closed(&mut self) {
        self.game.tool_window_closed();
    }
}
