use crate::{
    Frame, Framebuffer, Game, GameResult, PixelFitPresentation, PixelPresentation, Rect, Size,
    ToolFrame, ToolWindowConfig, Viewport, present_pixel_surface, present_pixel_surface_fit,
    presentation::pixel_fit_presentation,
};

/// Provisional P6 helper for hosting one low-resolution `Game` inside a
/// high-resolution parent frame.
///
/// Hosts a low-resolution game inside a higher-resolution parent frame.
/// Keyboard/gamepad state is forwarded unchanged. Pointer and touch coordinates
/// are mapped into child framebuffer space by the fitted-presentation methods.
pub struct PixelGameHost {
    game: Box<dyn Game>,
    framebuffer: Framebuffer,
    size: Size,
}

impl PixelGameHost {
    pub fn from_boxed(game: Box<dyn Game>, size: Size) -> Self {
        Self {
            game,
            framebuffer: Framebuffer::new(size.width, size.height),
            size,
        }
    }

    pub fn new<G: Game + 'static>(game: G, size: Size) -> Self {
        Self::from_boxed(Box::new(game), size)
    }

    pub const fn size(&self) -> Size {
        self.size
    }

    pub fn framebuffer(&self) -> &Framebuffer {
        &self.framebuffer
    }

    pub fn tool_window_config(&self) -> Option<ToolWindowConfig> {
        self.game.tool_window_config()
    }

    pub fn update_tool_window(&mut self, frame: &mut ToolFrame<'_>) {
        self.game.update_tool_window(frame);
    }

    pub fn tool_window_closed(&mut self) {
        self.game.tool_window_closed();
    }

    fn update_child(&mut self, host: &mut Frame<'_>) -> GameResult {
        let mut child = Frame {
            framebuffer: &mut self.framebuffer,
            input: host.input,
            delta_time: host.delta_time,
            storage: &mut *host.storage,
            audio: &mut *host.audio,
            surface_size: self.size,
            viewport: Viewport::new(self.size, self.size),
        };
        self.game.update(&mut child)
    }

    fn update_child_fit(&mut self, host: &mut Frame<'_>, bounds: Rect) -> GameResult {
        let Some(presentation) = pixel_fit_presentation(self.size, bounds) else {
            return self.update_child(host);
        };

        let mut mapped_input = host.input.clone();
        mapped_input.map_pointer_positions(|point| presentation.map_point(point));

        let mut child = Frame {
            framebuffer: &mut self.framebuffer,
            input: &mapped_input,
            delta_time: host.delta_time,
            storage: &mut *host.storage,
            audio: &mut *host.audio,
            surface_size: self.size,
            viewport: Viewport::new(self.size, self.size),
        };
        self.game.update(&mut child)
    }

    /// Updates a keyboard/gamepad-oriented child and presents its low-resolution
    /// framebuffer into `bounds` using integer-nearest composition.
    ///
    /// Pointer/touch consumers must not use this provisional method until the
    /// mapped-input contract is implemented.
    pub fn update_and_present(
        &mut self,
        host: &mut Frame<'_>,
        bounds: Rect,
    ) -> (GameResult, Option<PixelPresentation>) {
        let result = self.update_child(host);
        let presentation = present_pixel_surface(host.framebuffer, &self.framebuffer, bounds);
        (result, presentation)
    }

    /// Updates a child and fits its framebuffer into `bounds` using aspect-ratio
    /// preserving nearest-neighbour sampling at any scale.
    ///
    /// Pointer and touch positions are translated from host space into the child
    /// framebuffer using exactly the same fitted presentation geometry.
    pub fn update_and_present_fit(
        &mut self,
        host: &mut Frame<'_>,
        bounds: Rect,
    ) -> (GameResult, Option<PixelFitPresentation>) {
        let result = self.update_child_fit(host, bounds);
        let presentation = present_pixel_surface_fit(host.framebuffer, &self.framebuffer, bounds);
        (result, presentation)
    }

    /// Compatibility alias kept during the P6 API-shape audit.
    pub fn update_and_present_shared_input(
        &mut self,
        host: &mut Frame<'_>,
        bounds: Rect,
    ) -> (GameResult, Option<PixelPresentation>) {
        self.update_and_present(host, bounds)
    }

    pub fn present(&self, host: &mut Framebuffer, bounds: Rect) -> Option<PixelPresentation> {
        present_pixel_surface(host, &self.framebuffer, bounds)
    }

    pub fn present_fit(
        &self,
        host: &mut Framebuffer,
        bounds: Rect,
    ) -> Option<PixelFitPresentation> {
        present_pixel_surface_fit(host, &self.framebuffer, bounds)
    }
}
